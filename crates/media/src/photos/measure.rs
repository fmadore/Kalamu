//! What one photograph gives the checks: its sharpness, where its writing is densest, and the
//! small grey copies the comparisons read. Each photograph is decoded once, reduced, and let go.

use std::path::Path;

use super::grey::{Grey, fit};
use crate::{ImageSource, MediaError};

/// The long side a photograph is reduced to before its sharpness is measured: about what a
/// model is given of a page (Claude scales larger images down to 1,568 pixels), so blur too
/// fine to survive that reduction, which a model would not see either, is not reported.
pub(crate) const WORK_SIDE: usize = 1_536;

/// The long side of the grey copy two photographs are compared on.
pub(crate) const COMPARE_LONG: usize = 384;
/// Its short side: every copy has the same size, so two photographs of one camera line up.
pub(crate) const COMPARE_SHORT: usize = 288;

/// The size of the smallest copy, read for every pair of neighbours to find candidates.
pub(crate) const TINY_LONG: usize = 32;
pub(crate) const TINY_SHORT: usize = 24;

/// The share of a photograph's strongest edges its sharpness is read from. A page of writing
/// has a few per cent of its pixels on strokes; the strongest 0.5% are strokes on any page that
/// holds some writing, and not the paper's grain.
const EDGE_SHARE: f32 = 0.005;

/// Below this contrast at its strongest edges (in grey levels of 255, at an eighth of the
/// working size), a photograph holds too little to judge: a blank page is never called
/// blurred.
pub(crate) const CONTENT_FLOOR: f32 = 10.0;

/// One photograph, measured.
#[derive(Debug, Clone)]
pub(crate) struct Measured {
    /// Wider than tall: two photographs are compared only when both are, or both are not.
    pub landscape: bool,
    /// The compared copy, `COMPARE_LONG` × `COMPARE_SHORT` or turned, as bytes.
    pub compare: Vec<u8>,
    /// The smallest copy, brought to a mean of 0 and a spread of 1.
    pub tiny: Grey,
    /// A 64-bit difference hash of the page's layout.
    pub hash: u64,
    /// How sharp the page is; `None` when it holds too little to judge.
    pub sharpness: Option<f32>,
    /// Where the writing is densest, in fractions of the width and height: the place shown
    /// close up.
    pub focus: (f32, f32),
}

impl Measured {
    pub fn compare_size(&self) -> (usize, usize) {
        if self.landscape {
            (COMPARE_LONG, COMPARE_SHORT)
        } else {
            (COMPARE_SHORT, COMPARE_LONG)
        }
    }

    pub fn compare_grey(&self) -> Grey {
        let (w, h) = self.compare_size();
        Grey::from_u8(w, h, &self.compare)
    }
}

/// Decodes the photograph at `path` upright (its EXIF orientation applied), reduces it to
/// [`WORK_SIDE`] straight from its grey bytes, lets the decoded image go, and measures it.
/// (Decoding a JPEG straight to grey with `zune-jpeg`, the decoder `image` uses, was tried and
/// saved nothing measurable.)
pub(crate) fn measure_file(path: &Path) -> Result<Measured, MediaError> {
    let source = ImageSource::open(path)?;
    if !source.is_decodable() || source.page_count() != 1 {
        return Err(MediaError::UnsupportedKind);
    }
    let luma = source.upright(1, None)?.into_luma8();
    drop(source);
    let (w, h) = (luma.width() as usize, luma.height() as usize);
    let (ww, wh) = fit(w, h, WORK_SIDE);
    let work = super::grey::reduce(luma.as_raw(), w, h, ww, wh);
    drop(luma);
    Ok(measure(&work))
}

/// Measures a photograph already reduced to at most [`WORK_SIDE`].
pub(crate) fn measure(work: &Grey) -> Measured {
    let landscape = work.w > work.h;
    let (cw, ch) = if landscape {
        (COMPARE_LONG, COMPARE_SHORT)
    } else {
        (COMPARE_SHORT, COMPARE_LONG)
    };
    let compare = work.reduce(cw, ch);
    let (tw, th) = if landscape {
        (TINY_LONG, TINY_SHORT)
    } else {
        (TINY_SHORT, TINY_LONG)
    };
    let tiny = compare.reduce(tw, th).normalised();
    let hash = difference_hash(&compare);
    let (sharpness, focus) = sharpness(work);
    Measured {
        landscape,
        compare: compare.to_u8(),
        tiny,
        hash,
        sharpness,
        focus,
    }
}

/// Each pixel's step to its right, and its step to the pixel below.
fn steps(g: &Grey) -> (Vec<f32>, Vec<f32>) {
    let mut across = vec![0.0f32; g.w * g.h];
    let mut down = vec![0.0f32; g.w * g.h];
    for y in 0..g.h.saturating_sub(1) {
        for x in 0..g.w.saturating_sub(1) {
            let v = g.at(x, y);
            across[y * g.w + x] = (g.at(x + 1, y) - v).abs();
            down[y * g.w + x] = (g.at(x, y + 1) - v).abs();
        }
    }
    (across, down)
}

/// The value under which all but `share` of `values` fall (values of 0 to 255).
fn top_share(values: &[f32], share: f32) -> f32 {
    const BINS: usize = 1024;
    let mut hist = [0u32; BINS];
    for &v in values {
        let bin = ((v * 4.0) as usize).min(BINS - 1);
        hist[bin] += 1;
    }
    let wanted = (values.len() as f32 * share).max(1.0) as u32;
    let mut seen = 0u32;
    for bin in (0..BINS).rev() {
        seen += hist[bin];
        if seen >= wanted {
            return bin as f32 / 4.0;
        }
    }
    0.0
}

/// How sharp a page is, from its strongest edges at two sizes: at the working size and at
/// half of it. A sharp stroke steps from ink to paper within a pixel at both sizes, so its
/// steepest step is about the same at both (a ratio near 1, more with the paper's grain); a
/// blurred stroke spreads its step over several pixels, which halving the size halves, so its
/// steps are twice as steep at half the size (a ratio towards 0.5). The ratio does not depend
/// on how much is written, nor on the light or the ink's contrast. Steps across and steps
/// down are read apart and the lower ratio kept, since a hand that moved blurs only the edges
/// across its movement.
///
/// `None` when the page holds too little to judge: its strongest edges at an eighth of the
/// working size, where no blur a photograph keeps its page legible through still shows, are
/// fainter than [`CONTENT_FLOOR`]. Also gives where the strongest edges are densest.
pub(crate) fn sharpness(work: &Grey) -> (Option<f32>, (f32, f32)) {
    let (fine_x, fine_y) = steps(work);
    let half = work.reduce(work.w.div_ceil(2), work.h.div_ceil(2));
    let (coarse_x, coarse_y) = steps(&half);
    let both: Vec<f32> = fine_x.iter().zip(&fine_y).map(|(a, b)| a.max(*b)).collect();
    let focus = densest(&both, work.w, work.h);
    if content(work) < CONTENT_FLOOR {
        return (None, focus);
    }
    let ratio = |fine: &[f32], coarse: &[f32]| {
        let coarse = top_share(coarse, EDGE_SHARE);
        (coarse > 0.0).then(|| top_share(fine, EDGE_SHARE) / coarse)
    };
    let sharp = match (ratio(&fine_x, &coarse_x), ratio(&fine_y, &coarse_y)) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (one, other) => one.or(other),
    };
    (sharp, focus)
}

/// How much a page holds: the contrast of its strongest edges at an eighth of the working
/// size, in grey levels.
pub(crate) fn content(work: &Grey) -> f32 {
    let eighth = work.reduce(work.w.div_ceil(8), work.h.div_ceil(8));
    let (x, y) = steps(&eighth);
    top_share(&x, EDGE_SHARE).max(top_share(&y, EDGE_SHARE))
}

/// The centre of the eighth by eighth of the page where the steps are strongest, in fractions.
fn densest(steps: &[f32], w: usize, h: usize) -> (f32, f32) {
    const GRID: usize = 8;
    let mut cells = [0.0f64; GRID * GRID];
    for y in 0..h {
        let cy = y * GRID / h.max(1);
        for x in 0..w {
            let cx = x * GRID / w.max(1);
            cells[cy * GRID + cx] += f64::from(steps[y * w + x]);
        }
    }
    let best = cells
        .iter()
        .enumerate()
        .fold(
            (0, f64::MIN),
            |acc, (i, &v)| if v > acc.1 { (i, v) } else { acc },
        )
        .0;
    (
        ((best % GRID) as f32 + 0.5) / GRID as f32,
        ((best / GRID) as f32 + 0.5) / GRID as f32,
    )
}

/// A difference hash: the page reduced to 9 × 8, each bit saying whether a cell is lighter
/// than its right neighbour. Retakes of a page differ in a few bits.
fn difference_hash(g: &Grey) -> u64 {
    let small = g.reduce(9, 8);
    let mut hash = 0u64;
    for y in 0..8 {
        for x in 0..8 {
            hash <<= 1;
            if small.at(x, y) > small.at(x + 1, y) {
                hash |= 1;
            }
        }
    }
    hash
}
