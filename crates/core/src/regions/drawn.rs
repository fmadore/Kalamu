//! The researcher's own regions over the scan (#84, second step): boxes they draw, and the boxes
//! they give an engine's region, kept in a layer of their own beside the stored result, which
//! is never changed.
//!
//! A region is kept by its page's number and half, not by the index of a text piece: the
//! analysis and contents reviews show a page of the source with no text piece for it, and show
//! the same regions. A box adjusting an engine's region belongs to one reading of the page (the
//! stored result's cache key) and one region id in it. When the page is read again, that
//! region no longer exists; the box is then shown as one the researcher drew, so nothing they
//! did is lost.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use super::{Granularity, PageRegions, RegionOrigin, ShownRegion};
use crate::{JobId, PageHalf, SourceId, Timestamp};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

/// How far past the page's edge a box may reach and be clamped to it, as a fraction of the
/// page: pointer arithmetic overshoots by a hair. A box reaching further was not meant for
/// this page, and is refused.
pub const SLACK: f32 = 0.01;
/// The smallest width or height a box may have, as a fraction of the page: about a letter's
/// stroke on a printed page. A smaller box is a click that slipped, not a region.
pub const MIN_SIZE: f32 = 0.004;

/// The engine's region a box adjusts: the stored result it was shown from, and its id there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AdjustedRegion {
    /// The cache key of the stored result ([`PageRegions::result`]).
    pub result: String,
    /// The region's id in that result ([`super::block_id`]).
    pub region: String,
}

/// A region the researcher drew, or the box they gave an engine's region, as the store keeps
/// it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DrawnRegion {
    /// Made by the engine when the region is first saved.
    pub id: String,
    pub job_id: JobId,
    pub source_id: SourceId,
    /// The page's number in the source, from 1, as [`crate::PieceRef::Page`] numbers it.
    pub page: u32,
    /// The half of a split spread the box is drawn on; its fractions are of that half.
    pub half: Option<PageHalf>,
    /// Fractions of the page, or of the half, as the review shows it; see the module's notes
    /// on coordinates.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// What the researcher wrote for it; possibly nothing.
    pub text: String,
    /// The engine's region whose box this replaces; none for a region drawn on its own.
    pub adjusts: Option<AdjustedRegion>,
    /// The source's content hash when the region was saved: the version of the file the box
    /// was drawn on.
    pub source_hash: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DrawnRegion {
    /// A new region's id.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Whether the region lies on page `page`, or on that half of it.
    pub fn on(&self, page: u32, half: Option<PageHalf>) -> bool {
        self.page == page && self.half == half
    }

    /// The region as the review draws it among the researcher's own.
    pub fn shown(&self) -> ShownRegion {
        ShownRegion {
            id: self.id.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            text: self.text.clone(),
            granularity: Granularity::Block,
            origin: RegionOrigin::Manual,
            span: None,
            adjusted_by: None,
        }
    }
}

/// What the interface sends to save a region: a new one, or a change to one saved (`id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegionDraft {
    /// The region to change; none for a new one.
    pub id: Option<String>,
    pub job_id: JobId,
    pub source_id: SourceId,
    pub page: u32,
    pub half: Option<PageHalf>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub adjusts: Option<AdjustedRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RegionError {
    #[error("The region's box is not a number")]
    NotFinite,
    #[error("The region reaches outside the page")]
    OutsidePage,
    #[error("The region is too small")]
    TooSmall,
    /// A page the source does not have, or a half of a page that was not split.
    #[error("The source has no such page")]
    NoPage,
    /// The region it adjusts is not on this page as it was read, or that reading is gone.
    #[error("The region it adjusts is no longer on this page")]
    Adjusts,
    #[error("The region was not found")]
    NotFound,
}

impl RegionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFinite => "region.not_finite",
            Self::OutsidePage => "region.outside",
            Self::TooSmall => "region.too_small",
            Self::NoPage => "region.no_page",
            Self::Adjusts => "region.adjusts",
            Self::NotFound => "region.not_found",
        }
    }
}

/// A draft's box checked and made tidy: drawn in any direction, it is given from its top-left
/// corner; reaching past the page by up to [`SLACK`], it is clamped to it. Refused when a value
/// is not a number, the box reaches further, or it is smaller than [`MIN_SIZE`] either way, or
/// the page is numbered 0.
pub fn check(draft: &RegionDraft) -> Result<RegionDraft, RegionError> {
    let (x, y, w, h) = (draft.x, draft.y, draft.width, draft.height);
    if ![x, y, w, h].iter().all(|v| v.is_finite()) {
        return Err(RegionError::NotFinite);
    }
    if draft.page == 0 {
        return Err(RegionError::NoPage);
    }
    let (x0, x1) = (x.min(x + w), x.max(x + w));
    let (y0, y1) = (y.min(y + h), y.max(y + h));
    let low = -SLACK;
    let high = 1.0 + SLACK;
    if x0 < low || y0 < low || x1 > high || y1 > high {
        return Err(RegionError::OutsidePage);
    }
    let (x0, y0) = (x0.max(0.0), y0.max(0.0));
    let (x1, y1) = (x1.min(1.0), y1.min(1.0));
    if x1 - x0 < MIN_SIZE || y1 - y0 < MIN_SIZE {
        return Err(RegionError::TooSmall);
    }
    Ok(RegionDraft {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
        ..draft.clone()
    })
}

impl PageRegions {
    /// Whether the engine's regions on this page hold one with this id, at any granularity.
    pub fn holds(&self, id: &str) -> bool {
        self.blocks
            .iter()
            .chain(&self.lines)
            .chain(&self.words)
            .any(|r| r.id == id)
    }

    /// The researcher's regions laid on the page. A box adjusting one of the regions shown
    /// (the same result, the same id) takes that region's place, its text and its place in the
    /// text kept, its origin the researcher's and `adjusted_by` the box's id; any other is shown
    /// among `drawn`. `regions`
    /// are this page's, in the order they were made.
    pub fn with_drawn(mut self, regions: &[DrawnRegion]) -> Self {
        for region in regions {
            let applied = match (&region.adjusts, &self.result) {
                (Some(adjusts), Some(result)) if adjusts.result == *result => {
                    self.adjust(&adjusts.region, region)
                }
                _ => false,
            };
            if !applied {
                self.drawn.push(region.shown());
            }
        }
        self
    }

    /// Gives region `id` the researcher's box; false when the page has no such region.
    fn adjust(&mut self, id: &str, by: &DrawnRegion) -> bool {
        for list in [&mut self.blocks, &mut self.lines, &mut self.words] {
            if let Some(shown) = list.iter_mut().find(|r| r.id == id) {
                shown.x = by.x;
                shown.y = by.y;
                shown.width = by.width;
                shown.height = by.height;
                shown.origin = RegionOrigin::Manual;
                shown.adjusted_by = Some(by.id.clone());
                return true;
            }
        }
        false
    }
}
