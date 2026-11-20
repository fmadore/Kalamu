//! A grey image of floats and the few operations the photograph checks need: reducing by
//! averaging, warping by a small turn, scale and shift, and smoothing with a box.

/// A grey image, row by row, one float a pixel (0 to 255 as decoded). A pixel a warp could not
/// fill is NaN, and every measure skips it.
#[derive(Debug, Clone)]
pub(crate) struct Grey {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

impl Grey {
    pub fn new(w: usize, h: usize, px: Vec<f32>) -> Self {
        debug_assert_eq!(px.len(), w * h);
        Self { w, h, px }
    }

    pub fn from_u8(w: usize, h: usize, px: &[u8]) -> Self {
        Self::new(w, h, px.iter().map(|&v| f32::from(v)).collect())
    }

    #[inline]
    pub fn at(&self, x: usize, y: usize) -> f32 {
        self.px[y * self.w + x]
    }

    /// The image as bytes, rounded and clamped; NaN becomes mid-grey.
    pub fn to_u8(&self) -> Vec<u8> {
        self.px
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    128
                } else {
                    v.round().clamp(0.0, 255.0) as u8
                }
            })
            .collect()
    }

    /// The image reduced to `dw` × `dh` by averaging every source pixel under each new one
    /// (an area average, weighted at the edges of each footprint), which is how a camera's own
    /// pixels would have seen the page at that size. Never enlarges detail.
    pub fn reduce(&self, dw: usize, dh: usize) -> Grey {
        reduce(&self.px, self.w, self.h, dw, dh)
    }

    /// The image seen through a similarity: the pixel at `p` of the result is this image's
    /// pixel at `scale · turn(angle) · (p − centre) + centre + shift`, read between pixels.
    /// Pixels that fall outside are NaN.
    pub fn warp(&self, angle_deg: f32, scale: f32, shift: (f32, f32)) -> Grey {
        let (sin, cos) = angle_deg.to_radians().sin_cos();
        let (cx, cy) = ((self.w as f32 - 1.0) / 2.0, (self.h as f32 - 1.0) / 2.0);
        let (a, b) = (scale * cos, scale * sin);
        let mut out = vec![f32::NAN; self.w * self.h];
        for y in 0..self.h {
            let dy = y as f32 - cy;
            for x in 0..self.w {
                let dx = x as f32 - cx;
                let sx = a * dx - b * dy + cx + shift.0;
                let sy = b * dx + a * dy + cy + shift.1;
                out[y * self.w + x] = self.bilinear(sx, sy);
            }
        }
        Grey::new(self.w, self.h, out)
    }

    fn bilinear(&self, x: f32, y: f32) -> f32 {
        if x < 0.0 || y < 0.0 || x > (self.w - 1) as f32 || y > (self.h - 1) as f32 {
            return f32::NAN;
        }
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let top = self.at(x0, y0) * (1.0 - fx) + self.at(x1, y0) * fx;
        let bottom = self.at(x0, y1) * (1.0 - fx) + self.at(x1, y1) * fx;
        top * (1.0 - fy) + bottom * fy
    }

    /// The image minus its average over a `(2r+1)`-pixel square: the detail of strokes and
    /// letters, without the paper's shading or the exposure. NaN stays NaN.
    pub fn detail(&self, r: usize) -> Grey {
        let (w, h) = (self.w, self.h);
        // Integral images of the values and of how many pixels are filled.
        let stride = w + 1;
        let mut sum = vec![0.0f64; stride * (h + 1)];
        let mut count = vec![0u32; stride * (h + 1)];
        for y in 0..h {
            let mut row_sum = 0.0f64;
            let mut row_count = 0u32;
            for x in 0..w {
                let v = self.at(x, y);
                if !v.is_nan() {
                    row_sum += f64::from(v);
                    row_count += 1;
                }
                let i = (y + 1) * stride + x + 1;
                sum[i] = sum[i - stride] + row_sum;
                count[i] = count[i - stride] + row_count;
            }
        }
        let mut out = vec![f32::NAN; w * h];
        for y in 0..h {
            let (y0, y1) = (y.saturating_sub(r), (y + r + 1).min(h));
            for x in 0..w {
                let v = self.at(x, y);
                if v.is_nan() {
                    continue;
                }
                let (x0, x1) = (x.saturating_sub(r), (x + r + 1).min(w));
                let box_sum = sum[y1 * stride + x1] - sum[y0 * stride + x1] - sum[y1 * stride + x0]
                    + sum[y0 * stride + x0];
                // Added before subtracting, so no step goes below zero.
                let n = (count[y1 * stride + x1] + count[y0 * stride + x0])
                    - count[y0 * stride + x1]
                    - count[y1 * stride + x0];
                out[y * w + x] = v - (box_sum / f64::from(n.max(1))) as f32;
            }
        }
        Grey::new(w, h, out)
    }

    /// The image averaged over a `(2r+1)`-pixel square around each pixel. NaN stays NaN.
    pub fn smoothed(&self, r: usize) -> Grey {
        let detail = self.detail(r);
        let px = self.px.iter().zip(&detail.px).map(|(v, d)| v - d).collect();
        Grey::new(self.w, self.h, px)
    }

    /// The image without what runs straight across it or straight down it: each pixel minus
    /// the average of the `2r+1` pixels around it along its row, then along its column. A
    /// ruled line, which is the same along its length, goes; handwriting and print, which
    /// change from one letter to the next, stay. NaN stays NaN.
    pub fn without_lines(&self, r: usize) -> Grey {
        let across = self.minus_run_average(r, true);
        across.minus_run_average(r, false)
    }

    fn minus_run_average(&self, r: usize, along_rows: bool) -> Grey {
        let (len, lines) = if along_rows {
            (self.w, self.h)
        } else {
            (self.h, self.w)
        };
        let index = |line: usize, i: usize| {
            if along_rows {
                line * self.w + i
            } else {
                i * self.w + line
            }
        };
        let mut out = self.px.clone();
        let mut sum = vec![0.0f64; len + 1];
        let mut count = vec![0u32; len + 1];
        for line in 0..lines {
            for i in 0..len {
                let v = self.px[index(line, i)];
                let (s, c) = if v.is_nan() {
                    (0.0, 0)
                } else {
                    (f64::from(v), 1)
                };
                sum[i + 1] = sum[i] + s;
                count[i + 1] = count[i] + c;
            }
            for i in 0..len {
                let at = index(line, i);
                if out[at].is_nan() {
                    continue;
                }
                let (i0, i1) = (i.saturating_sub(r), (i + r + 1).min(len));
                let n = count[i1] - count[i0];
                out[at] -= ((sum[i1] - sum[i0]) / f64::from(n.max(1))) as f32;
            }
        }
        Grey::new(self.w, self.h, out)
    }

    /// The image with its filled pixels brought to a mean of 0 and a spread of 1, so two
    /// photographs taken in different light compare; a flat image stays all zeros.
    pub fn normalised(&self) -> Grey {
        let (mut n, mut s, mut s2) = (0.0f64, 0.0f64, 0.0f64);
        for &v in &self.px {
            if !v.is_nan() {
                n += 1.0;
                s += f64::from(v);
                s2 += f64::from(v) * f64::from(v);
            }
        }
        let mean = s / n.max(1.0);
        let sd = (s2 / n.max(1.0) - mean * mean).max(0.0).sqrt();
        let px = self
            .px
            .iter()
            .map(|&v| {
                if v.is_nan() {
                    v
                } else if sd < 1e-6 {
                    0.0
                } else {
                    ((f64::from(v) - mean) / sd) as f32
                }
            })
            .collect();
        Grey::new(self.w, self.h, px)
    }
}

/// Grey pixels `w` × `h` (bytes as decoded, or floats) reduced to `dw` × `dh` by averaging:
/// see [`Grey::reduce`]. Reading bytes directly spares a copy of a whole photograph in floats.
pub(crate) fn reduce<T: Copy + Into<f32>>(
    px: &[T],
    w: usize,
    h: usize,
    dw: usize,
    dh: usize,
) -> Grey {
    let dw = dw.max(1);
    let dh = dh.max(1);
    let cols = spans(w, dw);
    let rows = spans(h, dh);
    // Columns first, into a buffer as tall as the source.
    let mut narrow = vec![0.0f32; dw * h];
    let mut row = vec![0.0f32; w];
    for y in 0..h {
        for (to, from) in row.iter_mut().zip(&px[y * w..(y + 1) * w]) {
            *to = (*from).into();
        }
        for (dx, span) in cols.iter().enumerate() {
            narrow[y * dw + dx] = span.average(&row, 1, 0);
        }
    }
    let mut out = vec![0.0f32; dw * dh];
    for (dy, span) in rows.iter().enumerate() {
        for dx in 0..dw {
            out[dy * dw + dx] = span.average(&narrow, dw, dx);
        }
    }
    Grey::new(dw, dh, out)
}

/// The source pixels one reduced pixel covers along one axis: from `first` to `last`
/// inclusive, the two ends weighted by how much of them it covers.
struct Span {
    first: usize,
    last: usize,
    first_weight: f32,
    last_weight: f32,
    total: f32,
}

impl Span {
    /// The weighted average of `data[(i * step) + offset]` over the span.
    #[inline]
    fn average(&self, data: &[f32], step: usize, offset: usize) -> f32 {
        if self.first == self.last {
            return data[self.first * step + offset];
        }
        let mut sum = data[self.first * step + offset] * self.first_weight
            + data[self.last * step + offset] * self.last_weight;
        for i in self.first + 1..self.last {
            sum += data[i * step + offset];
        }
        sum / self.total
    }
}

fn spans(from: usize, to: usize) -> Vec<Span> {
    let ratio = from as f64 / to as f64;
    (0..to)
        .map(|i| {
            let start = i as f64 * ratio;
            let end = ((i + 1) as f64 * ratio).min(from as f64);
            let first = (start.floor() as usize).min(from - 1);
            let last = ((end.ceil() as usize).max(first + 1) - 1).min(from - 1);
            if first == last {
                return Span {
                    first,
                    last,
                    first_weight: 1.0,
                    last_weight: 1.0,
                    total: 1.0,
                };
            }
            let first_weight = (first as f64 + 1.0 - start) as f32;
            let last_weight = (end - last as f64) as f32;
            Span {
                first,
                last,
                first_weight,
                last_weight,
                total: (end - start) as f32,
            }
        })
        .collect()
}

/// Width and height that fit `(w, h)` inside a square of `side`, keeping its proportions;
/// unchanged when it already fits.
pub(crate) fn fit(w: usize, h: usize, side: usize) -> (usize, usize) {
    let long = w.max(h);
    if long <= side {
        return (w, h);
    }
    let scale = side as f64 / long as f64;
    (
        ((w as f64 * scale).round() as usize).max(1),
        ((h as f64 * scale).round() as usize).max(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reducing_averages_what_each_new_pixel_covers() {
        let g = Grey::new(4, 2, vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]);
        let r = g.reduce(2, 1);
        assert_eq!(r.px, vec![25.0, 45.0]);
        let r = Grey::new(3, 1, vec![0.0, 30.0, 60.0]).reduce(2, 1);
        // The middle pixel is shared: each side takes half of it.
        assert!((r.px[0] - 10.0).abs() < 1e-4 && (r.px[1] - 50.0).abs() < 1e-4);
    }

    #[test]
    fn a_warp_with_nothing_to_do_gives_the_image_back() {
        let g = Grey::new(5, 4, (0..20).map(|v| v as f32).collect());
        assert_eq!(g.warp(0.0, 1.0, (0.0, 0.0)).px, g.px);
        let moved = g.warp(0.0, 1.0, (1.0, 0.0));
        assert_eq!(moved.at(0, 0), 1.0);
        assert!(moved.at(4, 0).is_nan());
    }

    #[test]
    fn detail_takes_away_shading() {
        let ramp = Grey::new(9, 9, (0..81).map(|v| (v % 9) as f32 * 10.0).collect());
        let d = ramp.detail(1);
        assert!(d.at(4, 4).abs() < 1e-3);
    }

    #[test]
    fn fitting_keeps_proportions() {
        assert_eq!(fit(4000, 3000, 1000), (1000, 750));
        assert_eq!(fit(300, 200, 1000), (300, 200));
    }
}
