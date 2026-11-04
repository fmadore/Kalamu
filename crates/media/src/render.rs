//! PDF pages drawn as images, for engines that read images only (Tesseract).
//!
//! hayro, a PDF rasterizer written in Rust with no unsafe code, draws the single-page PDF that
//! the other models are sent, so a page split into halves is drawn as the half the others read.
//! The specification first named PDFium; hayro needs no library shipped beside the app on each
//! system ([decision](../../../docs/decisions/2026-11-04-tesseract.md)).

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings, render};

use crate::MediaError;

/// Pages are drawn at this resolution, Tesseract's own recommendation for print.
pub const DPI: f32 = 300.0;
/// The longest side a page is drawn at, so a poster-sized page does not take gigabytes.
const MAX_SIDE: f32 = 7000.0;

/// The first page of `pdf` as a PNG on a white ground, at [`DPI`] or less for a very large page.
pub fn first_page_png(pdf: &[u8]) -> Result<Vec<u8>, MediaError> {
    let pdf = Pdf::new(pdf.to_vec())
        .map_err(|e| MediaError::Pdf(format!("the page could not be drawn: {e:?}")))?;
    let pages = pdf.pages();
    let page = pages
        .first()
        .ok_or_else(|| MediaError::Pdf("the page could not be drawn: no page".into()))?;
    let (width, height) = page.render_dimensions();
    let mut scale = DPI / 72.0;
    let longest = width.max(height) * scale;
    if longest > MAX_SIDE {
        scale *= MAX_SIDE / longest;
    }
    let pixmap = render(
        page,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
        },
    );
    pixmap
        .into_png()
        .map_err(|e| MediaError::Image(format!("the drawn page could not be saved: {e:?}")))
}
