//! The turns of a snapshot's text: who speaks where, and when, so a later step can say which
//! turn a quotation was found in and play the recording there (#38, #47).
//!
//! A turn is a span of the frozen text in characters, like an [`Anchor`], always inside one
//! part's span, in order and never overlapping another. It covers the words said and nothing
//! else: a `[hh:mm:ss]` stamp and a `Speaker:` label written before them are outside it, so a
//! quotation anchored in a turn is anchored in words. Text between turns (a page marker, blank
//! lines, the words a cut recording repeats at the start of a piece) belongs to none.
//!
//! Turns are read, part by part, in this order:
//!
//! 1. **The part's timed segments** (a provider's turns, or a translated transcript's lines),
//!    times on the recording's clock: each segment is a turn, placed where its words stand in
//!    the part's text, in order, the spacing between words forgiven. If one segment cannot be
//!    placed, none of the part's are trusted and its turns are read from its text instead.
//! 2. **`[hh:mm:ss] Speaker:` lines** a transcription wrote into its text (also once corrected,
//!    when the segments are gone): a turn at each stamp, its time put on the recording's clock
//!    by the piece's start, ending where the part's next turn starts (the piece's end for the
//!    last one); lines without a stamp continue the turn.
//! 3. **`Speaker:` lines** without times, in a transcription: a turn at each label.
//! 4. **Paragraphs**, separated by blank lines: any other text, a page read from a scan
//!    included, so every text a later step reads has numbered passages to point to.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Anchor, slice};
use crate::{PieceRef, Segment, Task};

/// One turn of a snapshot's text: characters `start..end`, inside one part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Turn {
    pub start: u32,
    pub end: u32,
    /// Who speaks, as the researcher named them; `None` for a paragraph or an unlabelled turn.
    pub speaker: Option<String>,
    /// When the turn starts on the recording's clock, when the text has times.
    #[ts(type = "number | null")]
    pub start_ms: Option<u64>,
    /// When it ends, when known.
    #[ts(type = "number | null")]
    pub end_ms: Option<u64>,
}

impl Turn {
    /// Whether an anchor's span lies inside this turn.
    pub fn contains(&self, anchor: &Anchor) -> bool {
        self.start <= anchor.start && anchor.end <= self.end
    }
}

/// The number (place in the list) of the turn an anchor lies in, when one holds it whole.
pub fn turn_of(turns: &[Turn], anchor: &Anchor) -> Option<usize> {
    turns.iter().position(|t| t.contains(anchor))
}

/// A turn's words in the snapshot's text, or `None` when the turn is not in it.
pub fn turn_text<'a>(text: &'a str, turn: &Turn) -> Option<&'a str> {
    if turn.end < turn.start {
        return None;
    }
    slice(text, turn.start, turn.end)
}

/// The turns of one part, whose text (its end trimmed, as the snapshot holds it) starts at
/// character `base` of the snapshot.
pub(super) fn of_part(
    task: Task,
    piece: &PieceRef,
    text: &str,
    segments: &[Segment],
    base: u32,
) -> Vec<Turn> {
    let placed = if segments.is_empty() {
        None
    } else {
        from_segments(text, segments)
    };
    let mut turns = placed.unwrap_or_else(|| from_text(task, piece, text));
    for turn in &mut turns {
        turn.start += base;
        turn.end += base;
    }
    turns
}

/// Each segment placed where its words stand, in order; `None` when one cannot be.
fn from_segments(text: &str, segments: &[Segment]) -> Option<Vec<Turn>> {
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0usize;
    let mut out = Vec::new();
    for segment in segments {
        // A segment with no words says nothing about where the next one is.
        if segment.text.trim().is_empty() {
            continue;
        }
        let (start, end) = find_words(&chars, at, &segment.text)?;
        out.push(Turn {
            start: start as u32,
            end: end as u32,
            speaker: segment.speaker.clone(),
            start_ms: Some(segment.start_ms),
            end_ms: (segment.end_ms > segment.start_ms).then_some(segment.end_ms),
        });
        at = end;
    }
    Some(out)
}

/// The first place at or after `from` where `words` stand, the spaces between them matching
/// any run of white space: the span from its first character to its last.
fn find_words(chars: &[char], from: usize, words: &str) -> Option<(usize, usize)> {
    let words: Vec<Vec<char>> = words
        .split_whitespace()
        .map(|w| w.chars().collect())
        .collect();
    let first = words.first()?.first().copied()?;
    'candidates: for start in from..chars.len() {
        if chars[start] != first {
            continue;
        }
        let mut i = start;
        for (n, word) in words.iter().enumerate() {
            if n > 0 {
                let gap = i;
                while chars.get(i).is_some_and(|c| c.is_whitespace()) {
                    i += 1;
                }
                if i == gap {
                    continue 'candidates;
                }
            }
            if chars.get(i..i + word.len()) != Some(word.as_slice()) {
                continue 'candidates;
            }
            i += word.len();
        }
        return Some((start, i));
    }
    None
}

/// How a part's text is cut into turns when its segments do not say.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    /// A turn at each `[hh:mm:ss]` stamp.
    Stamped,
    /// A turn at each `Speaker:` label.
    Labelled,
    /// A turn per paragraph.
    Paragraphs,
}

/// A speaker label opening `line` (already trimmed): the label and where the words after its
/// colon start, in bytes.
fn label(line: &str) -> Option<(&str, usize)> {
    let colon = line.find(':')?;
    let name = line[..colon].trim_end();
    if !crate::speakers::looks_like_label(name) {
        return None;
    }
    let after = &line[colon + 1..];
    Some((name, colon + 1 + after.len() - after.trim_start().len()))
}

fn chars_in(s: &str) -> u32 {
    s.chars().count() as u32
}

/// A turn being read: its span once its first words are seen.
struct Open {
    span: Option<(u32, u32)>,
    speaker: Option<String>,
    start_ms: Option<u64>,
}

impl Open {
    fn reach(&mut self, start: u32, end: u32) {
        if start == end {
            return;
        }
        self.span = Some(match self.span {
            Some((from, _)) => (from, end),
            None => (start, end),
        });
    }

    fn close(self, out: &mut Vec<Turn>) {
        if let Some((start, end)) = self.span {
            out.push(Turn {
                start,
                end,
                speaker: self.speaker,
                start_ms: self.start_ms,
                end_ms: None,
            });
        }
    }
}

/// A part's turns read from its text: stamped lines, labelled lines or paragraphs.
fn from_text(task: Task, piece: &PieceRef, text: &str) -> Vec<Turn> {
    let first_line = text.split('\n').map(str::trim).find(|l| !l.is_empty());
    let reading = if task != Task::AudioTranscription {
        Reading::Paragraphs
    } else if text
        .split('\n')
        .any(|l| crate::timed::parse_stamp(l.trim()).is_some())
    {
        Reading::Stamped
    } else if first_line.is_some_and(|l| label(l).is_some()) {
        Reading::Labelled
    } else {
        Reading::Paragraphs
    };
    let (offset, piece_end) = match piece {
        PieceRef::Segment {
            start_ms, end_ms, ..
        } => (*start_ms, Some(*end_ms)),
        PieceRef::Page { .. }
        | PieceRef::Whole
        | PieceRef::Passage { .. }
        | PieceRef::Turns { .. } => (0, None),
    };
    let mut out = Vec::new();
    let mut open: Option<Open> = None;
    let mut at = 0u32;
    for line in text.split('\n') {
        let line_start = at;
        at += chars_in(line) + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if reading == Reading::Paragraphs
                && let Some(turn) = open.take()
            {
                turn.close(&mut out);
            }
            continue;
        }
        // Where the trimmed line starts, in characters from the start of the part.
        let from = line_start + chars_in(&line[..line.len() - line.trim_start().len()]);
        // A new turn's time and speaker, and where its words start in the trimmed line.
        let begins: Option<(Option<u64>, Option<String>, usize)> = match reading {
            Reading::Stamped => crate::timed::parse_stamp(trimmed).map(|(ms, rest)| {
                let stamp = trimmed.len() - rest.len();
                match label(rest) {
                    Some((name, words)) => {
                        (Some(offset + ms), Some(name.to_string()), stamp + words)
                    }
                    None => (Some(offset + ms), None, stamp),
                }
            }),
            Reading::Labelled => {
                label(trimmed).map(|(name, words)| (None, Some(name.to_string()), words))
            }
            Reading::Paragraphs => open.is_none().then_some((None, None, 0)),
        };
        let words_at = match begins {
            Some((start_ms, speaker, words)) => {
                if let Some(turn) = open.take() {
                    turn.close(&mut out);
                }
                open = Some(Open {
                    span: None,
                    speaker,
                    start_ms,
                });
                words
            }
            None => 0,
        };
        let start = from + chars_in(&trimmed[..words_at]);
        let end = from + chars_in(trimmed);
        // Words before the first stamp are a turn of their own, with no time.
        open.get_or_insert(Open {
            span: None,
            speaker: None,
            start_ms: None,
        })
        .reach(start, end);
    }
    if let Some(turn) = open {
        turn.close(&mut out);
    }
    if reading == Reading::Stamped {
        close_times(&mut out, piece_end);
    }
    out
}

/// Each timed turn ends where the next timed turn of the part starts, the last at the piece's
/// end; a time that would not be after the start is left unknown.
fn close_times(turns: &mut [Turn], piece_end: Option<u64>) {
    for i in 0..turns.len() {
        let Some(start) = turns[i].start_ms else {
            continue;
        };
        let next = turns[i + 1..].iter().find_map(|t| t.start_ms).or(piece_end);
        turns[i].end_ms = next.filter(|&end| end > start);
    }
}

#[cfg(test)]
mod tests;
