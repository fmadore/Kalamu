//! What Whisper in the app (`whisper/mod.rs`) and the Vulkan helper (`helpers/whisper-vulkan`)
//! share: how a window is decoded, how its language is detected, what whisper.cpp reports of
//! each line, and the messages between the two processes. Compiled into both from this one file
//! (the helper includes it by path), so a window decoded on the graphics processor is asked for
//! exactly as one decoded on the processor. Ported from Live Translation's
//! `whisper-vulkan/src/engine.rs`, with Kalamu's decoding: beam search, subtitle-length lines.
//!
//! The messages are frames: a 32-bit little-endian length, then that many bytes. A request or a
//! response is JSON; `Detect` and `Decode` are followed by one more frame holding the samples as
//! 32-bit little-endian floats.

use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperError, WhisperSegment, WhisperState};

/// whisper.cpp's own command-line default for file transcription.
pub const BEAM_SIZE: i32 = 5;
/// Longest line, as the subtitle export's cues, so a cue's times are Whisper's own.
pub const MAX_LINE_CHARS: i32 = 84;

/// Far more than a window can hold (two and a half minutes of samples is under 10 MiB).
const MAX_FRAME: usize = 64 << 20;

/// How one window is decoded. `language` is the language it was spoken in, or `None` to detect
/// it; `translate` writes English; `prompt` is the names and words to spell as tokens.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decode {
    pub language: Option<String>,
    pub translate: bool,
    pub threads: i32,
    pub prompt: Vec<i32>,
}

/// whisper.cpp's parameters for `decode`.
pub fn params(decode: &Decode) -> FullParams<'_, '_> {
    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: BEAM_SIZE,
        patience: -1.0,
    });
    params.set_language(decode.language.as_deref());
    params.set_translate(decode.translate);
    params.set_n_threads(decode.threads);
    // Each window starts fresh: carrying text over is what lets Whisper fall into repeating
    // itself for minutes. The names to spell are given to every window, as tokens: whisper.cpp
    // clears the earlier text first, then puts them in front of each decoding pass.
    params.set_no_context(true);
    if !decode.prompt.is_empty() {
        params.set_tokens(&decode.prompt);
    }
    // Lines of subtitle length, cut between words at the tokens' own times.
    params.set_token_timestamps(true);
    params.set_split_on_word(true);
    params.set_max_len(MAX_LINE_CHARS);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_print_special(false);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params
}

/// The language spoken in `samples`, as whisper.cpp's id; `None` when it cannot tell.
pub fn detect_language(
    state: &mut WhisperState,
    samples: &[f32],
    threads: usize,
) -> Result<Option<i32>, WhisperError> {
    state.pcm_to_mel(samples, threads)?;
    let (id, _) = state.lang_detect(0, threads)?;
    Ok((id >= 0).then_some(id))
}

/// One line as whisper.cpp reported it, before Kalamu decides whether to keep it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawSegment {
    /// Lossy: a token boundary inside a multi-byte character costs one character.
    pub text: String,
    /// Centiseconds from the start of the audio decoded.
    pub start_cs: i64,
    pub end_cs: i64,
    /// One value for the whole window: whisper.cpp asks once, before decoding.
    pub no_speech: f32,
    /// Mean log-probability of the line's tokens.
    pub mean_logprob: f32,
}

/// Every line of the window `state` last decoded.
pub fn segments(state: &WhisperState) -> Vec<RawSegment> {
    state.as_iter().filter_map(|s| raw_segment(&s)).collect()
}

fn raw_segment(segment: &WhisperSegment<'_>) -> Option<RawSegment> {
    let text = segment.to_str_lossy().ok()?.into_owned();
    let tokens = segment.n_tokens();
    let mean_logprob = if tokens > 0 {
        (0..tokens)
            .filter_map(|i| segment.get_token(i))
            .map(|token| token.token_data().plog)
            .sum::<f32>()
            / tokens as f32
    } else {
        0.0
    };
    Some(RawSegment {
        text,
        start_cs: segment.start_timestamp(),
        end_cs: segment.end_timestamp(),
        no_speech: segment.no_speech_probability(),
        mean_logprob,
    })
}

/// What the app asks of the helper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Request {
    /// Load the model at `path`, provided it is `bytes` long with this SHA-256, on the graphics
    /// processor the helper chooses.
    Load {
        path: PathBuf,
        bytes: u64,
        sha256: String,
    },
    /// The text as the model's tokens, for the names-and-words prompt.
    Tokenize { text: String },
    /// Detect the language of the samples in the frame that follows.
    Detect { threads: i32 },
    /// Decode the samples in the frame that follows.
    Decode(Decode),
}

/// The helper's answer to each request, in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Response {
    /// The model is loaded on `device`, as its driver names it.
    #[serde(rename_all = "camelCase")]
    Loaded {
        device: String,
        text_context: i32,
    },
    /// `None` when the model's tokenizer refused the text.
    Tokens {
        tokens: Option<Vec<i32>>,
    },
    Detected {
        language: Option<i32>,
    },
    Decoded {
        segments: Vec<RawSegment>,
    },
    /// The request failed; the helper is still running and can take another.
    Failed {
        message: String,
    },
}

/// What `whisper-vulkan --list-devices` prints, as one line of JSON: the graphics processors
/// it finds and the one it would use.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Devices {
    pub devices: Vec<String>,
    pub chosen: Option<String>,
}

fn invalid(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Write one frame and flush it: the other side is waiting for it.
pub fn write_frame(output: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    let length = u32::try_from(bytes.len()).map_err(|_| invalid("frame too large"))?;
    output.write_all(&length.to_le_bytes())?;
    output.write_all(bytes)?;
    output.flush()
}

/// The next frame, or `None` once the other side has closed the stream between frames.
pub fn read_frame(input: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut length = [0u8; 4];
    match input.read_exact(&mut length) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    let length = usize::try_from(u32::from_le_bytes(length)).unwrap_or(usize::MAX);
    if length > MAX_FRAME {
        return Err(invalid("frame too large"));
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes)?;
    Ok(Some(bytes))
}

/// Write `message` as a JSON frame.
pub fn send<T: Serialize>(output: &mut impl Write, message: &T) -> io::Result<()> {
    write_frame(output, &serde_json::to_vec(message).map_err(invalid)?)
}

/// Read a JSON frame as `T`; `None` once the stream has ended.
pub fn receive<T: for<'de> Deserialize<'de>>(input: &mut impl Read) -> io::Result<Option<T>> {
    read_frame(input)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(invalid))
        .transpose()
}

pub fn samples_to_bytes(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

pub fn bytes_to_samples(bytes: &[u8]) -> io::Result<Vec<f32>> {
    let (samples, rest) = bytes.as_chunks::<4>();
    if !rest.is_empty() {
        return Err(invalid("samples are 4 bytes each"));
    }
    Ok(samples.iter().map(|b| f32::from_le_bytes(*b)).collect())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_and_a_closed_stream_ends_between_them() {
        let mut stream = Vec::new();
        let request = Request::Decode(Decode {
            language: Some("fr".into()),
            translate: true,
            threads: 4,
            prompt: vec![50, 51],
        });
        send(&mut stream, &request).unwrap();
        write_frame(&mut stream, &samples_to_bytes(&[0.25, -1.0])).unwrap();
        let mut reader = stream.as_slice();
        assert_eq!(receive::<Request>(&mut reader).unwrap(), Some(request));
        let samples = read_frame(&mut reader).unwrap().unwrap();
        assert_eq!(bytes_to_samples(&samples).unwrap(), [0.25, -1.0]);
        assert_eq!(read_frame(&mut reader).unwrap(), None);
    }

    #[test]
    fn broken_streams_are_errors_not_requests() {
        assert!(read_frame(&mut u32::MAX.to_le_bytes().as_slice()).is_err());
        let mut cut = Vec::new();
        write_frame(&mut cut, b"abcdef").unwrap();
        cut.truncate(7);
        assert!(read_frame(&mut cut.as_slice()).is_err());
        assert!(bytes_to_samples(&[0, 0, 0]).is_err());
    }

    #[test]
    fn the_messages_read_as_both_sides_write_them() {
        let json = serde_json::to_string(&Request::Detect { threads: 4 }).unwrap();
        assert_eq!(json, r#"{"type":"detect","threads":4}"#);
        let loaded: Response =
            serde_json::from_str(r#"{"type":"loaded","device":"NVIDIA","textContext":448}"#)
                .unwrap();
        assert_eq!(
            loaded,
            Response::Loaded {
                device: "NVIDIA".into(),
                text_context: 448
            }
        );
    }

    #[test]
    fn the_decoding_asked_for_is_kalamus() {
        // whisper-rs exposes no getters; its Debug output shows the native parameters.
        let decode = Decode {
            language: Some("fr".into()),
            translate: false,
            threads: 4,
            prompt: vec![1, 2, 3],
        };
        let shown = format!("{:?}", params(&decode));
        for expected in [
            "beam_size: 5,",
            "no_context: true,",
            "token_timestamps: true,",
            "split_on_word: true,",
            "max_len: 84,",
            "prompt_n_tokens: 3,",
            "n_threads: 4,",
        ] {
            assert!(shown.contains(expected), "{expected} not in {shown}");
        }
    }
}
