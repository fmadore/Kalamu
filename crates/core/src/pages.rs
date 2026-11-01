//! Several pages read in one call: the instruction that makes the model mark where each page
//! begins, and the answer split back into pages at those marks. Each page is then stored as its
//! own result, so the review, corrections, exports and error rates stay page by page. A page
//! whose mark is missing is not guessed at: the engine reads it again on its own.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How many pages of a PDF a text extraction sends in one call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum PagesPerCall {
    /// One page per call: the safest, and what every job did before.
    #[default]
    One,
    /// Up to this many consecutive pages per call.
    Several { pages: u32 },
    /// As many as the model can safely write in one answer, up to the whole document.
    Whole,
}

impl PagesPerCall {
    /// The pages asked for in one call, before the model's output limit caps them.
    pub fn asked(self, document_pages: u32) -> u32 {
        match self {
            Self::One => 1,
            Self::Several { pages } => pages.max(1),
            Self::Whole => document_pages.max(1),
        }
    }
}

/// The line that begins page `number` in the answer.
pub fn marker(number: u32) -> String {
    format!("--- Page {number} ---")
}

/// What is added to the request when pages `first..=last` go in one file: mark each page, in
/// order, numbered as the document numbers them.
pub fn instruction(first: u32, last: u32) -> String {
    format!(
        "This file holds pages {first} to {last} of the document ({count} pages). Transcribe \
         every page, in order. Begin each page's transcription with a line of its own reading \
         exactly `{start}`, then `{next}` and so on, numbered as here even where the page \
         prints another number. Write `[BLANK PAGE]` after the line of a page with no text.",
        count = last - first + 1,
        start = marker(first),
        next = marker(first + 1),
    )
}

/// The page number a line marks, if it is a page mark: `--- Page 3 ---`, also with other dashes
/// or `=`, Markdown emphasis or a heading's `#`, and any capitals.
fn marked(line: &str) -> Option<u32> {
    let decoration =
        |c: char| matches!(c, '-' | '=' | '#' | '*' | '_' | '—' | '–') || c.is_whitespace();
    let core = line.trim_matches(decoration);
    let word = core.get(..4)?;
    if !word.eq_ignore_ascii_case("page") {
        return None;
    }
    let number = core[4..].trim();
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    number.parse().ok()
}

/// The answer for pages `first..=last` split at its page marks: each page found, with its text.
/// Marks numbered as the document numbers the pages are used as they are; marks numbered from 1
/// (a model that counted the pages of the file) are moved to the document's numbers. Marks out
/// of order, repeated or outside the range make the answer unusable: nothing is returned, and
/// every page is read again on its own. Text before the first mark is not a page's.
pub fn split(text: &str, first: u32, last: u32) -> Vec<(u32, String)> {
    let mut marks: Vec<(u32, usize, usize)> = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        if let Some(n) = marked(line) {
            marks.push((n, at, at + line.len()));
        }
        at += line.len();
    }
    if marks.is_empty() {
        return Vec::new();
    }
    let increasing = marks.windows(2).all(|w| w[0].0 < w[1].0);
    let count = last - first + 1;
    let shift = if !increasing {
        return Vec::new();
    } else if marks.iter().all(|(n, ..)| (first..=last).contains(n)) {
        0
    } else if first > 1 && marks.iter().all(|(n, ..)| (1..=count).contains(n)) {
        first - 1
    } else {
        return Vec::new();
    };
    marks
        .iter()
        .enumerate()
        .map(|(i, &(n, _, body))| {
            let end = marks.get(i + 1).map_or(text.len(), |next| next.1);
            (n + shift, text[body..end].trim().to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_is_split_at_its_page_marks() {
        let answer = "Here is the text.\n--- Page 5 ---\nLettre du 3 mai.\n\n--- Page 6 ---\n\
                      Suite.\n--- Page 7 ---\n[BLANK PAGE]\n";
        assert_eq!(
            split(answer, 5, 7),
            vec![
                (5, "Lettre du 3 mai.".to_string()),
                (6, "Suite.".to_string()),
                (7, "[BLANK PAGE]".to_string()),
            ]
        );
    }

    #[test]
    fn marks_are_read_loosely_but_never_guessed() {
        // Other dashes, a heading, capitals; numbered from 1 by a model counting the file.
        let loose = "## PAGE 1\nUn.\n**Page 2**\nDeux.\n=== page 3 ===\nTrois.";
        assert_eq!(
            split(loose, 9, 11)
                .iter()
                .map(|(n, _)| *n)
                .collect::<Vec<_>>(),
            [9, 10, 11]
        );
        // A page whose mark is missing is simply not found.
        let gap = "--- Page 1 ---\nUn.\n--- Page 3 ---\nTrois.";
        assert_eq!(
            split(gap, 1, 3).iter().map(|(n, _)| *n).collect::<Vec<_>>(),
            [1, 3]
        );
        // Out of order, repeated, out of range or absent: nothing.
        assert!(split("--- Page 2 ---\nb\n--- Page 1 ---\na", 1, 2).is_empty());
        assert!(split("--- Page 1 ---\na\n--- Page 1 ---\nb", 1, 2).is_empty());
        assert!(split("--- Page 8 ---\na", 1, 2).is_empty());
        assert!(split("No marks at all.", 1, 2).is_empty());
        // A line that only mentions a page is not a mark.
        assert!(split("See page 2 of the register.\n", 1, 2).is_empty());
    }

    #[test]
    fn the_instruction_names_the_marks_to_write() {
        let text = instruction(5, 8);
        assert!(text.contains("pages 5 to 8") && text.contains("(4 pages)"));
        assert!(text.contains("`--- Page 5 ---`") && text.contains("`--- Page 6 ---`"));
        assert_eq!(PagesPerCall::Whole.asked(40), 40);
        assert_eq!(PagesPerCall::Several { pages: 0 }.asked(40), 1);
    }
}
