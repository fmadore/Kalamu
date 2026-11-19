//! Finding a model's quotation in the text it quotes, forgiving only what a model changes
//! without meaning to when it copies words, and nothing that changes the words themselves.
//!
//! Forgiven:
//!
//! - **White space**: a run of spaces, tabs, line breaks or no-break spaces (U+00A0, U+202F) on
//!   either side matches any run on the other.
//! - **The kind of apostrophe** (`'`, `’`, `‘`, `ʼ`) and **of quotation mark** (`"`, `“`, `”`,
//!   `„`, `«`, `»`, `‹`, `›`), with the spaces French typography puts inside guillemets (or a
//!   model leaves out around a mark): `« oui »` matches `"oui"`.
//! - **An ellipsis character** for three dots, either way.
//! - **What a model puts around a quotation**: an ellipsis or quotation marks before or after
//!   it, and a final punctuation mark the text does not have there.
//!
//! Nothing else: letters keep their case and their accents, no word may be missing, added or
//! changed, and words left out in the middle (`[…]`) are not found. What is kept is always the
//! text's own characters between the two ends found, never the model's copy.

use super::{Anchor, SnapshotPart};

/// A text reduced to what is compared, each character with the span of the original
/// characters (counted in `char`s) it stands for.
struct Folded {
    chars: Vec<char>,
    spans: Vec<(u32, u32)>,
}

fn is_quote_mark(c: char) -> bool {
    matches!(
        c,
        '"' | '“' | '”' | '„' | '‟' | '«' | '»' | '‹' | '›' | '〝' | '〞' | '＂'
    )
}

fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '’' | '‘' | 'ʼ' | '‛' | '＇')
}

/// What one character is compared as.
fn fold_char(c: char) -> char {
    if is_quote_mark(c) {
        '"'
    } else if is_apostrophe(c) {
        '\''
    } else {
        c
    }
}

/// The text folded: white space runs made one space (none beside a quotation mark, nor at
/// either end), every quotation mark and apostrophe one of each, an ellipsis three dots.
fn fold(text: &str) -> Folded {
    let mut chars = Vec::new();
    let mut spans = Vec::new();
    // A run of white space not yet written: where it started and ended.
    let mut gap: Option<(u32, u32)> = None;
    for (i, c) in text.chars().enumerate() {
        let i = i as u32;
        if c.is_whitespace() {
            gap = Some(gap.map_or((i, i + 1), |(from, _)| (from, i + 1)));
            continue;
        }
        let folded = fold_char(c);
        if let Some(span) = gap.take() {
            let after_mark = chars.last() == Some(&'"');
            if !chars.is_empty() && !after_mark && folded != '"' {
                chars.push(' ');
                spans.push(span);
            }
        }
        if c == '…' {
            for _ in 0..3 {
                chars.push('.');
                spans.push((i, i + 1));
            }
        } else {
            chars.push(folded);
            spans.push((i, i + 1));
        }
    }
    Folded { chars, spans }
}

/// What a model puts before or after a quotation: an ellipsis (also in brackets), white space,
/// and with `marks` quotation marks.
fn strip_wrapping(quote: &str, marks: bool) -> &str {
    const ELLIPSES: [&str; 6] = ["[...]", "[…]", "(...)", "(…)", "...", "…"];
    let mut s = quote.trim();
    loop {
        let before = s.len();
        for e in ELLIPSES {
            s = s.strip_prefix(e).unwrap_or(s).trim_start();
            s = s.strip_suffix(e).unwrap_or(s).trim_end();
        }
        if !marks {
            if s.len() == before {
                return s;
            }
            continue;
        }
        s = s.trim_start_matches(is_quote_mark).trim_start();
        s = s.trim_end_matches(is_quote_mark).trim_end();
        // Single marks only when they wrap the whole quotation: an apostrophe may begin or end
        // a word.
        if let (Some(first), Some(last)) = (s.chars().next(), s.chars().last())
            && s.chars().count() > 1
            && is_apostrophe(first)
            && is_apostrophe(last)
        {
            s = &s[first.len_utf8()..s.len() - last.len_utf8()];
            s = s.trim();
        }
        if s.len() == before {
            return s;
        }
    }
}

/// A punctuation mark that may end a quotation, which the text need not have there.
fn is_final_punctuation(c: char) -> bool {
    matches!(
        c,
        '.' | ',' | ';' | ':' | '!' | '?' | '،' | '؛' | '؟' | '۔' | '。'
    )
}

/// The quotation as it is looked for, and the final punctuation mark it ended with, if any.
fn needle(quote: &str, marks: bool) -> (Vec<char>, Option<char>) {
    let stripped = strip_wrapping(quote, marks);
    let mut chars = fold(stripped).chars;
    let last = chars.last().copied().filter(|&c| is_final_punctuation(c));
    // Three dots at the end are an ellipsis, already stripped; one mark is the final one.
    if last.is_some() {
        chars.pop();
        while chars.last() == Some(&' ') {
            chars.pop();
        }
    }
    (chars, last)
}

/// Every place `quote` stands in `text`, as character spans of `text`, in order, the forgiving
/// rules above applied. Overlapping repeats are all given. The quotation is looked for first
/// with the quotation marks it begins or ends with, which may be the text's own (`« on ne
/// savait pas » est devenu…`), and only then without them, as a model's wrapping.
pub fn find(text: &str, quote: &str) -> Vec<(u32, u32)> {
    let hay = fold(text);
    let with_marks = find_folded(text, &hay, needle(quote, false));
    if !with_marks.is_empty() {
        return with_marks;
    }
    find_folded(text, &hay, needle(quote, true))
}

fn find_folded(
    text: &str,
    hay: &Folded,
    (needle, punctuation): (Vec<char>, Option<char>),
) -> Vec<(u32, u32)> {
    // A quotation of nothing, or of punctuation alone, quotes nothing.
    if !needle.iter().any(|c| c.is_alphanumeric()) {
        return Vec::new();
    }
    let text_chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    if hay.chars.len() < needle.len() {
        return out;
    }
    for start in 0..=hay.chars.len() - needle.len() {
        if hay.chars[start..start + needle.len()] != needle[..] {
            continue;
        }
        let from = hay.spans[start].0;
        let mut to = hay.spans[start + needle.len() - 1].1;
        // The text's own final mark is kept when the model gave the same one, after the
        // no-break space French puts before some.
        if let Some(mark) = punctuation {
            let mut at = to as usize;
            while text_chars
                .get(at)
                .is_some_and(|c| c.is_whitespace() && *c != '\n')
            {
                at += 1;
            }
            if text_chars.get(at) == Some(&mark) {
                to = at as u32 + 1;
            }
        }
        out.push((from, to));
    }
    out
}

/// Every place a quotation stands in a snapshot, each inside one piece, in order: [`find`]
/// over the whole text, as [`super::locate`] does with exact matching.
pub fn locate(text: &str, parts: &[SnapshotPart], quote: &str) -> Vec<Anchor> {
    find(text, quote)
        .into_iter()
        .filter_map(|(start, end)| {
            let part = parts
                .iter()
                .position(|p| p.start <= start && end <= p.end)?;
            Some(Anchor {
                part: part as u32,
                start,
                end,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
