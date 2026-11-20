//! A text too long for its model to read in one request, read in parts of whole turns
//! ([`crate::PieceRef::Turns`]), each part sent with its turns numbered as in the whole text, so
//! that every quotation is found in the whole text as it is for a text read at once.
//!
//! The parts are as even as the text allows and each stays well under what the model reads
//! (`part_chars`), leaving room for the instructions and the answer; each part after the first
//! repeats the last turns of the one before (`overlap`), so a passage across a cut is read whole
//! in one of them. A text refused for its length is one that even one turn alone does not fit.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::snapshot::{Turn, turn_text};

/// How a cited analysis reads a text too long for its model, settled when the job is created
/// from the catalogue's limit, so a catalogue refreshed before the job resumes never cuts the
/// parts already read otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TurnWindows {
    /// The most characters of numbered text the model reads in one request (its catalogue limit
    /// at four characters a token): a text longer than this is read in parts.
    pub most_chars: u32,
    /// The most characters of numbered text one part sends, leaving room for the instructions,
    /// the request and the answer.
    pub part_chars: u32,
    /// How many turns a part repeats from the end of the one before.
    pub overlap: u32,
}

/// The share of what the model reads that one part sends, in tenths.
const PART_TENTHS: u32 = 7;
/// Turns repeated at the start of a part.
const OVERLAP_TURNS: u32 = 2;

impl TurnWindows {
    /// The parts for a model that reads `most_chars` characters in one request.
    pub fn new(most_chars: u32) -> Self {
        Self {
            most_chars,
            part_chars: (u64::from(most_chars) * u64::from(PART_TENTHS) / 10).max(1) as u32,
            overlap: OVERLAP_TURNS,
        }
    }
}

/// One turn the model would be sent alone, and still could not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnTooLong {
    /// The turn's number, from 1.
    pub turn: u32,
    pub chars: u32,
}

/// One turn as the model reads it: `[3] [00:04:12] Awa: what she said`, on one line; `None`
/// for a turn with no words in the text. `index` counts from 0.
pub fn numbered_line(text: &str, turn: &Turn, index: usize) -> Option<String> {
    // On one line, its own spaces (no-break ones included) kept within each line.
    let words = turn_text(text, turn)?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut line = format!("[{}] ", index + 1);
    if let Some(ms) = turn.start_ms {
        line.push_str(&format!("[{}] ", crate::result::clock(ms)));
    }
    if let Some(speaker) = &turn.speaker {
        line.push_str(&format!("{speaker}: "));
    }
    line.push_str(&words);
    Some(line)
}

/// Turns `first` to `last` (numbered from 1, inclusive) as the model reads them.
pub fn window_text(text: &str, turns: &[Turn], first: u32, last: u32) -> String {
    let from = first.saturating_sub(1) as usize;
    let to = (last as usize).min(turns.len());
    turns
        .iter()
        .enumerate()
        .take(to)
        .skip(from)
        .filter_map(|(i, turn)| numbered_line(text, turn, i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Each turn's length as sent, in characters with its line break; 0 for a turn not sent.
pub fn line_sizes(text: &str, turns: &[Turn]) -> Vec<u32> {
    turns
        .iter()
        .enumerate()
        .map(|(i, turn)| numbered_line(text, turn, i).map_or(0, |l| l.chars().count() as u32 + 1))
        .collect()
}

/// The parts a text is read in, as (first, last) turn numbers from 1, inclusive; `None` when the
/// whole text fits in one request, read whole as before; an error when one turn alone does not.
pub fn plan(sizes: &[u32], windows: &TurnWindows) -> Result<Option<Vec<(u32, u32)>>, TurnTooLong> {
    let total: u64 = sizes.iter().map(|&s| u64::from(s)).sum();
    // The last line has no line break after it.
    if total.saturating_sub(1) <= u64::from(windows.most_chars) {
        return Ok(None);
    }
    if let Some((i, &chars)) = sizes
        .iter()
        .enumerate()
        .find(|&(_, &s)| s.saturating_sub(1) > windows.most_chars)
    {
        return Err(TurnTooLong {
            turn: i as u32 + 1,
            chars: chars.saturating_sub(1),
        });
    }
    let budget = u64::from(windows.part_chars.max(1));
    // As many parts as the text needs, each about as long, a little left for the turns repeated.
    let parts = total.div_ceil(budget).max(1);
    let target = budget.min(total.div_ceil(parts) + budget / 8);
    let size =
        |from: usize, to: usize| -> u64 { sizes[from..to].iter().map(|&s| u64::from(s)).sum() };
    let mut out: Vec<(u32, u32)> = Vec::new();
    let (mut first, mut fresh) = (0usize, 0usize);
    while fresh < sizes.len() {
        // At least one turn not read before; the turns repeated go when they leave it no room.
        let mut used = size(first, fresh + 1);
        if used > budget {
            first = fresh;
            used = size(first, fresh + 1);
        }
        let mut last = fresh;
        while last + 1 < sizes.len() && used + u64::from(sizes[last + 1]) <= target {
            last += 1;
            used += u64::from(sizes[last]);
        }
        out.push((first as u32 + 1, last as u32 + 1));
        fresh = last + 1;
        // The next part repeats up to `overlap` turns, never more than a third of a part, and
        // never the whole of this one.
        let mut back = fresh;
        let mut repeated = 0u64;
        while back > first + 1
            && ((fresh - back) as u32) < windows.overlap
            && repeated + u64::from(sizes[back - 1]) <= budget / 3
        {
            back -= 1;
            repeated += u64::from(sizes[back]);
        }
        first = back;
    }
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn windows(most: u32) -> TurnWindows {
        TurnWindows::new(most)
    }

    #[test]
    fn a_text_that_fits_is_read_whole() {
        assert_eq!(plan(&[50, 50, 50], &windows(150)).unwrap(), None);
        assert_eq!(plan(&[], &windows(10)).unwrap(), None);
    }

    #[test]
    fn a_long_text_is_cut_at_whole_turns_into_even_overlapping_parts() {
        let sizes = vec![100u32; 30];
        let w = windows(1_000);
        let parts = plan(&sizes, &w).unwrap().unwrap();
        // Every turn is read, in order, each part under the share of the limit.
        assert_eq!(parts.first().unwrap().0, 1);
        assert_eq!(parts.last().unwrap().1, 30);
        for pair in parts.windows(2) {
            let ((_, last), (first, _)) = (pair[0], pair[1]);
            // The next part repeats two turns of the one before.
            assert_eq!(last - first + 1, 2, "{parts:?}");
        }
        for (first, last) in &parts {
            let chars: u32 = sizes[*first as usize - 1..*last as usize].iter().sum();
            assert!(chars <= w.part_chars, "{parts:?}");
        }
    }

    #[test]
    fn a_turn_longer_than_a_part_is_a_part_of_its_own_without_repetition() {
        let w = windows(1_000);
        let parts = plan(&[300, 900, 300, 300], &w).unwrap().unwrap();
        assert!(parts.contains(&(2, 2)), "{parts:?}");
        assert_eq!(parts.last().unwrap().1, 4);
    }

    #[test]
    fn only_a_turn_the_model_cannot_read_alone_refuses_the_text() {
        let w = windows(1_000);
        assert_eq!(
            plan(&[300, 1_200, 300], &w),
            Err(TurnTooLong {
                turn: 2,
                chars: 1_199
            })
        );
        // Just under the limit alone, it is read alone.
        assert!(plan(&[300, 1_001, 300], &w).unwrap().is_some());
    }

    #[test]
    fn a_window_is_sent_with_the_whole_texts_turn_numbers() {
        let text = "Awa: one\n\nBen: two\n\nAwa: three";
        let turns = vec![
            Turn {
                start: 5,
                end: 8,
                speaker: Some("Awa".into()),
                start_ms: Some(0),
                end_ms: None,
            },
            Turn {
                start: 15,
                end: 18,
                speaker: Some("Ben".into()),
                start_ms: Some(65_000),
                end_ms: None,
            },
            Turn {
                start: 25,
                end: 30,
                speaker: Some("Awa".into()),
                start_ms: None,
                end_ms: None,
            },
        ];
        assert_eq!(
            window_text(text, &turns, 2, 3),
            "[2] [00:01:05] Ben: two\n[3] Awa: three"
        );
        assert_eq!(
            line_sizes(text, &turns)[2],
            "[3] Awa: three".len() as u32 + 1
        );
    }
}
