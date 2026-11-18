//! Where a chunk begins, given as a word search's match, so the review opens there as it does
//! for words found: the chunk's opening words as the words searched for, which of their
//! matches in the piece's text it is, and, in a recording, when its line begins.

use kalamu_core::{Passage, PieceRef};

use crate::Hit;
use crate::fold::{fold, folded, long_enough, spots};

/// The most words searched for to find a chunk's start.
const OPENING_WORDS: usize = 6;
/// The most characters of a chunk shown.
const SHOWN: usize = 280;

/// The chunk's opening words: up to [`OPENING_WORDS`] words on its first line, enough to fold
/// to three characters at least; empty when none do.
fn opening(chars: &[char], start: usize, end: usize) -> String {
    let mut words = 0;
    let mut out = String::new();
    let mut in_word = false;
    for &c in &chars[start..end] {
        if c == '\n' {
            break;
        }
        if c.is_whitespace() {
            if in_word {
                words += 1;
                if words >= OPENING_WORDS {
                    break;
                }
            }
            in_word = false;
        } else {
            in_word = true;
        }
        out.push(c);
    }
    let out = out.trim().to_string();
    let needle: Vec<char> = folded(&out).chars().collect();
    if long_enough(&needle) {
        out
    } else {
        String::new()
    }
}

/// The chunk's words shown: its first [`SHOWN`] characters, cut at a word, on one line.
fn shown(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= SHOWN {
        return flat;
    }
    let cut: String = flat.chars().take(SHOWN).collect();
    let cut = match cut.rfind(' ') {
        Some(at) if at > SHOWN / 2 => cut[..at].to_string(),
        _ => cut,
    };
    format!("{cut}…")
}

/// The chunk `[start, end)` of `passage` as a match.
pub(crate) fn hit(passage: &Passage, start: usize, end: usize) -> Hit {
    let part = passage.parts.first();
    let text = part.map_or("", |p| p.text.as_str());
    let chars: Vec<char> = text.chars().collect();
    let end = end.min(chars.len());
    let start = start.min(end);
    let term = opening(&chars, start, end);
    let mut nth = 0;
    let mut at_ms = None;
    if !term.is_empty() {
        let needle: Vec<char> = folded(&term).chars().collect();
        let found = spots(&fold(text), &needle);
        nth = found
            .iter()
            .position(|s| s.span.0 >= start)
            .unwrap_or(found.len()) as u32;
        // A recording's line, when its lines hold as many matches as its text.
        if let (PieceRef::Segment { start_ms, .. }, Some(part)) = (&passage.piece, part) {
            let at_lines: Vec<u64> = part
                .lines
                .iter()
                .flat_map(|(at, line)| spots(&fold(line), &needle).into_iter().map(move |_| *at))
                .collect();
            if !at_lines.is_empty() && at_lines.len() == found.len() {
                at_ms = at_lines.get(nth as usize).map(|at| start_ms + at);
            }
        }
    }
    Hit {
        index: passage.key.index,
        piece: passage.piece.clone(),
        term,
        nth,
        field: None,
        entry: None,
        entry_number: None,
        reason: false,
        before: String::new(),
        words: shown(&chars[start..end].iter().collect::<String>()),
        after: String::new(),
        marks: Vec::new(),
        at_ms,
        reviewed: passage.reviewed,
        corrected: passage.corrected,
    }
}
