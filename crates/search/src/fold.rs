//! Matching as the review's search matches (`src/lib/search.ts`): capitals, accents and Arabic
//! vowel marks set aside, any run of spaces or line breaks matching one space. The text shown
//! is never changed: each folded character is tied back to the stretch of the text it came
//! from, so a match is given in the text's own offsets, counted in characters as anchors are.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// The fewest characters a query may fold to: the index matches three at a time.
pub const MIN_QUERY: usize = 3;

/// A text folded for matching, each folded character tied to the characters `[from, to)` of
/// the text it came from.
pub(crate) struct Folded {
    pub text: Vec<char>,
    from: Vec<usize>,
    to: Vec<usize>,
}

pub(crate) fn fold(text: &str) -> Folded {
    let mut out = Folded {
        text: Vec::new(),
        from: Vec::new(),
        to: Vec::new(),
    };
    for (i, ch) in text.chars().enumerate() {
        let next = i + 1;
        if ch.is_whitespace() {
            // A run of white space is one space.
            if out.text.last() == Some(&' ') {
                if let Some(to) = out.to.last_mut() {
                    *to = next;
                }
            } else {
                out.text.push(' ');
                out.from.push(i);
                out.to.push(next);
            }
            continue;
        }
        let letters: Vec<char> = std::iter::once(ch)
            .nfd()
            .filter(|c| !is_combining_mark(*c))
            .flat_map(char::to_lowercase)
            .collect();
        // A mark on its own (a vowel mark, an accent typed apart) belongs to the letter before.
        if letters.is_empty()
            && let Some(to) = out.to.last_mut().filter(|to| **to == i)
        {
            *to = next;
        }
        for c in letters {
            out.text.push(c);
            out.from.push(i);
            out.to.push(next);
        }
    }
    out
}

/// A text as the index holds it and a query as it is matched: folded, without space at either
/// end.
pub fn folded(text: &str) -> String {
    fold(text)
        .text
        .into_iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// Whether a query is long enough to search the project for.
pub fn searchable(query: &str) -> bool {
    folded(query).chars().count() >= MIN_QUERY
}

/// Every place the folded `needle` is found in `text`, in order, none overlapping, as
/// `[start, end)` in characters of `text`.
pub(crate) fn find(text: &str, needle: &[char]) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    let hay = fold(text);
    let mut out = Vec::new();
    let mut at = 0;
    while at + needle.len() <= hay.text.len() {
        if hay.text[at..at + needle.len()] == *needle {
            out.push((hay.from[at], hay.to[at + needle.len() - 1]));
            at += needle.len();
        } else {
            at += 1;
        }
    }
    out
}

/// A match with some of the words around it, on one line, cut at word boundaries: the words
/// before, the words found and the words after.
pub(crate) fn context(text: &str, (start, end): (usize, usize)) -> (String, String, String) {
    const AROUND: usize = 48;
    let chars: Vec<char> = text.chars().collect();
    let slice = |a: usize, b: usize| -> String {
        chars[a..b]
            .iter()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut from = start.saturating_sub(AROUND);
    if from > 0
        && let Some(space) = (from..start).find(|&i| chars[i].is_whitespace())
    {
        from = space + 1;
    }
    let mut to = (end + AROUND).min(chars.len());
    if to < chars.len()
        && let Some(space) = (end..to).rev().find(|&i| chars[i].is_whitespace())
    {
        to = space;
    }
    let lead = if from > 0 { "…" } else { "" };
    let tail = if to < chars.len() { "…" } else { "" };
    // A space next to the words found is kept, so the snippet reads as one line.
    let before = slice(from, start);
    let gap_before = start > from && chars[start - 1].is_whitespace();
    let after = slice(end, to);
    let gap_after = to > end && chars[end].is_whitespace();
    (
        format!(
            "{lead}{before}{}",
            if gap_before && !before.is_empty() {
                " "
            } else {
                ""
            }
        ),
        slice(start, end),
        format!(
            "{}{after}{tail}",
            if gap_after && !after.is_empty() {
                " "
            } else {
                ""
            }
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(text: &str, query: &str) -> Vec<String> {
        let needle: Vec<char> = folded(query).chars().collect();
        let chars: Vec<char> = text.chars().collect();
        find(text, &needle)
            .into_iter()
            .map(|(a, b)| chars[a..b].iter().collect())
            .collect()
    }

    #[test]
    fn capitals_and_accents_are_set_aside_but_the_text_is_kept() {
        assert_eq!(
            found("Le Père Éric à Ouagadougou", "pere eric"),
            vec!["Père Éric"]
        );
        assert_eq!(
            found("OUÉDRAOGO, Ouedraogo", "ouédraogo"),
            vec!["OUÉDRAOGO", "Ouedraogo"]
        );
        // An accent typed apart belongs to its letter.
        assert_eq!(found("Pe\u{0300}re", "pere"), vec!["Pe\u{0300}re"]);
    }

    #[test]
    fn arabic_vowel_marks_are_set_aside() {
        let marked = "قَالَ الشَّيْخُ فِي المَسْجِدِ";
        assert_eq!(found(marked, "الشيخ"), vec!["الشَّيْخُ"]);
        // Inside a word, as the article is joined to it.
        assert_eq!(found("بالمسجد", "مسجد"), vec!["مسجد"]);
    }

    #[test]
    fn a_run_of_spaces_and_line_breaks_is_one_space() {
        assert_eq!(
            found("la mission\n  catholique", "Mission Catholique"),
            vec!["mission\n  catholique"]
        );
        assert_eq!(folded("  Union   culturelle\n"), "union culturelle");
    }

    #[test]
    fn repeated_words_are_each_found_once() {
        assert_eq!(found("UCM, ucm et Ucm", "ucm").len(), 3);
        assert_eq!(found("aaaa", "aa").len(), 2);
    }

    #[test]
    fn three_letters_at_least() {
        assert!(!searchable(" é "));
        assert!(!searchable("ab"));
        assert!(searchable("UCM"));
    }

    #[test]
    fn context_cuts_at_words_and_keeps_one_line() {
        let text = "Au début de l'année, la mission catholique de Koupéla a reçu\nune lettre du \
                    cercle, qui annonçait la visite prochaine du gouverneur général.";
        let needle: Vec<char> = folded("mission catholique").chars().collect();
        let m = find(text, &needle)[0];
        let (before, words, after) = context(text, m);
        assert_eq!(words, "mission catholique");
        assert_eq!(before, "Au début de l'année, la ");
        assert!(after.starts_with(" de Koupéla a reçu une lettre"));
        assert!(after.ends_with('…'));
    }
}
