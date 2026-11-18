use super::*;
use crate::regions::{block_id, line_id, word_id};
use crate::{TextLine, TextRegion, TextWord};

/// A block of lines of words, as Tesseract places them.
fn tesseract_block(x: f32, y: f32, lines: &[&str]) -> TextRegion {
    let lines: Vec<TextLine> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let ly = y + i as f32 * 0.03;
            TextLine {
                x,
                y: ly,
                width: 0.4,
                height: 0.02,
                words: line
                    .split_whitespace()
                    .enumerate()
                    .map(|(k, w)| TextWord {
                        x: x + k as f32 * 0.06,
                        y: ly,
                        width: 0.05,
                        height: 0.02,
                        text: w.to_owned(),
                        confidence: Some(0.9),
                    })
                    .collect(),
            }
        })
        .collect();
    TextRegion {
        x,
        y,
        width: 0.4,
        height: 0.03 * lines.len() as f32,
        text: lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n"),
        lines,
    }
}

/// Two blocks of a page read by Tesseract under result `key`: "Le cercle de Kita" over
/// "Dakar, le 3 mai", then "Fin du rapport.".
fn page(key: &str) -> PageRegions {
    let regions = vec![
        tesseract_block(0.1, 0.1, &["Le cercle de Kita", "Dakar, le 3 mai"]),
        tesseract_block(0.1, 0.5, &["Fin du rapport."]),
    ];
    let text = "Le cercle de Kita\nDakar, le 3 mai\n\nFin du rapport.";
    PageRegions {
        result: Some(key.to_owned()),
        ..PageRegions::of(&regions, text, text, false, Some(1.414))
    }
}

fn drawn(id: &str, adjusts: Option<(&str, &str)>) -> DrawnRegion {
    DrawnRegion {
        id: id.to_owned(),
        job_id: JobId::new(),
        source_id: SourceId::new(),
        page: 1,
        half: None,
        x: 0.2,
        y: 0.3,
        width: 0.25,
        height: 0.05,
        text: "note en marge".to_owned(),
        adjusts: adjusts.map(|(result, region)| AdjustedRegion {
            result: result.to_owned(),
            region: region.to_owned(),
        }),
        source_hash: "abc".to_owned(),
        created_at: Timestamp(1),
        updated_at: Timestamp(2),
    }
}

fn draft(x: f32, y: f32, width: f32, height: f32) -> RegionDraft {
    RegionDraft {
        id: None,
        job_id: JobId::new(),
        source_id: SourceId::new(),
        page: 1,
        half: None,
        x,
        y,
        width,
        height,
        text: String::new(),
        adjusts: None,
    }
}

#[test]
fn every_region_has_an_id_by_its_place_at_each_granularity() {
    let page = page("k1");
    let ids = |list: &[ShownRegion]| list.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids(&page.blocks), ["r1", "r2"]);
    assert_eq!(ids(&page.lines), ["r1_l1", "r1_l2", "r2_l1"]);
    assert_eq!(page.words[0].id, "r1_l1_w1");
    assert_eq!(page.words[4].id, "r1_l2_w1");
    assert_eq!(page.words.last().map(|w| w.id.as_str()), Some("r2_l1_w3"));
    assert_eq!(
        (block_id(2), line_id(2, 1), word_id(2, 1, 3)),
        ("r2".to_owned(), "r2_l1".to_owned(), "r2_l1_w3".to_owned())
    );
    assert!(
        page.blocks
            .iter()
            .all(|r| r.granularity == Granularity::Block)
    );
    assert!(
        page.lines
            .iter()
            .all(|r| r.granularity == Granularity::Line)
    );
    assert!(
        page.words
            .iter()
            .all(|r| r.granularity == Granularity::Word)
    );
    // The same result shown again gives the same ids.
    assert_eq!(ids(&page.words), ids(&self::page("k1").words));
    // Lines estimated within a block placed whole are numbered the same way.
    let whole = TextRegion::block(0.1, 0.2, 0.4, 0.1, "Dakar, le\n3 mai");
    let estimated = PageRegions::of(&[whole], "Dakar, le\n3 mai", "x", false, Some(1.414));
    assert_eq!(ids(&estimated.lines), ["r1_l1", "r1_l2"]);
    assert_eq!(estimated.words[2].id, "r1_l2_w1");
    assert_eq!(estimated.words[2].origin, RegionOrigin::Estimated);
}

#[test]
fn an_adjusted_region_keeps_its_place_text_and_span_with_the_researcher_s_box() {
    let before = page("k1");
    let after = page("k1").with_drawn(&[drawn("d1", Some(("k1", "r1_l2")))]);
    let line = &after.lines[1];
    assert_eq!(line.id, "r1_l2");
    assert_eq!(
        (line.x, line.y, line.width, line.height),
        (0.2, 0.3, 0.25, 0.05)
    );
    assert_eq!(line.origin, RegionOrigin::Manual);
    // The box's id, for putting the engine's own back by removing it.
    assert_eq!(line.adjusted_by.as_deref(), Some("d1"));
    assert_eq!(before.lines[1].adjusted_by, None);
    assert_eq!(line.text, before.lines[1].text);
    assert_eq!(line.span, before.lines[1].span);
    assert!(line.span.is_some());
    // Not shown twice, and nothing else moved.
    assert!(after.drawn.is_empty());
    assert_eq!(after.words, before.words);
    assert_eq!(after.blocks, before.blocks);
}

#[test]
fn under_another_reading_an_adjustment_is_shown_as_the_researcher_s_own() {
    let read_again = page("k2").with_drawn(&[
        drawn("d1", Some(("k1", "r1_l2"))),
        drawn("d2", None),
        // The same result, but a region it does not have.
        drawn("d3", Some(("k2", "r9"))),
    ]);
    assert_eq!(read_again.lines, page("k2").lines);
    let ids: Vec<&str> = read_again.drawn.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["d1", "d2", "d3"]);
    for r in &read_again.drawn {
        assert_eq!(r.origin, RegionOrigin::Manual);
        assert_eq!(r.granularity, Granularity::Block);
        assert_eq!(r.span, None);
        assert_eq!(r.text, "note en marge");
        // The researcher's own region adjusts nothing shown.
        assert_eq!(r.adjusted_by, None);
    }
    assert!(read_again.lines.iter().all(|l| l.adjusted_by.is_none()));
    // A page not read yet shows the researcher's regions too.
    let unread = PageRegions::none(false).with_drawn(&[drawn("d1", Some(("k1", "r1")))]);
    assert_eq!(unread.drawn.len(), 1);
    assert!(unread.blocks.is_empty());
}

#[test]
fn a_box_is_refused_when_it_is_not_a_number_far_outside_or_tiny() {
    assert_eq!(
        check(&draft(f32::NAN, 0.1, 0.2, 0.2)),
        Err(RegionError::NotFinite)
    );
    assert_eq!(
        check(&draft(0.1, 0.1, f32::INFINITY, 0.2)),
        Err(RegionError::NotFinite)
    );
    assert_eq!(
        check(&draft(-0.05, 0.1, 0.2, 0.2)),
        Err(RegionError::OutsidePage)
    );
    assert_eq!(
        check(&draft(0.9, 0.1, 0.2, 0.2)),
        Err(RegionError::OutsidePage)
    );
    assert_eq!(
        check(&draft(0.1, 0.1, 0.002, 0.2)),
        Err(RegionError::TooSmall)
    );
    let page_zero = RegionDraft {
        page: 0,
        ..draft(0.1, 0.1, 0.2, 0.2)
    };
    assert_eq!(check(&page_zero), Err(RegionError::NoPage));
    // Every refusal has an id the interface translates.
    for e in [
        RegionError::NotFinite,
        RegionError::OutsidePage,
        RegionError::TooSmall,
        RegionError::NoPage,
        RegionError::Adjusts,
        RegionError::NotFound,
    ] {
        assert!(e.code().starts_with("region."));
    }
}

#[test]
fn a_hair_past_the_edge_is_clamped_and_a_box_drawn_backwards_is_turned_round() {
    let over = check(&draft(-0.005, 0.5, 0.5, 0.508)).unwrap();
    assert_eq!((over.x, over.y), (0.0, 0.5));
    assert!((over.width - 0.495).abs() < 1e-6);
    assert!((over.y + over.height - 1.0).abs() < 1e-6);
    // Dragged up and to the left: the same box from its top-left corner.
    let back = check(&draft(0.6, 0.4, -0.2, -0.1)).unwrap();
    assert!((back.x - 0.4).abs() < 1e-6 && (back.y - 0.3).abs() < 1e-6);
    assert!((back.width - 0.2).abs() < 1e-6 && (back.height - 0.1).abs() < 1e-6);
}

#[test]
fn a_split_spread_s_halves_keep_their_regions_apart() {
    let left = DrawnRegion {
        half: Some(PageHalf::Left),
        ..drawn("d1", None)
    };
    assert!(left.on(1, Some(PageHalf::Left)));
    assert!(!left.on(1, Some(PageHalf::Right)));
    assert!(!left.on(1, None));
    assert!(!left.on(2, Some(PageHalf::Left)));
    // Its box stays in fractions of its half, as the review shows the half alone.
    assert_eq!(left.shown().x, 0.2);
}

#[test]
fn the_researcher_s_text_is_kept_as_written_in_any_script() {
    let arabic = DrawnRegion {
        text: "بسم الله الرحمن الرحيم — Réception à Bamako".to_owned(),
        ..drawn("d1", None)
    };
    let shown = PageRegions::none(false).with_drawn(std::slice::from_ref(&arabic));
    assert_eq!(shown.drawn[0].text, arabic.text);
    let json = serde_json::to_string(&arabic).unwrap();
    let back: DrawnRegion = serde_json::from_str(&json).unwrap();
    assert_eq!(back, arabic);
}
