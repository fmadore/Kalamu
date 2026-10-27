use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage};
use scriptorium_core::{PageHalf, WarningKind};

use super::*;
use crate::pdf::sample_pdf;

const A4: (f32, f32) = (595.0, 842.0);
const SPREAD: (f32, f32) = (1190.0, 842.0);

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let _ = DynamicImage::ImageRgb8(RgbImage::new(w, h))
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png);
    out
}

#[test]
fn pdf_pages_become_standalone_single_page_pdfs() -> Result<(), MediaError> {
    let pdf = PdfSource::from_bytes(&sample_pdf(&[A4, A4, A4], true))?;
    assert_eq!(pdf.page_count(), 3);
    let page = pdf.page_pdf(2, None)?;
    let reread = PdfSource::from_bytes(&page)?;
    assert_eq!(reread.page_count(), 1);
    // Resources were inherited from the page tree and must survive the cut.
    assert!(reread.has_text(1));
    assert!(
        lopdf::Document::load_mem(&page)
            .is_ok_and(|d| d.extract_text(&[1]).is_ok_and(|t| t.contains("Page 2")))
    );
    Ok(())
}

#[test]
fn spreads_are_detected_and_split_in_two() -> Result<(), MediaError> {
    let bytes = sample_pdf(&[A4, SPREAD], false);
    let open = OpenSource::Pdf(Box::new(PdfSource::from_bytes(&bytes)?));
    let warnings = open.warnings(SourceId::new());
    assert!(warnings.iter().any(|w| matches!(
        &w.kind,
        WarningKind::DoublePageSpread { pages } if pages == &vec![2]
    )));

    let pieces = open.pieces(true, None, crate::DEFAULT_PIECE_MS);
    assert_eq!(
        pieces,
        vec![
            PieceRef::page(1),
            PieceRef::Page {
                number: 2,
                half: Some(PageHalf::Left)
            },
            PieceRef::Page {
                number: 2,
                half: Some(PageHalf::Right)
            },
        ]
    );
    let right = open.prepare(&pieces[2], &["application/pdf"], "issue")?;
    assert_eq!(right.file_name, "issue-p2-right.pdf");
    let half = PdfSource::from_bytes(&right.bytes)?;
    let (w, h) = half.display_size(1).unwrap_or_default();
    assert!((w - 595.0).abs() < 1.0 && (h - 842.0).abs() < 1.0);
    Ok(())
}

#[test]
fn samples_take_only_the_first_pages() -> Result<(), MediaError> {
    let open = OpenSource::Pdf(Box::new(PdfSource::from_bytes(&sample_pdf(
        &[A4; 5], false,
    ))?));
    assert_eq!(
        open.pieces(false, Some(2), crate::DEFAULT_PIECE_MS).len(),
        2
    );
    assert_eq!(
        open.pieces(false, Some(50), crate::DEFAULT_PIECE_MS).len(),
        5
    );
    Ok(())
}

#[test]
fn pdf_needs_a_model_that_reads_pdfs() -> Result<(), MediaError> {
    let open = OpenSource::Pdf(Box::new(PdfSource::from_bytes(&sample_pdf(&[A4], false))?));
    let result = open.prepare(&PieceRef::page(1), &["image/png"], "x");
    assert!(matches!(result, Err(MediaError::NeedsConversion(_))));
    Ok(())
}

#[test]
fn a_youtube_video_is_one_piece_sent_as_its_link() -> Result<(), MediaError> {
    let source = SourceFile::youtube("https://youtu.be/dQw4w9WgXcQ")
        .ok_or_else(|| MediaError::NeedsConversion("not a YouTube link".into()))?;
    let open = OpenSource::open(&source, None)?;
    assert_eq!(
        open.pieces(false, None, DEFAULT_PIECE_MS),
        vec![PieceRef::Whole]
    );
    assert!(open.warnings(source.id).is_empty());
    let sent = open.prepare(&PieceRef::Whole, &["audio/mpeg", YOUTUBE_LINK], "video")?;
    assert_eq!(sent.mime, YOUTUBE_LINK);
    assert_eq!(sent.bytes, b"https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    let refused = open.prepare(&PieceRef::Whole, &["audio/mpeg"], "video");
    assert!(matches!(refused, Err(MediaError::NeedsConversion(_))));
    Ok(())
}

#[test]
fn accepted_images_pass_through_unchanged() -> Result<(), MediaError> {
    let bytes = png(2400, 3300);
    let img = ImageSource::from_bytes(bytes.clone(), "png")?;
    let (mime, out) = img.prepare(1, None, &["image/png", "image/jpeg"])?;
    assert_eq!(mime, "image/png");
    assert_eq!(out, bytes);
    Ok(())
}

#[test]
fn a_photographed_page_is_analysed_as_one_image() -> Result<(), MediaError> {
    let bytes = png(2400, 3300);
    let photo = OpenSource::Image(ImageSource::from_bytes(bytes.clone(), "png")?);
    let whole = photo.prepare(&PieceRef::Whole, &["text/plain", "image/png"], "letter")?;
    assert_eq!(
        (whole.mime.as_str(), whole.file_name.as_str()),
        ("image/png", "letter.png")
    );
    assert_eq!(whole.bytes, bytes);
    Ok(())
}

#[test]
fn images_are_converted_split_and_checked() -> Result<(), MediaError> {
    let img = ImageSource::from_bytes(png(3000, 2000), "png")?;
    assert!(img.is_spread(1));
    let (mime, left) = img.prepare(1, Some(PageHalf::Left), &["image/jpeg"])?;
    assert_eq!(mime, "image/jpeg");
    let decoded = image::load_from_memory(&left).map_err(|e| MediaError::Image(e.to_string()))?;
    assert_eq!((decoded.width(), decoded.height()), (1500, 2000));

    let small = OpenSource::Image(ImageSource::from_bytes(png(800, 1100), "png")?);
    let warnings = small.warnings(SourceId::new());
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w.kind, WarningKind::LowResolution { dpi, .. } if dpi < 200))
    );
    Ok(())
}

#[test]
fn files_are_inspected_and_hashed() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(|e| MediaError::Image(e.to_string()))?;
    let path = dir.path().join("issue.pdf");
    std::fs::write(&path, sample_pdf(&[A4, A4], false))
        .map_err(|e| MediaError::Image(e.to_string()))?;
    let source = inspect(&path, None)?;
    assert_eq!(source.kind, InputKind::Pdf);
    assert_eq!(source.page_count, Some(2));
    assert_eq!(source.sha256.len(), 64);
    assert_eq!(source.file_name, "issue.pdf");
    Ok(())
}

/// Cuts every page of a real PDF and checks each one opens: `SCRIPTORIUM_SAMPLE_PDF=path cargo
/// test -p scriptorium-media real_pdf -- --ignored --nocapture`.
#[test]
#[ignore = "needs a PDF on disk"]
fn real_pdf_pages_open() -> Result<(), MediaError> {
    let Ok(path) = std::env::var("SCRIPTORIUM_SAMPLE_PDF") else {
        return Ok(());
    };
    let started = std::time::Instant::now();
    let pdf = PdfSource::open(std::path::Path::new(&path))?;
    let mut total = 0usize;
    for n in 1..=pdf.page_count() {
        let page = pdf.page_pdf(n, None)?;
        total += page.len();
        assert_eq!(PdfSource::from_bytes(&page)?.page_count(), 1, "page {n}");
    }
    let open = OpenSource::Pdf(Box::new(pdf));
    println!(
        "{} pages, {} KB of single-page PDFs in {:?}; warnings: {:?}",
        match &open {
            OpenSource::Pdf(p) => p.page_count(),
            _ => 0,
        },
        total / 1024,
        started.elapsed(),
        open.warnings(SourceId::new())
            .iter()
            .map(|w| &w.kind)
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn recordings_go_whole_when_the_model_takes_their_format() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(|e| MediaError::Image(e.to_string()))?;
    let path = dir.path().join("entretien.m4a");
    std::fs::write(&path, b"fake m4a").map_err(|e| MediaError::Image(e.to_string()))?;
    let source = inspect(&path, None)?;
    let open = OpenSource::open(&source, None)?;
    let pieces = open.pieces(false, None, crate::DEFAULT_PIECE_MS);
    assert_eq!(pieces.len(), 1);
    let input = open.prepare(&pieces[0], &["audio/mpeg", "audio/mp4"], "entretien")?;
    assert_eq!(input.mime, "audio/mp4");
    assert_eq!(input.file_name, "entretien.m4a");
    assert!(matches!(
        open.prepare(&pieces[0], &["audio/mpeg"], "entretien"),
        Err(MediaError::NeedsConversion(_))
    ));
    Ok(())
}
