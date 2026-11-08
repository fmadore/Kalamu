//! Speech detection before Whisper, with Silero's voice-activity model, when it is downloaded.
//!
//! Each window is looked through for speech, and only the speech is decoded: the stretches
//! found, a little widened, joined with a tenth of a second of silence between them, as
//! whisper.cpp's own `vad` mode does, and every line's times mapped back to the recording. A
//! window with no speech is not decoded at all, so silence and music cost nothing and Whisper has
//! no chance to invent text over them.
//!
//! whisper.cpp's `vad` switch is not used: it applies only in `whisper_full`, while whisper-rs
//! calls `whisper_full_with_state`, where it does nothing. Before this module, Kalamu set it and
//! its provenance said `vad: true`, but every window was decoded whole.

use std::ops::Range;

use whisper_rs::{WhisperVadContext, WhisperVadContextParams, WhisperVadParams};

const SAMPLE_RATE: usize = 16_000;
const SAMPLES_PER_CENTISECOND: f32 = 160.0;
/// Kept on each side of a stretch of speech, so a soft first or last syllable is not cut.
const MARGIN: usize = SAMPLE_RATE / 5;
/// The silence put between two stretches joined: enough for Whisper to hear a pause.
const GAP: usize = SAMPLE_RATE / 10;

/// The voice-activity model, loaded once and kept with the speech model.
pub(super) struct Vad(WhisperVadContext);

impl Vad {
    /// The model at `path`; `None` (logged) when it will not load, and the windows are then
    /// decoded whole.
    pub fn load(path: &str) -> Option<Self> {
        let mut params = WhisperVadContextParams::new();
        params.set_n_threads(2);
        params.set_use_gpu(false);
        match WhisperVadContext::new(path, params) {
            Ok(context) => Some(Self(context)),
            Err(error) => {
                tracing::warn!(path, %error, "the voice-activity model did not load");
                None
            }
        }
    }

    /// Where `samples` hold speech, in samples; empty when nowhere. `None` when detection
    /// failed: the caller then decodes everything, as speech detection must never lose words.
    pub fn speech(&mut self, samples: &[f32]) -> Option<Vec<Range<usize>>> {
        match self
            .0
            .segments_from_samples(WhisperVadParams::new(), samples)
        {
            Ok(segments) => Some(
                segments
                    .map(|s| to_samples(s.start)..to_samples(s.end))
                    .collect(),
            ),
            Err(error) => {
                tracing::warn!(%error, "speech detection failed on a window");
                None
            }
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to zero and rounded; a window is a few million samples at most"
)]
fn to_samples(centiseconds: f32) -> usize {
    (centiseconds.max(0.0) * SAMPLES_PER_CENTISECOND).round() as usize
}

/// A window's speech, joined, and where each part of it came from.
#[derive(Debug, PartialEq)]
pub(super) struct Speech {
    pub samples: Vec<f32>,
    /// Each stretch: where it starts in `samples`, where in the window, and its length.
    stretches: Vec<(usize, usize, usize)>,
}

impl Speech {
    /// All of `samples`, as one stretch: for a window decoded without speech detection.
    pub fn whole(samples: &[f32]) -> Self {
        Self {
            samples: samples.to_vec(),
            stretches: vec![(0, 0, samples.len())],
        }
    }

    /// The stretches of `samples` in `ranges`, each widened by [`MARGIN`], those that then touch
    /// merged, joined with [`GAP`] of silence between them. `None` when there is no speech.
    pub fn join(samples: &[f32], ranges: &[Range<usize>]) -> Option<Self> {
        let mut merged: Vec<Range<usize>> = Vec::new();
        let mut sorted = ranges.to_vec();
        sorted.sort_by_key(|r| r.start);
        for range in sorted {
            let start = range.start.saturating_sub(MARGIN).min(samples.len());
            let end = (range.end + MARGIN).min(samples.len());
            if start >= end {
                continue;
            }
            match merged.last_mut() {
                Some(last) if start <= last.end => last.end = last.end.max(end),
                _ => merged.push(start..end),
            }
        }
        if merged.is_empty() {
            return None;
        }
        let mut speech = Self {
            samples: Vec::new(),
            stretches: Vec::new(),
        };
        for range in merged {
            if !speech.samples.is_empty() {
                speech.samples.resize(speech.samples.len() + GAP, 0.0);
            }
            speech
                .stretches
                .push((speech.samples.len(), range.start, range.len()));
            speech.samples.extend_from_slice(&samples[range]);
        }
        Some(speech)
    }

    /// Where the sample at `joined` in the joined speech was in the window. A time in a gap
    /// between stretches counts as the start of the next one; one past the end, as the end of
    /// the last.
    pub fn original(&self, joined: usize) -> usize {
        for &(at, from, len) in &self.stretches {
            if joined < at {
                return from;
            }
            if joined <= at + len {
                return from + (joined - at);
            }
        }
        self.stretches
            .last()
            .map_or(joined, |&(_, from, len)| from + len)
    }

    /// [`Self::original`] for whisper.cpp's centiseconds, in milliseconds.
    pub fn original_ms(&self, centiseconds: i64) -> u64 {
        let joined = usize::try_from(centiseconds).unwrap_or(0) * (SAMPLE_RATE / 100);
        (self.original(joined) * 1000 / SAMPLE_RATE) as u64
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::single_range_in_vec_init)]
mod tests {
    use super::*;

    const SECOND: usize = SAMPLE_RATE;

    #[test]
    fn speech_is_joined_with_short_pauses_and_times_map_back() {
        // Ten seconds; speech from 1 to 2 s and from 6 to 7 s.
        let samples: Vec<f32> = (0..10 * SECOND).map(|i| i as f32).collect();
        let speech = Speech::join(&samples, &[6 * SECOND..7 * SECOND, SECOND..2 * SECOND]).unwrap();
        let first = SECOND - MARGIN..2 * SECOND + MARGIN;
        let second = 6 * SECOND - MARGIN..7 * SECOND + MARGIN;
        assert_eq!(speech.samples.len(), first.len() + GAP + second.len());
        assert_eq!(speech.samples[0], first.start as f32);
        // The first sample of the second stretch, after the pause, is 6 s less the margin.
        let at = first.len() + GAP;
        assert_eq!(speech.samples[at], second.start as f32);
        assert_eq!(speech.original(0), first.start);
        assert_eq!(speech.original(at + 10), second.start + 10);
        // In the pause: the start of what follows. Past the end: the end of the last.
        assert_eq!(speech.original(first.len() + 1), second.start);
        assert_eq!(speech.original(usize::MAX / 2), second.end);
        // whisper.cpp's centiseconds: one second into the joined speech.
        assert_eq!(speech.original_ms(100), 1_800);
    }

    #[test]
    fn stretches_that_touch_are_merged_and_silence_has_no_speech() {
        let samples = vec![0.0; 10 * SECOND];
        let close = Speech::join(
            &samples,
            &[SECOND..2 * SECOND, 2 * SECOND + 100..3 * SECOND],
        )
        .unwrap();
        assert_eq!(close.stretches.len(), 1);
        assert_eq!(Speech::join(&samples, &[]), None);
        // Within the window whatever whisper.cpp reports.
        let edge = Speech::join(&samples, &[9 * SECOND..12 * SECOND]).unwrap();
        assert_eq!(edge.samples.len(), SECOND + MARGIN);
    }

    #[test]
    fn a_window_decoded_whole_maps_to_itself() {
        let speech = Speech::whole(&[0.0; 32_000]);
        assert_eq!(speech.original(16_000), 16_000);
        assert_eq!(speech.original_ms(150), 1_500);
    }
}
