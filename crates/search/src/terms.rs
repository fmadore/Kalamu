//! What a query looks for and where it stands in a passage: one literal phrase, every word
//! anywhere in the same text, or every word within a few words of the others; and in each,
//! words or phrases given as alternatives, any of which will do.
//!
//! Each match is also given as one of its words and which of their matches it is in its piece,
//! so the review, opened on it, finds the same place with its own search for those words.

use std::collections::HashMap;
use std::rc::Rc;

use kalamu_core::{Passage, PieceRef};

use crate::fold::{context, fold, folded, index_term, is_word, long_enough, spots};
use crate::{FieldRef, Hit, Mark, MatchMode};

/// Marks that keep words together in a query that looks for each word: straight, curly and
/// French quotes.
const QUOTES: [char; 5] = ['"', '\u{201C}', '\u{201D}', '\u{AB}', '\u{BB}'];

/// Written in capitals between two words or phrases, outside quotes: either will do. English's
/// and French's are both read whatever the interface's language, so a query means the same in
/// each; written in small letters, they are words like any other.
pub(crate) const OPERATORS: [&str; 2] = ["OR", "OU"];

/// One thing a query looks for: a word, or words that stand together.
pub(crate) struct Term {
    /// As the researcher wrote it: the review searches for it when a match is opened.
    pub typed: String,
    /// Folded, as it is matched.
    pub needle: Vec<char>,
    /// A word of two letters, found only on its own.
    pub word: bool,
    /// The group of alternatives it belongs to.
    pub group: usize,
}

pub(crate) struct Query {
    /// Every term, group after group.
    pub terms: Vec<Term>,
    /// How many groups: a match holds a term of each.
    pub groups: usize,
    /// Words too short to look for, left out of the query.
    pub ignored: Vec<String>,
}

impl Query {
    /// The terms as the index is asked for them, group by group.
    pub fn index_terms(&self) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = vec![Vec::new(); self.groups];
        for t in &self.terms {
            out[t.group].push(index_term(&t.needle));
        }
        out
    }
}

/// A piece of a query: a word, or words between quotes.
struct Token {
    text: String,
    quoted: bool,
}

impl Token {
    fn operator(&self) -> bool {
        !self.quoted && OPERATORS.contains(&self.text.as_str())
    }
}

/// The words of a query, each group of alternatives as written: for a phrase, the words
/// between operators, each run one alternative; otherwise each word or quoted run, punctuation
/// around a word set aside, an operator between two of them putting the second beside the
/// first. An operator with nothing on one side is left out.
fn written(query: &str, mode: MatchMode) -> Vec<Vec<String>> {
    let mut tokens = Vec::new();
    for (i, chunk) in query.split(QUOTES).enumerate() {
        if i % 2 == 1 {
            tokens.push(Token {
                text: chunk.split_whitespace().collect::<Vec<_>>().join(" "),
                quoted: true,
            });
        } else {
            tokens.extend(chunk.split_whitespace().map(|w| Token {
                text: if mode == MatchMode::Phrase {
                    w.to_string()
                } else {
                    w.trim_matches(|c: char| c.is_ascii_punctuation())
                        .to_string()
                },
                quoted: false,
            }));
        }
    }
    let mut out: Vec<Vec<String>> = Vec::new();
    if mode == MatchMode::Phrase {
        let mut alternatives = Vec::new();
        let mut run: Vec<String> = Vec::new();
        for t in tokens {
            if t.operator() {
                alternatives.push(run.join(" "));
                run.clear();
            } else if !t.text.is_empty() {
                run.push(t.text);
            }
        }
        alternatives.push(run.join(" "));
        out.push(alternatives);
    } else {
        let mut or = false;
        for t in tokens.into_iter().filter(|t| !t.text.is_empty()) {
            if t.operator() {
                or = !out.is_empty();
            } else {
                match out.last_mut() {
                    Some(group) if or => group.push(t.text),
                    _ => out.push(vec![t.text]),
                }
                or = false;
            }
        }
    }
    out
}

/// The terms of a query as `mode` reads it ([`written`]). A term that folds to fewer than three
/// characters and is not a word of two letters is left out and named; one written twice is
/// looked for once, in the first group that has it; a group left with nothing is dropped.
pub(crate) fn parse(query: &str, mode: MatchMode) -> Query {
    let mut out = Query {
        terms: Vec::new(),
        groups: 0,
        ignored: Vec::new(),
    };
    for alternatives in written(query, mode) {
        let mut any = false;
        for typed in alternatives.into_iter().filter(|w| !w.is_empty()) {
            let needle: Vec<char> = folded(&typed).chars().collect();
            if !long_enough(&needle) {
                if !out.ignored.contains(&typed) {
                    out.ignored.push(typed);
                }
            } else if !out.terms.iter().any(|t| t.needle == needle) {
                out.terms.push(Term {
                    typed,
                    word: is_word(&needle),
                    needle,
                    group: out.groups,
                });
                any = true;
            }
        }
        if any {
            out.groups += 1;
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
    group: usize,
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

/// Where a term of every group stands with at most `near` words between the first and the
/// last: the first such stretch to end, then the next after it, none overlapping. `places` are
/// in reading order.
fn clusters<'a>(text: &str, places: &'a [Place], groups: usize, near: u32) -> Vec<Vec<&'a Place>> {
    let words = word_numbers(text);
    let word = |at: usize| words.get(at).copied().unwrap_or_default();
    let mut latest: Vec<Option<&Place>> = vec![None; groups];
    let mut out = Vec::new();
    for p in places {
        latest[p.group] = Some(p);
        let chosen: Vec<&Place> = latest.iter().flatten().copied().collect();
        if chosen.len() < groups {
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
            latest = vec![None; groups];
        }
    }
    out
}

/// The matches of `query` in one passage, as `mode` finds them, within at most `near` words in
/// [`MatchMode::Near`]. A match never runs from one part of the passage into the next. Each
/// part, and each of a recording's lines, is folded once for every term.
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
    // every group and those of a word of two letters inside a longer one too, as the review's
    // search counts them.
    let mut nth = vec![0u32; query.terms.len()];
    for (p, part) in passage.parts.iter().enumerate() {
        let hay = fold(&part.text);
        let lines: Vec<_> = match start_ms {
            Some(_) => part.lines.iter().map(|(at, t)| (*at, fold(t))).collect(),
            None => Vec::new(),
        };
        let mut places = Vec::new();
        let mut held = vec![false; query.groups];
        for (t, term) in query.terms.iter().enumerate() {
            let found = spots(&hay, &term.needle);
            // A recording's match is tied to its line when the lines hold as many matches as
            // the text: the n-th match is then in the line holding the n-th match.
            let at_lines: Vec<u64> = lines
                .iter()
                .flat_map(|(at, line)| spots(line, &term.needle).into_iter().map(move |_| *at))
                .collect();
            let timed = !at_lines.is_empty() && at_lines.len() == found.len();
            for (i, spot) in found.into_iter().enumerate() {
                let this = nth[t];
                nth[t] += 1;
                if term.word && !spot.alone {
                    continue;
                }
                held[term.group] = true;
                places.push(Place {
                    term: t,
                    group: term.group,
                    nth: this,
                    span: spot.span,
                    at_ms: timed.then(|| start_ms.unwrap_or_default() + at_lines[i]),
                });
            }
        }
        if !held.iter().all(|h| *h) {
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
                for cluster in clusters(&part.text, &places, query.groups, near) {
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
mod tests;
