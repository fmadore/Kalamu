//! Matching as the review's search matches (`src/lib/search.ts`): capitals, accents and Arabic
//! vowel marks set aside, any run of spaces or line breaks matching one space. The text shown
//! is never changed: each folded character is tied back to the stretch of the text it came
//! from, so a match is given in the text's own offsets, counted in characters as anchors are.
//!
//! A term of three characters or more is found anywhere, inside words too. One of two is
//! found only as a word on its own, with no letter or digit just before or after it, and only
//! when both are letters or digits of a script written with spaces between words: inside
//! longer words, two letters would be found on nearly every page. The index matches three
//! characters at a time, so it holds the text with every mark that is not a letter, a digit
//! or a space set between spaces, and a space at either end ([`index_text`]): a word of two
//! letters then always stands between two spaces there, and is looked for as ` ab `.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// The fewest characters a term may fold to: two, a word on its own; three or more are found
/// anywhere.
pub const MIN_QUERY: usize = 2;

/// Scripts written without spaces between words, as ranges of code points: two of their
/// letters are not a word on its own, and the index cannot find them. Thai and Lao, Myanmar,
/// Khmer, the Japanese kana and the Chinese characters.
pub const UNSPACED: [(u32, u32); 10] = [
    (0x0E00, 0x0EFF),
    (0x1000, 0x109F),
    (0x1780, 0x17FF),
    (0x3040, 0x30FF),
    (0x31F0, 0x31FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xF900, 0xFAFF),
    (0xFF66, 0xFF9F),
    (0x20000, 0x323AF),
];

/// A text folded for matching, each folded character tied to the characters `[from, to)` of
/// the text it came from.
pub(crate) struct Folded {
    pub text: Vec<char>,
    from: Vec<usize>,
    to: Vec<usize>,
}

pub(crate) fn fold(text: &str) -> Folded {
    let mut out = Folded {
        text: Vec::with_capacity(text.len()),
        from: Vec::with_capacity(text.len()),
        to: Vec::with_capacity(text.len()),
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
        if ch.is_ascii() {
            // Most of a Latin text: no accent to set aside, so nothing to decompose.
            out.text.push(ch.to_ascii_lowercase());
            out.from.push(i);
            out.to.push(next);
            continue;
        }
        let mut any = false;
        for c in std::iter::once(ch)
            .nfd()
            .filter(|c| !is_combining_mark(*c))
            .flat_map(char::to_lowercase)
        {
            any = true;
            out.text.push(c);
            out.from.push(i);
            out.to.push(next);
        }
        // A mark on its own (a vowel mark, an accent typed apart) belongs to the letter before.
        if !any && let Some(to) = out.to.last_mut().filter(|to| **to == i) {
            *to = next;
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

/// A letter or digit of a script written with spaces between words.
fn spaced_letter(c: char) -> bool {
    let code = c as u32;
    c.is_alphanumeric() && !UNSPACED.iter().any(|&(a, b)| (a..=b).contains(&code))
}

/// Whether a folded term is a word of two letters, found only on its own.
pub(crate) fn is_word(needle: &[char]) -> bool {
    needle.len() == 2 && needle.iter().all(|c| spaced_letter(*c))
}

/// Whether a folded term can be looked for: three characters or more, or a word of two letters.
pub(crate) fn long_enough(needle: &[char]) -> bool {
    needle.len() >= 3 || is_word(needle)
}

/// Whether a query is long enough to search the project for.
pub fn searchable(query: &str) -> bool {
    long_enough(&folded(query).chars().collect::<Vec<_>>())
}

/// Set apart in the index: neither a letter, a digit nor a space.
fn apart(c: char) -> bool {
    c != ' ' && !c.is_alphanumeric()
}

/// Every character set apart between spaces. The same for each character wherever it stands,
/// so a term is in a text exactly when its spaced form is in the text's.
fn spaced(text: impl Iterator<Item = char>, out: &mut String) {
    for c in text {
        if apart(c) {
            out.push(' ');
            out.push(c);
            out.push(' ');
        } else {
            out.push(c);
        }
    }
}

/// A folded text as the index holds it: spaced, with a space at either end.
pub(crate) fn index_text(folded: &str) -> String {
    let mut out = String::with_capacity(folded.len() + folded.len() / 8 + 2);
    out.push(' ');
    spaced(folded.chars(), &mut out);
    out.push(' ');
    out
}

/// A folded term as the index is asked for it: spaced, and a word of two letters between
/// spaces.
pub(crate) fn index_term(needle: &[char]) -> String {
    let mut out = String::new();
    if is_word(needle) {
        out.push(' ');
        out.extend(needle);
        out.push(' ');
    } else {
        spaced(needle.iter().copied(), &mut out);
    }
    out
}

/// One place a folded term is found in a text.
pub(crate) struct Spot {
    /// `[start, end)` in characters of the text.
    pub span: (usize, usize),
    /// No letter or digit stands just before or after it.
    pub alone: bool,
}

/// Every place the folded `needle` is found in the folded `hay`, in order, none overlapping,
/// as the review's search finds them.
pub(crate) fn spots(hay: &Folded, needle: &[char]) -> Vec<Spot> {
    let Some(&first) = needle.first() else {
        return Vec::new();
    };
    let joined = |at: Option<&char>| at.is_some_and(|c| c.is_alphanumeric());
    let n = needle.len();
    let mut out = Vec::new();
    let mut at = 0;
    while at + n <= hay.text.len() {
        if hay.text[at] == first && hay.text[at..at + n] == *needle {
            out.push(Spot {
                span: (hay.from[at], hay.to[at + n - 1]),
                alone: !joined(at.checked_sub(1).and_then(|b| hay.text.get(b)))
                    && !joined(hay.text.get(at + n)),
            });
            at += n;
        } else {
            at += 1;
        }
    }
    out
}

/// Every place the folded `needle` is found in `text`, in order, none overlapping, as
/// `[start, end)` in characters of `text`.
#[cfg(test)]
pub(crate) fn find(text: &str, needle: &[char]) -> Vec<(usize, usize)> {
    spots(&fold(text), needle)
        .into_iter()
        .map(|s| s.span)
        .collect()
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

    /// Worked cases of what a query matches, written for the interface, whose review search
    /// (`src/lib/search.ts`) must find the same words in the same order: project search opens
    /// the review at the n-th match of a piece, and both sides count it. `cargo test` keeps the
    /// file current, as it does `searchRules.json`; the interface's tests read it.
    #[test]
    fn export_match_cases_for_the_interface() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            ("La Réfection du minaret. REFECTION faite.", "refection"),
            ("Le père, le Père et le PÈRE.", "pere"),
            ("ٱلْحَمْدُ لِلَّهِ، الحمد لله", "الحمد"),
            ("Ḥajj, hajj et Hajj", "hajj"),
            ("cercle  de\nDakar, cercle de Dakar", "cercle de dakar"),
            ("aaaaa", "aaa"),
            ("cœur, Cœur, coeur", "cœur"),
            ("Straße et STRASSE", "straße"),
            ("Côte d’Ivoire, Côte d'Ivoire", "cote d"),
            ("e\u{301}cole et école", "ecole"),
            ("Saint-Louis du Sénégal", "senegal"),
        ];
        let cases: Vec<serde_json::Value> = cases
            .iter()
            .map(|(text, query)| {
                serde_json::json!({ "text": text, "query": query, "found": found(text, query) })
            })
            .collect();
        kalamu_core::test_support::write_interface_json("src/lib/matchCases.json", &cases)
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
    fn two_letters_of_a_spaced_script_at_least() {
        assert!(!searchable(" é "));
        assert!(searchable("ab"));
        assert!(searchable(" Ba "));
        assert!(searchable("في"));
        assert!(searchable("UCM"));
        // Not letters, or letters of a script written without spaces: too short.
        assert!(!searchable("l'"));
        assert!(!searchable("北京"));
        assert!(!searchable("ไท"));
        assert!(searchable("北京市"));
    }

    #[test]
    fn a_word_of_two_letters_stands_alone() {
        let alone = |text: &str, query: &str| -> Vec<bool> {
            let needle: Vec<char> = folded(query).chars().collect();
            spots(&fold(text), &needle)
                .into_iter()
                .map(|s| s.alone)
                .collect()
        };
        // Inside Bamako and Kaba it is found, as the review counts it, but not alone.
        assert_eq!(
            alone("Bamako, Amadou Bâ (Kaba) et BA.", "ba"),
            [false, true, false, true]
        );
        assert_eq!(alone("ba", "ba"), [true]);
        assert_eq!(alone("l'or et d’or, ORS", "or"), [true, true, false]);
        assert_eq!(alone("فيه في", "في"), [false, true]);
        // A digit joins as a letter does.
        assert_eq!(alone("Ba2 ba", "ba"), [false, true]);
    }

    /// A term is in a text exactly when its index form is in the text's: what the index finds
    /// is what the search places.
    #[test]
    fn the_index_holds_two_letters_between_spaces() {
        let index = |text: &str| index_text(&folded(text));
        let term = |query: &str| index_term(&folded(query).chars().collect::<Vec<_>>());
        assert_eq!(index("(Bâ), l'Or"), "  ( ba )  ,  l ' or ");
        assert_eq!(term("Ba"), " ba ");
        assert_eq!(term("Côte d'Ivoire"), "cote d ' ivoire");
        for (text, query, found) in [
            ("Amadou (Bâ).", "ba", true),
            ("Bâ", "ba", true),
            ("Bamako", "ba", false),
            ("Kaba", "ba", false),
            ("Côte d'Ivoire", "cote d'ivoire", true),
            ("Côte d'Ivoire", "e d'i", true),
            ("Côte d'Ivoire", "cote d ivoire", false),
            ("« Union », 1957", "union »,", true),
        ] {
            assert_eq!(
                index(text).contains(&term(query)),
                found,
                "{query} in {text}"
            );
        }
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
