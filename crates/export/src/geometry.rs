//! What ALTO, PAGE XML and the searchable PDF write where the text sits, page by page: the
//! regions of the page's parts put back on the whole page, whose each box is where the researcher
//! had a hand in it, and the researcher's own regions (#84).
//!
//! All of it is decided before an exporter sees it (`kalamu_engine`'s export document, and
//! [`kalamu_core::export::ResearcherGeometry`]): the formats write the boxes they are given and
//! say whose they are, each in its own way.

use std::collections::BTreeMap;

use kalamu_core::export::BoxKind;
use kalamu_core::{ExportDocument, PieceRef, TextRegion, Timestamp};

/// Whose a box is, where the researcher had a hand in it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Mark {
    /// The researcher's region, as `provenance.json` lists it.
    pub id: String,
    pub kind: BoxKind,
}

impl Mark {
    /// The mark in words, for a format that says it in a note: a box the researcher placed is
    /// theirs, the model's box widened for one of theirs is still the model's.
    pub(crate) fn note(&self) -> String {
        let id = &self.id;
        match self.kind {
            BoxKind::Researcher => format!("position set by the researcher (region {id})"),
            BoxKind::ResearcherWidened => format!(
                "position set by the researcher (region {id}), widened to hold what the model \
                 placed within it"
            ),
            BoxKind::ModelWidened => format!(
                "the model's position, widened to hold one set by the researcher (region {id})"
            ),
        }
    }
}

/// The marks of one block of the model's, and of its lines and words.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BlockMarks {
    pub block: Option<Mark>,
    /// By line, from 0.
    pub lines: BTreeMap<usize, Mark>,
    /// By line and word, from 0.
    pub words: BTreeMap<(usize, usize), Mark>,
    /// A block placed whole whose estimated lines were written into it, so the researcher's box
    /// for one of them is kept: its lines are estimated all the same.
    pub estimated: bool,
}

/// A region the researcher placed that adjusts nothing of the model's, on the whole page.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Drawn {
    pub id: String,
    pub region: TextRegion,
}

/// One page's regions, on the whole page, in reading order.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct PageGeometry {
    /// The model's regions, with the researcher's boxes.
    pub regions: Vec<TextRegion>,
    /// One for each of `regions`.
    pub marks: Vec<BlockMarks>,
    /// After the model's, in the order made; with or without text.
    pub drawn: Vec<Drawn>,
    /// When the researcher last saved a box on the page.
    pub changed: Option<Timestamp>,
}

impl PageGeometry {
    /// What a format that writes text sets on the page: the model's regions, then the
    /// researcher's own that hold text, as blocks placed whole.
    pub(crate) fn texts(&self) -> Vec<TextRegion> {
        let drawn = self
            .drawn
            .iter()
            .filter(|d| !d.region.text.trim().is_empty())
            .map(|d| d.region.clone());
        self.regions.iter().cloned().chain(drawn).collect()
    }
}

/// Each page's geometry, by page number: a page split into halves put back together, its left
/// half's regions first. A page appears when a part of it has regions, or the researcher placed
/// a region of their own on it.
pub(crate) fn by_page(doc: &ExportDocument) -> BTreeMap<u32, PageGeometry> {
    let mut pages: BTreeMap<u32, PageGeometry> = BTreeMap::new();
    for part in &doc.parts {
        let PieceRef::Page { number, half } = part.piece else {
            continue;
        };
        let researcher = &part.researcher;
        if part.regions.is_empty() && researcher.drawn.is_empty() {
            continue;
        }
        let page = pages.entry(number).or_default();
        let base = page.regions.len();
        page.regions
            .extend(part.regions.iter().map(|r| r.onto_whole_page(half)));
        page.marks
            .resize_with(page.regions.len(), BlockMarks::default);
        for &b in &researcher.estimated {
            if let Some(marks) = page.marks.get_mut(base + b) {
                marks.estimated = true;
            }
        }
        for adjusted in &researcher.adjusted {
            let at = adjusted.at;
            let Some(marks) = page.marks.get_mut(base + at.block) else {
                continue;
            };
            let mark = Mark {
                id: adjusted.id.clone(),
                kind: adjusted.kind,
            };
            match (at.line, at.word) {
                (None, _) => marks.block = Some(mark),
                (Some(l), None) => {
                    marks.lines.insert(l, mark);
                }
                (Some(l), Some(w)) => {
                    marks.words.insert((l, w), mark);
                }
            }
            page.changed = page.changed.max(Some(adjusted.updated_at));
        }
        for drawn in &researcher.drawn {
            page.drawn.push(Drawn {
                id: drawn.id.clone(),
                region: drawn.region.onto_whole_page(drawn.half),
            });
            page.changed = page.changed.max(Some(drawn.updated_at));
        }
    }
    pages
}

/// The id a format gives a region of the researcher's own: theirs, which may start with a
/// digit, after `researcher_`, so it is a valid XML id; anything an XML name may not hold
/// becomes `_`.
pub(crate) fn researcher_id(id: &str) -> String {
    let safe: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("researcher_{safe}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_researcher_s_id_is_a_valid_xml_id() {
        assert_eq!(
            researcher_id("6f1c2a3b-0000-4000-8000-000000000001"),
            "researcher_6f1c2a3b-0000-4000-8000-000000000001"
        );
        assert_eq!(researcher_id("a b:c"), "researcher_a_b_c");
    }
}
