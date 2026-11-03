//! What a query looks for and where it stands in a passage: one literal phrase, every word
//! anywhere in the same text, or every word within a few words of the others.
//!
//! Each match is also given as one of its words and which of their matches it is in its piece,
//! so the review, opened on it, finds the same place with its own search for those words.

use std::collections::HashMap;
use std::rc::Rc;

use kalamu_core::{Passage, PieceRef};

use crate::fold::{MIN_QUERY, context, find, folded};
use crate::{FieldRef, Hit, Mark, MatchMode};

/// Marks that keep words together in a query that looks for each word: straight, curly and
/// French quotes.
const QUOTES: [char; 5] = ['"', '\u{201C}', '\u{201D}', '\u{AB}', '\u{BB}'];

/// One thing a query looks for: a word, or words that stand together.
pub(crate) struct Term {
    /// As the researcher wrote it: the review searches for it when a match is opened.
    pub typed: String,
    /// Folded, as it is matched.
    pub needle: Vec<char>,
}

pub(crate) struct Query {
    pub terms: Vec<Term>,
    /// Words too short to look for, left out of the query.
    pub ignored: Vec<String>,
}

impl Query {
    /// The terms as the index matches them.
    pub fn folded(&self) -> Vec<String> {
        self.terms
            .iter()
            .map(|t| t.needle.iter().collect())
            .collect()
    }
}

/// The terms of a query as `mode` reads it: for a phrase the whole query, otherwise each word,
/// words between quotes kept together and punctuation around a word set aside. A word that
/// folds to fewer than three letters is left out and named; one written twice is looked for
/// once.
pub(crate) fn parse(query: &str, mode: MatchMode) -> Query {
    let mut written: Vec<String> = Vec::new();
    if mode == MatchMode::Phrase {
        written.push(query.trim().to_string());
    } else {
        for (i, chunk) in query.split(QUOTES).enumerate() {
            if i % 2 == 1 {
                written.push(chunk.split_whitespace().collect::<Vec<_>>().join(" "));
            } else {
                written.extend(
                    chunk
                        .split_whitespace()
                        .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
                        .map(str::to_string),
                );
            }
        }
    }
    let mut out = Query {
        terms: Vec::new(),
        ignored: Vec::new(),
    };
    for typed in written.into_iter().filter(|w| !w.is_empty()) {
        let needle: Vec<char> = folded(&typed).chars().collect();
        if needle.len() < MIN_QUERY {
            if !out.ignored.contains(&typed) {
                out.ignored.push(typed);
            }
        } else if !out.terms.iter().any(|t| t.needle == needle) {
            out.terms.push(Term { typed, needle });
        }
    }
    out
}

/// A match before it is listed: the words around it are cut from its text only if it is.
#[derive(Clone)]
pub(crate) struct Found {
    pub hit: Hit,
    /// Where it stands in its passage, to list a passage's matches in reading order: its part,
    /// then its first character.
    pub order: (usize, usize),
    text: Rc<str>,
    span: (usize, usize),
    /// The terms' own places within `span`.
    marks: Vec<(usize, usize)>,
}

impl Found {
    pub fn listed(mut self) -> Hit {
        let (before, words, after) = context(&self.text, self.span);
        // Each term's place in the words shown, where a run of white space is one space.
        let start = self.span.0;
        let shown = |at: usize| -> u32 {
            let mut n = 0u32;
            let mut space = false;
            for c in self.text.chars().skip(start).take(at.saturating_sub(start)) {
                if !(c.is_whitespace() && space) {
                    n += 1;
                }
                space = c.is_whitespace();
            }
            n
        };
        self.hit.marks = self
            .marks
            .iter()
            .map(|&(a, b)| Mark {
                start: shown(a),
                end: shown(b),
            })
            .collect();
        self.hit.before = before;
        self.hit.words = words;
        self.hit.after = after;
        self.hit
    }
}

/// One place a term stands in a part of a passage.
struct Place {
    term: usize,
    /// Which match of its term it is in the piece, as the review counts them.
    nth: u32,
    span: (usize, usize),
    at_ms: Option<u64>,
}

/// For each character of `text`, the number of the word it is in, from 1; white space takes
/// the number of the word before it.
fn word_numbers(text: &str) -> Vec<u32> {
    let mut n = 0;
    let mut space = true;
    text.chars()
        .map(|c| {
            if !c.is_whitespace() && space {
                n += 1;
            }
            space = c.is_whitespace();
            n
        })
        .collect()
}

/// Where every term stands with at most `near` words between the first and the last: the
/// first such stretch to end, then the next after it, none overlapping. `places` are in
/// reading order.
fn clusters<'a>(text: &str, places: &'a [Place], terms: usize, near: u32) -> Vec<Vec<&'a Place>> {
    let words = word_numbers(text);
    let word = |at: usize| words.get(at).copied().unwrap_or_default();
    let mut latest: Vec<Option<&Place>> = vec![None; terms];
    let mut out = Vec::new();
    for p in places {
        latest[p.term] = Some(p);
        let chosen: Vec<&Place> = latest.iter().flatten().copied().collect();
        if chosen.len() < terms {
            continue;
        }
        let Some(first) = chosen.iter().min_by_key(|q| q.span.0) else {
            continue;
        };
        let between = word(p.span.0)
            .saturating_sub(word(first.span.1.saturating_sub(1)))
            .saturating_sub(1);
        if between <= near {
            let mut chosen = chosen;
            chosen.sort_by_key(|q| (q.span.0, q.term));
            out.push(chosen);
            latest = vec![None; terms];
        }
    }
    out
}

/// The matches of `query` in one passage, as `mode` finds them, within at most `near` words in
/// [`MatchMode::Near`]. A match never runs from one part of the passage into the next.
pub(crate) fn hits(
    passage: &Passage,
    query: &Query,
    mode: MatchMode,
    near: u32,
    labels: &HashMap<String, String>,
) -> Vec<Found> {
    let start_ms = match passage.piece {
        PieceRef::Segment { start_ms, .. } => Some(start_ms),
        _ => None,
    };
    let mut out = Vec::new();
    // Every term's matches are counted through the whole piece, those in parts that hold not
    // every term too, as the review's search counts them.
    let mut nth = vec![0u32; query.terms.len()];
    for (p, part) in passage.parts.iter().enumerate() {
        let mut places = Vec::new();
        let mut every = true;
        for (t, term) in query.terms.iter().enumerate() {
            let found = find(&part.text, &term.needle);
            every &= !found.is_empty();
            // A recording's match is tied to its line when the lines hold as many matches as
            // the text: the n-th match is then in the line holding the n-th match.
            let lines: Vec<u64> = part
                .lines
                .iter()
                .flat_map(|(at, text)| find(text, &term.needle).into_iter().map(move |_| *at))
                .collect();
            let timed = start_ms.is_some() && !lines.is_empty() && lines.len() == found.len();
            for (i, span) in found.into_iter().enumerate() {
                places.push(Place {
                    term: t,
                    nth: nth[t],
                    span,
                    at_ms: timed.then(|| start_ms.unwrap_or_default() + lines[i]),
                });
                nth[t] += 1;
            }
        }
        if !every {
            continue;
        }
        places.sort_by_key(|q| (q.span.0, q.term));
        let text: Rc<str> = Rc::from(part.text.as_str());
        let found = |first: &Place, span: (usize, usize), marks: Vec<(usize, usize)>| Found {
            hit: Hit {
                index: passage.key.index,
                piece: passage.piece.clone(),
                term: query.terms[first.term].typed.clone(),
                nth: first.nth,
                field: part.field.as_ref().map(|key| FieldRef {
                    key: key.clone(),
                    label: labels.get(key).cloned().unwrap_or_else(|| key.clone()),
                }),
                entry: part.entry.clone(),
                entry_number: part.entry_number,
                reason: part.reason,
                before: String::new(),
                words: String::new(),
                after: String::new(),
                marks: Vec::new(),
                at_ms: first.at_ms,
                reviewed: passage.reviewed,
                corrected: passage.corrected,
            },
            order: (p, span.0),
            text: text.clone(),
            span,
            marks,
        };
        match mode {
            MatchMode::Phrase | MatchMode::AllWords => {
                out.extend(places.iter().map(|q| found(q, q.span, vec![q.span])));
            }
            MatchMode::Near => {
                for cluster in clusters(&part.text, &places, query.terms.len(), near) {
                    let (Some(first), Some(end)) =
                        (cluster.first(), cluster.iter().map(|q| q.span.1).max())
                    else {
                        continue;
                    };
                    let marks = cluster.iter().map(|q| q.span).collect();
                    out.push(found(first, (first.span.0, end), marks));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use kalamu_core::{JobId, PassageKey, PassagePart, SetupId, SourceId};

    use super::*;

    fn passage(texts: &[&str]) -> Passage {
        Passage {
            key: PassageKey {
                job_id: JobId::new(),
                setup_id: SetupId::new(),
                source_id: SourceId::new(),
                index: 0,
            },
            stamp: String::new(),
            piece: PieceRef::page(1),
            reviewed: false,
            corrected: false,
            parts: texts
                .iter()
                .map(|t| PassagePart {
                    field: None,
                    entry: None,
                    entry_number: None,
                    reason: false,
                    text: (*t).into(),
                    lines: Vec::new(),
                })
                .collect(),
        }
    }

    /// Each match listed as `term#nth: words [marked]`.
    fn shown(texts: &[&str], query: &str, mode: MatchMode, near: u32) -> Vec<String> {
        let q = parse(query, mode);
        let mut found = hits(&passage(texts), &q, mode, near, &HashMap::new());
        found.sort_by_key(|f| f.order);
        found
            .into_iter()
            .map(Found::listed)
            .map(|h| {
                let chars: Vec<char> = h.words.chars().collect();
                let marked: Vec<String> = h
                    .marks
                    .iter()
                    .map(|m| chars[m.start as usize..m.end as usize].iter().collect())
                    .collect();
                format!("{}#{}: {} [{}]", h.term, h.nth, h.words, marked.join("|"))
            })
            .collect()
    }

    #[test]
    fn words_are_read_as_the_mode_asks() {
        let q = parse("  la mission catholique ", MatchMode::Phrase);
        assert_eq!(q.terms.len(), 1);
        assert_eq!(q.terms[0].typed, "la mission catholique");
        assert!(q.ignored.is_empty());

        let q = parse(
            "« mission catholique » Koupéla, la de UCM ucm",
            MatchMode::AllWords,
        );
        let typed: Vec<&str> = q.terms.iter().map(|t| t.typed.as_str()).collect();
        assert_eq!(typed, ["mission catholique", "Koupéla", "UCM"]);
        assert_eq!(q.ignored, ["la", "de"]);
        assert_eq!(q.folded(), ["mission catholique", "koupela", "ucm"]);
        // Quotes left open keep the rest together.
        let q = parse("\"union culturelle musulmane", MatchMode::Near);
        assert_eq!(q.terms[0].typed, "union culturelle musulmane");
    }

    #[test]
    fn every_word_anywhere_in_a_part_each_place_its_own_match() {
        let texts = [
            "La mission de Koupéla. Plus loin, la mission encore.",
            "Koupéla seule",
            "Mission et KOUPELA",
        ];
        assert_eq!(
            shown(&texts, "koupela mission", MatchMode::AllWords, 0),
            [
                "mission#0: mission [mission]",
                "koupela#0: Koupéla [Koupéla]",
                "mission#1: mission [mission]",
                // The second part holds one word only, but its match is counted.
                "mission#2: Mission [Mission]",
                "koupela#2: KOUPELA [KOUPELA]",
            ]
        );
    }

    #[test]
    fn words_near_each_other_within_the_distance_asked() {
        let text = "L'imam Ouédraogo a parlé. Bien plus tard, dans un autre quartier de la \
                    ville, le grand imam est venu, et Ouédraogo aussi.";
        // Two words apart at most: the first imam only.
        assert_eq!(
            shown(&[text], "imam ouedraogo", MatchMode::Near, 2),
            ["imam#0: imam Ouédraogo [imam|Ouédraogo]"]
        );
        // Further apart, the second pair too, each word marked and the words between not.
        assert_eq!(
            shown(&[text], "imam ouedraogo", MatchMode::Near, 4),
            [
                "imam#0: imam Ouédraogo [imam|Ouédraogo]",
                "imam#1: imam est venu, et Ouédraogo [imam|Ouédraogo]",
            ]
        );
        // In either order.
        assert_eq!(
            shown(
                &["Ouédraogo, dit l'imam"],
                "imam ouedraogo",
                MatchMode::Near,
                2
            ),
            ["ouedraogo#0: Ouédraogo, dit l'imam [Ouédraogo|imam]"]
        );
    }

    #[test]
    fn near_marks_follow_the_words_shown_across_line_breaks() {
        assert_eq!(
            shown(
                &["union\n\n  des   musulmans"],
                "union musulmans",
                MatchMode::Near,
                1
            ),
            ["union#0: union des musulmans [union|musulmans]"]
        );
    }

    #[test]
    fn a_phrase_is_one_term_marked_whole() {
        assert_eq!(
            shown(&["UCM, ucm et Ucm"], "ucm", MatchMode::Phrase, 0),
            ["ucm#0: UCM [UCM]", "ucm#1: ucm [ucm]", "ucm#2: Ucm [Ucm]"]
        );
    }
}
