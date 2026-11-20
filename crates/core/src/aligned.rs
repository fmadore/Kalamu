//! A transcript translated line by line, so that its subtitles keep the times of the lines they
//! translate (#167).
//!
//! When a transcript's text is frozen for a later job, its timed lines are kept with it, on the
//! recording's clock, if every part read has its times ([`lines_of`]). A translation of that
//! text sends each passage of lines as numbered units, `[12] what was said`, one per line: never
//! a time or a speaker, which the model has no business rewriting. The model answers with the
//! same numbers, each with its line translated, in one field of entries ([`schema`]); the
//! answer is taken only when it gives back every number sent, once, and no other ([`read`]).
//! Times and speakers are then copied from the lines by number, so a cue can never move.
//!
//! The translation's text is its lines, one paragraph each ([`text`]). A correction keeps the
//! times when it keeps one paragraph per line ([`as_corrected`]); one that joins or splits
//! lines loses them, and subtitles are refused for it rather than shifted.

use std::ops::Range;

use crate::{AnalysisSchema, ExportPart, Field, FieldAnswer, FieldKind, PieceRef, Segment, Task};

/// The field that holds the translated lines.
pub const LINES: &str = "lines";
/// A line's number, as sent.
pub const ID: &str = "id";
/// A line's translation.
pub const TEXT: &str = "text";

/// At most this many characters of lines in a passage: well within every model's output
/// limit once each line comes back with its number around it.
pub const PASSAGE_CHARS: usize = 6_000;
/// At most this many lines in a passage, so a long run of short turns is not one long answer.
pub const PASSAGE_LINES: usize = 120;

/// Added to the request of a passage of lines, after the prompt's own.
pub const REQUEST: &str = "The text is a transcript, one numbered line at a time: the line's \
number in brackets, such as [12], then what was said. Give back every line, translated, as one \
entry with its number exactly as given and its translation, in the same order. Never join, split, \
leave out or add a line, and write no number, time or speaker in a line's translation.";

/// A translation's answer when it reads timed lines: each line's number and its translation.
pub fn schema() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![Field {
            key: LINES.to_string(),
            label: "Translated lines".to_string(),
            instructions: "Every numbered line of the text, translated: one entry per line, in \
                           the same order."
                .to_string(),
            kind: FieldKind::Entries {
                fields: vec![
                    Field {
                        key: ID.to_string(),
                        label: "Number".to_string(),
                        instructions: "The line's number, exactly as given, without brackets."
                            .to_string(),
                        kind: FieldKind::Text,
                        reason: false,
                    },
                    Field {
                        key: TEXT.to_string(),
                        label: "Translation".to_string(),
                        instructions: "The line translated, and nothing else.".to_string(),
                        kind: FieldKind::Text,
                        reason: false,
                    },
                ],
            },
            reason: false,
        }],
    }
}

/// A line's words on one line, its spaces evened out.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every timed line of a transcript, or of a translated transcript, as the researcher left it
/// (the parts of its export document),
/// on the recording's clock and each on one line: `None` unless every part read has its times
/// (a model's timed segments, or `[hh:mm:ss]` markers a transcription wrote into its text). A
/// part corrected without them, or a translation made without lines, has none, and then the
/// text keeps no times at all rather than some.
pub fn lines_of(task: Task, parts: &[ExportPart]) -> Option<Vec<Segment>> {
    if !matches!(task, Task::AudioTranscription | Task::Translation) {
        return None;
    }
    let mut out: Vec<Segment> = Vec::new();
    for part in parts {
        if part.failed.is_some() {
            continue;
        }
        let read = if !part.segments.is_empty() {
            part.segments.clone()
        } else if task == Task::AudioTranscription {
            let offset = match part.piece {
                PieceRef::Segment { start_ms, .. } => start_ms,
                PieceRef::Page { .. }
                | PieceRef::Whole
                | PieceRef::Passage { .. }
                | PieceRef::Turns { .. } => 0,
            };
            crate::timed::segments_from_text(&part.text, offset)
        } else {
            Vec::new()
        };
        if read.is_empty() && !part.text.trim().is_empty() {
            return None;
        }
        out.extend(read.into_iter().filter_map(|s| {
            let text = one_line(&s.text);
            (!text.is_empty()).then_some(Segment { text, ..s })
        }));
    }
    if out.is_empty() {
        return None;
    }
    crate::timed::close_ends(&mut out);
    Some(out)
}

/// The lines cut into passages, in order, none split: at most [`PASSAGE_CHARS`] characters and
/// [`PASSAGE_LINES`] lines each. The same lines always give the same passages.
pub fn passages(lines: &[Segment]) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let (mut start, mut chars) = (0, 0);
    for (i, line) in lines.iter().enumerate() {
        let len = line.text.chars().count() + 8;
        if i > start && (chars + len > PASSAGE_CHARS || i - start >= PASSAGE_LINES) {
            out.push(start..i);
            (start, chars) = (i, 0);
        }
        chars += len;
    }
    if start < lines.len() {
        out.push(start..lines.len());
    }
    out
}

/// A passage as the model reads it: each line after its number (counted from 1 over the whole
/// transcript), one per line, with neither time nor speaker.
pub fn units(lines: &[Segment], range: Range<usize>) -> String {
    lines[range.clone()]
        .iter()
        .zip(range)
        .map(|(line, i)| format!("[{}] {}", i + 1, line.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A passage as the review shows it beside its translation: each line with its time and
/// speaker, one paragraph each.
pub fn source_text(lines: &[Segment], range: Range<usize>) -> String {
    lines[range]
        .iter()
        .map(|line| {
            let speaker = line
                .speaker
                .as_deref()
                .map(|s| format!("{s}: "))
                .unwrap_or_default();
            format!(
                "[{}] {speaker}{}",
                crate::result::clock(line.start_ms),
                line.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// The translated lines as one text: one paragraph per line.
pub fn text(lines: &[Segment]) -> String {
    lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A line's number as the model gave it, brackets and spaces allowed.
fn number(raw: &str) -> Option<usize> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim()
        .parse()
        .ok()
}

/// The passage's lines translated: each line's time and speaker, the model's translation of it.
/// Refused, with the reason, unless the answer gives back every number sent, each once, no
/// other, and none empty: the engine asks again, and a passage that never comes back whole
/// fails rather than placing a translation under another line's time.
pub fn read(
    answers: &[FieldAnswer],
    lines: &[Segment],
    range: Range<usize>,
) -> Result<Vec<Segment>, String> {
    let entries = answers
        .iter()
        .find(|a| a.key == LINES)
        .map(|a| a.value.entries())
        .ok_or_else(|| "the answer has no translated lines".to_string())?;
    let mut translated: Vec<Option<String>> = vec![None; range.len()];
    for entry in entries {
        let field = |key: &str| {
            entry
                .iter()
                .find(|a| a.key == key)
                .map(|a| a.value.joined())
                .unwrap_or_default()
        };
        let raw = field(ID);
        let at = number(&raw)
            .filter(|n| range.contains(&n.wrapping_sub(1)))
            .map(|n| n - 1 - range.start)
            .ok_or_else(|| {
                format!(
                    "the answer gives a line \"{}\" that was not sent",
                    raw.trim()
                )
            })?;
        let text = one_line(&field(TEXT));
        if text.is_empty() {
            return Err(format!("line {} came back empty", at + range.start + 1));
        }
        if translated[at].replace(text).is_some() {
            return Err(format!("line {} came back twice", at + range.start + 1));
        }
    }
    let missing: Vec<String> = translated
        .iter()
        .enumerate()
        .filter(|(_, t)| t.is_none())
        .map(|(i, _)| (i + range.start + 1).to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} of {} lines did not come back translated: {}",
            missing.len(),
            range.len(),
            missing.join(", ")
        ));
    }
    Ok(lines[range]
        .iter()
        .zip(translated)
        .map(|(line, text)| Segment {
            text: text.unwrap_or_default(),
            ..line.clone()
        })
        .collect())
}

/// A translation's lines with the researcher's correction: the corrected text's paragraphs in
/// place of the lines' words, each keeping its line's time and speaker. A correction that no
/// longer has one paragraph per line gives the number of paragraphs it has instead, and its
/// times are lost: which paragraph would go where is not for Kalamu to guess.
pub fn as_corrected(lines: &[Segment], corrected: &str) -> Result<Vec<Segment>, usize> {
    let paragraphs: Vec<String> = corrected
        .replace("\r\n", "\n")
        .split("\n\n")
        .map(one_line)
        .filter(|p| !p.is_empty())
        .collect();
    if paragraphs.len() != lines.len() {
        return Err(paragraphs.len());
    }
    Ok(lines
        .iter()
        .zip(paragraphs)
        .map(|(line, text)| Segment {
            text,
            ..line.clone()
        })
        .collect())
}

/// What a translation's cache keys carry of the lines it read: their times, speakers and words,
/// so that another set of lines behind the same text is never answered from this one's.
pub fn fingerprint(lines: &[Segment]) -> String {
    let material = serde_json::to_string(lines).unwrap_or_default();
    crate::fingerprint::sha256_hex(material.as_bytes())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
