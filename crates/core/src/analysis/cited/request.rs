//! What the model is told about the numbered text it reads: a transcript's turns, each with its
//! time and its speaker, or a document's passages (a page's paragraphs, a text file's), which
//! have neither. The two say the same about quotations, and are numbered alike; only the words
//! for what is numbered differ, so a document is not described to the model as words said.
//!
//! Neither enters a cache key: the numbered text does (`cited_turns:<sha256>`), and a transcript
//! is told exactly what it always was.

use crate::PieceRef;
use crate::snapshot::{SnapshotPart, Turn};

/// Added to the request of an analysis with a field that cites the document, after the
/// prompt's own, for a transcript: a text whose turns have a speaker or a time.
pub const REQUEST: &str = "The text is sent one numbered turn at a time: the turn's number in \
brackets, such as [12], then its time and its speaker when they are known, then what was said. \
For every quotation, give the number of the turn it comes from and the words exactly as they \
stand in that turn, copied character for character: never correct, translate, shorten or join \
words from two turns, and never quote words that are not in the text. Give only the words said, \
without the number, the time or the speaker.";

/// [`REQUEST`] for a document: a text read from pages, or a text file, whose numbered passages
/// have no speaker and no time. The quotation's number is still given as `turn`, the property
/// the answer's fields name.
pub const REQUEST_PASSAGES: &str = "The text is sent one numbered passage at a time: the \
passage's number in brackets, such as [12], then its words. For every quotation, give the \
number of the passage it comes from (as its turn) and the words exactly as they stand in that \
passage, copied character for character: never correct, translate, shorten or join words from \
two passages, and never quote words that are not in the text. Give only the words written, \
without the number.";

/// Whether a text's turns are a transcript's: one at least has a speaker or a time.
pub fn is_transcript(turns: &[Turn]) -> bool {
    turns
        .iter()
        .any(|t| t.speaker.is_some() || t.start_ms.is_some())
}

/// The request added for a text with these turns.
pub fn request_for(turns: &[Turn]) -> &'static str {
    if is_transcript(turns) {
        REQUEST
    } else {
        REQUEST_PASSAGES
    }
}

/// Added to the request of one part of a long text, after [`request_for`]'s: which turns, or
/// which passages, it holds.
pub fn part_request(turns: &[Turn], first: u32, last: u32) -> String {
    let what = if is_transcript(turns) {
        "turns"
    } else {
        "passages"
    };
    format!(
        " The text is long and is read in parts: this part holds {what} {first} to {last}. \
         Answer from this part alone; the parts are brought together afterwards."
    )
}

/// The page turn `turn` (numbered from 1) lies on, when the text was read from pages: the part
/// that holds its start. `None` for a transcript, a text file, or a turn past the last.
pub fn page_of(parts: &[SnapshotPart], turns: &[Turn], turn: u32) -> Option<PieceRef> {
    let start = turns.get(turn.checked_sub(1)? as usize)?.start;
    parts
        .iter()
        .find(|p| p.start <= start && start < p.end.max(p.start + 1))
        .map(|p| &p.piece)
        .filter(|piece| matches!(piece, PieceRef::Page { .. }))
        .cloned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn turn(start: u32, end: u32, speaker: Option<&str>, ms: Option<u64>) -> Turn {
        Turn {
            start,
            end,
            speaker: speaker.map(String::from),
            start_ms: ms,
            end_ms: None,
        }
    }

    fn page(number: u32, start: u32, end: u32) -> SnapshotPart {
        SnapshotPart {
            piece: PieceRef::page(number),
            start,
            end,
            reviewed: false,
            corrected: false,
        }
    }

    #[test]
    fn a_transcript_is_told_of_turns_and_a_document_of_passages() {
        let said = [turn(0, 4, Some("Awa"), None)];
        let timed = [turn(0, 4, None, Some(1_000))];
        let written = [turn(0, 4, None, None), turn(6, 9, None, None)];
        assert_eq!(request_for(&said), REQUEST);
        assert_eq!(request_for(&timed), REQUEST);
        assert_eq!(request_for(&written), REQUEST_PASSAGES);
        assert!(!REQUEST_PASSAGES.contains("said"));
        assert!(part_request(&said, 3, 9).contains("turns 3 to 9"));
        assert!(part_request(&written, 3, 9).contains("passages 3 to 9"));
    }

    #[test]
    fn a_turn_is_on_the_page_that_holds_its_start() {
        let parts = [page(1, 0, 10), page(2, 10, 20)];
        let turns = [turn(0, 4, None, None), turn(12, 18, None, None)];
        assert_eq!(page_of(&parts, &turns, 1), Some(PieceRef::page(1)));
        assert_eq!(page_of(&parts, &turns, 2), Some(PieceRef::page(2)));
        assert_eq!(page_of(&parts, &turns, 3), None);
        assert_eq!(page_of(&parts, &turns, 0), None);
        let whole = [SnapshotPart {
            piece: PieceRef::Whole,
            ..page(1, 0, 20)
        }];
        assert_eq!(page_of(&whole, &turns, 1), None);
    }
}
