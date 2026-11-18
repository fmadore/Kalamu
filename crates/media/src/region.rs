//! A region of a page cut from the original at full resolution, for a model to read that part
//! again (#84).
//!
//! The box is given as the review gives it (`kalamu_core::regions`): fractions (0 to 1) of the
//! piece as shown, from its top-left corner. A PDF page is shown with its own `/Rotate`
//! applied; one half of a split spread is shown as that half alone; an image is shown turned
//! upright by its EXIF orientation. The box is cut exactly, clamped to the page: any margin is
//! the caller's to add.

use thiserror::Error;

/// The fewest pixels a region may have on a side once cut: below this a model has nothing to
/// read, and a box drawn by a slip of the mouse is refused rather than sent.
pub const MIN_REGION_PIXELS: u32 = 8;

/// A box on a piece, in fractions of its width and height as the review shows it, from its
/// top-left corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crop {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Why a region cannot be cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegionRefusal {
    #[error("The region could not be read: its position or size is not a number")]
    NotFinite,
    #[error("The region lies outside the page. Draw it again over the part to read")]
    OutsidePage,
    #[error(
        "The region is too small to read: draw it at least {MIN_REGION_PIXELS} pixels wide and \
         high on the scan"
    )]
    TooSmall,
    #[error("Only a page of a PDF or an image can be read again by region")]
    NoPage,
    #[error(
        "This model does not read images, and a region is sent as one. Choose a model that reads \
         page images"
    )]
    NoImages,
}

impl RegionRefusal {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFinite => "media.region_not_finite",
            Self::OutsidePage => "media.region_outside_page",
            Self::TooSmall => "media.region_too_small",
            Self::NoPage => "media.region_no_page",
            Self::NoImages => "media.region_no_images",
        }
    }
}

impl Crop {
    /// The box clamped to the piece, as `[left, top, right, bottom]` fractions; refused when a
    /// value is not finite or nothing of it lies on the piece.
    pub(crate) fn clamped(&self) -> Result<[f32; 4], RegionRefusal> {
        let Self {
            x,
            y,
            width,
            height,
        } = *self;
        if ![x, y, width, height].iter().all(|v| v.is_finite()) {
            return Err(RegionRefusal::NotFinite);
        }
        let (left, right) = (x.min(x + width), x.max(x + width));
        let (top, bottom) = (y.min(y + height), y.max(y + height));
        let left = left.max(0.0);
        let top = top.max(0.0);
        let right = right.min(1.0);
        let bottom = bottom.min(1.0);
        if left >= right || top >= bottom {
            return Err(RegionRefusal::OutsidePage);
        }
        Ok([left, top, right, bottom])
    }
}

/// The part of a PDF page box `[x0, y0, x1, y1]` (points, y upwards) that the fractions
/// `[left, top, right, bottom]` of the page as shown cover, given the page's rotation
/// (clockwise, in degrees), as `half_box` turns a side of the screen into a half of the box.
pub(crate) fn region_box(
    [x0, y0, x1, y1]: [f32; 4],
    rotation: i64,
    [left, top, right, bottom]: [f32; 4],
) -> [f32; 4] {
    let (w, h) = (x1 - x0, y1 - y0);
    let (ax, bx, ay, by) = match rotation {
        // Turned a quarter clockwise: the page's left edge is at the top of the screen, its
        // bottom edge at the left.
        90 => (x0 + top * w, x0 + bottom * w, y0 + left * h, y0 + right * h),
        180 => (x1 - right * w, x1 - left * w, y0 + top * h, y0 + bottom * h),
        // Turned a quarter anticlockwise: the page's left edge at the bottom, its top at the
        // left.
        270 => (x1 - bottom * w, x1 - top * w, y1 - right * h, y1 - left * h),
        _ => (x0 + left * w, x0 + right * w, y1 - bottom * h, y1 - top * h),
    };
    [ax, ay, bx, by]
}

/// The pixels `[left, top, right, bottom]` fractions cover on a picture `w` × `h` pixels:
/// `(x, y, width, height)`, at least one pixel each way and within the picture.
pub(crate) fn pixel_box((w, h): (u32, u32), [left, top, right, bottom]: [f32; 4]) -> [u32; 4] {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a fraction from 0 to 1 of a picture's side, rounded"
    )]
    let at = |fraction: f32, side: u32| ((fraction * side as f32).round() as u32).min(side);
    let (x, y) = (at(left, w), at(top, h));
    let (x, y) = (x.min(w.saturating_sub(1)), y.min(h.saturating_sub(1)));
    let width = at(right, w).saturating_sub(x).max(1);
    let height = at(bottom, h).saturating_sub(y).max(1);
    [x, y, width, height]
}
