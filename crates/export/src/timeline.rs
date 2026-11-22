//! Turning transcription parts into one continuous timeline, and the timeline into subtitle
//! cues.

use kalamu_core::timed::{close_ends, segments_from_text};
use kalamu_core::{ExportDocument, PieceRef, Segment};

/// Longest subtitle line pair, as in zmo-ai-pipelines' subtitle export.
const MAX_CUE_CHARS: usize = 84;
/// Longest time a cue stays on screen.
const MAX_CUE_MS: u64 = 6_000;
/// Shortest time a cue is shown where its line leaves room: a word flashed for a quarter of a
/// second cannot be read. A line too short for its cues at this pace is cut into fewer, longer
/// ones.
const MIN_CUE_MS: u64 = 1_000;

/// Every segment of the document with times relative to the start of the recording. Parts
/// without timed segments are read for `[hh:mm:ss]` markers, as written by the interview
/// prompts.
pub fn timeline(doc: &ExportDocument) -> Vec<Segment> {
    let mut out = Vec::new();
    for part in &doc.parts {
        let offset = match part.piece {
            PieceRef::Segment { start_ms, .. } => start_ms,
            PieceRef::Page { .. }
            | PieceRef::Whole
            | PieceRef::Passage { .. }
            | PieceRef::Turns { .. } => 0,
        };
        if part.segments.is_empty() {
            out.extend(segments_from_text(&part.text, offset));
        } else {
            out.extend(part.segments.iter().cloned());
        }
    }
    close_ends(&mut out);
    out
}

/// A subtitle cue: when it shows, who speaks, its words, and whether it opens its line, where
/// a subtitle file that names the speakers writes the name.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: Option<String>,
    pub text: String,
    pub opens_line: bool,
}

/// Subtitle cues, the same in every subtitle format: a long line is cut at word boundaries into
/// cues of even length, the speaker's name taking its room in the first where names are written
/// (`speaker_labels`), so no cue is a sliver of a word or two; a line too short to show each cue
/// for [`MIN_CUE_MS`] is cut into fewer. Time is shared out by length, the last cue ending with
/// the line.
pub fn cues(segments: &[Segment], speaker_labels: bool) -> Vec<Cue> {
    let mut out = Vec::new();
    for seg in segments {
        let words: Vec<&str> = seg.text.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        // "Name: " before the first cue's words.
        let label = match (&seg.speaker, speaker_labels) {
            (Some(s), true) => name(s).chars().count() + 2,
            _ => 0,
        };
        let duration = seg.end_ms.saturating_sub(seg.start_ms);
        let chunks = cut(&words, label, duration);
        let lengths: Vec<u64> = chunks
            .iter()
            .enumerate()
            .map(|(i, c)| (c.chars().count() + if i == 0 { label } else { 0 }) as u64)
            .collect();
        let total = lengths.iter().sum::<u64>().max(1);
        let last = chunks.len() - 1;
        let mut before = 0;
        for (i, chunk) in chunks.into_iter().enumerate() {
            let start = seg.start_ms + duration * before / total;
            before += lengths[i];
            let boundary = if i == last {
                seg.end_ms
            } else {
                seg.start_ms + duration * before / total
            };
            out.push(Cue {
                start_ms: start,
                end_ms: boundary.min(start + MAX_CUE_MS).max(start + 1),
                speaker: seg.speaker.clone(),
                text: chunk,
                opens_line: i == 0,
            });
        }
    }
    out
}

/// A speaker's name as a cue writes it, on one line.
pub(crate) fn name(speaker: &str) -> String {
    speaker.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A line's words in as few cues as fit [`MAX_CUE_CHARS`] (fewer where the line lasts less
/// than [`MIN_CUE_MS`] a cue), each about as long as the others: the narrowest width that
/// still cuts it in that many.
fn cut(words: &[&str], label: usize, duration: u64) -> Vec<String> {
    let fit = wrap(words, label, MAX_CUE_CHARS).len();
    let room = usize::try_from(duration / MIN_CUE_MS).unwrap_or(usize::MAX);
    let count = fit.min(room).max(1);
    let length = label
        + words.iter().map(|w| w.chars().count()).sum::<usize>()
        + words.len().saturating_sub(1);
    let mut width = length.div_ceil(count);
    loop {
        let chunks = wrap(words, label, width);
        if chunks.len() <= count {
            return chunks;
        }
        width += 1;
    }
}

/// Words cut into cues of at most `max` characters (a word longer stands alone), the first cue
/// starting with `offset` taken.
fn wrap(words: &[&str], offset: usize, max: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut used = offset;
    for word in words {
        let len = word.chars().count();
        if !current.is_empty() && used + 1 + len > max {
            chunks.push(std::mem::take(&mut current));
            used = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            used += 1;
        }
        current.push_str(word);
        used += len;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}
