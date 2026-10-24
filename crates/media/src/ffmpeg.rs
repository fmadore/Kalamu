//! FFmpeg and ffprobe, run as separate programs: audio and video are probed, analysed for
//! silences and loudness, and cut into pieces in the format a provider takes.
//!
//! The app ships its own LGPL-only build next to the executable (README, "Bundled conversion
//! tools"); FFmpeg is never linked into the app. Development builds also accept one on `PATH`.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::MediaError;
pub use crate::audio_format::AudioFormat;

/// Where the two programs are, once found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ffmpeg {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    /// Whether this build has the Opus encoder; without it pieces fall back to FLAC.
    opus: bool,
}

/// Windows' `CREATE_NO_WINDOW`: a console program started from the app must not flash a
/// console window.
#[cfg(windows)]
fn quiet(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000)
}

#[cfg(not(windows))]
fn quiet(command: &mut Command) -> &mut Command {
    command
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

impl Ffmpeg {
    /// Looks in `SCRIPTORIUM_FFMPEG_DIR`, then in `dirs` (the app's own folder), then, in
    /// development builds only, on `PATH`.
    pub fn locate(dirs: &[PathBuf]) -> Option<Self> {
        let mut candidates: Vec<PathBuf> = std::env::var_os("SCRIPTORIUM_FFMPEG_DIR")
            .map(PathBuf::from)
            .into_iter()
            .chain(dirs.iter().cloned())
            .collect();
        if cfg!(debug_assertions)
            && let Some(path) = std::env::var_os("PATH")
        {
            candidates.extend(std::env::split_paths(&path));
        }
        candidates.into_iter().find_map(|dir| Self::in_dir(&dir))
    }

    fn in_dir(dir: &Path) -> Option<Self> {
        let ffmpeg = dir.join(exe("ffmpeg"));
        let ffprobe = dir.join(exe("ffprobe"));
        if !ffmpeg.is_file() || !ffprobe.is_file() {
            return None;
        }
        let encoders = quiet(Command::new(&ffmpeg).args(["-hide_banner", "-encoders"]))
            .stdin(Stdio::null())
            .output()
            .ok()?;
        let opus = String::from_utf8_lossy(&encoders.stdout).contains("libopus");
        Some(Self {
            ffmpeg,
            ffprobe,
            opus,
        })
    }

    fn run(&self, program: &Path, args: &[&str]) -> Result<std::process::Output, MediaError> {
        quiet(Command::new(program).args(args))
            .stdin(Stdio::null())
            .output()
            .map_err(|e| MediaError::Tool(format!("could not start {}: {e}", program.display())))
    }

    /// Length and streams of an audio or video file.
    pub fn probe(&self, path: &Path) -> Result<Probe, MediaError> {
        let path_text = path.to_string_lossy();
        let output = self.run(
            &self.ffprobe,
            &[
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                &path_text,
            ],
        )?;
        if !output.status.success() {
            return Err(MediaError::Tool(last_line(&output.stderr)));
        }
        parse_probe(&output.stdout)
    }

    /// Silences and loudness over the whole recording, in one decoding pass.
    pub fn analyse(&self, path: &Path) -> Result<Analysis, MediaError> {
        let path_text = path.to_string_lossy();
        let output = self.run(
            &self.ffmpeg,
            &[
                "-hide_banner",
                "-nostats",
                "-i",
                &path_text,
                "-map",
                "0:a:0",
                "-vn",
                "-af",
                SILENCE_AND_VOLUME,
                "-f",
                "null",
                "-",
            ],
        )?;
        if !output.status.success() {
            return Err(MediaError::Tool(last_line(&output.stderr)));
        }
        Ok(parse_analysis(&String::from_utf8_lossy(&output.stderr)))
    }

    /// Mean loudness from up to three one-minute excerpts (start, middle, end): enough to warn
    /// about a quiet recording without decoding all of it while files are being added.
    pub fn sample_loudness(&self, path: &Path, duration_ms: u64) -> Result<f32, MediaError> {
        let path_text = path.to_string_lossy();
        let starts: Vec<u64> = if duration_ms <= 180_000 {
            vec![0]
        } else {
            vec![0, duration_ms / 2 - 30_000, duration_ms - 60_000]
        };
        let mut means = Vec::new();
        for start in starts {
            let start = seconds(start);
            let output = self.run(
                &self.ffmpeg,
                &[
                    "-hide_banner",
                    "-nostats",
                    "-ss",
                    &start,
                    "-t",
                    "60",
                    "-i",
                    &path_text,
                    "-map",
                    "0:a:0",
                    "-vn",
                    "-af",
                    "volumedetect",
                    "-f",
                    "null",
                    "-",
                ],
            )?;
            if let Some(mean) = parse_analysis(&String::from_utf8_lossy(&output.stderr)).mean_db {
                means.push(mean);
            }
        }
        if means.is_empty() {
            return Err(MediaError::Tool("no audio to measure".into()));
        }
        Ok(means.iter().sum::<f32>() / means.len() as f32)
    }

    /// The span `[start_ms, end_ms)` as mono 16 kHz audio in `format`, optionally with the
    /// volume evened out. Read into memory: a twenty-minute piece is a few megabytes.
    pub fn extract(
        &self,
        path: &Path,
        start_ms: u64,
        end_ms: u64,
        format: AudioFormat,
        even_out: bool,
    ) -> Result<Vec<u8>, MediaError> {
        let path_text = path.to_string_lossy();
        let (start, length) = (seconds(start_ms), seconds(end_ms.saturating_sub(start_ms)));
        // Input seeking (`-ss` before `-i`) is fast and, for audio, exact.
        let mut args = vec![
            "-hide_banner",
            "-nostats",
            "-v",
            "error",
            "-ss",
            &start,
            "-t",
            &length,
            "-i",
            &path_text,
            "-map",
            "0:a:0",
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
        ];
        if even_out {
            // Dynamic normalisation: a quiet interviewee and a loud interviewer both end up
            // audible, which is what speech recognition needs (unlike loudnorm, which keeps
            // the gap and resamples to 192 kHz).
            args.extend(["-af", "dynaudnorm"]);
        }
        args.extend(format.encoder_args());
        args.push("pipe:1");
        let output = self.run(&self.ffmpeg, &args)?;
        if !output.status.success() || output.stdout.is_empty() {
            return Err(MediaError::Tool(last_line(&output.stderr)));
        }
        Ok(output.stdout)
    }

    /// The best format among `accepted` (in the provider's order) that this build can write.
    pub fn choose(&self, accepted: &[&str]) -> Option<AudioFormat> {
        // Compressed formats first: Opus is a tenth of FLAC's size at the 16 kHz providers
        // keep anyway, then lossless, then uncompressed.
        let preference = [
            AudioFormat::Pcm16k,
            AudioFormat::OggOpus,
            AudioFormat::WebmOpus,
            AudioFormat::Flac,
            AudioFormat::Wav,
        ];
        preference
            .into_iter()
            .filter(|f| self.opus || !f.is_opus())
            .find(|f| accepted.contains(&f.mime()))
    }
}

const SILENCE_AND_VOLUME: &str = "silencedetect=noise=-35dB:duration=0.5,volumedetect";

fn seconds(ms: u64) -> String {
    format!("{}.{:03}", ms / 1000, ms % 1000)
}

fn last_line(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("FFmpeg failed")
        .trim()
        .to_owned()
}

/// What ffprobe says about a file.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub duration_ms: u64,
    pub has_audio: bool,
    pub has_video: bool,
}

#[derive(Deserialize)]
struct ProbeJson {
    #[serde(default)]
    streams: Vec<StreamJson>,
    format: Option<FormatJson>,
}

#[derive(Deserialize)]
struct StreamJson {
    codec_type: Option<String>,
    disposition: Option<Disposition>,
}

#[derive(Deserialize)]
struct Disposition {
    #[serde(default)]
    attached_pic: u8,
}

#[derive(Deserialize)]
struct FormatJson {
    duration: Option<String>,
}

fn parse_probe(json: &[u8]) -> Result<Probe, MediaError> {
    let probe: ProbeJson =
        serde_json::from_slice(json).map_err(|e| MediaError::Tool(format!("ffprobe: {e}")))?;
    let duration_ms = probe
        .format
        .and_then(|f| f.duration)
        .and_then(|d| d.parse::<f64>().ok())
        .map_or(0, |s| (s * 1000.0).round() as u64);
    let kind = |k: &str| {
        probe.streams.iter().any(|s| {
            s.codec_type.as_deref() == Some(k)
                // Cover art in an MP3 or M4A is a "video" stream; it is not a video.
                && s.disposition.as_ref().is_none_or(|d| d.attached_pic == 0)
        })
    };
    Ok(Probe {
        duration_ms,
        has_audio: kind("audio"),
        has_video: kind("video"),
    })
}

/// Silences and loudness, from FFmpeg's log.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Analysis {
    /// `(start_ms, end_ms)` of each pause of half a second or more.
    pub silences: Vec<(u64, u64)>,
    pub mean_db: Option<f32>,
    pub max_db: Option<f32>,
}

fn parse_analysis(log: &str) -> Analysis {
    let mut analysis = Analysis::default();
    let mut open: Option<u64> = None;
    let value = |line: &str, key: &str| -> Option<f64> {
        let rest = &line[line.find(key)? + key.len()..];
        rest.trim_start()
            .split(|c: char| c.is_whitespace() || c == '|')
            .next()?
            .parse()
            .ok()
    };
    for line in log.lines() {
        if let Some(start) = value(line, "silence_start:") {
            open = Some((start.max(0.0) * 1000.0) as u64);
        } else if let Some(end) = value(line, "silence_end:") {
            let end = (end * 1000.0) as u64;
            analysis.silences.push((open.take().unwrap_or(0), end));
        } else if let Some(mean) = value(line, "mean_volume:") {
            analysis.mean_db = Some(mean as f32);
        } else if let Some(max) = value(line, "max_volume:") {
            analysis.max_db = Some(max as f32);
        }
    }
    analysis
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reads_length_and_streams_and_ignores_cover_art() {
        let json = br#"{"streams":[{"codec_type":"audio"},{"codec_type":"video","disposition":{"attached_pic":1}}],
            "format":{"duration":"3725.480000"}}"#;
        let probe = parse_probe(json).unwrap();
        assert_eq!(probe.duration_ms, 3_725_480);
        assert!(probe.has_audio);
        assert!(!probe.has_video);
    }

    #[test]
    fn reads_silences_and_loudness_from_the_log() {
        let log = "[silencedetect @ 0x1] silence_start: 12.5\n\
                   [silencedetect @ 0x1] silence_end: 14.25 | silence_duration: 1.75\n\
                   [silencedetect @ 0x1] silence_start: -0.01\n\
                   [silencedetect @ 0x1] silence_end: 0.8 | silence_duration: 0.81\n\
                   [Parsed_volumedetect_1 @ 0x2] mean_volume: -27.4 dB\n\
                   [Parsed_volumedetect_1 @ 0x2] max_volume: -3.1 dB\n";
        let analysis = parse_analysis(log);
        assert_eq!(analysis.silences, [(12_500, 14_250), (0, 800)]);
        assert_eq!(analysis.mean_db, Some(-27.4));
        assert_eq!(analysis.max_db, Some(-3.1));
    }

    #[test]
    fn picks_compressed_formats_the_provider_takes() {
        let with_opus = Ffmpeg {
            ffmpeg: PathBuf::new(),
            ffprobe: PathBuf::new(),
            opus: true,
        };
        let without = Ffmpeg {
            opus: false,
            ..with_opus.clone()
        };
        let openai = ["audio/webm", "audio/mpeg", "audio/mp4", "audio/wav"];
        assert_eq!(with_opus.choose(&openai), Some(AudioFormat::WebmOpus));
        assert_eq!(without.choose(&openai), Some(AudioFormat::Wav));
        let gemini = ["audio/flac", "audio/ogg", "audio/mpeg"];
        assert_eq!(with_opus.choose(&gemini), Some(AudioFormat::OggOpus));
        assert_eq!(without.choose(&gemini), Some(AudioFormat::Flac));
        assert_eq!(
            without.choose(&[scriptorium_core::PCM_16K_MONO]),
            Some(AudioFormat::Pcm16k)
        );
        assert_eq!(with_opus.choose(&["audio/mpeg"]), None);
    }

    /// With a real FFmpeg (on PATH in development; skipped where there is none): a recording
    /// with a pause in the middle is measured, its pause found, and pieces written in every
    /// format.
    #[test]
    fn probes_analyses_and_converts_a_real_recording() {
        let Some(ffmpeg) = Ffmpeg::locate(&[]) else {
            eprintln!("no FFmpeg found; skipped");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let wav = dir.path().join("pause.wav");
        let tone = "aevalsrc='if(between(t,3,5),0,0.5*sin(2*PI*440*t))':s=44100:d=8";
        let made = quiet(Command::new(&ffmpeg.ffmpeg).args([
            "-hide_banner",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            tone,
            "-ac",
            "2",
        ]))
        .arg(&wav)
        .status()
        .unwrap();
        assert!(made.success());

        let probe = ffmpeg.probe(&wav).unwrap();
        assert!((7_900..=8_100).contains(&probe.duration_ms), "{probe:?}");
        assert!(probe.has_audio && !probe.has_video);

        let analysis = ffmpeg.analyse(&wav).unwrap();
        assert!(
            analysis
                .silences
                .iter()
                .any(|(s, e)| (2_900..3_200).contains(s) && (4_800..5_100).contains(e)),
            "{analysis:?}"
        );

        let pcm = ffmpeg
            .extract(&wav, 1_000, 3_000, AudioFormat::Pcm16k, false)
            .unwrap();
        assert_eq!(
            pcm.len(),
            2 * 16_000 * 2,
            "two seconds of 16 kHz mono samples"
        );
        for format in [AudioFormat::Flac, AudioFormat::Wav] {
            assert!(
                !ffmpeg
                    .extract(&wav, 0, 8_000, format, true)
                    .unwrap()
                    .is_empty()
            );
        }
        if ffmpeg.opus {
            let ogg = ffmpeg
                .extract(&wav, 0, 8_000, AudioFormat::OggOpus, false)
                .unwrap();
            assert!(ogg.starts_with(b"OggS"));
        }
    }

    #[test]
    fn times_are_written_as_seconds_with_milliseconds() {
        assert_eq!(seconds(1_234_567), "1234.567");
        assert_eq!(seconds(5), "0.005");
    }
}
