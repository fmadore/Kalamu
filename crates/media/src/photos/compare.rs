//! Whether two photographs show the same page: the second laid over the first (a small turn,
//! scale and shift found from coarse to fine), then their detail compared tile by tile, so that
//! two pages of one register, alike in their ruling and layout but not in what is written on
//! them, are told apart.

use super::grey::Grey;
use super::measure::Measured;

/// Turns tried at the coarsest size, in degrees: a page photographed again by hand turns by a
/// degree or two.
const ANGLES: [f32; 7] = [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0];
/// Scales tried at the coarsest size: the hand a little nearer or farther.
const SCALES: [f32; 3] = [0.95, 1.0, 1.05];
/// Shifts tried at the coarsest size (36 × 48 pixels), in its pixels: about a tenth of the page.
const COARSE_SHIFT: i32 = 4;
/// Tiles across the long side of the compared copy, and across the short side.
const TILES_LONG: usize = 8;
const TILES_SHORT: usize = 6;
/// The radius of the box whose average is taken away to keep a page's detail (strokes and
/// letters) on the compared copy.
const DETAIL_RADIUS: usize = 2;
/// A photograph whose sharpness is below this is soft enough for both photographs to be
/// softened before their writing is compared, when the other is sharper by [`SOFTER_BY`].
const SOFT: f32 = 0.75;
const SOFTER_BY: f32 = 0.1;
/// How far, in pixels of the compared copy, each tile may still move to fit (the page is not
/// flat, the phone not square to it).
const TILE_SHIFT: i32 = 2;
/// Half the length, in pixels of the compared copy, along which ruled lines are averaged away
/// before tiles are compared: the ruling of a register or a printed form is the same on every
/// page, and would otherwise make two pages of it look alike.
const LINE_RADIUS: usize = 12;

/// A tile holds writing when its detail is at least this strong, relative to the page's.
const TILE_CONTENT: f32 = 0.6;
/// A tile with writing on one photograph and next to none at the same place on the other.
const TILE_BALANCE: f32 = 0.35;
/// Two tiles whose detail correlates less than this do not show the same writing.
pub(crate) const TILE_MATCH: f32 = 0.5;
/// At most this share of the tiles with writing may disagree for two photographs to be the
/// same page.
pub(crate) const MAX_MISMATCH: f32 = 0.25;
/// And the tiles with writing must correlate this well, at the median.
pub(crate) const MIN_MEDIAN: f32 = 0.45;
/// Fewer tiles with writing than this, and two photographs are not judged: a blank page looks
/// like every other blank page.
const MIN_CONTENT_TILES: usize = 6;

/// How alike the smallest copies of two photographs are at their best shift of up to two
/// pixels (a correlation, -1 to 1).
pub(crate) fn tiny_likeness(a: &Measured, b: &Measured) -> f32 {
    if a.landscape != b.landscape {
        return -1.0;
    }
    best_shift(&a.tiny, &b.tiny, 2).0
}

/// What laying one photograph over the other found.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Verdict {
    pub same: bool,
    /// Tiles holding writing on either photograph.
    pub content_tiles: usize,
    /// Of those, tiles that do not show the same writing.
    pub mismatched: usize,
    /// Of those, tiles with writing on one photograph and next to none on the other.
    pub unbalanced: usize,
    /// The median correlation of the tiles with writing.
    pub median: f32,
}

/// Whether `a` and `b` show the same page.
pub(crate) fn compare(a: &Measured, b: &Measured) -> Verdict {
    let not_judged = Verdict {
        same: false,
        content_tiles: 0,
        mismatched: 0,
        unbalanced: 0,
        median: 0.0,
    };
    if a.landscape != b.landscape {
        return not_judged;
    }
    let ga = a.compare_grey();
    let gb = b.compare_grey();
    let (angle, scale, shift) = align(&ga, &gb);
    let laid = gb.warp(angle, scale, shift);
    // A blurred retake beside a sharp take: both are softened, so that their writing compares
    // rather than their sharpness.
    let soft = match (a.sharpness, b.sharpness) {
        (Some(x), Some(y)) => x.min(y) < SOFT && (x - y).abs() >= SOFTER_BY,
        _ => false,
    };
    let (ga, laid) = if soft {
        (ga.smoothed(1), laid.smoothed(1))
    } else {
        (ga, laid)
    };
    let writing = |g: &Grey| g.detail(DETAIL_RADIUS).without_lines(LINE_RADIUS);
    tiles(&writing(&ga), &writing(&laid))
}

/// The turn, scale and shift (in pixels of the compared copy) that lay `b` over `a`: tried
/// widely at an eighth of the size, then narrowed at a quarter and at a half.
fn align(a: &Grey, b: &Grey) -> (f32, f32, (f32, f32)) {
    let pyramid = |g: &Grey, k: usize| g.reduce(g.w / k, g.h / k).normalised();
    let (a8, b8) = (pyramid(a, 8), pyramid(b, 8));
    // The three best turns and scales at the coarsest size, each with its best shift.
    let mut coarse: Vec<(f32, f32, f32, (i32, i32))> = Vec::new();
    for &angle in &ANGLES {
        for &scale in &SCALES {
            let warped = b8.warp(angle, scale, (0.0, 0.0));
            let (score, shift) = best_shift(&a8, &warped, COARSE_SHIFT);
            coarse.push((score, angle, scale, shift));
        }
    }
    coarse.sort_by(|x, y| y.0.total_cmp(&x.0));
    coarse.truncate(3);

    let mut best = (f32::MIN, 0.0, 1.0, (0.0f32, 0.0f32));
    let (a4, b4) = (pyramid(a, 4), pyramid(b, 4));
    for &(_, angle, scale, (sx, sy)) in &coarse {
        for da in [-0.5f32, 0.0, 0.5] {
            for ds in [-0.025f32, 0.0, 0.025] {
                let (angle, scale) = (angle + da, scale + ds);
                // The coarse shift, doubled to this size, is applied by the warp; what is left
                // is found by shifting.
                let start = (2.0 * sx as f32, 2.0 * sy as f32);
                let warped = b4.warp(angle, scale, start);
                let (score, (dx, dy)) = best_shift(&a4, &warped, 2);
                if score > best.0 {
                    best = (
                        score,
                        angle,
                        scale,
                        (start.0 + dx as f32, start.1 + dy as f32),
                    );
                }
            }
        }
    }
    let (_, angle, scale, shift4) = best;
    let (a2, b2) = (pyramid(a, 2), pyramid(b, 2));
    let mut fine = (f32::MIN, angle, scale, (2.0 * shift4.0, 2.0 * shift4.1));
    for da in [-0.25f32, 0.0, 0.25] {
        for ds in [-0.01f32, 0.0, 0.01] {
            let start = (2.0 * shift4.0, 2.0 * shift4.1);
            let warped = b2.warp(angle + da, scale + ds, start);
            let (score, (dx, dy)) = best_shift(&a2, &warped, 2);
            if score > fine.0 {
                fine = (
                    score,
                    angle + da,
                    scale + ds,
                    (start.0 + dx as f32, start.1 + dy as f32),
                );
            }
        }
    }
    let (_, angle, scale, shift2) = fine;
    (angle, scale, (2.0 * shift2.0, 2.0 * shift2.1))
}

/// The best correlation of `a` with `b` moved by up to `reach` pixels each way, and that move
/// (as `b`'s pixel read for `a`'s: `b(x + dx, y + dy)`). Unfilled pixels are skipped.
pub(crate) fn best_shift(a: &Grey, b: &Grey, reach: i32) -> (f32, (i32, i32)) {
    let mut best = (f32::MIN, (0, 0));
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let score = correlation(a, b, (0, 0, a.w, a.h), (dx, dy));
            if score > best.0 {
                best = (score, (dx, dy));
            }
        }
    }
    best
}

/// The correlation of `a` over the rectangle `(x0, y0, x1, y1)` with `b` read `shift` away.
/// Pixels unfilled in either, or read outside `b`, are skipped; fewer than half the rectangle's
/// pixels left, and it is 0.
fn correlation(a: &Grey, b: &Grey, rect: (usize, usize, usize, usize), shift: (i32, i32)) -> f32 {
    let (x0, y0, x1, y1) = rect;
    let (dx, dy) = shift;
    let (mut n, mut sa, mut sb, mut sab, mut saa, mut sbb) = (0u32, 0.0, 0.0, 0.0, 0.0, 0.0);
    for y in y0..y1 {
        let by = y as i32 + dy;
        if by < 0 || by >= b.h as i32 {
            continue;
        }
        let a_row = &a.px[y * a.w..(y + 1) * a.w];
        let b_row = &b.px[by as usize * b.w..(by as usize + 1) * b.w];
        for (x, &va) in a_row.iter().enumerate().take(x1).skip(x0) {
            let bx = x as i32 + dx;
            if bx < 0 || bx >= b.w as i32 {
                continue;
            }
            let vb = b_row[bx as usize];
            if va.is_nan() || vb.is_nan() {
                continue;
            }
            let (va, vb) = (f64::from(va), f64::from(vb));
            n += 1;
            sa += va;
            sb += vb;
            sab += va * vb;
            saa += va * va;
            sbb += vb * vb;
        }
    }
    let area = ((x1 - x0) * (y1 - y0)) as u32;
    if n < area / 2 || n == 0 {
        return 0.0;
    }
    let nf = f64::from(n);
    let cov = sab - sa * sb / nf;
    let va = saa - sa * sa / nf;
    let vb = sbb - sb * sb / nf;
    if va <= 1e-9 || vb <= 1e-9 {
        return 0.0;
    }
    (cov / (va * vb).sqrt()) as f32
}

/// The root mean square of the filled pixels of `g` in a rectangle, and how many were filled.
fn energy(g: &Grey, (x0, y0, x1, y1): (usize, usize, usize, usize)) -> (f32, usize) {
    let (mut n, mut s) = (0usize, 0.0f64);
    for y in y0..y1 {
        for x in x0..x1 {
            let v = g.at(x, y);
            if !v.is_nan() {
                n += 1;
                s += f64::from(v) * f64::from(v);
            }
        }
    }
    (((s / n.max(1) as f64).sqrt()) as f32, n)
}

/// Compares the detail of two photographs laid one over the other, tile by tile.
fn tiles(a: &Grey, b: &Grey) -> Verdict {
    let (whole_a, _) = energy(a, (0, 0, a.w, a.h));
    let (whole_b, _) = energy(b, (0, 0, b.w, b.h));
    let (cols, rows) = if a.w > a.h {
        (TILES_LONG, TILES_SHORT)
    } else {
        (TILES_SHORT, TILES_LONG)
    };
    let (tw, th) = (a.w / cols, a.h / rows);
    let mut scores = Vec::new();
    let mut mismatched = 0;
    let mut unbalanced = 0;
    for row in 0..rows {
        for col in 0..cols {
            let rect = (col * tw, row * th, (col + 1) * tw, (row + 1) * th);
            let (ea, na) = energy(a, rect);
            let (eb, nb) = energy(b, rect);
            // A tile mostly outside the other photograph after laying it over is left out.
            if na.min(nb) < tw * th * 4 / 5 {
                continue;
            }
            let (ea, eb) = (ea / whole_a.max(1e-6), eb / whole_b.max(1e-6));
            if ea.max(eb) < TILE_CONTENT {
                continue;
            }
            if ea.min(eb) / ea.max(eb) < TILE_BALANCE {
                // At the edge, one photograph may simply show less of the page (the table
                // under it): that tile says nothing either way.
                if row == 0 || col == 0 || row == rows - 1 || col == cols - 1 {
                    continue;
                }
                mismatched += 1;
                unbalanced += 1;
                scores.push(0.0);
                continue;
            }
            let mut best = f32::MIN;
            for dy in -TILE_SHIFT..=TILE_SHIFT {
                for dx in -TILE_SHIFT..=TILE_SHIFT {
                    best = best.max(correlation(a, b, rect, (dx, dy)));
                }
            }
            if best < TILE_MATCH {
                mismatched += 1;
            }
            scores.push(best);
        }
    }
    let content_tiles = scores.len();
    scores.sort_by(f32::total_cmp);
    let median = scores.get(content_tiles / 2).copied().unwrap_or(0.0);
    let same = content_tiles >= MIN_CONTENT_TILES
        && (mismatched as f32) <= MAX_MISMATCH * content_tiles as f32
        && median >= MIN_MEDIAN;
    Verdict {
        same,
        content_tiles,
        mismatched,
        unbalanced,
        median,
    }
}
