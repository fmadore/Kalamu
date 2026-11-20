//! Photographs of one batch checked against each other before a job (#216): the same page
//! photographed twice, and a blurred photograph. In a reading room a page is often taken two or
//! three times, the hand having moved or the focus hesitated; sent as they are, the retakes are
//! paid for and reviewed twice, and the blurred take is sometimes the one read.
//!
//! Each photograph is decoded once, upright, and reduced (`measure`): its sharpness is read at
//! about the size a model sees, and small grey copies are kept for the comparisons; the decoded
//! image is let go at once. Neighbours in the order given are compared first, since that is
//! where retakes sit; farther pairs only when their layouts already agree closely. A candidate
//! pair is then laid one over the other and compared tile by tile (`compare`), so that two
//! pages of one register are not taken for one. Sharpness is judged against the batch as well
//! as against a floor, since paper, light and phones differ.
//!
//! Local only: nothing is sent anywhere, and originals are only read. The work runs on a few
//! threads, and stops between photographs when `stop` is raised.
//! See [the decision](../../../docs/decisions/2026-11-20-duplicate-and-blurred-photographs.md).

mod compare;
mod grey;
mod measure;

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::{DynamicImage, GenericImageView};
use kalamu_core::{InputKind, InputWarning, SourceFile, SourceId, StopSignal, WarningKind};

use crate::{ImageSource, MediaError};
use measure::Measured;

/// Each photograph is compared with this many that follow it in the order.
const NEIGHBOURS: usize = 2;
/// Photographs farther apart in the order are compared only when their layouts' hashes differ
/// in at most this many bits of 64…
const FAR_HASH_BITS: u32 = 8;
/// …and their smallest copies correlate at least this well.
const FAR_LIKENESS: f32 = 0.75;

/// A photograph is blurred when its sharpness is below this, whatever the batch: on the
/// sample pages, a Gaussian blur of 1 pixel at the working size gives 0.60 to 0.62, sharp
/// pages 0.91 to 0.98.
pub(crate) const BLUR_FLOOR: f32 = 0.6;
/// Or, in a batch of at least [`BATCH_FOR_RELATIVE`] photographs that can be judged, below
/// this share of the batch's median sharpness…
pub(crate) const BLUR_RELATIVE: f32 = 0.78;
/// …as long as it is also below this, so a sharp batch does not make a sharp page look blurred.
pub(crate) const BLUR_CEILING: f32 = 0.85;
pub(crate) const BATCH_FOR_RELATIVE: usize = 4;
/// Of two photographs of one page, one is called clearly the sharper when its sharpness is
/// higher by at least this share.
const CLEARLY_SHARPER: f32 = 0.06;

/// The widest a photograph is shown beside a warning, in pixels.
pub const VIEW_WIDTH: u32 = 400;

/// What the photographs of a batch showed, and how to show the photographs warned about.
#[derive(Debug, Clone, Default)]
pub struct PhotoReport {
    pub warnings: Vec<InputWarning>,
    pub views: Vec<PhotoView>,
}

/// A photograph warned about, drawn small (`page`) and a part of it close up (`detail`), as
/// JPEGs, so its sharpness can be judged by eye.
#[derive(Debug, Clone)]
pub struct PhotoView {
    pub source_id: SourceId,
    pub page: Vec<u8>,
    pub detail: Vec<u8>,
}

/// Checks the photographs among `sources`, in the order given (the order of the files, or of
/// a document's pages): which look like the same page as another, and which look blurred.
/// Other sources, multi-page TIFFs and files that cannot be decoded are left out.
/// [`MediaError::Stopped`] when `stop` is raised before the end.
pub fn check_photos(sources: &[SourceFile], stop: &StopSignal) -> Result<PhotoReport, MediaError> {
    let photos: Vec<&SourceFile> = sources
        .iter()
        .filter(|s| s.kind == InputKind::Image && !s.is_page_set() && s.page_count == Some(1))
        .collect();
    let workers = workers();
    let measured = parallel(&photos, workers, stop, |s| {
        measure::measure_file(Path::new(&s.path)).ok()
    })?;
    let (pairs, blurred) = judge(&measured, workers, stop)?;

    let mut warnings = Vec::new();
    let mut shown: Vec<(usize, (f32, f32))> = Vec::new();
    let mut show = |i: usize, focus: (f32, f32)| {
        if !shown.iter().any(|&(j, _)| j == i) {
            shown.push((i, focus));
        }
    };
    for pair in &pairs {
        let (first, second) = (photos[pair.first].id, photos[pair.second].id);
        warnings.push(InputWarning {
            source_id: second,
            kind: WarningKind::SamePage {
                other: first,
                keep: photos[pair.keep].id,
                clearly_sharper: pair.clearly_sharper,
            },
        });
        // Both shown close up at the same place, the sharper one's densest writing.
        let focus = measured[pair.keep].as_ref().map_or((0.5, 0.5), |m| m.focus);
        show(pair.keep, focus);
        show(pair.other(), focus);
    }
    for &i in &blurred {
        warnings.push(InputWarning {
            source_id: photos[i].id,
            kind: WarningKind::Blurred,
        });
        show(i, measured[i].as_ref().map_or((0.5, 0.5), |m| m.focus));
    }
    let views = parallel(&shown, workers, stop, |&(i, focus)| {
        view(photos[i], focus).ok()
    })?
    .into_iter()
    .flatten()
    .collect();
    Ok(PhotoReport { warnings, views })
}

/// Two photographs found to be the same page: `first` comes before `second` in the order, and
/// `keep` is the sharper of the two.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SamePair {
    pub first: usize,
    pub second: usize,
    pub keep: usize,
    pub clearly_sharper: bool,
}

impl SamePair {
    fn other(&self) -> usize {
        if self.keep == self.first {
            self.second
        } else {
            self.first
        }
    }
}

/// The pairs that show the same page, and the photographs that look blurred, from what each
/// photograph measured (`None` for one that could not be read).
pub(crate) fn judge(
    measured: &[Option<Measured>],
    workers: usize,
    stop: &StopSignal,
) -> Result<(Vec<SamePair>, Vec<usize>), MediaError> {
    let candidates = candidates(measured);
    let verdicts = parallel(&candidates, workers, stop, |&(i, j)| {
        match (&measured[i], &measured[j]) {
            (Some(a), Some(b)) => compare::compare(a, b).same,
            _ => false,
        }
    })?;
    let sharpness = |i: usize| measured[i].as_ref().and_then(|m| m.sharpness);
    let pairs: Vec<SamePair> = candidates
        .iter()
        .zip(verdicts)
        .filter(|&(_, same)| same)
        .map(|(&(first, second), _)| {
            let (a, b) = (sharpness(first), sharpness(second));
            let (keep, clearly_sharper) = match (a, b) {
                (Some(a), Some(b)) => {
                    let keep = if b > a { second } else { first };
                    (keep, (a - b).abs() >= CLEARLY_SHARPER * a.max(b))
                }
                (None, Some(_)) => (second, false),
                _ => (first, false),
            };
            SamePair {
                first,
                second,
                keep,
                clearly_sharper,
            }
        })
        .collect();

    // A photograph left out by a pair's suggestion is not warned about again.
    let set_aside: Vec<usize> = pairs.iter().map(SamePair::other).collect();
    let judged: Vec<f32> = (0..measured.len()).filter_map(sharpness).collect();
    let median = median(&judged);
    let blurred = (0..measured.len())
        .filter(|i| !set_aside.contains(i))
        .filter(|&i| {
            sharpness(i).is_some_and(|s| {
                s < BLUR_FLOOR
                    || (judged.len() >= BATCH_FOR_RELATIVE
                        && s < BLUR_CEILING
                        && median.is_some_and(|m| s < BLUR_RELATIVE * m))
            })
        })
        .collect();
    Ok((pairs, blurred))
}

/// The pairs worth laying over each other: every neighbour (a retake turned or moved can look
/// unlike its page at the smallest size, so neighbours are always laid over each other), and
/// pairs farther apart whose layouts agree closely.
fn candidates(measured: &[Option<Measured>]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (i, a) in measured.iter().enumerate() {
        let Some(a) = a else { continue };
        for (j, b) in measured.iter().enumerate().skip(i + 1) {
            let Some(b) = b else { continue };
            if a.landscape != b.landscape {
                continue;
            }
            let near = j - i <= NEIGHBOURS;
            if near
                || ((a.hash ^ b.hash).count_ones() <= FAR_HASH_BITS
                    && compare::tiny_likeness(a, b) >= FAR_LIKENESS)
            {
                out.push((i, j));
            }
        }
    }
    out
}

fn median(values: &[f32]) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    Some(sorted[sorted.len() / 2])
}

/// Threads for the work: half the processors, at most six, so New job and the system stay
/// responsive, and at most six photographs are held decoded at once (a 12-megapixel JPEG
/// decoded to grey is 12 MB).
fn workers() -> usize {
    std::thread::available_parallelism()
        .map_or(1, |n| n.get() / 2)
        .clamp(1, 6)
}

/// `work` done for every item on `workers` threads, the results in the items' order.
/// [`MediaError::Stopped`] as soon as `stop` is raised: each thread stops before its next item.
pub(crate) fn parallel<T: Sync, R: Send>(
    items: &[T],
    workers: usize,
    stop: &StopSignal,
    work: impl Fn(&T) -> R + Sync,
) -> Result<Vec<R>, MediaError> {
    let next = AtomicUsize::new(0);
    let done: Vec<Vec<(usize, R)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers.clamp(1, items.len().max(1)))
            .map(|_| {
                scope.spawn(|| {
                    let mut out = Vec::new();
                    loop {
                        if stop.is_stopped() {
                            break;
                        }
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(i) else { break };
                        out.push((i, work(item)));
                    }
                    out
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    if stop.is_stopped() {
        return Err(MediaError::Stopped);
    }
    let mut all: Vec<(usize, R)> = done.into_iter().flatten().collect();
    if all.len() != items.len() {
        // A thread ended early without being stopped: it failed, and its items are lost.
        return Err(MediaError::Image(
            "a photograph could not be checked".into(),
        ));
    }
    all.sort_by_key(|&(i, _)| i);
    Ok(all.into_iter().map(|(_, r)| r).collect())
}

/// A photograph drawn [`VIEW_WIDTH`] wide, and the part of it around `focus` close up.
fn view(source: &SourceFile, focus: (f32, f32)) -> Result<PhotoView, MediaError> {
    let img = ImageSource::open(Path::new(&source.path))?.upright(1, None)?;
    let page = jpeg(&img.thumbnail(VIEW_WIDTH, VIEW_WIDTH.saturating_mul(2)))?;
    let (w, h) = img.dimensions();
    // A sixth of the page's width, at most as wide as shown, so what is shown is the
    // photograph's own pixels or fewer, never enlarged.
    let cw = (w / 6).max(VIEW_WIDTH.min(w)).min(w);
    let ch = (cw * 3 / 5).min(h);
    let x = ((focus.0 * w as f32) as u32)
        .saturating_sub(cw / 2)
        .min(w - cw);
    let y = ((focus.1 * h as f32) as u32)
        .saturating_sub(ch / 2)
        .min(h - ch);
    let part = img.crop_imm(x, y, cw, ch);
    let detail = jpeg(&part.thumbnail(VIEW_WIDTH, VIEW_WIDTH))?;
    Ok(PhotoView {
        source_id: source.id,
        page,
        detail,
    })
}

fn jpeg(img: &DynamicImage) -> Result<Vec<u8>, MediaError> {
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85)
        .encode_image(&img.to_rgb8())
        .map_err(|e| MediaError::Image(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests;
