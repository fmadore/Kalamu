//! Document units (#50): the articles, letters or items a table of contents found in one
//! source, each a set of the source's pages in reading order, kept as a layer of its own over
//! the scan and the table of contents, which never change.
//!
//! A unit starts from a reviewed entry (its title, its pages, the researcher's fields) or is
//! made by hand, and can be renamed, given other pages, joined to another or split in two. Its
//! pages are the source's own, numbered as the job's pieces are (the file's pages, not the
//! numbers printed on them), so a unit's precision is the whole page: two articles that share
//! a page share it whole, and the units say so rather than pretend to cut it.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::analysis::answer_text;
use crate::contents::{PAGES, TITLE};
use crate::{FieldAnswer, JobId, PageHalf, PieceRef, SetupId, SourceId, Timestamp, UnitId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentUnit {
    pub id: UnitId,
    /// The table of contents it belongs to: one source under one set-up.
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    /// Its place in the list: the entries' order, then the researcher's.
    pub position: u32,
    pub title: String,
    /// The entry's other fields (authors, a date, a summary) as the researcher left them.
    pub fields: Vec<FieldAnswer>,
    /// The pages in reading order, as the job's pieces number them.
    pub pages: Vec<PieceRef>,
    /// The entry it was made from, numbered from 1; none for a unit made by hand.
    pub entry: Option<u32>,
    /// Raised by every change, so a frozen copy says which version it read.
    pub revision: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DocumentUnit {
    pub fn new(job_id: JobId, setup_id: SetupId, source_id: SourceId, position: u32) -> Self {
        let now = Timestamp::now();
        Self {
            id: UnitId::new(),
            job_id,
            setup_id,
            source_id,
            position,
            title: String::new(),
            fields: Vec::new(),
            pages: Vec::new(),
            entry: None,
            revision: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// The pages as `(number, half)`, for cutting them out of the file.
    pub fn page_numbers(&self) -> Vec<(u32, Option<PageHalf>)> {
        self.pages
            .iter()
            .filter_map(|p| match p {
                PieceRef::Page { number, half } => Some((*number, *half)),
                _ => None,
            })
            .collect()
    }
}

/// What is wrong with a list of pages; the interface words it, the message is the detail.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PagesError {
    #[error("no pages")]
    Empty,
    #[error("“{0}”")]
    Unreadable(String),
    #[error("page {0}")]
    NoSuchPage(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UnitError {
    #[error(transparent)]
    Pages(#[from] PagesError),
    #[error("not a table of contents")]
    NotContents,
    #[error("units were already made from these entries")]
    AlreadyMade,
    /// A unit without pages, named by its title, cannot be read on its own.
    #[error("{0}")]
    NoPages(String),
    /// Only a PDF's pages are cut out for now; the file's name.
    #[error("{0}")]
    NotPdf(String),
}

impl UnitError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Pages(PagesError::Empty) => "unit.pages_empty",
            Self::Pages(PagesError::Unreadable(_)) => "unit.pages_unreadable",
            Self::Pages(PagesError::NoSuchPage(_)) => "unit.no_such_page",
            Self::NotContents => "unit.not_contents",
            Self::AlreadyMade => "unit.already_made",
            Self::NoPages(_) => "unit.no_pages",
            Self::NotPdf(_) => "unit.not_pdf",
        }
    }
}

/// A unit as a later job read it: kept with that job's source, so a change to the unit
/// afterwards marks the job out of date without changing what it read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FrozenUnit {
    pub id: UnitId,
    pub title: String,
    pub revision: u32,
    pub pages: Vec<PieceRef>,
}

impl From<&DocumentUnit> for FrozenUnit {
    fn from(unit: &DocumentUnit) -> Self {
        Self {
            id: unit.id,
            title: unit.title.clone(),
            revision: unit.revision,
            pages: unit.pages.clone(),
        }
    }
}

/// The pages a list such as `"1, 3-5, 8 (left)"` names, among the source's pieces, in the
/// order written and each once. A page split into halves is both halves unless one is named.
pub fn parse_pages(text: &str, pieces: &[PieceRef]) -> Result<Vec<PieceRef>, PagesError> {
    let (pages, unknown) = read_pages(text, pieces)?;
    if let Some(first) = unknown.first() {
        return Err(PagesError::NoSuchPage(first.clone()));
    }
    if pages.is_empty() {
        return Err(PagesError::Empty);
    }
    Ok(pages)
}

/// The pages of a list among the source's pieces, and the numbers it names that the source
/// does not have. Fails only on what is not a page number at all.
fn read_pages(text: &str, pieces: &[PieceRef]) -> Result<(Vec<PieceRef>, Vec<String>), PagesError> {
    let mut pages: Vec<PieceRef> = Vec::new();
    let mut unknown = Vec::new();
    for item in text.split([',', ';']) {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (range, half) = split_half(item);
        let (first, last) = read_range(range).ok_or_else(|| PagesError::Unreadable(item.into()))?;
        for number in first..=last {
            let found: Vec<&PieceRef> = pieces
                .iter()
                .filter(|p| match p {
                    PieceRef::Page { number: n, half: h } => {
                        *n == number && (half.is_none() || *h == half || h.is_none())
                    }
                    _ => false,
                })
                .collect();
            if found.is_empty() {
                unknown.push(match half {
                    Some(half) => PieceRef::Page {
                        number,
                        half: Some(half),
                    }
                    .label(),
                    None => number.to_string(),
                });
            }
            for piece in found {
                if !pages.contains(piece) {
                    pages.push(piece.clone());
                }
            }
        }
    }
    Ok((pages, unknown))
}

/// `"8 (left)"` → `("8", Left)`; `"3-5"` → `("3-5", None)`.
fn split_half(item: &str) -> (&str, Option<PageHalf>) {
    let lower = item.to_lowercase();
    for (word, half) in [("left", PageHalf::Left), ("right", PageHalf::Right)] {
        if let Some(at) = lower.find(word) {
            let range = item[..at].trim_end().trim_end_matches('(').trim_end();
            return (range, Some(half));
        }
    }
    (item, None)
}

/// `"3-5"`, `"3–5"`, `"p. 3"`, `"pp. 3-5"` → `(3, 5)`; a reversed range is unreadable.
fn read_range(range: &str) -> Option<(u32, u32)> {
    let range = range
        .trim()
        .trim_start_matches(|c: char| c.is_alphabetic() || c == '.' || c.is_whitespace());
    let mut ends = range.split(['-', '–', '—']).map(str::trim);
    let first: u32 = ends.next()?.parse().ok()?;
    let last: u32 = match ends.next() {
        Some(end) => end.parse().ok()?,
        None => first,
    };
    (ends.next().is_none() && first >= 1 && first <= last).then_some((first, last))
}

/// The pages as the researcher reads and types them: `1, 3-5, 8 (left)`. Both halves of a
/// page in order read as the page.
pub fn format_pages(pages: &[PieceRef]) -> String {
    let mut items: Vec<(u32, Option<PageHalf>)> = Vec::new();
    for page in pages {
        let PieceRef::Page { number, half } = page else {
            continue;
        };
        if *half == Some(PageHalf::Right) && items.last() == Some(&(*number, Some(PageHalf::Left)))
        {
            items.pop();
            items.push((*number, None));
        } else {
            items.push((*number, *half));
        }
    }
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < items.len() {
        let (first, half) = items[i];
        if half.is_some() {
            out.push(
                PieceRef::Page {
                    number: first,
                    half,
                }
                .label(),
            );
            i += 1;
            continue;
        }
        let mut last = first;
        while i + 1 < items.len() && items[i + 1] == (last + 1, None) {
            last += 1;
            i += 1;
        }
        out.push(if last > first {
            format!("{first}-{last}")
        } else {
            first.to_string()
        });
        i += 1;
    }
    out.join(", ")
}

/// What could not be taken from an entry when units were made from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SeedProblem {
    /// The entry, numbered from 1.
    pub entry: u32,
    /// Its pages as the table of contents gives them.
    pub pages: String,
    /// Pages it names that the document does not have; empty when none could be read.
    pub unknown: Vec<String>,
}

/// One unit per entry of a table of contents, in its order, numbered from `first_position`:
/// the title and pages from the entry's fixed fields, the rest as the entry's other fields.
/// Pages the document does not have are left out and reported; an entry whose pages cannot
/// be read at all still gives a unit, without pages, for the researcher to complete.
pub fn seed(
    (job_id, setup_id, source_id): (JobId, SetupId, SourceId),
    entries: &[Vec<FieldAnswer>],
    pieces: &[PieceRef],
    first_position: u32,
) -> (Vec<DocumentUnit>, Vec<SeedProblem>) {
    let mut units = Vec::new();
    let mut problems = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let number = i as u32 + 1;
        let pages_text = answer_text(entry, PAGES);
        let mut unit = DocumentUnit::new(job_id, setup_id, source_id, first_position + i as u32);
        unit.title = answer_text(entry, TITLE).trim().to_string();
        unit.entry = Some(number);
        unit.fields = entry
            .iter()
            .filter(|a| a.key != TITLE && a.key != PAGES)
            .cloned()
            .collect();
        match read_pages(&pages_text, pieces) {
            Ok((pages, unknown)) => {
                if !unknown.is_empty() || pages.is_empty() {
                    problems.push(SeedProblem {
                        entry: number,
                        pages: pages_text.clone(),
                        unknown,
                    });
                }
                unit.pages = pages;
            }
            Err(_) => problems.push(SeedProblem {
                entry: number,
                pages: pages_text.clone(),
                unknown: Vec::new(),
            }),
        }
        units.push(unit);
    }
    (units, problems)
}

/// `second`'s pages after `first`'s, each once: an article continued in an entry of its own.
pub fn join(first: &DocumentUnit, second: &DocumentUnit) -> Vec<PieceRef> {
    let mut pages = first.pages.clone();
    for page in &second.pages {
        if !pages.contains(page) {
            pages.push(page.clone());
        }
    }
    pages
}

/// The pages before `at` and from `at` on: two items an entry took for one.
pub fn split(unit: &DocumentUnit, at: usize) -> Option<(Vec<PieceRef>, Vec<PieceRef>)> {
    (at > 0 && at < unit.pages.len())
        .then(|| (unit.pages[..at].to_vec(), unit.pages[at..].to_vec()))
}

/// A page a unit shares with others: the precision of a whole-page cut, said outright.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SharedPage {
    pub page: PieceRef,
    /// The other units on it.
    pub with: Vec<UnitId>,
}

/// For each unit, in order, the pages it shares with others.
pub fn shared_pages(units: &[DocumentUnit]) -> Vec<Vec<SharedPage>> {
    units
        .iter()
        .map(|unit| {
            let mut seen = BTreeSet::new();
            unit.pages
                .iter()
                .filter(|page| seen.insert(page.descriptor()))
                .filter_map(|page| {
                    let with: Vec<UnitId> = units
                        .iter()
                        .filter(|other| other.id != unit.id && overlaps(&other.pages, page))
                        .map(|other| other.id)
                        .collect();
                    (!with.is_empty()).then(|| SharedPage {
                        page: page.clone(),
                        with,
                    })
                })
                .collect()
        })
        .collect()
}

/// Whether `pages` cover any of `page`: a whole page and either of its halves overlap.
fn overlaps(pages: &[PieceRef], page: &PieceRef) -> bool {
    let PieceRef::Page { number, half } = page else {
        return false;
    };
    pages.iter().any(|p| match p {
        PieceRef::Page { number: n, half: h } => {
            n == number && (h.is_none() || half.is_none() || h == half)
        }
        _ => false,
    })
}

/// A file name for a unit's pages: the source's name, the unit's place and its title, kept
/// short and free of the characters Windows and macOS refuse.
pub fn file_stem(source_stem: &str, position: u32, title: &str) -> String {
    let title: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '[' | ']' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let title: String = title.chars().take(60).collect();
    let title = title.trim_end_matches(['.', ' ']);
    if title.is_empty() {
        format!("{source_stem} {:02}", position + 1)
    } else {
        format!("{source_stem} {:02} {title}", position + 1)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::FieldValue;

    fn pieces() -> Vec<PieceRef> {
        // Six pages, the fourth a spread split in two.
        let mut out: Vec<PieceRef> = (1..=3).map(PieceRef::page).collect();
        out.push(PieceRef::Page {
            number: 4,
            half: Some(PageHalf::Left),
        });
        out.push(PieceRef::Page {
            number: 4,
            half: Some(PageHalf::Right),
        });
        out.extend((5..=6).map(PieceRef::page));
        out
    }

    fn left(number: u32) -> PieceRef {
        PieceRef::Page {
            number,
            half: Some(PageHalf::Left),
        }
    }

    fn right(number: u32) -> PieceRef {
        PieceRef::Page {
            number,
            half: Some(PageHalf::Right),
        }
    }

    #[test]
    fn page_lists_are_read_in_the_order_written() {
        let pieces = pieces();
        assert_eq!(
            parse_pages("1, 5-6, 2", &pieces).unwrap(),
            [
                PieceRef::page(1),
                PieceRef::page(5),
                PieceRef::page(6),
                PieceRef::page(2)
            ]
        );
        assert_eq!(
            parse_pages("pp. 3–4", &pieces).unwrap(),
            [PieceRef::page(3), left(4), right(4)],
            "a page split in two is both halves"
        );
        assert_eq!(
            parse_pages("4 (right); 1", &pieces).unwrap(),
            [right(4), PieceRef::page(1)]
        );
        assert_eq!(parse_pages("2, 2", &pieces).unwrap(), [PieceRef::page(2)]);
        assert_eq!(parse_pages(" ", &pieces), Err(PagesError::Empty));
        assert_eq!(
            parse_pages("5-3", &pieces),
            Err(PagesError::Unreadable("5-3".into()))
        );
        assert_eq!(
            parse_pages("two", &pieces),
            Err(PagesError::Unreadable("two".into()))
        );
        assert_eq!(
            parse_pages("6-8", &pieces),
            Err(PagesError::NoSuchPage("7".into()))
        );
        assert_eq!(
            parse_pages("1 (left)", &pieces),
            Ok(vec![PieceRef::page(1)]),
            "a page not split is the page, whichever half is named"
        );
    }

    #[test]
    fn page_lists_are_written_back_as_typed() {
        let pieces = pieces();
        for text in ["1, 5-6, 2", "2-6", "4 (right), 1", "3, 4 (left)"] {
            assert_eq!(format_pages(&parse_pages(text, &pieces).unwrap()), text);
        }
        assert_eq!(format_pages(&[]), "");
    }

    fn entry(title: &str, pages: &str, author: &str) -> Vec<FieldAnswer> {
        let text = |key: &str, value: &str| FieldAnswer {
            key: key.into(),
            value: FieldValue::Text(value.into()),
            reason: None,
        };
        vec![
            text(TITLE, title),
            text(PAGES, pages),
            text("author", author),
        ]
    }

    #[test]
    fn units_are_made_from_the_entries() {
        let ids = (JobId::new(), SetupId::new(), SourceId::new());
        let entries = [
            entry(" Le Hajj ", "1, 5", "La Rédaction"),
            entry("Lettre de Ouagadougou", "2-3, 9", ""),
            entry("Avis", "dernière page", ""),
        ];
        let (units, problems) = seed(ids, &entries, &pieces(), 0);
        assert_eq!(units.len(), 3);
        assert_eq!(units[0].title, "Le Hajj");
        assert_eq!(units[0].pages, [PieceRef::page(1), PieceRef::page(5)]);
        assert_eq!(units[0].entry, Some(1));
        assert_eq!(
            units[0].fields.len(),
            1,
            "title and pages are the unit's own"
        );
        assert_eq!(units[1].pages, [PieceRef::page(2), PieceRef::page(3)]);
        assert_eq!(units[2].position, 2);
        assert!(units[2].pages.is_empty());
        assert_eq!(
            problems,
            [
                SeedProblem {
                    entry: 2,
                    pages: "2-3, 9".into(),
                    unknown: vec!["9".into()],
                },
                SeedProblem {
                    entry: 3,
                    pages: "dernière page".into(),
                    unknown: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn units_are_joined_split_and_their_shared_pages_named() {
        let ids = (JobId::new(), SetupId::new(), SourceId::new());
        let entries = [
            entry("A", "1-2", ""),
            entry("B", "2, 4 (left)", ""),
            entry("C", "4", ""),
        ];
        let (units, _) = seed(ids, &entries, &pieces(), 0);
        assert_eq!(
            join(&units[0], &units[1]),
            [PieceRef::page(1), PieceRef::page(2), left(4)]
        );
        assert_eq!(split(&units[2], 1), Some((vec![left(4)], vec![right(4)])));
        assert_eq!(split(&units[2], 0), None);
        assert_eq!(split(&units[2], 2), None);

        let shared = shared_pages(&units);
        assert_eq!(shared[0].len(), 1);
        assert_eq!(shared[0][0].page, PieceRef::page(2));
        assert_eq!(shared[0][0].with, [units[1].id]);
        assert_eq!(shared[1].len(), 2, "page 2 with A, the left of 4 with C");
        assert_eq!(shared[2].len(), 1, "only the left half of 4 is shared");
        assert_eq!(shared[2][0].page, left(4));
    }

    #[test]
    fn file_names_are_short_and_safe() {
        assert_eq!(
            file_stem("issue", 2, "[Éditorial] Le Hajj: 2024?"),
            "issue 03 Éditorial Le Hajj 2024"
        );
        assert_eq!(file_stem("issue", 0, "  "), "issue 01");
        assert_eq!(
            file_stem("issue", 9, &"long ".repeat(30)).chars().count(),
            "issue 10 ".len() + 59
        );
    }
}
