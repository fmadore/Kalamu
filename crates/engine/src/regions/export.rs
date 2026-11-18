//! The geometry a page's exports write (#84): the model's regions with the researcher's boxes
//! laid on them, as the review shows them, and the researcher's own regions beside them. Core
//! lays them ([`kalamu_core::regions::lay_for_export`]); here they are gathered for one set-up
//! and follow a correction.
//!
//! # Which of the researcher's regions
//!
//! Those `provenance.json` lists for the set-up ([`Engine::regions_exported`]): drawn on the
//! source, adjusting one of its readings, or adjusting a reading no set-up shows any more. A
//! region goes with the piece of its page and half; one on a half the job did not read alone
//! (drawn on a whole page later split, or on a half of a page that was not) goes with the
//! page's first piece, in its own fractions, and the formats put it on the whole page.
//!
//! # After a correction
//!
//! The boxes are laid first, on the model's regions whose ids they name, and the corrected words
//! then laid onto the words' boxes ([`crate::realign`]): a word given a box passes it to the
//! corrected word or words that take its place, a line given one keeps it (realigning fits a line
//! to its words; it is then widened only to hold them), a block keeps it. Where the corrected text
//! cannot be laid on the words (a block placed whole, text the model never placed) it is one
//! block over the page, which claims nothing more than that; a box the researcher gave a region
//! of the model's is then one of their own, with no text, the page's text being in that block.
//! So is a box for a line or word the correction removed.

use std::collections::HashSet;

use kalamu_core::export::{RegionPath, ResearcherGeometry};
use kalamu_core::regions::{LaidForExport, contain, lay_for_export};
use kalamu_core::{
    DrawnRegion, Job, PageHalf, PieceRef, PieceState, SourceFile, SourceId, TextRegion,
};

use crate::{Engine, EngineError};

/// The researcher's regions a set-up's export writes, handed out piece by piece.
pub(crate) struct ExportRegions {
    regions: Vec<DrawnRegion>,
    /// The pages and halves the job read as pieces.
    pieces: HashSet<(u32, Option<PageHalf>)>,
    /// The pages whose first piece was given its regions on halves read no other way.
    given: HashSet<u32>,
}

impl ExportRegions {
    /// The regions that go with `piece`, a part of the export, in the order made. Each page's
    /// first part asked for takes those on a half no piece reads.
    pub(crate) fn on(&mut self, piece: &PieceRef) -> Vec<DrawnRegion> {
        let PieceRef::Page { number, half } = *piece else {
            return Vec::new();
        };
        let first = self.given.insert(number);
        self.regions
            .iter()
            .filter(|r| r.page == number)
            .filter(|r| r.half == half || (first && !self.pieces.contains(&(number, r.half))))
            .cloned()
            .collect()
    }
}

impl Engine {
    /// The researcher's regions for a set-up's export of a source, `states` its pieces.
    pub(crate) fn export_regions(
        &self,
        job: &Job,
        source: SourceId,
        states: &[PieceState],
    ) -> Result<ExportRegions, EngineError> {
        Ok(ExportRegions {
            regions: self.regions_exported(job, source, states)?,
            pieces: states
                .iter()
                .filter_map(|s| match s.piece {
                    PieceRef::Page { number, half } => Some((number, half)),
                    _ => None,
                })
                .collect(),
            given: HashSet::new(),
        })
    }

    /// What an export part of `piece` writes where the text sits: the model's regions `model`
    /// of the reading `result` (none for a piece that failed), the researcher's boxes among
    /// `drawn` laid on them, the corrected text `corrected` laid on the words where it can be,
    /// and whose each box is. See the module's notes.
    pub(crate) fn export_geometry(
        &self,
        source: &SourceFile,
        piece: &PieceRef,
        model: &[TextRegion],
        result: Option<&str>,
        corrected: Option<&str>,
        drawn: &[DrawnRegion],
    ) -> (Vec<TextRegion>, ResearcherGeometry) {
        // Only a line of a block placed whole needs the page's proportions, to estimate it as
        // the review did.
        let adjusts_lines = drawn.iter().any(|r| {
            r.adjusts
                .as_ref()
                .is_some_and(|a| Some(a.result.as_str()) == result)
        });
        let aspect = if adjusts_lines && model.iter().any(|r| r.lines.is_empty()) {
            self.page_aspect(source, piece)
        } else {
            None
        };
        let LaidForExport {
            mut regions,
            mut boxes,
            mut estimated,
            mut own,
        } = lay_for_export(model, result, drawn, aspect);
        let whole_page = |text: &str| vec![TextRegion::block(0.05, 0.05, 0.9, 0.9, text)];
        match corrected {
            Some(text) if !model.is_empty() => {
                // A block placed whole has no words to lay the correction on, even where its
                // estimated lines were written in for a box.
                let placed = model.iter().all(|r| !r.lines.is_empty());
                let realigned = placed
                    .then(|| crate::realign::realign_traced(&regions, text))
                    .flatten();
                match realigned {
                    Some(realigned) => {
                        let mut moved: Vec<(RegionPath, DrawnRegion)> = Vec::new();
                        for (old, by) in boxes {
                            let places = realigned.places(old);
                            if places.is_empty() {
                                own.push(without_text(by));
                            } else {
                                moved.extend(places.into_iter().map(|p| (p, by.clone())));
                            }
                        }
                        regions = realigned.regions;
                        keep_line_boxes(&mut regions, &moved);
                        boxes = moved;
                    }
                    None => {
                        regions = whole_page(text);
                        own.extend(boxes.drain(..).map(|(_, by)| without_text(by)));
                        estimated.clear();
                    }
                }
            }
            Some(text) if !text.trim().is_empty() && matches!(piece, PieceRef::Page { .. }) => {
                regions = whole_page(text);
            }
            _ => {}
        }
        // In the order made, the boxes that adjust nothing any more among them.
        own.sort_by_key(|r| r.created_at);
        let geometry = contain(&mut regions, &boxes, estimated, &own);
        (regions, geometry)
    }
}

/// A box given to a region no longer exported, as one of the researcher's own: it says a region
/// is there, with no text, which the page's text holds.
fn without_text(by: DrawnRegion) -> DrawnRegion {
    DrawnRegion {
        text: String::new(),
        ..by
    }
}

/// Gives each line the researcher placed their box back, which realigning fitted to its words;
/// and fits a line holding a word they placed to the model's words alone, which realigning had
/// fitted to theirs too, so the line is said to be widened for it.
fn keep_line_boxes(regions: &mut [TextRegion], boxes: &[(RegionPath, DrawnRegion)]) {
    let placed = |b: usize, l: usize| {
        boxes
            .iter()
            .any(|(p, _)| p.block == b && p.line == Some(l) && p.word.is_none())
    };
    for (path, by) in boxes {
        let Some(l) = path.line else {
            continue;
        };
        let Some(line) = regions.get_mut(path.block).and_then(|r| r.lines.get_mut(l)) else {
            continue;
        };
        if path.word.is_none() {
            (line.x, line.y, line.width, line.height) = (by.x, by.y, by.width, by.height);
            continue;
        }
        if placed(path.block, l) {
            continue;
        }
        let theirs = |w: usize| {
            boxes
                .iter()
                .any(|(p, _)| p.block == path.block && p.line == Some(l) && p.word == Some(w))
        };
        let model = line
            .words
            .iter()
            .enumerate()
            .filter(|(w, _)| !theirs(*w))
            .map(|(_, w)| (w.x, w.y, w.x + w.width, w.y + w.height))
            .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)));
        if let Some((x0, y0, x1, y1)) = model {
            (line.x, line.y, line.width, line.height) = (x0, y0, x1 - x0, y1 - y0);
        }
    }
}
