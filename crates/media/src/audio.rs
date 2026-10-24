//! Recordings: split into pieces at pauses, converted for each provider, and checked before a
//! job starts.
//!
//! iwac-ai-pipelines cuts every twenty minutes exactly; here the cut moves to the longest
//! pause in the minute and a half before that mark, so no word is split. Only when there is
//! no pause at all is the cut made at the mark, and the next piece then starts two seconds
//! earlier so the word on the cut is heard whole once.

use std::path::{Path, PathBuf};

use scriptorium_core::{InputKind, PieceRef, PreparedInput, SourceFile, WarningKind};

use crate::MediaError;
use crate::ffmpeg::{Ffmpeg, Probe};

/// Default length of a piece: what iwac-ai-pipelines found works for Gemini, and short
/// enough that a failed or interrupted piece costs little to redo.
pub const DEFAULT_PIECE_MS: u64 = 20 * 60_000;
/// How far before the target a cut may move to find a pause.
const SEARCH_MS: u64 = 90_000;
/// Repeated at a cut with no pause, so the word on it is heard whole.
const OVERLAP_MS: u64 = 2_000;
/// Mean loudness below which a recording is flagged as quiet; speech is usually -30 to -18 dB.
const QUIET_DB: f32 = -35.0;
/// Recordings longer than this get a note that they will be split.
const LONG_MS: u64 = 2 * 60 * 60_000;

pub struct AudioSource {
    path: PathBuf,
    kind: InputKind,
    /// The file's own MIME type, when a provider might take it as it is.
    mime: Option<&'static str>,
    duration_ms: u64,
    even_out: bool,
    ffmpeg: Option<Ffmpeg>,
}

/// The MIME type providers expect for an audio file, from its extension.
pub fn audio_mime(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "ogg" | "opus" => "audio/ogg",
        "webm" => "audio/webm",
        _ => return None,
    })
}

/// Pieces of at most `target_ms` (plus a quarter for the last one rather than leaving a
/// scrap), cut at the longest pause before each mark. Returns `(start_ms, end_ms)` pairs that
/// cover the whole recording; consecutive pieces touch, or overlap by [`OVERLAP_MS`] where no
/// pause was found.
pub fn plan(duration_ms: u64, silences: &[(u64, u64)], target_ms: u64) -> Vec<(u64, u64)> {
    let target_ms = target_ms.max(60_000);
    let mut pieces = Vec::new();
    let mut start = 0;
    while duration_ms.saturating_sub(start) > target_ms + target_ms / 4 {
        let mark = start + target_ms;
        let pause = silences
            .iter()
            .filter(|(s, e)| *s >= mark.saturating_sub(SEARCH_MS) && *e <= mark && e > s)
            .max_by_key(|(s, e)| e - s);
        match pause {
            Some((s, e)) => {
                let cut = (s + e) / 2;
                pieces.push((start, cut));
                start = cut;
            }
            None => {
                pieces.push((start, mark));
                start = mark - OVERLAP_MS;
            }
        }
    }
    pieces.push((start, duration_ms));
    pieces
}

impl AudioSource {
    pub fn open(source: &SourceFile, ffmpeg: Option<&Ffmpeg>) -> Result<Self, MediaError> {
        let path = PathBuf::from(&source.path);
        let mime = match source.kind {
            InputKind::Audio => audio_mime(&path),
            _ => None,
        };
        if mime.is_none() && ffmpeg.is_none() {
            return Err(MediaError::NeedsConversion(
                "Reading this file needs FFmpeg, which was not found next to the app.".into(),
            ));
        }
        Ok(Self {
            path,
            kind: source.kind,
            mime,
            duration_ms: source.duration_ms.unwrap_or(0),
            even_out: source.even_out_volume,
            ffmpeg: ffmpeg.cloned(),
        })
    }

    /// Analyses the recording (one decoding pass) and plans its pieces. Without FFmpeg, the
    /// recording is one piece sent as it is.
    pub fn pieces(&self, target_ms: u64) -> Vec<PieceRef> {
        let spans = match &self.ffmpeg {
            Some(ffmpeg) if self.duration_ms > target_ms + target_ms / 4 => {
                let silences = ffmpeg
                    .analyse(&self.path)
                    .map(|a| a.silences)
                    .unwrap_or_else(|e| {
                        tracing::warn!(error = %e, "no silence analysis; cutting at fixed marks");
                        Vec::new()
                    });
                plan(self.duration_ms, &silences, target_ms)
            }
            _ => vec![(0, self.duration_ms)],
        };
        spans
            .into_iter()
            .enumerate()
            .map(|(index, (start_ms, end_ms))| PieceRef::Segment {
                index: index as u32,
                start_ms,
                end_ms,
            })
            .collect()
    }

    /// One piece in a format the provider takes: the file itself when it is a single piece in
    /// an accepted format with nothing to fix, otherwise cut and converted by FFmpeg.
    pub fn prepare(
        &self,
        piece: &PieceRef,
        accepted: &[&str],
        file_stem: &str,
    ) -> Result<PreparedInput, MediaError> {
        let PieceRef::Segment {
            index,
            start_ms,
            end_ms,
        } = piece
        else {
            return Err(MediaError::NoSuchPage(piece.label()));
        };
        let whole = *start_ms == 0 && (*end_ms >= self.duration_ms || self.duration_ms == 0);
        if let Some(mime) = self.mime
            && whole
            && !self.even_out
            && accepted.contains(&mime)
        {
            let bytes = std::fs::read(&self.path).map_err(|e| MediaError::Read {
                path: self.path.display().to_string(),
                message: e.to_string(),
            })?;
            let ext = self
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp3");
            return Ok(PreparedInput {
                piece: piece.clone(),
                file_name: format!("{file_stem}.{ext}"),
                mime: mime.to_owned(),
                bytes,
            });
        }
        let ffmpeg = self.ffmpeg.as_ref().ok_or_else(|| {
            MediaError::NeedsConversion(
                "This model needs the recording converted or split, which needs FFmpeg; it was \
                 not found next to the app."
                    .into(),
            )
        })?;
        let format = ffmpeg.choose(accepted).ok_or_else(|| {
            MediaError::NeedsConversion("No audio format this model takes can be written.".into())
        })?;
        let end = if *end_ms == 0 { u64::MAX / 2 } else { *end_ms };
        let bytes = ffmpeg.extract(&self.path, *start_ms, end, format, self.even_out)?;
        Ok(PreparedInput {
            piece: piece.clone(),
            file_name: format!("{file_stem}-{:02}.{}", index + 1, format.extension()),
            mime: format.mime().to_owned(),
            bytes,
        })
    }

    pub fn is_video(&self) -> bool {
        self.kind == InputKind::Video
    }
}

/// Length and streams, for a source being added.
pub fn probe(path: &Path, ffmpeg: Option<&Ffmpeg>) -> Option<Probe> {
    ffmpeg?.probe(path).ok()
}

/// Checks on a recording before a job: no FFmpeg to read it, no sound track, very quiet, very
/// long. Loudness is sampled, not measured over the whole file, to keep adding files quick.
pub fn warnings(
    path: &Path,
    kind: InputKind,
    probe: Option<&Probe>,
    ffmpeg: Option<&Ffmpeg>,
) -> Vec<WarningKind> {
    let mut out = Vec::new();
    let Some(ffmpeg) = ffmpeg else {
        if kind == InputKind::Video || audio_mime(path).is_none() {
            out.push(WarningKind::ToolMissing);
        }
        return out;
    };
    let Some(probe) = probe else {
        return out;
    };
    if !probe.has_audio {
        out.push(WarningKind::NoSoundTrack);
        return out;
    }
    if probe.duration_ms > LONG_MS {
        out.push(WarningKind::LongRecording {
            minutes: (probe.duration_ms / 60_000) as u32,
            pieces: plan(probe.duration_ms, &[], DEFAULT_PIECE_MS).len() as u32,
        });
    }
    if let Ok(mean) = ffmpeg.sample_loudness(path, probe.duration_ms)
        && mean < QUIET_DB
    {
        out.push(WarningKind::QuietRecording {
            mean_db: mean.round() as i32,
        });
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const MIN: u64 = 60_000;

    #[test]
    fn short_recordings_are_one_piece() {
        assert_eq!(plan(10 * MIN, &[], DEFAULT_PIECE_MS), [(0, 10 * MIN)]);
        // Up to a quarter over the target stays whole rather than leaving a scrap.
        assert_eq!(plan(25 * MIN, &[], DEFAULT_PIECE_MS), [(0, 25 * MIN)]);
    }

    #[test]
    fn cuts_fall_in_the_longest_pause_before_each_mark() {
        let silences = [
            (19 * MIN, 19 * MIN + 600),
            (19 * MIN + 30_000, 19 * MIN + 32_000),
            // After the mark: never used, so a piece never exceeds the target.
            (20 * MIN + 1_000, 20 * MIN + 9_000),
        ];
        let pieces = plan(50 * MIN, &silences, DEFAULT_PIECE_MS);
        assert_eq!(pieces[0], (0, 19 * MIN + 31_000));
        assert_eq!(pieces[1].0, pieces[0].1, "pieces touch at a pause");
        assert_eq!(pieces.last().unwrap().1, 50 * MIN);
        assert!(
            pieces
                .iter()
                .all(|(s, e)| e - s <= DEFAULT_PIECE_MS + DEFAULT_PIECE_MS / 4)
        );
    }

    #[test]
    fn without_a_pause_the_next_piece_repeats_two_seconds() {
        let pieces = plan(60 * MIN, &[], DEFAULT_PIECE_MS);
        assert_eq!(pieces[0], (0, 20 * MIN));
        assert_eq!(pieces[1].0, 20 * MIN - OVERLAP_MS);
        assert_eq!(pieces.last().unwrap().1, 60 * MIN);
    }

    #[test]
    fn a_shorter_target_for_a_stricter_model_makes_more_pieces() {
        assert!(plan(60 * MIN, &[], 10 * MIN).len() > plan(60 * MIN, &[], DEFAULT_PIECE_MS).len());
    }
}
