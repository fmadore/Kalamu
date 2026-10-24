use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader};
use scriptorium_core::PageHalf;

use crate::MediaError;

/// Longest side sent to a provider; larger scans are scaled down (Claude refuses above 8000 px).
const MAX_SIDE: u32 = 7_680;

/// An image file: one page, or several for a multi-page TIFF.
pub struct ImageSource {
    bytes: Vec<u8>,
    mime: String,
    /// Width and height of each page, as stored (before EXIF orientation).
    sizes: Vec<(u32, u32)>,
    /// Whether EXIF orientation turns the image a quarter turn.
    quarter_turn: bool,
}

impl ImageSource {
    pub fn open(path: &Path) -> Result<Self, MediaError> {
        let bytes = std::fs::read(path).map_err(|e| MediaError::Read {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        Self::from_bytes(bytes, &ext)
    }

    pub fn from_bytes(bytes: Vec<u8>, ext: &str) -> Result<Self, MediaError> {
        let mime = match ext {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "tif" | "tiff" => "image/tiff",
            "bmp" => "image/bmp",
            "heic" => "image/heic",
            "heif" => "image/heif",
            _ => "application/octet-stream",
        }
        .to_string();
        let (sizes, quarter_turn) = if mime == "image/tiff" {
            (tiff_sizes(&bytes)?, false)
        } else if mime.starts_with("image/hei") {
            // No decoder for HEIC here; providers that accept it get the file as it is.
            (vec![(0, 0)], false)
        } else {
            let mut decoder = ImageReader::new(Cursor::new(&bytes))
                .with_guessed_format()
                .map_err(|e| MediaError::Image(e.to_string()))?
                .into_decoder()
                .map_err(|e| MediaError::Image(e.to_string()))?;
            let size = decoder.dimensions();
            let turn = decoder.orientation().is_ok_and(|o| {
                use image::metadata::Orientation::*;
                matches!(o, Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH)
            });
            (vec![size], turn)
        };
        Ok(Self {
            bytes,
            mime,
            sizes,
            quarter_turn,
        })
    }

    pub fn page_count(&self) -> u32 {
        self.sizes.len() as u32
    }

    /// Width and height as displayed, in pixels; `None` when the format cannot be read here.
    pub fn display_size(&self, number: u32) -> Option<(u32, u32)> {
        let &(w, h) = self.sizes.get(number.checked_sub(1)? as usize)?;
        if w == 0 {
            return None;
        }
        Some(if self.quarter_turn { (h, w) } else { (w, h) })
    }

    pub fn is_spread(&self, number: u32) -> bool {
        self.display_size(number)
            .is_some_and(|(w, h)| h > 0 && w as f32 / h as f32 > crate::checks::SPREAD_RATIO)
    }

    pub fn is_decodable(&self) -> bool {
        !self.mime.starts_with("image/hei")
    }

    /// The page as bytes in an accepted format: the original file when possible, otherwise
    /// decoded, turned upright, cropped to the requested half and re-encoded.
    pub fn prepare(
        &self,
        number: u32,
        half: Option<PageHalf>,
        accepted: &[&str],
    ) -> Result<(String, Vec<u8>), MediaError> {
        let single_page = self.sizes.len() == 1;
        let oversized = self
            .display_size(number)
            .is_some_and(|(w, h)| w.max(h) > MAX_SIDE);
        if single_page
            && half.is_none()
            && !oversized
            && !self.quarter_turn
            && accepted.contains(&self.mime.as_str())
        {
            return Ok((self.mime.clone(), self.bytes.clone()));
        }
        if !self.is_decodable() {
            return Err(MediaError::NeedsConversion(format!(
                "This model does not accept {} images and they cannot be converted yet. Convert \
                 them to JPEG or PNG, or choose Gemini, which reads them directly.",
                self.mime.trim_start_matches("image/").to_uppercase()
            )));
        }
        let mut img = self.decode(number)?;
        if let Some(half) = half {
            let (w, h) = img.dimensions();
            let mid = w / 2;
            img = match half {
                PageHalf::Left => img.crop_imm(0, 0, mid, h),
                PageHalf::Right => img.crop_imm(mid, 0, w - mid, h),
            };
        }
        let (w, h) = img.dimensions();
        if w.max(h) > MAX_SIDE {
            img = img.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Lanczos3);
        }
        encode(&img, accepted)
    }

    fn decode(&self, number: u32) -> Result<DynamicImage, MediaError> {
        if self.mime == "image/tiff" {
            return tiff_page(&self.bytes, number);
        }
        let mut decoder = ImageReader::new(Cursor::new(&self.bytes))
            .with_guessed_format()
            .map_err(|e| MediaError::Image(e.to_string()))?
            .into_decoder()
            .map_err(|e| MediaError::Image(e.to_string()))?;
        let orientation = decoder.orientation().ok();
        let mut img =
            DynamicImage::from_decoder(decoder).map_err(|e| MediaError::Image(e.to_string()))?;
        if let Some(o) = orientation {
            img.apply_orientation(o);
        }
        Ok(img)
    }
}

/// PNG keeps fine strokes and is the safe choice for scans; JPEG when PNG is not accepted or
/// would be too large.
fn encode(img: &DynamicImage, accepted: &[&str]) -> Result<(String, Vec<u8>), MediaError> {
    const PNG_LIMIT: usize = 15 * 1024 * 1024;
    if accepted.contains(&"image/png") {
        let mut out = Vec::new();
        img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .map_err(|e| MediaError::Image(e.to_string()))?;
        if out.len() <= PNG_LIMIT || !accepted.contains(&"image/jpeg") {
            return Ok(("image/png".into(), out));
        }
    }
    if accepted.contains(&"image/jpeg") {
        let mut out = Vec::new();
        let rgb = DynamicImage::ImageRgb8(img.to_rgb8());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 92);
        rgb.write_with_encoder(encoder)
            .map_err(|e| MediaError::Image(e.to_string()))?;
        return Ok(("image/jpeg".into(), out));
    }
    Err(MediaError::NeedsConversion(
        "This model does not accept images.".into(),
    ))
}

fn tiff_sizes(bytes: &[u8]) -> Result<Vec<(u32, u32)>, MediaError> {
    let err = |e: tiff::TiffError| MediaError::Image(e.to_string());
    let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).map_err(err)?;
    let mut sizes = vec![decoder.dimensions().map_err(err)?];
    while decoder.more_images() {
        decoder.next_image().map_err(err)?;
        sizes.push(decoder.dimensions().map_err(err)?);
    }
    Ok(sizes)
}

/// One page of a TIFF; `image` reads only the first, so pages are decoded with `tiff` directly.
fn tiff_page(bytes: &[u8], number: u32) -> Result<DynamicImage, MediaError> {
    use tiff::ColorType;
    use tiff::decoder::DecodingResult;

    let err = |e: tiff::TiffError| MediaError::Image(e.to_string());
    let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).map_err(err)?;
    for _ in 1..number {
        decoder.next_image().map_err(err)?;
    }
    let (w, h) = decoder.dimensions().map_err(err)?;
    let color = decoder.colortype().map_err(err)?;
    let data = decoder.read_image().map_err(err)?;
    let unsupported = || {
        MediaError::Image(format!(
            "TIFF page {number} uses a colour format that cannot be converted ({color:?})"
        ))
    };
    let img = match (color, data) {
        (ColorType::Gray(8), DecodingResult::U8(buf)) => {
            image::GrayImage::from_raw(w, h, buf).map(DynamicImage::ImageLuma8)
        }
        (ColorType::GrayA(8), DecodingResult::U8(buf)) => {
            image::GrayAlphaImage::from_raw(w, h, buf).map(DynamicImage::ImageLumaA8)
        }
        (ColorType::RGB(8), DecodingResult::U8(buf)) => {
            image::RgbImage::from_raw(w, h, buf).map(DynamicImage::ImageRgb8)
        }
        (ColorType::RGBA(8), DecodingResult::U8(buf)) => {
            image::RgbaImage::from_raw(w, h, buf).map(DynamicImage::ImageRgba8)
        }
        (ColorType::Gray(16), DecodingResult::U16(buf)) => {
            image::ImageBuffer::from_raw(w, h, buf).map(DynamicImage::ImageLuma16)
        }
        (ColorType::RGB(16), DecodingResult::U16(buf)) => {
            image::ImageBuffer::from_raw(w, h, buf).map(DynamicImage::ImageRgb16)
        }
        _ => None,
    };
    img.ok_or_else(unsupported)
}
