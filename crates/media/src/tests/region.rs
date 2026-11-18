//! Regions of a page cut from the original (#84).

use std::io::Cursor;

use image::{DynamicImage, GrayImage, ImageFormat, RgbImage};
use kalamu_core::{PageHalf, PieceRef, SourceFile};
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};

use super::oriented_png;
use crate::{Crop, ImageSource, MediaError, OpenSource, PdfSource, RegionRefusal};

/// A one-page PDF of `media` (`[x0, y0, x1, y1]` in points), with an optional CropBox and
/// `/Rotate`, and a black square filled over each of `marks` (`[x0, y0, x1, y1]`, points).
fn marked_pdf(media: [f32; 4], crop: Option<[f32; 4]>, rotate: i64, marks: &[[f32; 4]]) -> Vec<u8> {
    let real = |b: [f32; 4]| b.iter().map(|v| Object::Real(*v)).collect::<Vec<_>>();
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut operations = vec![Operation::new("rg", vec![0.into(), 0.into(), 0.into()])];
    for [x0, y0, x1, y1] in marks {
        operations.push(Operation::new(
            "re",
            vec![
                Object::Real(*x0),
                Object::Real(*y0),
                Object::Real(x1 - x0),
                Object::Real(y1 - y0),
            ],
        ));
        operations.push(Operation::new("f", vec![]));
    }
    let content = Content { operations };
    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        content.encode().unwrap_or_default(),
    ));
    let mut page = dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => real(media),
        "Contents" => content_id,
        "Rotate" => rotate,
    };
    if let Some(crop) = crop {
        page.set("CropBox", real(crop));
    }
    let page_id = doc.add_object(page);
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![Object::Reference(page_id)],
            "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    let _ = doc.save_to(&mut out);
    out
}

fn pdf(bytes: &[u8]) -> Result<OpenSource, MediaError> {
    Ok(OpenSource::Pdf(Box::new(PdfSource::from_bytes(bytes)?)))
}

fn crop(x: f32, y: f32, width: f32, height: f32) -> Crop {
    Crop {
        x,
        y,
        width,
        height,
    }
}

/// The region cut, as grey levels.
fn cut(
    open: &OpenSource,
    piece: &PieceRef,
    region: Crop,
) -> Result<(GrayImage, String), MediaError> {
    let input = open.prepare_region(piece, region, &["application/pdf", "image/png"], "x")?;
    assert_eq!(input.mime, "image/png");
    assert_eq!(&input.piece, piece);
    let image = image::load_from_memory(&input.bytes)
        .map_err(|e| MediaError::Image(e.to_string()))?
        .to_luma8();
    Ok((image, input.file_name))
}

fn dark(image: &GrayImage) -> usize {
    image.pixels().filter(|p| p.0[0] < 128).count()
}

fn is_dark(image: &GrayImage, x: u32, y: u32) -> bool {
    image.get_pixel(x, y).0[0] < 128
}

/// Where a corner of the box as shown lies within a region cut there: its pixel, a pixel in.
#[derive(Clone, Copy)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl Corner {
    fn region(self) -> Crop {
        match self {
            Self::TopLeft => crop(0.0, 0.0, 0.2, 0.2),
            Self::TopRight => crop(0.8, 0.0, 0.2, 0.2),
            Self::BottomRight => crop(0.8, 0.8, 0.2, 0.2),
            Self::BottomLeft => crop(0.0, 0.8, 0.2, 0.2),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::TopLeft => Self::BottomRight,
            Self::TopRight => Self::BottomLeft,
            Self::BottomRight => Self::TopLeft,
            Self::BottomLeft => Self::TopRight,
        }
    }

    fn pixel(self, image: &GrayImage) -> (u32, u32) {
        let (r, b) = (image.width() - 3, image.height() - 3);
        match self {
            Self::TopLeft => (2, 2),
            Self::TopRight => (r, 2),
            Self::BottomRight => (r, b),
            Self::BottomLeft => (2, b),
        }
    }
}

/// The crop of the corner holding the mark holds it, at that corner; the crop of the opposite
/// corner holds nothing.
fn assert_mark_at(open: &OpenSource, piece: &PieceRef, corner: Corner, what: &str) {
    let Ok((near, _)) = cut(open, piece, corner.region()) else {
        panic!("{what}: the region could not be cut");
    };
    let (x, y) = corner.pixel(&near);
    assert!(
        is_dark(&near, x, y),
        "{what}: the mark is not in the corner"
    );
    assert!(dark(&near) > 100, "{what}: {} dark pixels", dark(&near));
    let Ok((far, _)) = cut(open, piece, corner.opposite().region()) else {
        panic!("{what}: the opposite region could not be cut");
    };
    assert_eq!(dark(&far), 0, "{what}: the opposite corner holds a mark");
}

/// 600 by 800 points, marked at its top-left corner as stored: the turn decides where it shows.
const PAGE: [f32; 4] = [0.0, 0.0, 600.0, 800.0];
const TOP_LEFT_MARK: [f32; 4] = [0.0, 740.0, 60.0, 800.0];

#[test]
fn a_pdf_region_is_cut_where_the_page_shows_it_whatever_its_turn() -> Result<(), MediaError> {
    // The page's top-left corner turned clockwise lands at each corner of the screen in turn.
    for (rotate, corner) in [
        (0, Corner::TopLeft),
        (90, Corner::TopRight),
        (180, Corner::BottomRight),
        (270, Corner::BottomLeft),
    ] {
        let open = pdf(&marked_pdf(PAGE, None, rotate, &[TOP_LEFT_MARK]))?;
        assert_mark_at(
            &open,
            &PieceRef::page(1),
            corner,
            &format!("Rotate {rotate}"),
        );
    }
    Ok(())
}

#[test]
fn a_pdf_region_is_measured_on_the_crop_box() -> Result<(), MediaError> {
    // A mark at the CropBox's top-left corner, and another outside it, at the MediaBox's, which
    // the page never shows.
    let crop_box = [100.0, 100.0, 500.0, 700.0];
    let marks = [[100.0, 640.0, 160.0, 700.0], TOP_LEFT_MARK];
    for (rotate, corner) in [(0, Corner::TopLeft), (90, Corner::TopRight)] {
        let open = pdf(&marked_pdf(PAGE, Some(crop_box), rotate, &marks))?;
        assert_mark_at(
            &open,
            &PieceRef::page(1),
            corner,
            &format!("CropBox, Rotate {rotate}"),
        );
    }
    // 0.2 of the CropBox's 400 by 600 points, at 300 dpi: 333 by 500 pixels.
    let open = pdf(&marked_pdf(PAGE, Some(crop_box), 0, &marks))?;
    let (image, _) = cut(&open, &PieceRef::page(1), crop(0.0, 0.0, 0.2, 0.2))?;
    assert!((333..=334).contains(&image.width()), "{}", image.width());
    assert!((499..=501).contains(&image.height()), "{}", image.height());
    Ok(())
}

#[test]
fn a_pdf_region_is_drawn_at_300_dpi() -> Result<(), MediaError> {
    // A4: 0.3 by 0.4 of 595 by 842 points is 743.75 by 1,403.3 pixels at 300 dpi.
    let open = pdf(&marked_pdf([0.0, 0.0, 595.0, 842.0], None, 0, &[]))?;
    let (image, name) = cut(&open, &PieceRef::page(1), crop(0.1, 0.2, 0.3, 0.4))?;
    assert_eq!(name, "x-p1-region.png");
    assert!((743..=745).contains(&image.width()), "{}", image.width());
    assert!(
        (1403..=1404).contains(&image.height()),
        "{}",
        image.height()
    );
    // Turned a quarter, the page shows 842 wide: the box's share of that.
    let turned = pdf(&marked_pdf([0.0, 0.0, 595.0, 842.0], None, 90, &[]))?;
    let (image, _) = cut(&turned, &PieceRef::page(1), crop(0.0, 0.0, 0.5, 0.5))?;
    assert!((1753..=1755).contains(&image.width()), "{}", image.width());
    assert!(
        (1239..=1241).contains(&image.height()),
        "{}",
        image.height()
    );
    // A box running off the page is clamped to it: 0.2 of each side is left.
    let (image, _) = cut(&open, &PieceRef::page(1), crop(-0.1, -0.1, 0.3, 0.3))?;
    assert!((495..=497).contains(&image.width()), "{}", image.width());
    assert!((701..=703).contains(&image.height()), "{}", image.height());
    // A model that takes JPEG only gets one.
    let jpeg = open.prepare_region(
        &PieceRef::page(1),
        crop(0.1, 0.1, 0.5, 0.5),
        &["image/jpeg"],
        "x",
    )?;
    assert_eq!(
        (jpeg.mime.as_str(), jpeg.file_name.as_str()),
        ("image/jpeg", "x-p1-region.jpg")
    );
    Ok(())
}

#[test]
fn a_region_of_a_spread_s_half_is_taken_from_that_half() -> Result<(), MediaError> {
    // A spread of two 600 by 800 pages, marked at the right page's top-left corner.
    let open = pdf(&marked_pdf(
        [0.0, 0.0, 1200.0, 800.0],
        None,
        0,
        &[[600.0, 740.0, 660.0, 800.0]],
    ))?;
    let right = PieceRef::Page {
        number: 1,
        half: Some(PageHalf::Right),
    };
    assert_mark_at(&open, &right, Corner::TopLeft, "right half");
    let (image, name) = cut(&open, &right, crop(0.0, 0.0, 0.2, 0.2))?;
    assert_eq!(name, "x-p1-right-region.png");
    // 0.2 of the half's 600 points, not of the spread's 1,200.
    assert!((499..=501).contains(&image.width()), "{}", image.width());
    let left = PieceRef::Page {
        number: 1,
        half: Some(PageHalf::Left),
    };
    let (image, _) = cut(&open, &left, crop(0.0, 0.0, 0.2, 0.2))?;
    assert_eq!(dark(&image), 0, "the left half's corner is blank");

    // The same with an image: 1,200 by 800 pixels, the right page's top-left 60 pixels black.
    let mut spread = RgbImage::from_pixel(1200, 800, image::Rgb([255, 255, 255]));
    for x in 600..660 {
        for y in 0..60 {
            spread.put_pixel(x, y, image::Rgb([0, 0, 0]));
        }
    }
    let mut bytes = Vec::new();
    DynamicImage::ImageRgb8(spread)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|e| MediaError::Image(e.to_string()))?;
    let open = OpenSource::Image(ImageSource::from_bytes(bytes, "png")?);
    let (image, _) = cut(&open, &right, crop(0.0, 0.0, 0.1, 0.1))?;
    assert_eq!(image.dimensions(), (60, 80));
    assert_eq!(dark(&image), 60 * 60);
    let (image, _) = cut(&open, &left, crop(0.0, 0.0, 0.1, 0.1))?;
    assert_eq!(dark(&image), 0);
    Ok(())
}

#[test]
fn an_image_region_is_cut_after_its_exif_turn() -> Result<(), MediaError> {
    // Stored 40 by 20 pixels on its side, its top-right 10 pixels red; EXIF says to turn it a
    // quarter clockwise. Upright it is 20 by 40, the red at the bottom right.
    let mut stored = RgbImage::from_pixel(40, 20, image::Rgb([255, 255, 255]));
    for x in 30..40 {
        for y in 0..10 {
            stored.put_pixel(x, y, image::Rgb([255, 0, 0]));
        }
    }
    let mut plain = Vec::new();
    DynamicImage::ImageRgb8(stored)
        .write_to(&mut Cursor::new(&mut plain), ImageFormat::Png)
        .map_err(|e| MediaError::Image(e.to_string()))?;
    let open = OpenSource::Image(ImageSource::from_bytes(oriented_png(&plain, 6), "png")?);
    let red = |region: Crop| -> Result<(u32, u32, usize), MediaError> {
        let input = open.prepare_region(&PieceRef::page(1), region, &["image/png"], "photo")?;
        let image = image::load_from_memory(&input.bytes)
            .map_err(|e| MediaError::Image(e.to_string()))?
            .to_rgb8();
        let count = image.pixels().filter(|p| p.0 == [255, 0, 0]).count();
        Ok((image.width(), image.height(), count))
    };
    assert_eq!(red(crop(0.5, 0.75, 0.5, 0.25))?, (10, 10, 100));
    assert_eq!(red(crop(0.0, 0.0, 0.5, 0.25))?, (10, 10, 0));
    Ok(())
}

#[test]
fn an_image_region_is_its_share_of_the_image_s_pixels() -> Result<(), MediaError> {
    let open = OpenSource::Image(ImageSource::from_bytes(super::png(1000, 500), "png")?);
    let (image, name) = cut(&open, &PieceRef::page(1), crop(0.1, 0.2, 0.3, 0.4))?;
    assert_eq!(name, "x-p1-region.png");
    assert_eq!(image.dimensions(), (300, 200));
    let (image, _) = cut(&open, &PieceRef::page(1), crop(0.9, 0.9, 0.5, 0.5))?;
    assert_eq!(image.dimensions(), (100, 50), "clamped to the image");
    Ok(())
}

#[test]
fn regions_that_cannot_be_cut_are_refused() -> Result<(), MediaError> {
    let refusal = |result: Result<_, MediaError>| match result {
        Err(MediaError::Region(r)) => Some(r),
        _ => None,
    };
    let page = PieceRef::page(1);
    let images = ["image/png"];
    let open = pdf(&marked_pdf([0.0, 0.0, 595.0, 842.0], None, 0, &[]))?;
    let photo = OpenSource::Image(ImageSource::from_bytes(super::png(1000, 500), "png")?);
    for source in [&open, &photo] {
        let try_cut = |region| source.prepare_region(&page, region, &images, "x");
        assert_eq!(
            refusal(try_cut(crop(f32::NAN, 0.0, 0.2, 0.2))),
            Some(RegionRefusal::NotFinite)
        );
        assert_eq!(
            refusal(try_cut(crop(0.0, 0.0, f32::INFINITY, 0.2))),
            Some(RegionRefusal::NotFinite)
        );
        assert_eq!(
            refusal(try_cut(crop(1.2, 0.1, 0.2, 0.2))),
            Some(RegionRefusal::OutsidePage)
        );
        assert_eq!(
            refusal(try_cut(crop(0.1, 0.1, 0.0, 0.2))),
            Some(RegionRefusal::OutsidePage)
        );
        // A few points of an A4 page, a few pixels of the image.
        assert_eq!(
            refusal(try_cut(crop(0.5, 0.5, 0.002, 0.2))),
            Some(RegionRefusal::TooSmall)
        );
        // A model that reads no images, though it reads PDFs.
        assert_eq!(
            refusal(source.prepare_region(
                &page,
                crop(0.1, 0.1, 0.5, 0.5),
                &["application/pdf"],
                "x"
            )),
            Some(RegionRefusal::NoImages)
        );
        // No such page.
        assert!(matches!(
            source.prepare_region(&PieceRef::page(2), crop(0.1, 0.1, 0.5, 0.5), &images, "x"),
            Err(MediaError::NoSuchPage(_))
        ));
    }
    let message = refusal(open.prepare_region(&page, crop(0.5, 0.5, 0.001, 0.2), &images, "x"))
        .map(|r| (r.to_string(), r.code()));
    assert_eq!(
        message,
        Some((
            "The region is too small to read: draw it at least 8 pixels wide and high on the scan"
                .to_string(),
            "media.region_too_small"
        ))
    );

    // Recordings, texts and videos have no page.
    let video = SourceFile::youtube("https://youtu.be/dQw4w9WgXcQ")
        .ok_or_else(|| MediaError::NeedsConversion("not a YouTube link".into()))?;
    let video = OpenSource::open(&video, None)?;
    assert_eq!(
        refusal(video.prepare_region(&page, crop(0.1, 0.1, 0.5, 0.5), &images, "x")),
        Some(RegionRefusal::NoPage)
    );
    let dir = tempfile::tempdir().map_err(|e| MediaError::Tool(e.to_string()))?;
    let path = dir.path().join("notes.txt");
    std::fs::write(&path, "Some notes.").map_err(|e| MediaError::Tool(e.to_string()))?;
    let text = OpenSource::Text(crate::TextSource::open(&path)?);
    assert_eq!(
        refusal(text.prepare_region(&page, crop(0.1, 0.1, 0.5, 0.5), &images, "x")),
        Some(RegionRefusal::NoPage)
    );

    // A HEIC that cannot be decoded is refused as a page of it is.
    let heic = OpenSource::Image(ImageSource::from_bytes(
        b"not decoded here".to_vec(),
        "heic",
    )?);
    let Err(MediaError::NeedsConversion(message)) =
        heic.prepare_region(&page, crop(0.1, 0.1, 0.5, 0.5), &images, "x")
    else {
        panic!("a HEIC region is refused");
    };
    assert!(message.contains("HEIC"), "{message}");
    Ok(())
}

#[test]
fn the_original_is_only_read() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(|e| MediaError::Tool(e.to_string()))?;
    let path = dir.path().join("page.pdf");
    let bytes = marked_pdf(PAGE, None, 90, &[TOP_LEFT_MARK]);
    std::fs::write(&path, &bytes).map_err(|e| MediaError::Tool(e.to_string()))?;
    let open = OpenSource::Pdf(Box::new(PdfSource::open(&path)?));
    open.prepare_region(
        &PieceRef::page(1),
        crop(0.0, 0.0, 0.5, 0.5),
        &["image/png"],
        "x",
    )?;
    let after = std::fs::read(&path).map_err(|e| MediaError::Tool(e.to_string()))?;
    assert_eq!(after, bytes);
    Ok(())
}
