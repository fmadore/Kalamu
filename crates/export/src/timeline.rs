//! Turning transcription parts into one continuous timeline, and the timeline into subtitle
//! cues.

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
            PieceRef::Page { .. } | PieceRef::Whole => 0,
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

/// The timed lines of one piece whose model wrote `[hh:mm:ss]` markers into its text, each
/// ending where the next begins, for the review.
pub fn timed_lines(text: &str) -> Vec<Segment> {
    let mut out = segments_from_text(text, 0);
    close_ends(&mut out);
    out
}

/// Closes each segment at the next one's start when the model gave no end.
fn close_ends(segments: &mut [Segment]) {
    for i in 0..segments.len() {
        if segments[i].end_ms <= segments[i].start_ms {
            let next = segments.get(i + 1).map(|s| s.start_ms);
            segments[i].end_ms = next
                .filter(|&n| n > segments[i].start_ms)
                .unwrap_or(segments[i].start_ms + 4_000);
        }
    }
}

/// Reads lines like `[00:01:05] Speaker 2: text`. Lines without a timestamp continue the
/// previous segment. Times are shifted by `offset_ms`.
pub fn segments_from_text(text: &str, offset_ms: u64) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((ms, rest)) = parse_stamp(line) {
            let (speaker, body) = split_speaker(rest);
            out.push(Segment {
                start_ms: offset_ms + ms,
                end_ms: 0,
                speaker,
                text: body.to_string(),
            });
        } else if let Some(last) = out.last_mut() {
            last.text.push(' ');
            last.text.push_str(line);
        }
    }
    out
}

fn parse_stamp(line: &str) -> Option<(u64, &str)> {
    let rest = line.strip_prefix('[')?;
    let (stamp, rest) = rest.split_once(']')?;
    let parts: Vec<u64> = stamp
        .split(':')
        .map(|p| p.trim().parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let secs = match parts.as_slice() {
        [h, m, s] => h * 3600 + m * 60 + s,
        [m, s] => m * 60 + s,
        _ => return None,
    };
    Some((secs * 1000, rest.trim_start()))
}

fn split_speaker(rest: &str) -> (Option<String>, &str) {
    if let Some((label, body)) = rest.split_once(':') {
        let label = label.trim();
        let looks_like_label = !label.is_empty()
            && label.len() <= 40
            && label.split_whitespace().count() <= 4
            && !label.contains('[');
        if looks_like_label {
            return (Some(label.to_string()), body.trim_start());
        }
    }
    (None, rest)
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
