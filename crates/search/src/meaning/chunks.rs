//! A passage's text cut into chunks for embedding: about [`TARGET`] characters each, never more
//! than the model takes, each overlapping the one before by about [`OVERLAP`] of its length, cut
//! at a paragraph, else a sentence, else a word, and never inside a word or a grapheme, in any
//! script. Boundaries are Unicode's (UAX #29): a Chinese or Japanese text with no spaces is cut
//! between two of its characters, an Arabic one between its words.
//!
//! Each chunk keeps where it stands in the passage's text, in characters, so a result opens the
//! review at that place.

use unicode_segmentation::UnicodeSegmentation;

/// The length aimed at, in characters.
pub const TARGET: usize = 1200;
/// How much of a chunk the next one repeats.
pub const OVERLAP: f32 = 0.15;
/// Fewer letters than this and a chunk is not worth embedding: a page number, a stamp.
pub const MIN_LETTERS: usize = 8;
/// Raised whenever how a text is cut changes: every passage is then cut and embedded again.
pub const VERSION: u32 = 1;

/// One stretch of a passage's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Where it stands in the passage's text, in characters: `[start, end)`.
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// What cutting one text gave.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cut {
    pub chunks: Vec<Chunk>,
    /// Chunks left out for having almost no letters.
    pub skipped: u32,
}

/// The kinds of place a text may be cut at, strongest first.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Strength {
    Grapheme,
    Word,
    Sentence,
    Paragraph,
}

/// Every place the text may be cut, as a character offset and how good a place it is; the
/// start and the end included.
fn boundaries(text: &str) -> Vec<(usize, Strength)> {
    // Byte offsets to character offsets.
    let mut char_at = vec![0usize; text.len() + 1];
    let mut n = 0;
    for (b, _) in text.char_indices() {
        char_at[b] = n;
        n += 1;
    }
    char_at[text.len()] = n;
    let mut strength: Vec<Option<Strength>> = vec![None; n + 1];
    let mut mark = |byte: usize, s: Strength| {
        let c = char_at[byte];
        if strength[c].is_none_or(|old| old < s) {
            strength[c] = Some(s);
        }
    };
    for (b, _) in text.grapheme_indices(true) {
        mark(b, Strength::Grapheme);
    }
    for (b, _) in text.split_word_bound_indices() {
        mark(b, Strength::Word);
    }
    for (b, _) in text.split_sentence_bound_indices() {
        mark(b, Strength::Sentence);
    }
    // A line break ends a sentence (a speaker's line, a heading); a blank line, a paragraph.
    let bytes = text.as_bytes();
    for (b, _) in text.match_indices('\n') {
        let after = b + 1;
        mark(after, Strength::Sentence);
        let mut j = after;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r') {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'\n' {
            mark(j + 1, Strength::Paragraph);
        }
    }
    mark(0, Strength::Paragraph);
    mark(text.len(), Strength::Paragraph);
    strength
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| Some((i, s?)))
        .collect()
}

fn letters(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

/// Cuts `text` into chunks of about `target` characters, never more than `max`.
pub fn cut(text: &str, target: usize, max: usize) -> Cut {
    let size = target.min(max).max(1);
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let slice = |a: usize, b: usize| -> String { chars[a..b].iter().collect() };
    let mut out = Cut::default();
    let keep = |start: usize, end: usize, out: &mut Cut| {
        // White space at either end is left out of the chunk and of its place.
        let mut a = start;
        let mut b = end;
        while a < b && chars[a].is_whitespace() {
            a += 1;
        }
        while b > a && chars[b - 1].is_whitespace() {
            b -= 1;
        }
        if a == b {
            return;
        }
        let text = slice(a, b);
        if letters(&text) < MIN_LETTERS {
            out.skipped += 1;
        } else {
            out.chunks.push(Chunk {
                start: a,
                end: b,
                text,
            });
        }
    };
    if total <= size {
        keep(0, total, &mut out);
        return out;
    }
    let places = boundaries(text);
    let overlap = ((size as f32) * OVERLAP) as usize;
    let mut start = 0;
    loop {
        let limit = start + size;
        if limit >= total {
            keep(start, total, &mut out);
            break;
        }
        // The strongest place in the second half of the window, the last of its kind; a word
        // anywhere in it; a grapheme as a last resort.
        let window = &places[places.partition_point(|(at, _)| *at <= start)
            ..places.partition_point(|(at, _)| *at <= limit)];
        let half = start + size / 2;
        let end = [Strength::Paragraph, Strength::Sentence]
            .iter()
            .find_map(|s| {
                window
                    .iter()
                    .rev()
                    .find(|(at, k)| *at > half && k >= s)
                    .map(|(at, _)| *at)
            })
            .or_else(|| {
                window
                    .iter()
                    .rev()
                    .find(|(_, k)| *k >= Strength::Word)
                    .map(|(at, _)| *at)
            })
            .or_else(|| window.last().map(|(at, _)| *at))
            .unwrap_or(limit);
        keep(start, end, &mut out);
        // The next begins a little before: at a sentence where one starts in the overlap, else
        // at a word; always after this one's start, so the cutting moves on.
        let from = end.saturating_sub(overlap).max(start + 1);
        let next = [Strength::Sentence, Strength::Word, Strength::Grapheme]
            .iter()
            .find_map(|s| {
                places[places.partition_point(|(at, _)| *at < from)..]
                    .iter()
                    .take_while(|(at, _)| *at < end)
                    .find(|(_, k)| k >= s)
                    .map(|(at, _)| *at)
            })
            .unwrap_or(end);
        start = next;
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn lengths(cut: &Cut) -> Vec<usize> {
        cut.chunks.iter().map(|c| c.end - c.start).collect()
    }

    /// Each chunk's text is the text at its place.
    fn placed(text: &str, cut: &Cut) {
        let chars: Vec<char> = text.chars().collect();
        for c in &cut.chunks {
            assert_eq!(c.text, chars[c.start..c.end].iter().collect::<String>());
        }
    }

    #[test]
    fn a_short_piece_is_one_chunk_and_a_page_number_is_skipped() {
        let one = cut("  La lettre du cheikh, datée de 1932.  ", TARGET, 2000);
        assert_eq!(one.chunks.len(), 1);
        assert_eq!(one.chunks[0].start, 2);
        assert_eq!(one.chunks[0].text, "La lettre du cheikh, datée de 1932.");
        let page = cut("— 12 —", TARGET, 2000);
        assert!(page.chunks.is_empty());
        assert_eq!(page.skipped, 1);
        assert_eq!(cut("   ", TARGET, 2000), Cut::default());
    }

    #[test]
    fn a_long_text_is_cut_at_paragraphs_with_overlap_never_above_the_limit() {
        let paragraph = "Le congrès de l'Union culturelle musulmane s'ouvrit à Dakar. \
                         Les délégués venaient de tout le pays. "
            .repeat(6);
        let text = format!("{paragraph}\n\n{paragraph}\n\n{paragraph}\n\n{paragraph}");
        let c = cut(&text, 1200, 900);
        placed(&text, &c);
        assert!(c.chunks.len() >= 3);
        assert!(lengths(&c).iter().all(|n| *n <= 900));
        // Each chunk begins before the last one ended: they overlap.
        for pair in c.chunks.windows(2) {
            assert!(pair[1].start < pair[0].end, "{pair:?}");
            assert!(pair[1].start > pair[0].start);
        }
        // Cut at a sentence's end or a paragraph's: each chunk ends on a full stop.
        assert!(c.chunks.iter().all(|ch| ch.text.ends_with('.')));
        // The whole text is covered.
        assert_eq!(
            c.chunks.last().unwrap().end,
            text.trim_end().chars().count()
        );
    }

    #[test]
    fn chinese_without_spaces_and_arabic_are_cut_between_characters_or_words() {
        let chinese = "北京大学图书馆收藏了许多关于西非伊斯兰教的文献资料".repeat(20);
        let c = cut(&chinese, 100, 100);
        placed(&chinese, &c);
        assert!(c.chunks.len() > 4);
        assert!(lengths(&c).iter().all(|n| *n <= 100));

        let arabic =
            "انعقد مؤتمر الاتحاد الثقافي الإسلامي في داكار وحضره مندوبون من كل البلاد ".repeat(10);
        let c = cut(&arabic, 120, 120);
        placed(&arabic, &c);
        let words: Vec<&str> = arabic.split_whitespace().collect();
        for ch in &c.chunks {
            // Neither end inside a word.
            let first = ch.text.split_whitespace().next().unwrap();
            let last = ch.text.split_whitespace().last().unwrap();
            assert!(
                words.contains(&first) && words.contains(&last),
                "{}",
                ch.text
            );
        }
    }

    #[test]
    fn graphemes_are_never_split_even_in_a_text_without_any_break() {
        // One "word" of letters with combining accents, longer than the limit.
        let text = "e\u{301}".repeat(300);
        let c = cut(&text, 100, 101);
        placed(&text, &c);
        for ch in &c.chunks {
            assert!(!ch.text.starts_with('\u{301}'));
            assert!(ch.text.ends_with('\u{301}'));
            assert!(ch.end - ch.start <= 101);
        }
    }
}
