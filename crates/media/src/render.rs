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

/// The `tEXt` keyword under which a drawn page states the resolution it was drawn at, in dots
/// per inch (`300`, `241.32`), which Tesseract's record of the page reads back. A scan given
/// as a PNG never carries it, so a scan's own resolution tag is not taken for a drawing.
pub const RESOLUTION_KEYWORD: &str = "Kalamu drawn dpi";

/// The first page of `pdf` as a PNG on a white ground, at [`DPI`] or less for a very large page.
/// The PNG states the resolution used twice: as its `pHYs` chunk, for any program that reads
/// one, and exactly, under [`RESOLUTION_KEYWORD`]. Its pixels are the same either way.
pub fn first_page_png(pdf: &[u8]) -> Result<Vec<u8>, MediaError> {
    let mut used = DPI;
    let png = draw_first_page(pdf, |width, height| {
        let scale = DPI / 72.0;
        let longest = width.max(height) * scale;
        let scale = if longest > MAX_SIDE {
            scale * MAX_SIDE / longest
        } else {
            scale
        };
        used = scale * 72.0;
        scale
    })?;
    Ok(with_resolution(png, used))
}

/// A resolution as written in the PNG: two decimals at most, `300` rather than `300.00`.
fn written(dpi: f32) -> String {
    let text = format!("{dpi:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// `png` with a `pHYs` chunk (pixels per metre) and a `tEXt` chunk stating `dpi`, both placed
/// right after the header, where the format allows them. Nothing else changes.
fn with_resolution(png: Vec<u8>, dpi: f32) -> Vec<u8> {
    // The signature (8 bytes), then the header chunk: length, type, 13 bytes of data, CRC.
    const AFTER_HEADER: usize = 8 + 4 + 4 + 13 + 4;
    if png.len() < AFTER_HEADER || &png[12..16] != b"IHDR" {
        return png;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a resolution of a few hundred dots per inch, rounded"
    )]
    let per_metre = (dpi / 0.0254).round() as u32;
    let mut physical = Vec::with_capacity(9);
    physical.extend_from_slice(&per_metre.to_be_bytes());
    physical.extend_from_slice(&per_metre.to_be_bytes());
    physical.push(1); // the unit is the metre
    let mut text = RESOLUTION_KEYWORD.as_bytes().to_vec();
    text.push(0);
    text.extend_from_slice(written(dpi).as_bytes());
    let mut out = Vec::with_capacity(png.len() + 64);
    out.extend_from_slice(&png[..AFTER_HEADER]);
    chunk(&mut out, b"pHYs", &physical);
    chunk(&mut out, b"tEXt", &text);
    out.extend_from_slice(&png[AFTER_HEADER..]);
    out
}

/// A PNG chunk: its length, its type, its data and the CRC of type and data.
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    let length = u32::try_from(data.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(kind.iter().chain(data)).to_be_bytes());
}

/// The CRC-32 PNG chunks carry (ISO 3309, as zlib computes it).
fn crc32<'a>(bytes: impl Iterator<Item = &'a u8>) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
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
fn draw_first_page(
    pdf: &[u8],
    mut scale: impl FnMut(f32, f32) -> f32,
) -> Result<Vec<u8>, MediaError> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The chunks of a PNG, in order: type and data, each CRC checked.
    fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        let mut out = Vec::new();
        let mut at = 8;
        while at + 12 <= png.len() {
            let length = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
            let kind: [u8; 4] = png[at + 4..at + 8].try_into().unwrap();
            let data = png[at + 8..at + 8 + length].to_vec();
            let crc =
                u32::from_be_bytes(png[at + 8 + length..at + 12 + length].try_into().unwrap());
            assert_eq!(crc, crc32(kind.iter().chain(&data)), "{kind:?}");
            out.push((kind, data));
            at += 12 + length;
        }
        out
    }

    fn stated(png: &[u8]) -> Option<String> {
        chunks(png).into_iter().find_map(|(kind, data)| {
            let text = String::from_utf8(data).ok()?;
            let (keyword, value) = text.split_once('\0')?;
            (&kind == b"tEXt" && keyword == RESOLUTION_KEYWORD).then(|| value.to_owned())
        })
    }

    #[test]
    fn the_crc_is_pngs() {
        // The IEND chunk every PNG ends with, and the check value of CRC-32.
        assert_eq!(crc32(b"IEND".iter()), 0xAE42_6082);
        assert_eq!(crc32(b"123456789".iter()), 0xCBF4_3926);
    }

    #[test]
    fn a_page_drawn_states_its_resolution_and_keeps_its_pixels() {
        let a4 = crate::sample_pdf(&[(595.0, 842.0)], true);
        let png = first_page_png(&a4).unwrap();
        assert_eq!(stated(&png).as_deref(), Some("300"));
        let kinds: Vec<[u8; 4]> = chunks(&png).into_iter().map(|(k, _)| k).collect();
        assert_eq!(&kinds[..3], [*b"IHDR", *b"pHYs", *b"tEXt"]);
        let (_, physical) = chunks(&png)
            .into_iter()
            .find(|(k, _)| k == b"pHYs")
            .unwrap();
        // 300 dpi is 11,811 pixels per metre, both ways.
        assert_eq!(physical, [0, 0, 0x2E, 0x23, 0, 0, 0x2E, 0x23, 1]);
        // The same pixels as the page drawn without them.
        let plain = draw_first_page(&a4, |_, _| DPI / 72.0).unwrap();
        let drawn = image::load_from_memory(&png).unwrap();
        assert_eq!(drawn, image::load_from_memory(&plain).unwrap());
        assert_eq!((drawn.width(), drawn.height()), (2479, 3508));
    }

    #[test]
    fn a_poster_is_drawn_smaller_and_says_so() {
        // 2,000 by 1,000 points: 8,333 pixels wide at 300 dpi, drawn 7,000 wide at 252 dpi.
        let poster = crate::sample_pdf(&[(2000.0, 1000.0)], false);
        let png = first_page_png(&poster).unwrap();
        assert_eq!(stated(&png).as_deref(), Some("252"));
        let drawn = image::load_from_memory(&png).unwrap();
        assert_eq!(drawn.width(), 7000);
    }

    #[test]
    fn resolutions_are_written_without_trailing_zeros() {
        assert_eq!(written(300.0), "300");
        assert_eq!(written(241.316), "241.32");
        assert_eq!(written(250.5), "250.5");
    }

    #[test]
    fn a_thumbnail_carries_no_resolution() {
        let a4 = crate::sample_pdf(&[(595.0, 842.0)], true);
        let (_, png) = thumbnail(&a4, "application/pdf", 100).unwrap();
        assert_eq!(stated(&png), None);
    }
}
