use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader};
use kalamu_core::PageHalf;

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
    /// Whether EXIF orientation changes the image at all (a turn, half a turn, a mirror):
    /// such a file is sent upright, decoded and turned, since not every reader honours it
    /// (Tesseract does not).
    oriented: bool,
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
        let (sizes, quarter_turn, oriented) = if mime == "image/tiff" {
            (tiff_sizes(&bytes)?, false, false)
        } else if mime.starts_with("image/hei") {
            // No decoder for HEIC here; providers that accept it get the file as it is.
            (vec![(0, 0)], false, false)
        } else {
            let mut decoder = ImageReader::new(Cursor::new(&bytes))
                .with_guessed_format()
                .map_err(|e| MediaError::Image(e.to_string()))?
                .into_decoder()
                .map_err(|e| MediaError::Image(e.to_string()))?;
            let size = decoder.dimensions();
            let orientation = decoder.orientation().ok();
            let turn = orientation.is_some_and(|o| {
                use image::metadata::Orientation::*;
                matches!(o, Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH)
            });
            let oriented =
                orientation.is_some_and(|o| o != image::metadata::Orientation::NoTransforms);
            (vec![size], turn, oriented)
        };
        Ok(Self {
            bytes,
            mime,
            sizes,
            quarter_turn,
            oriented,
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
            && !self.oriented
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

fn tiff_err(e: tiff::TiffError) -> MediaError {
    MediaError::Image(e.to_string())
}

/// A TIFF decoder that takes the pages archives scan. The crate's defaults refuse a page that
/// decodes to more than 256 MiB, which a colour A3 page at 600 dpi in 16 bits per channel
/// (7,016 × 9,921 pixels, 418 MB) passes; a gigabyte takes an A2 page so scanned, and a strip
/// read at once up to half that.
fn tiff_decoder(bytes: &[u8]) -> Result<tiff::decoder::Decoder<Cursor<&[u8]>>, MediaError> {
    let limits = {
        let mut limits = tiff::decoder::Limits::default();
        limits.decoding_buffer_size = 1 << 30;
        limits.intermediate_buffer_size = 1 << 29;
        limits
    };
    Ok(tiff::decoder::Decoder::new(Cursor::new(bytes))
        .map_err(tiff_err)?
        .with_limits(limits))
}

fn tiff_sizes(bytes: &[u8]) -> Result<Vec<(u32, u32)>, MediaError> {
    let mut decoder = tiff_decoder(bytes)?;
    let mut sizes = vec![decoder.dimensions().map_err(tiff_err)?];
    while decoder.more_images() {
        decoder.next_image().map_err(tiff_err)?;
        sizes.push(decoder.dimensions().map_err(tiff_err)?);
    }
    Ok(sizes)
}

/// One page of a TIFF; `image` reads only the first, so pages are decoded with `tiff` directly.
/// Every bit depth of grey up to 16 bits, palette colour and CMYK are converted: a microfilm's
/// or a library's black-and-white scan is 1-bit, usually compressed as CCITT Group 4.
fn tiff_page(bytes: &[u8], number: u32) -> Result<DynamicImage, MediaError> {
    let mut decoder = tiff_decoder(bytes)?;
    let index = number
        .checked_sub(1)
        .ok_or_else(|| MediaError::NoSuchPage(number.to_string()))?;
    decoder
        .seek_to_image(index as usize)
        .map_err(|_| MediaError::NoSuchPage(number.to_string()))?;
    let (w, h) = decoder.dimensions().map_err(tiff_err)?;
    let unsupported = |what: String| {
        MediaError::Image(format!(
            "TIFF page {number} uses a colour format that cannot be converted ({what})"
        ))
    };
    let photometric = decoder
        .find_tag_unsigned::<u16>(tiff::tags::Tag::PhotometricInterpretation)
        .map_err(tiff_err)?;
    if photometric == Some(PALETTE) {
        return palette_page(bytes, &mut decoder, index, (w, h))
            .and_then(|img| img.ok_or_else(|| unsupported("palette".into())));
    }
    let color = decoder.colortype().map_err(tiff_err)?;
    let data = decoder.read_image().map_err(tiff_err)?;
    to_image((w, h), color, data).ok_or_else(|| unsupported(format!("{color:?}")))
}

/// PhotometricInterpretation for colours looked up in the page's ColorMap.
const PALETTE: u16 = 3;

/// A page decoded by `tiff`, as an image to convert; `None` for a format it gives that is not
/// converted (floating point, Lab, YCbCr not turned into RGB by its compression).
fn to_image(
    (w, h): (u32, u32),
    color: tiff::ColorType,
    data: tiff::decoder::DecodingResult,
) -> Option<DynamicImage> {
    use tiff::ColorType;
    use tiff::decoder::DecodingResult::{U8, U16};
    Some(match (color, data) {
        // Below 8 bits, rows are packed and start on a byte; white is the highest value, the
        // decoder having turned a page that says "white is zero" round.
        (ColorType::Gray(bits @ (1 | 2 | 4)), U8(buf)) => {
            let max = (1u32 << bits) - 1;
            let row = (w as usize * bits as usize).div_ceil(8);
            let mut grey = Vec::with_capacity(w as usize * h as usize);
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let v = packed(&buf, row, x, y, bits)? as u32;
                    grey.push((v * 255 / max) as u8);
                }
            }
            DynamicImage::ImageLuma8(image::GrayImage::from_raw(w, h, grey)?)
        }
        (ColorType::Gray(8), U8(buf)) => {
            DynamicImage::ImageLuma8(image::GrayImage::from_raw(w, h, buf)?)
        }
        (ColorType::GrayA(8), U8(buf)) => {
            DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_raw(w, h, buf)?)
        }
        (ColorType::RGB(8), U8(buf)) => {
            DynamicImage::ImageRgb8(image::RgbImage::from_raw(w, h, buf)?)
        }
        (ColorType::RGBA(8), U8(buf)) => {
            DynamicImage::ImageRgba8(image::RgbaImage::from_raw(w, h, buf)?)
        }
        (ColorType::Gray(16), U16(buf)) => {
            DynamicImage::ImageLuma16(image::ImageBuffer::from_raw(w, h, buf)?)
        }
        (ColorType::GrayA(16), U16(buf)) => {
            DynamicImage::ImageLumaA16(image::ImageBuffer::from_raw(w, h, buf)?)
        }
        (ColorType::RGB(16), U16(buf)) => {
            DynamicImage::ImageRgb16(image::ImageBuffer::from_raw(w, h, buf)?)
        }
        (ColorType::RGBA(16), U16(buf)) => {
            DynamicImage::ImageRgba16(image::ImageBuffer::from_raw(w, h, buf)?)
        }
        // Ink to light, without a colour profile: near enough for reading a page. Alpha, when
        // there is one, is dropped.
        (ColorType::CMYK(8) | ColorType::CMYKA(8), U8(buf)) => {
            let step = if matches!(color, ColorType::CMYK(_)) {
                4
            } else {
                5
            };
            let rgb = buf
                .chunks_exact(step)
                .flat_map(|p| {
                    let light = |ink: u8| u32::from(255 - ink) * u32::from(255 - p[3]) / 255;
                    [light(p[0]) as u8, light(p[1]) as u8, light(p[2]) as u8]
                })
                .collect();
            DynamicImage::ImageRgb8(image::RgbImage::from_raw(w, h, rgb)?)
        }
        (ColorType::CMYK(16) | ColorType::CMYKA(16), U16(buf)) => {
            let step = if matches!(color, ColorType::CMYK(_)) {
                4
            } else {
                5
            };
            let rgb = buf
                .chunks_exact(step)
                .flat_map(|p| {
                    let light = |ink: u16| {
                        (u64::from(65_535 - ink) * u64::from(65_535 - p[3]) / 65_535) as u16
                    };
                    [light(p[0]), light(p[1]), light(p[2])]
                })
                .collect();
            DynamicImage::ImageRgb16(image::ImageBuffer::from_raw(w, h, rgb)?)
        }
        _ => return None,
    })
}

/// The `bits`-bit sample at `(x, y)` of rows `row` bytes long, packed from the high bit.
fn packed(buf: &[u8], row: usize, x: usize, y: usize, bits: u8) -> Option<u8> {
    let bit = x * bits as usize;
    let byte = buf.get(y * row + bit / 8)?;
    let shift = 8 - bits as usize - bit % 8;
    Some((byte >> shift) & ((1u16 << bits) - 1) as u8)
}

/// A palette page, which `tiff` does not decode: its indices are read from a copy of the file
/// that calls the page grey, then looked up in the page's ColorMap. `None` for a palette of
/// more than 8 bits or a ColorMap too short for it.
fn palette_page(
    bytes: &[u8],
    decoder: &mut tiff::decoder::Decoder<Cursor<&[u8]>>,
    index: u32,
    (w, h): (u32, u32),
) -> Result<Option<DynamicImage>, MediaError> {
    use tiff::tags::Tag;
    let bits = decoder
        .find_tag_unsigned::<u8>(Tag::BitsPerSample)
        .map_err(tiff_err)?
        .unwrap_or(1);
    let map = decoder.get_tag_u16_vec(Tag::ColorMap).map_err(tiff_err)?;
    let entries = 1usize << bits.min(16);
    if !matches!(bits, 1 | 2 | 4 | 8) || map.len() < 3 * entries {
        return Ok(None);
    }
    let Some(ifd) = decoder.ifd_pointer() else {
        return Ok(None);
    };
    let Some(grey) = called_grey(bytes, decoder.byte_order(), ifd.0) else {
        return Ok(None);
    };
    let mut indices = tiff_decoder(&grey)?;
    indices.seek_to_image(index as usize).map_err(tiff_err)?;
    let tiff::decoder::DecodingResult::U8(buf) = indices.read_image().map_err(tiff_err)? else {
        return Ok(None);
    };
    let row = (w as usize * bits as usize).div_ceil(8);
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for y in 0..h as usize {
        for x in 0..w as usize {
            let Some(i) = packed(&buf, row, x, y, bits).map(usize::from) else {
                return Ok(None);
            };
            // The ColorMap holds every red, then every green, then every blue, in 16 bits.
            rgb.extend([map[i], map[entries + i], map[2 * entries + i]].map(|c| (c >> 8) as u8));
        }
    }
    Ok(image::RgbImage::from_raw(w, h, rgb).map(DynamicImage::ImageRgb8))
}

/// A copy of a TIFF whose directory at `ifd` calls its page grey (black is zero) instead of
/// palette colour, so its samples decode as they are stored. `None` when the directory cannot
/// be found in the bytes as the format lays it out.
fn called_grey(bytes: &[u8], order: tiff::tags::ByteOrder, ifd: u64) -> Option<Vec<u8>> {
    let little = order == tiff::tags::ByteOrder::LittleEndian;
    let read = |at: usize, len: usize| -> Option<u64> {
        let field = bytes.get(at..at.checked_add(len)?)?;
        let fold = |n: u64, b: &u8| n << 8 | u64::from(*b);
        Some(if little {
            field.iter().rev().fold(0, fold)
        } else {
            field.iter().fold(0, fold)
        })
    };
    // Classic TIFF: a 2-byte entry count, entries of 12 bytes holding their value from byte 8.
    // BigTIFF (version 43): an 8-byte count, entries of 20 bytes, the value from byte 12.
    let big = read(2, 2)? == 43;
    let (count_len, entry_len, value_at) = if big { (8, 20, 12) } else { (2, 12, 8) };
    let ifd = usize::try_from(ifd).ok()?;
    let count = usize::try_from(read(ifd, count_len)?).ok()?;
    let mut copy = bytes.to_vec();
    for n in 0..count {
        let entry = ifd + count_len + n * entry_len;
        // The tag, stored as a SHORT (type 3), so its value is the field's first two bytes.
        if read(entry, 2)? == 262 && read(entry + 2, 2)? == 3 {
            let at = entry + value_at;
            let one = if little { [1, 0] } else { [0, 1] };
            copy.get_mut(at..at + 2)?.copy_from_slice(&one);
            return Some(copy);
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Small TIFFs of the formats scans come in, made for these tests with ImageMagick and
    /// Pillow: 13 × 6 pixels with the left six columns black (red in the palette pages, blue
    /// to their right), the right ones white, so every row ends part-way through a byte; the
    /// grey ones a gradient, one level a row.
    fn fixture(name: &str) -> ImageSource {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("tiff")
            .join(name);
        ImageSource::open(&path).unwrap()
    }

    fn rgb(source: &ImageSource, page: u32, x: u32, y: u32) -> [u8; 3] {
        source.decode(page).unwrap().to_rgb8().get_pixel(x, y).0
    }

    #[test]
    fn one_bit_pages_convert_however_compressed_and_whichever_way_round() {
        for name in ["g4-min-is-white.tif", "g4-min-is-black.tif", "bilevel.tif"] {
            let source = fixture(name);
            let img = source.decode(1).unwrap();
            assert_eq!((img.width(), img.height()), (13, 6), "{name}");
            assert_eq!(rgb(&source, 1, 0, 0), [0, 0, 0], "{name}");
            assert_eq!(rgb(&source, 1, 5, 5), [0, 0, 0], "{name}");
            assert_eq!(rgb(&source, 1, 6, 0), [255, 255, 255], "{name}");
            assert_eq!(rgb(&source, 1, 12, 5), [255, 255, 255], "{name}");
            let (mime, png) = source.prepare(1, None, &["image/png"]).unwrap();
            assert_eq!(mime, "image/png");
            assert!(png.starts_with(b"\x89PNG"), "{name}");
        }
    }

    #[test]
    fn every_page_of_a_multi_page_group_4_file_converts() {
        let source = fixture("g4-min-is-black.tif");
        assert_eq!(source.page_count(), 2);
        assert_eq!(rgb(&source, 2, 0, 0), [0, 0, 0]);
        assert_eq!(rgb(&source, 2, 12, 0), [255, 255, 255]);
        assert!(matches!(source.decode(3), Err(MediaError::NoSuchPage(_))));
        let (_, left) = source
            .prepare(2, Some(PageHalf::Left), &["image/png"])
            .unwrap();
        assert!(left.starts_with(b"\x89PNG"));
    }

    #[test]
    fn grey_of_two_and_four_bits_spreads_over_the_whole_range() {
        // A gradient from black at the top to white at the bottom, one level a row.
        for (name, levels) in [("gray2.tif", 4u32), ("gray4.tif", 16)] {
            let source = fixture(name);
            let img = source.decode(1).unwrap().to_luma8();
            let column: Vec<u8> = (0..levels).map(|y| img.get_pixel(3, y).0[0]).collect();
            assert_eq!(column[0], 0, "{name}");
            assert_eq!(column[levels as usize - 1], 255, "{name}");
            assert!(column.windows(2).all(|w| w[0] < w[1]), "{name}: {column:?}");
        }
    }

    #[test]
    fn palette_pages_take_their_colours_from_the_colour_map() {
        for name in ["palette4.tif", "palette8.tif"] {
            let source = fixture(name);
            assert_eq!(rgb(&source, 1, 0, 0), [255, 0, 0], "{name}");
            assert_eq!(rgb(&source, 1, 8, 0), [0, 0, 255], "{name}");
        }
    }

    #[test]
    fn cmyk_pages_are_turned_into_light() {
        let source = fixture("cmyk.tif");
        assert_eq!(rgb(&source, 1, 0, 0), [0, 0, 0]);
        assert_eq!(rgb(&source, 1, 12, 0), [255, 255, 255]);
        let (mime, _) = source.prepare(1, None, &["image/jpeg"]).unwrap();
        assert_eq!(mime, "image/jpeg");
    }

    /// The directory of an uncompressed 16-bit RGB page of `w` × `h` pixels, whose one strip
    /// runs past the end of the file: enough to see whether a decoder would take its size.
    fn page_header(w: u32, h: u32) -> Vec<u8> {
        let entries: [(u16, u16, u32); 10] = [
            (256, 4, w),         // ImageWidth
            (257, 4, h),         // ImageLength
            (258, 3, 16),        // BitsPerSample, the same for the three samples
            (259, 3, 1),         // Compression: none
            (262, 3, 2),         // PhotometricInterpretation: RGB
            (273, 4, 200),       // StripOffsets
            (277, 3, 3),         // SamplesPerPixel
            (278, 4, h),         // RowsPerStrip
            (279, 4, w * h * 6), // StripByteCounts
            (284, 3, 1),         // PlanarConfiguration: chunky
        ];
        let mut tiff = b"II*\0".to_vec();
        tiff.extend(8u32.to_le_bytes());
        tiff.extend((entries.len() as u16).to_le_bytes());
        for (tag, kind, value) in entries {
            tiff.extend(tag.to_le_bytes());
            tiff.extend(kind.to_le_bytes());
            tiff.extend(1u32.to_le_bytes());
            match kind {
                3 => tiff.extend((value as u16).to_le_bytes().into_iter().chain([0, 0])),
                _ => tiff.extend(value.to_le_bytes()),
            }
        }
        tiff.extend(0u32.to_le_bytes());
        tiff.resize(256, 0);
        tiff
    }

    #[test]
    fn the_decoder_takes_a_colour_a3_page_at_600_dpi_in_16_bits() {
        // 418 MB decoded: refused by the crate's default limits, taken by ours, which then
        // find the file shorter than its strip.
        let header = page_header(7_016, 9_921);
        let mut default = tiff::decoder::Decoder::new(Cursor::new(&header[..])).unwrap();
        assert!(matches!(
            default.read_image(),
            Err(tiff::TiffError::LimitsExceeded)
        ));
        let mut ours = tiff_decoder(&header).unwrap();
        let read = ours.read_image();
        assert!(
            !matches!(read, Err(tiff::TiffError::LimitsExceeded) | Ok(_)),
            "{read:?}"
        );
    }
}
