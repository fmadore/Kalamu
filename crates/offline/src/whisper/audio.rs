//! The recording as whisper.cpp reads it: samples as floats, cut into windows at pauses, and
//! the loudest stretch of a window, where its language is detected.

use super::SAMPLE_RATE;

/// Target window length, and how far back from it the cut may move to find a pause.
pub(super) const WINDOW: usize = 120 * SAMPLE_RATE;
const SEARCH_BACK: usize = 20 * SAMPLE_RATE;
/// The frame over which loudness is measured when looking for the quietest moment.
const FRAME: usize = SAMPLE_RATE / 10;

/// 16-bit little-endian samples to the floats whisper.cpp reads.
pub(super) fn to_float(pcm: &[u8]) -> Vec<f32> {
    pcm.as_chunks::<2>()
        .0
        .iter()
        .map(|b| f32::from(i16::from_le_bytes(*b)) / 32_768.0)
        .collect()
}

/// Where to cut `samples` into windows of about [`WINDOW`]: at the quietest frame of the last
/// [`SEARCH_BACK`] before each target, so a cut falls in a pause rather than inside a word. The
/// last window may be up to a quarter longer rather than leaving a scrap.
pub(super) fn windows(samples: &[f32]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    while samples.len() - start > WINDOW + WINDOW / 4 {
        let target = start + WINDOW;
        let from = target - SEARCH_BACK;
        let quietest = samples[from..target]
            .chunks(FRAME)
            .enumerate()
            .map(|(i, frame)| (i, frame.iter().map(|s| s * s).sum::<f32>()))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map_or(SEARCH_BACK, |(i, _)| i * FRAME + FRAME / 2);
        let cut = from + quietest;
        out.push((start, cut));
        start = cut;
    }
    out.push((start, samples.len()));
    out
}

/// Where the stretch of `len` samples holding the most sound begins and ends, in whole frames;
/// all of `samples` when shorter.
pub(super) fn loudest_stretch(samples: &[f32], len: usize) -> (usize, usize) {
    if samples.len() <= len {
        return (0, samples.len());
    }
    let energy: Vec<f32> = samples
        .chunks(FRAME)
        .map(|frame| frame.iter().map(|s| s * s).sum())
        .collect();
    let frames = (len / FRAME).clamp(1, energy.len());
    let mut sum: f32 = energy[..frames].iter().sum();
    let mut best = (sum, 0);
    for i in frames..energy.len() {
        sum += energy[i] - energy[i - frames];
        if sum > best.0 {
            best = (sum, i + 1 - frames);
        }
    }
    let start = (best.1 * FRAME).min(samples.len() - len);
    (start, start + len)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::DETECT;
    use super::*;

    #[test]
    fn windows_cover_the_audio_and_cut_in_the_quietest_moment() {
        // Five minutes of "speech" with one silent frame in the search range before 2:00.
        let mut samples = vec![0.5f32; 300 * SAMPLE_RATE];
        let quiet = WINDOW - 5 * SAMPLE_RATE;
        for s in &mut samples[quiet..quiet + FRAME] {
            *s = 0.0;
        }
        let cuts = windows(&samples);
        assert_eq!(cuts.first().unwrap().0, 0);
        assert_eq!(cuts.last().unwrap().1, samples.len());
        for pair in cuts.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "windows must touch without gaps");
        }
        assert_eq!(cuts[0].1, quiet + FRAME / 2);
        // Short audio is one window, and a scrap is folded into the last one.
        assert_eq!(windows(&samples[..60 * SAMPLE_RATE]).len(), 1);
        assert_eq!(windows(&samples[..145 * SAMPLE_RATE]).len(), 1);
    }

    #[test]
    fn language_is_detected_where_the_window_is_loudest() {
        // Ten seconds of silence, then forty of sound: the thirty seconds start in the sound.
        let mut samples = vec![0.0f32; 50 * SAMPLE_RATE];
        for s in &mut samples[10 * SAMPLE_RATE..] {
            *s = 0.3;
        }
        let (from, to) = loudest_stretch(&samples, DETECT);
        assert!(from >= 10 * SAMPLE_RATE, "starts at {from}");
        assert_eq!(to - from, DETECT);
        // A window shorter than the stretch is read whole.
        assert_eq!(
            loudest_stretch(&samples[..SAMPLE_RATE], DETECT),
            (0, SAMPLE_RATE)
        );
    }

    #[test]
    fn pcm_bytes_become_floats() {
        let pcm = [0x00, 0x80, 0xff, 0x7f, 0x00, 0x00];
        assert_eq!(to_float(&pcm), [-1.0, 32_767.0 / 32_768.0, 0.0]);
    }
}
