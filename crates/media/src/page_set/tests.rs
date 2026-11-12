use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, ImageFormat, RgbImage};
use kalamu_core::{PageFile, PieceRef, SourceFile, SourceId, WarningKind};

use crate::{MediaError, OpenSource, check_unchanged, inspect};

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let _ = DynamicImage::ImageRgb8(RgbImage::new(w, h))
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png);
    out
}

fn io(e: std::io::Error) -> MediaError {
    MediaError::Image(e.to_string())
}

/// A document of three image files in a folder: each inspected as New job does, then made one
/// document in the order given.
fn three_pages(dir: &Path, sizes: [(u32, u32); 3]) -> Result<SourceFile, MediaError> {
    let mut pages = Vec::new();
    for (i, (w, h)) in sizes.into_iter().enumerate() {
        let path = dir.join(format!("page {}.png", i + 1));
        std::fs::write(&path, png(w, h)).map_err(io)?;
        let inspected = inspect(&path, None)?;
        pages.extend(PageFile::of(&inspected));
    }
    Ok(SourceFile::image_document("Letters", pages))
}

#[test]
fn a_document_of_images_is_read_page_by_page_in_its_order() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let doc = three_pages(dir.path(), [(800, 1100), (1600, 1000), (900, 1200)])?;
    assert_eq!(doc.page_count, Some(3));
    check_unchanged(&doc)?;
    let open = OpenSource::open(&doc, None)?;
    assert_eq!(
        open.pieces(false, None, 0),
        [PieceRef::page(1), PieceRef::page(2), PieceRef::page(3)]
    );
    // Its second file is a spread: split as a PDF's would be.
    assert_eq!(open.pieces(true, None, 0).len(), 4);
    assert_eq!(
        open.pieces(false, Some(2), 0).len(),
        2,
        "a sample of the first pages"
    );
    let second = open.prepare(&PieceRef::page(2), &["image/png"], "Letters")?;
    assert_eq!(second.bytes, png(1600, 1000), "the original file, as it is");
    assert_eq!(second.file_name, "Letters-p2.png");
    assert!(
        open.warnings(SourceId::new())
            .iter()
            .any(|w| w.kind == WarningKind::DoublePageSpread { pages: vec![2] })
    );
    assert!(open.text_pieces(None).is_err(), "no text to correct");
    Ok(())
}

#[test]
fn a_page_whose_file_was_replaced_or_moved_fails_alone() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let doc = three_pages(dir.path(), [(800, 1100), (800, 1100), (800, 1100)])?;
    // The second file replaced by another image, the third removed.
    std::fs::write(dir.path().join("page 2.png"), png(900, 1100)).map_err(io)?;
    std::fs::remove_file(dir.path().join("page 3.png")).map_err(io)?;
    let open = OpenSource::open(&doc, None)?;
    assert!(
        open.prepare(&PieceRef::page(1), &["image/png"], "x")
            .is_ok()
    );
    assert!(matches!(
        open.prepare(&PieceRef::page(2), &["image/png"], "x"),
        Err(MediaError::PageChanged(name)) if name == "page 2.png"
    ));
    assert!(matches!(
        open.prepare(&PieceRef::page(3), &["image/png"], "x"),
        Err(MediaError::Read { path, .. }) if path.ends_with("page 3.png")
    ));
    Ok(())
}

#[test]
fn a_page_replaced_by_a_file_of_the_same_size_is_found_by_its_hash() -> Result<(), MediaError> {
    let dir = tempfile::tempdir().map_err(io)?;
    let mut doc = three_pages(dir.path(), [(800, 1100), (800, 1100), (800, 1100)])?;
    // Same size, other content: the time it changed is unknown, so the hash decides.
    doc.pages[0].modified_ms = None;
    doc.pages[0].sha256 = "0".repeat(64);
    let open = OpenSource::open(&doc, None)?;
    assert!(matches!(
        open.prepare(&PieceRef::page(1), &["image/png"], "x"),
        Err(MediaError::PageChanged(_))
    ));
    Ok(())
}
