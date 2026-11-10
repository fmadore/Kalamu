//! PDF pages drawn as images, for engines that read images only (Tesseract), and pages drawn
//! small for the review's strip of thumbnails.
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

/// The widest a thumbnail is drawn, whatever is asked.
pub const THUMBNAIL_MAX_WIDTH: u32 = 400;

/// The first page of `pdf` as a PNG on a white ground, at [`DPI`] or less for a very large page.
pub fn first_page_png(pdf: &[u8]) -> Result<Vec<u8>, MediaError> {
    draw_first_page(pdf, |width, height| {
        let scale = DPI / 72.0;
        let longest = width.max(height) * scale;
        if longest > MAX_SIDE {
            scale * MAX_SIDE / longest
        } else {
            scale
        }
    })
}

/// A page drawn `width` pixels wide (at most [`THUMBNAIL_MAX_WIDTH`]), for a thumbnail: a
/// single-page PDF drawn as a PNG, or an image made smaller as a JPEG. Returns the picture's
/// media type and bytes: a scan's thumbnail is then a few kilobytes, not the whole page.
pub fn thumbnail(bytes: &[u8], mime: &str, width: u32) -> Result<(String, Vec<u8>), MediaError> {
    let width = width.clamp(1, THUMBNAIL_MAX_WIDTH);
    if mime == "application/pdf" {
        let png = draw_first_page(bytes, |page_width, _| width as f32 / page_width.max(1.0))?;
        return Ok(("image/png".into(), png));
    }
    let image = image::load_from_memory(bytes)
        .map_err(|e| MediaError::Image(format!("the page could not be read: {e}")))?;
    let small = image.thumbnail(width, width.saturating_mul(4)).into_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 80)
        .encode_image(&small)
        .map_err(|e| MediaError::Image(format!("the thumbnail could not be saved: {e}")))?;
    Ok(("image/jpeg".into(), out))
}

/// The first page of `pdf` on a white ground as a PNG, at the scale `scale` gives for the
/// page's size in points.
fn draw_first_page(pdf: &[u8], scale: impl Fn(f32, f32) -> f32) -> Result<Vec<u8>, MediaError> {
    let pdf = Pdf::new(pdf.to_vec())
        .map_err(|e| MediaError::Pdf(format!("the page could not be drawn: {e:?}")))?;
    let pages = pdf.pages();
    let page = pages
        .first()
        .ok_or_else(|| MediaError::Pdf("the page could not be drawn: no page".into()))?;
    let (width, height) = page.render_dimensions();
    let scale = scale(width, height);
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
