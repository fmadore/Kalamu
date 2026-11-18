//! A region read again refused before anything is sent: a model that reads no pictures, a box
//! off the page or too small, a page the source lacks, a region gone, a file gone; each refusal
//! with the code the interface translates.

use kalamu_core::{PageHalf, ReadingTarget, RegionAsk};
use kalamu_media::{MediaError, RegionRefusal};

use super::*;

#[tokio::test]
async fn refusals_reach_the_interface_with_their_own_codes() {
    let s = setting().await;
    // A model that reads no pictures: refused before anything is cut or sent.
    let pdf_only = RegionAsk {
        provider: ProviderId::new("fake"),
        model: "m".into(),
        ..s.ask("g1")
    };
    let err = s
        .engine
        .region_reading_estimate(&pdf_only)
        .await
        .err()
        .unwrap_or_else(|| panic!("a model that reads no pictures was asked"));
    assert_eq!(err.code(), "media.region_no_images");
    // A box off the page, or a page the source does not have.
    let off = RegionAsk {
        x: 0.9,
        width: 0.5,
        ..s.ask("g2")
    };
    let err = s.engine.ask_region_reading(off).await.err();
    assert_eq!(err.map(|e| e.code()), Some("region.outside"));
    let no_page = RegionAsk {
        page: 4,
        ..s.ask("g3")
    };
    let err = s.engine.ask_region_reading(no_page).await.err();
    assert_eq!(err.map(|e| e.code()), Some("region.no_page"));
    let half = RegionAsk {
        half: Some(PageHalf::Left),
        ..s.ask("g4")
    };
    assert!(s.engine.ask_region_reading(half).await.is_err());
    // A region the researcher drew that is gone.
    let gone = RegionAsk {
        region_id: "nowhere".into(),
        ..s.ask("g5")
    };
    let err = s.engine.ask_region_reading(gone).await.err();
    assert_eq!(err.map(|e| e.code()), Some("region.not_found"));
    // Each of media's refusals keeps its own code rather than "the file could not be read".
    for refusal in [
        RegionRefusal::NotFinite,
        RegionRefusal::OutsidePage,
        RegionRefusal::TooSmall,
        RegionRefusal::NoPage,
        RegionRefusal::NoImages,
    ] {
        let err = EngineError::Media(MediaError::Region(refusal));
        assert_eq!(err.code(), refusal.code());
    }
    assert_eq!(s.calls(), 0);
}

#[tokio::test]
async fn a_region_too_small_on_a_small_picture_is_refused_by_media() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let fake = Fake::new();
    let reader = Reader::new("reader", false);
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = engine_with(
        store.clone(),
        vec![fake.clone(), reader.clone()],
        dir.path(),
    );
    let f = Fixture {
        engine: engine.clone(),
        fake,
        store,
        dir,
    };
    // A photograph 64 pixels wide: a box a twentieth of it is three pixels.
    let mut png = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(64, 48, image::Rgb([9, 9, 9])))
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap_or_else(|e| panic!("{e}"));
    let path = f.dir.path().join("lettre.png");
    std::fs::write(&path, png).unwrap_or_else(|e| panic!("{e}"));
    let source = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = f.new_job(vec![source], &["m"]);
    let ask = RegionAsk {
        id: "g1".into(),
        job_id: job.id,
        source_id: job.sources[0].id,
        page: 1,
        half: None,
        x: 0.2,
        y: 0.2,
        width: 0.05,
        height: 0.5,
        region_id: "r1".into(),
        target: ReadingTarget::Nowhere,
        provider: ProviderId::new("reader"),
        model: "reader-model".into(),
        preset_id: None,
    };
    let err = engine.region_reading_estimate(&ask).await.err();
    assert_eq!(err.map(|e| e.code()), Some("media.region_too_small"));
    assert_eq!(reader.calls.load(Ordering::SeqCst), 0);
    // The file gone: refused as a job's page would be, and nothing sent.
    let bigger = RegionAsk { width: 0.5, ..ask };
    std::fs::remove_file(&path).unwrap_or_else(|e| panic!("{e}"));
    let err = engine.ask_region_reading(bigger).await.err();
    assert!(err.is_some_and(|e| e.code().starts_with("media.")));
    assert_eq!(reader.calls.load(Ordering::SeqCst), 0);
}
