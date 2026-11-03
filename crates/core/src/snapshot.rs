//! A frozen copy of a result's text as the researcher left it, which later work reads instead
//! of the result itself, so a correction made afterwards never changes what an earlier
//! analysis read.
//!
//! The text is the corrected view: corrections over the model's output, speakers renamed,
//! pieces that could not be read left out and listed. Offsets are counted in Unicode scalar
//! values (`char`s), not bytes or UTF-16 units, so they mean the same in Rust, in the
//! interface and in a file written in French or Arabic.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{JobId, PieceRef, SetupId, SourceId, Task, Timestamp};

/// Where one piece's text sits in the snapshot: characters `start..end`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SnapshotPart {
    pub piece: PieceRef,
    pub start: u32,
    pub end: u32,
    /// Marked as reviewed by the researcher when the snapshot was taken.
    pub reviewed: bool,
    /// Corrected by the researcher when the snapshot was taken.
    pub corrected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Snapshot {
    pub id: String,
    /// The result it was taken from.
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub task: Task,
    /// The original the result was read from.
    pub original_path: String,
    /// SHA-256 of the text: what later work's cache keys carry.
    pub sha256: String,
    pub parts: Vec<SnapshotPart>,
    /// Pieces that could not be read, left out of the text (numbered from 1).
    pub missing: Vec<u32>,
    pub created_at: Timestamp,
}

impl Snapshot {
    /// A fresh id for a snapshot about to be kept.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// Where a source made from a snapshot came from, carried by the source itself so the engine,
/// the exports and provenance know without asking the store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DerivedFrom {
    pub snapshot_id: String,
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    /// The original, beside which the later results are written.
    pub original_path: String,
    /// For a document unit's pages (#50), cut out of the original as a PDF of their own: the
    /// unit as it was then. The set-up and source above are its table of contents'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unit: Option<crate::units::FrozenUnit>,
}

/// A piece's text as it goes into a snapshot.
pub struct PartText<'a> {
    pub piece: &'a PieceRef,
    pub text: &'a str,
    pub reviewed: bool,
    pub corrected: bool,
    pub failed: bool,
}

/// The snapshot's text and where each piece sits in it. Pages are introduced by a
/// `--- Page N ---` line, as in the text export, so a model can say where it read something;
/// the marker is outside every part's span. Failed pieces are left out.
pub fn build(parts: &[PartText<'_>]) -> (String, Vec<SnapshotPart>) {
    let mut text = String::new();
    let mut len: u32 = 0;
    let mut spans = Vec::new();
    let push = |text: &mut String, len: &mut u32, s: &str| {
        text.push_str(s);
        *len += s.chars().count() as u32;
    };
    for part in parts {
        if part.failed {
            continue;
        }
        if !text.is_empty() {
            push(&mut text, &mut len, "\n\n");
        }
        if let PieceRef::Page { .. } = part.piece {
            push(
                &mut text,
                &mut len,
                &format!("--- Page {} ---\n\n", part.piece.label()),
            );
        }
        let start = len;
        push(&mut text, &mut len, part.text.trim_end());
        spans.push(SnapshotPart {
            piece: part.piece.clone(),
            start,
            end: len,
            reviewed: part.reviewed,
            corrected: part.corrected,
        });
    }
    (text, spans)
}

/// A span of a snapshot's text: characters `start..end`, inside part `part`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Anchor {
    pub part: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnchorError {
    #[error("the span is empty or reversed")]
    Empty,
    #[error("the span goes past the end of the text")]
    OutOfText,
    #[error("the span is not inside one piece")]
    AcrossPieces,
    #[error("the text at the span is not the quotation")]
    Mismatch,
}

/// Characters `start..end` of `text`, or `None` past its end.
fn slice(text: &str, start: u32, end: u32) -> Option<&str> {
    let mut indices = text.char_indices().map(|(i, _)| i).chain([text.len()]);
    let from = indices.nth(start as usize)?;
    let to = if end == start {
        from
    } else {
        indices.nth((end - start - 1) as usize)?
    };
    Some(&text[from..to])
}

/// Every place a quotation stands in the snapshot, each inside one piece, in order: a phrase
/// repeated on two pages gives two anchors, never one taken for the other.
pub fn locate(text: &str, parts: &[SnapshotPart], quote: &str) -> Vec<Anchor> {
    if quote.is_empty() {
        return Vec::new();
    }
    let quote_len = quote.chars().count() as u32;
    let mut out = Vec::new();
    let mut char_at = 0u32;
    let mut byte_at = 0usize;
    while let Some(found) = text[byte_at..].find(quote) {
        let byte = byte_at + found;
        char_at += text[byte_at..byte].chars().count() as u32;
        let (start, end) = (char_at, char_at + quote_len);
        if let Some(part) = parts.iter().position(|p| p.start <= start && end <= p.end) {
            out.push(Anchor {
                part: part as u32,
                start,
                end,
            });
        }
        // Step one character on, so overlapping repeats are found too.
        let step = text[byte..].chars().next().map_or(1, char::len_utf8);
        byte_at = byte + step;
        char_at += 1;
    }
    out
}

/// Checks that an anchor is one piece's text and holds the quotation, as a later step must
/// before it shows a model's quotation as evidence.
pub fn check(
    text: &str,
    parts: &[SnapshotPart],
    anchor: &Anchor,
    quote: &str,
) -> Result<(), AnchorError> {
    if anchor.end <= anchor.start {
        return Err(AnchorError::Empty);
    }
    let found = slice(text, anchor.start, anchor.end).ok_or(AnchorError::OutOfText)?;
    let inside = parts
        .get(anchor.part as usize)
        .is_some_and(|p| p.start <= anchor.start && anchor.end <= p.end);
    if !inside {
        return Err(AnchorError::AcrossPieces);
    }
    if found == quote {
        Ok(())
    } else {
        Err(AnchorError::Mismatch)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn snapshot() -> (String, Vec<SnapshotPart>) {
        let pages = [PieceRef::page(1), PieceRef::page(2), PieceRef::page(3)];
        build(&[
            PartText {
                piece: &pages[0],
                text: "Le Père Éric écrit à l'imam.\n",
                reviewed: true,
                corrected: true,
                failed: false,
            },
            PartText {
                piece: &pages[1],
                text: "",
                reviewed: false,
                corrected: false,
                failed: true,
            },
            PartText {
                piece: &pages[2],
                text: "قال الإمام: السلام عليكم. Le Père Éric répond.",
                reviewed: false,
                corrected: false,
                failed: false,
            },
        ])
    }

    #[test]
    fn the_text_has_page_markers_outside_the_pieces_and_leaves_failures_out() {
        let (text, parts) = snapshot();
        assert!(text.starts_with("--- Page 1 ---\n\nLe Père Éric"));
        assert!(text.contains("\n\n--- Page 3 ---\n\nقال"));
        assert!(!text.contains("Page 2"), "the failed page is left out");
        assert_eq!(parts.len(), 2);
        assert_eq!(
            slice(&text, parts[0].start, parts[0].end),
            Some("Le Père Éric écrit à l'imam.")
        );
        assert_eq!(
            slice(&text, parts[1].start, parts[1].end),
            Some("قال الإمام: السلام عليكم. Le Père Éric répond.")
        );
        assert!(parts[0].reviewed && parts[0].corrected);
    }

    #[test]
    fn a_repeated_phrase_is_found_where_each_stands() {
        let (text, parts) = snapshot();
        let found = locate(&text, &parts, "Le Père Éric");
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].part, found[1].part), (0, 1));
        assert_ne!(found[0].start, found[1].start);
        for anchor in &found {
            assert_eq!(check(&text, &parts, anchor, "Le Père Éric"), Ok(()));
        }
        // Arabic is counted in characters, as French is.
        let arabic = locate(&text, &parts, "السلام عليكم");
        assert_eq!(arabic.len(), 1);
        assert_eq!(
            slice(&text, arabic[0].start, arabic[0].end),
            Some("السلام عليكم")
        );
        // A marker is not a piece's text.
        assert!(locate(&text, &parts, "--- Page 3").is_empty());
        assert!(locate(&text, &parts, "absent").is_empty());
        // Overlapping repeats are all found.
        let (t, p) = build(&[PartText {
            piece: &PieceRef::Whole,
            text: "aaa",
            reviewed: false,
            corrected: false,
            failed: false,
        }]);
        assert_eq!(locate(&t, &p, "aa").len(), 2);
    }

    #[test]
    fn an_invalid_anchor_is_refused() {
        let (text, parts) = snapshot();
        let good = locate(&text, &parts, "l'imam")[0].clone();
        assert_eq!(check(&text, &parts, &good, "l'imam"), Ok(()));
        let shifted = Anchor {
            start: good.start + 1,
            end: good.end + 1,
            ..good.clone()
        };
        assert_eq!(
            check(&text, &parts, &shifted, "l'imam"),
            Err(AnchorError::Mismatch)
        );
        let empty = Anchor {
            end: good.start,
            ..good.clone()
        };
        assert_eq!(check(&text, &parts, &empty, ""), Err(AnchorError::Empty));
        let past = Anchor {
            part: 1,
            start: 10_000,
            end: 10_010,
        };
        assert_eq!(
            check(&text, &parts, &past, "x"),
            Err(AnchorError::OutOfText)
        );
        // From one piece into the next.
        let across = Anchor {
            part: 0,
            start: parts[0].end - 3,
            end: parts[1].start + 3,
        };
        let quote = slice(&text, across.start, across.end).unwrap().to_string();
        assert_eq!(
            check(&text, &parts, &across, &quote),
            Err(AnchorError::AcrossPieces)
        );
    }
}
