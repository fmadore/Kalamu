//! Turning transcription parts into one continuous timeline, and the timeline into subtitle
//! cues.

use kalamu_core::timed::{close_ends, segments_from_text};
use kalamu_core::{ExportDocument, PieceRef, Segment};

/// Longest subtitle line pair, as in zmo-ai-pipelines' subtitle export.
const MAX_CUE_CHARS: usize = 84;
/// Longest time a cue stays on screen.
const MAX_CUE_MS: u64 = 6_000;

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

/// Subtitle cues: long segments are split at word boundaries, with time shared out by length.
pub fn cues(segments: &[Segment], speaker_labels: bool) -> Vec<Segment> {
    let mut out = Vec::new();
    for seg in segments {
        let text = match (&seg.speaker, speaker_labels) {
            (Some(s), true) => format!("{s}: {}", seg.text.trim()),
            _ => seg.text.trim().to_string(),
        };
        if text.is_empty() {
            continue;
        }
        let chunks = wrap(&text, MAX_CUE_CHARS);
        let duration = seg.end_ms.saturating_sub(seg.start_ms).max(1);
        let total: usize = chunks.iter().map(|c| c.len()).sum::<usize>().max(1);
        let mut start = seg.start_ms;
        for chunk in chunks {
            let share = duration * chunk.len() as u64 / total as u64;
            let end = (start + share.max(800))
                .min(start + MAX_CUE_MS)
                .max(start + 1);
            out.push(Segment {
                start_ms: start,
                end_ms: end.min(seg.end_ms.max(start + 1)),
                speaker: seg.speaker.clone(),
                text: chunk,
            });
            start = end;
        }
    }
    out
}

fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > max {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}
