//! The researcher's boxes laid on a page's regions for ALTO, PAGE XML and the searchable PDF
//! (#84): the exports read the corrected view, as the review shows it.
//!
//! A box adjusting a region of the model's belongs to one reading of the page (its cache key)
//! and one region id in it, the review's (`r2`, `r2_l3`, `r2_l3_w1`). Which of the researcher's
//! regions a set-up's export considers at all is the engine's rule, the one `provenance.json`
//! follows; here, each box whose reading is the piece's and whose region the reading holds takes
//! that region's place, and every other region of theirs is one of their own.
//!
//! # What lies within a box
//!
//! The researcher moves one box at a time, and the review draws each where they put it: a block
//! given another box keeps its lines where the model measured them, a line its words. A format
//! nests them, a line within its block, a word within its line. So, where the researcher had a
//! hand in a box, the block or line holding it is widened just enough to hold what lies within it
//! ([`contain`]), and said to be ([`BoxKind`]): the researcher's block widened to hold the lines
//! the model measured in it, or the model's block widened to hold a line the researcher moved.
//! Nothing is scaled or moved: a word the model measured stays where it was measured, and a box
//! never claims a place nobody gave it. A block the model placed whole has no lines of its own:
//! a format estimates them within whatever box the block has, so they follow the researcher's.
//!
//! The lines the review estimates within such a block can be given a box too. Only then are they
//! written into the block, estimated with the page's proportions exactly as the review estimated
//! them, so its ids name the same lines; the block is listed as estimated
//! ([`ResearcherGeometry::estimated`]), its other lines as approximate as before. A line of
//! those given a box takes its estimated words along into it, each keeping its share of the
//! line, since nobody measured them anywhere else.

use std::collections::BTreeSet;

use super::{DrawnRegion, block_id, line_id, word_id};
use crate::TextRegion;
use crate::export::{AdjustedBox, BoxKind, RegionPath, ResearcherGeometry, ResearcherRegion};

/// The model's regions of one piece with the researcher's boxes laid on them.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidForExport {
    pub regions: Vec<TextRegion>,
    /// Each box given to a block, line or word of `regions`, by its place there, with the region
    /// of the researcher's it comes from; one per place, the last made.
    pub boxes: Vec<(RegionPath, DrawnRegion)>,
    /// Blocks placed whole whose estimated lines were written into them.
    pub estimated: Vec<usize>,
    /// The researcher's regions that adjust nothing in `regions`, in the order made.
    pub own: Vec<DrawnRegion>,
}

/// The place of a region's id in its result (`r2_l3` is block 1, line 2, from 0), the inverse of
/// [`block_id`], [`line_id`] and [`word_id`]; none for an id they do not make.
pub fn path_of(id: &str) -> Option<RegionPath> {
    let mut parts = id.strip_prefix('r')?.split('_');
    let number = |part: &str, prefix: char| -> Option<usize> {
        part.strip_prefix(prefix)?
            .parse::<usize>()
            .ok()?
            .checked_sub(1)
    };
    let block = parts.next()?.parse::<usize>().ok()?.checked_sub(1)?;
    let line = match parts.next() {
        Some(part) => Some(number(part, 'l')?),
        None => None,
    };
    let word = match parts.next() {
        Some(part) => Some(number(part, 'w')?),
        None => None,
    };
    if parts.next().is_some() {
        return None;
    }
    let path = RegionPath { block, line, word };
    // Only the ids the review makes: no leading zeros, nothing more.
    let again = match (path.line, path.word) {
        (None, None) => block_id(block + 1),
        (Some(l), None) => line_id(block + 1, l + 1),
        (Some(l), Some(w)) => word_id(block + 1, l + 1, w + 1),
        (None, Some(_)) => return None,
    };
    (again == id).then_some(path)
}

/// The model's regions `model` of a piece read as `result`, with the boxes among `drawn` (the
/// researcher's regions on the piece, in the order made) that adjust one of them laid on them;
/// the rest are their own. `page_aspect`, the piece's height over its width as the review shows
/// it, estimates the lines of a block placed whole when one of them was given a box; without it
/// such a box adjusts nothing here, as the review could not have shown that line.
pub fn lay_for_export(
    model: &[TextRegion],
    result: Option<&str>,
    drawn: &[DrawnRegion],
    page_aspect: Option<f32>,
) -> LaidForExport {
    let target = |r: &DrawnRegion| -> Option<RegionPath> {
        let adjusts = r.adjusts.as_ref()?;
        (Some(adjusts.result.as_str()) == result)
            .then(|| path_of(&adjusts.region))
            .flatten()
    };
    let mut regions = model.to_vec();
    // The estimated lines of a block placed whole, written in where one is given a box.
    let mut estimated = Vec::new();
    if let Some(aspect) = page_aspect {
        for path in drawn.iter().filter_map(target) {
            let Some(region) = model.get(path.block) else {
                continue;
            };
            if !region.lines.is_empty() || estimated.contains(&path.block) {
                continue;
            }
            let lines = region.estimated_lines(aspect);
            if holds(&lines, path) {
                regions[path.block].lines = lines;
                estimated.push(path.block);
            }
        }
    }
    estimated.sort_unstable();

    // Lines first: a line estimated within a block placed whole carries its estimated words into
    // the researcher's box, and a word given a box of its own is then laid over that.
    let line_first =
        |r: &&DrawnRegion| !target(r).is_some_and(|p| p.line.is_some() && p.word.is_none());
    let mut order: Vec<&DrawnRegion> = drawn.iter().collect();
    order.sort_by_key(line_first);
    let mut boxes: Vec<(RegionPath, DrawnRegion)> = Vec::new();
    let mut own: Vec<&DrawnRegion> = Vec::new();
    for r in order {
        let estimated_line = |p: &RegionPath| p.word.is_none() && estimated.contains(&p.block);
        match target(r).filter(|path| set_box(&mut regions, *path, r, estimated_line(path))) {
            Some(path) => {
                boxes.retain(|(p, _)| *p != path);
                boxes.push((path, r.clone()));
            }
            None => own.push(r),
        }
    }
    // In the order made.
    let made = |r: &&DrawnRegion| drawn.iter().position(|d| std::ptr::eq(d, *r));
    own.sort_by_key(made);
    let own = own.into_iter().cloned().collect();
    LaidForExport {
        regions,
        boxes,
        estimated,
        own,
    }
}

/// Whether `lines` hold the line or word `path` names.
fn holds(lines: &[crate::TextLine], path: RegionPath) -> bool {
    match (path.line, path.word) {
        (Some(l), None) => l < lines.len(),
        (Some(l), Some(w)) => lines.get(l).is_some_and(|line| w < line.words.len()),
        _ => false,
    }
}

/// Gives the block, line or word at `path` the box of `by`; false where there is none. A line
/// whose words are `estimated` (Kalamu's, within a block placed whole) takes them along, each
/// keeping its share of the line: they were never measured anywhere else.
fn set_box(
    regions: &mut [TextRegion],
    path: RegionPath,
    by: &DrawnRegion,
    estimated: bool,
) -> bool {
    let Some(region) = regions.get_mut(path.block) else {
        return false;
    };
    let place = (by.x, by.y, by.width, by.height);
    match (path.line, path.word) {
        (None, None) => {
            (region.x, region.y, region.width, region.height) = place;
            true
        }
        (Some(l), None) => region.lines.get_mut(l).is_some_and(|line| {
            if estimated && line.width > 0.0 {
                let scale = by.width / line.width;
                for word in &mut line.words {
                    word.x = by.x + (word.x - line.x) * scale;
                    word.width *= scale;
                    (word.y, word.height) = (by.y, by.height);
                }
            }
            (line.x, line.y, line.width, line.height) = place;
            true
        }),
        (Some(l), Some(w)) => region
            .lines
            .get_mut(l)
            .and_then(|line| line.words.get_mut(w))
            .is_some_and(|word| {
                (word.x, word.y, word.width, word.height) = place;
                true
            }),
        (None, Some(_)) => false,
    }
}

/// A box as its left, top, right and bottom edges.
type Edges = (f32, f32, f32, f32);

fn edges(x: f32, y: f32, width: f32, height: f32) -> Edges {
    (x, y, x + width, y + height)
}

fn union(a: Edges, b: Edges) -> Edges {
    (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
}

/// Whether `outer` is `inner` widened: an edge moved by more than the arithmetic's hair.
fn grew(outer: Edges, inner: Edges) -> bool {
    const HAIR: f32 = 1e-6;
    inner.0 - outer.0 > HAIR
        || inner.1 - outer.1 > HAIR
        || outer.2 - inner.2 > HAIR
        || outer.3 - inner.3 > HAIR
}

/// Each block and line of `regions` that holds one of `boxes`, or is one, widened just enough to
/// hold its lines or words (see the module's notes); and the geometry the formats read, every
/// box said to be the researcher's or widened for one of theirs. `estimated` and `own` are
/// [`LaidForExport`]'s, `boxes` its boxes at their places in `regions` now.
pub fn contain(
    regions: &mut [TextRegion],
    boxes: &[(RegionPath, DrawnRegion)],
    estimated: Vec<usize>,
    own: &[DrawnRegion],
) -> ResearcherGeometry {
    let mut adjusted: Vec<AdjustedBox> = boxes
        .iter()
        .map(|(at, r)| AdjustedBox {
            at: *at,
            id: r.id.clone(),
            kind: BoxKind::Researcher,
            updated_at: r.updated_at,
        })
        .collect();
    let blocks: BTreeSet<usize> = boxes.iter().map(|(p, _)| p.block).collect();
    for b in blocks {
        let Some(region) = regions.get_mut(b) else {
            continue;
        };
        let within: Vec<&(RegionPath, DrawnRegion)> =
            boxes.iter().filter(|(p, _)| p.block == b).collect();
        let lines: BTreeSet<usize> = within.iter().filter_map(|(p, _)| p.line).collect();
        for l in lines {
            let Some(line) = region.lines.get_mut(l) else {
                continue;
            };
            let before = edges(line.x, line.y, line.width, line.height);
            let after = line
                .words
                .iter()
                .map(|w| edges(w.x, w.y, w.width, w.height))
                .fold(before, union);
            if grew(after, before) {
                (line.x, line.y) = (after.0, after.1);
                (line.width, line.height) = (after.2 - after.0, after.3 - after.1);
                let at = RegionPath {
                    block: b,
                    line: Some(l),
                    word: None,
                };
                widen(&mut adjusted, at, &within, before);
            }
        }
        let before = edges(region.x, region.y, region.width, region.height);
        let after = region
            .lines
            .iter()
            .map(|l| edges(l.x, l.y, l.width, l.height))
            .fold(before, union);
        if grew(after, before) {
            (region.x, region.y) = (after.0, after.1);
            (region.width, region.height) = (after.2 - after.0, after.3 - after.1);
            let at = RegionPath {
                block: b,
                line: None,
                word: None,
            };
            widen(&mut adjusted, at, &within, before);
        }
    }
    adjusted.sort_by_key(|a| a.at);
    ResearcherGeometry {
        adjusted,
        estimated,
        drawn: own
            .iter()
            .map(|r| ResearcherRegion {
                id: r.id.clone(),
                half: r.half,
                region: TextRegion::block(r.x, r.y, r.width, r.height, r.text.clone()),
                updated_at: r.updated_at,
            })
            .collect(),
    }
}

/// Says the box at `at`, `before` it was widened, was: the researcher's own, now widened; or
/// the model's, widened for the first box of theirs within it (of `within`) that it did not hold,
/// else the first within it.
fn widen(
    adjusted: &mut Vec<AdjustedBox>,
    at: RegionPath,
    within: &[&(RegionPath, DrawnRegion)],
    before: Edges,
) {
    if let Some(own) = adjusted.iter_mut().find(|a| a.at == at) {
        own.kind = BoxKind::ResearcherWidened;
        return;
    }
    let inside: Vec<&DrawnRegion> = within
        .iter()
        .filter(|(p, _)| match at.line {
            None => p.line.is_some(),
            Some(l) => p.line == Some(l) && p.word.is_some(),
        })
        .map(|(_, r)| r)
        .collect();
    let outside = inside
        .iter()
        .find(|r| grew(union(before, edges(r.x, r.y, r.width, r.height)), before));
    if let Some(r) = outside.or(inside.first()) {
        adjusted.push(AdjustedBox {
            at,
            id: r.id.clone(),
            kind: BoxKind::ModelWidened,
            updated_at: r.updated_at,
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
