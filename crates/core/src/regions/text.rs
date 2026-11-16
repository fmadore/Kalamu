//! A page's text as words, to find where a region's words stand in it.
//!
//! Words are what whitespace separates, HTML tags counting as whitespace (Mistral OCR writes
//! tables in HTML). Each is compared in a normal form that leaves out what Markdown adds around
//! words and the engines' block texts leave out ([`crate::plain_words`]): emphasis marks
//! (`*`, `_`, `` ` ``), heading, quotation and table marks at either end (`#`, `>`, `|`), and words
//! made only of such marks or rules (`---`, `|:--|`). The rest is compared exactly, after Unicode
//! composition (NFC), so an accent typed as a separate mark matches the accented letter, and
//! Arabic or any other script compares letter for letter. Capitals and punctuation count.

use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;

/// A text's words in their normal form, each with the stretch of the text it stands in.
pub(super) struct Words {
    pub(super) normal: Vec<String>,
    /// UTF-16 offsets of each word, the marks left out of its normal form at either end
    /// excluded.
    spans: Vec<(u32, u32)>,
}

/// Marks Markdown sets around words, left out of a word wherever they are.
fn dropped(c: char) -> bool {
    matches!(c, '*' | '_' | '`')
}

/// Marks left out at either end of a word.
fn trimmed(c: char) -> bool {
    dropped(c) || matches!(c, '#' | '>' | '|')
}

/// A word made of these alone is markup: a rule, a table's separator, a bullet.
fn markup(c: char) -> bool {
    trimmed(c) || matches!(c, '-' | '=' | ':' | '+')
}

impl Words {
    pub(super) fn of(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut words = Words {
            normal: Vec::new(),
            spans: Vec::new(),
        };
        // The current word's characters, each with its UTF-16 offset.
        let mut word: Vec<(char, u32)> = Vec::new();
        let mut at: u32 = 0;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '<'
                && let Some(end) = tag_ends(&chars, i)
            {
                words.push(&word);
                word.clear();
                for &skipped in &chars[i..=end] {
                    at += skipped.len_utf16() as u32;
                }
                i = end + 1;
                continue;
            }
            if c.is_whitespace() {
                words.push(&word);
                word.clear();
            } else {
                word.push((c, at));
            }
            at += c.len_utf16() as u32;
            i += 1;
        }
        words.push(&word);
        words
    }

    fn push(&mut self, word: &[(char, u32)]) {
        if word.iter().all(|&(c, _)| markup(c)) {
            return;
        }
        let first = word.iter().position(|&(c, _)| !trimmed(c));
        let last = word.iter().rposition(|&(c, _)| !trimmed(c));
        let (Some(first), Some(last)) = (first, last) else {
            return;
        };
        let kept = &word[first..=last];
        let normal: String = kept
            .iter()
            .map(|&(c, _)| c)
            .filter(|&c| !dropped(c))
            .nfc()
            .collect();
        if normal.is_empty() {
            return;
        }
        let start = kept[0].1;
        let (c, offset) = kept[kept.len() - 1];
        self.normal.push(normal);
        self.spans.push((start, offset + c.len_utf16() as u32));
    }

    /// The stretch of the text from word `from` to the word before `to`.
    pub(super) fn span(&self, from: usize, to: usize) -> Option<super::TextSpan> {
        let start = self.spans.get(from)?.0;
        let end = self.spans.get(to.checked_sub(1)?)?.1;
        Some(super::TextSpan { start, end })
    }

    /// Where each word stands, to find runs of words quickly.
    pub(super) fn index(&self) -> Index<'_> {
        let mut at: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, w) in self.normal.iter().enumerate() {
            at.entry(w.as_str()).or_default().push(i);
        }
        Index { words: self, at }
    }
}

/// The index of the `>` closing an HTML tag opened at `start`: a `<` followed by a letter or
/// `/`, closed on the same line. A `<` in prose (`a < b`) is not a tag.
fn tag_ends(chars: &[char], start: usize) -> Option<usize> {
    let next = chars.get(start + 1)?;
    if !(next.is_ascii_alphabetic() || *next == '/') {
        return None;
    }
    chars[start + 1..]
        .iter()
        .take_while(|c| **c != '\n' && **c != '<')
        .position(|c| *c == '>')
        .map(|p| start + 1 + p)
}

pub(super) struct Index<'a> {
    words: &'a Words,
    at: HashMap<&'a str, Vec<usize>>,
}

impl Index<'_> {
    /// Where the run of words `run` begins in the text, at most `most` places.
    pub(super) fn places(&self, run: &[String], most: usize) -> Vec<usize> {
        let Some(first) = run.first() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for &i in self.at.get(first.as_str()).into_iter().flatten() {
            if self.words.normal.get(i..i + run.len()) == Some(run) {
                out.push(i);
                if out.len() >= most {
                    break;
                }
            }
        }
        out
    }
}
