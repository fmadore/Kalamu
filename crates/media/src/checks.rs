//! Input checks: cheap tests on page sizes and metadata that predict a poor result.

use kalamu_core::WarningKind;

use crate::{ImageSource, PdfSource};

/// Wider than tall by this much looks like two pages side by side. A4 landscape is 1.41; a
/// double spread of A4 portrait pages is 1.41 too, so this flags both and lets the user decide.
pub(crate) const SPREAD_RATIO: f32 = 1.25;

/// Below this, small print, diacritics and handwriting become guesswork.
const MIN_DPI: u32 = 200;

/// Images whose resolution would be lower than this are figures, not page scans.
const MIN_SCAN_DPI: u32 = 50;

/// Pages checked for slow tests (text layer, image resolution) on long PDFs.
const SAMPLE: u32 = 10;

pub(crate) fn pdf_warnings(pdf: &PdfSource) -> Vec<WarningKind> {
    let mut out = Vec::new();
    let pages = pdf.page_count();

    let spreads: Vec<u32> = (1..=pages).filter(|&n| pdf.is_spread(n)).collect();
    if !spreads.is_empty() {
        out.push(WarningKind::DoublePageSpread { pages: spreads });
    }

    let sampled: Vec<u32> = (1..=pages.min(SAMPLE)).collect();
    let text_pages: Vec<u32> = sampled
        .iter()
        .copied()
        .filter(|&n| pdf.has_text(n))
        .collect();
    // Only scanned pages: a born-digital page's images are logos and figures. An image narrower
    // than a 50 dpi page is a figure, not a scan.
    let low: Vec<(u32, u32)> = sampled
        .iter()
        .copied()
        .filter(|n| !text_pages.contains(n))
        .filter_map(|n| pdf.image_dpi(n).map(|dpi| (n, dpi)))
        .filter(|&(_, dpi)| (MIN_SCAN_DPI..MIN_DPI).contains(&dpi))
        .collect();
    if let Some(lowest) = low.iter().map(|&(_, dpi)| dpi).min() {
        out.push(WarningKind::LowResolution {
            pages: low.iter().map(|&(n, _)| n).collect(),
            dpi: lowest,
        });
    }

    if !text_pages.is_empty() {
        out.push(WarningKind::HasTextLayer {
            pages_with_text: text_pages.len() as u32,
        });
    }
    out
}

pub(crate) fn image_warnings(img: &ImageSource) -> Vec<WarningKind> {
    let mut out = Vec::new();
    if !img.is_decodable() {
        out.push(WarningKind::UnsupportedImage {
            detail: "HEIC/HEIF photos are sent as they are; only Gemini reads them.".into(),
        });
        return out;
    }
    let pages = img.page_count();
    let spreads: Vec<u32> = (1..=pages).filter(|&n| img.is_spread(n)).collect();
    if !spreads.is_empty() {
        out.push(WarningKind::DoublePageSpread { pages: spreads });
    }
    // Without a reliable DPI tag, assume the long side of the image is an A4 page (11.7 in).
    let low: Vec<(u32, u32)> = (1..=pages)
        .filter_map(|n| {
            img.display_size(n)
                .map(|(w, h)| (n, (w.max(h) as f32 / 11.7) as u32))
        })
        .filter(|&(_, dpi)| dpi < MIN_DPI)
        .collect();
    if let Some(lowest) = low.iter().map(|&(_, dpi)| dpi).min() {
        out.push(WarningKind::LowResolution {
            pages: low.iter().map(|&(n, _)| n).collect(),
            dpi: lowest,
        });
    }
    out
}
