use super::*;
use crate::{AdjustedRegion, JobId, PageHalf, SourceId, TextLine, TextWord, Timestamp};

/// A block of one line of two words, as Tesseract places them.
fn block(y: f32) -> TextRegion {
    let word = |x: f32, text: &str| TextWord {
        x,
        y,
        width: 0.1,
        height: 0.02,
        text: text.into(),
        confidence: Some(0.9),
    };
    TextRegion {
        lines: vec![TextLine {
            x: 0.1,
            y,
            width: 0.25,
            height: 0.02,
            words: vec![word(0.1, "Dakar,"), word(0.25, "mai")],
        }],
        ..TextRegion::block(0.1, y, 0.25, 0.02, "Dakar, mai")
    }
}

fn drawn(id: &str, adjusts: Option<(&str, &str)>, place: (f32, f32, f32, f32)) -> DrawnRegion {
    DrawnRegion {
        id: id.into(),
        job_id: JobId::new(),
        source_id: SourceId::new(),
        page: 1,
        half: None,
        x: place.0,
        y: place.1,
        width: place.2,
        height: place.3,
        text: String::new(),
        adjusts: adjusts.map(|(result, region)| AdjustedRegion {
            result: result.into(),
            region: region.into(),
        }),
        source_hash: "ab".into(),
        created_at: Timestamp(1),
        updated_at: Timestamp(2),
    }
}

fn path(block: usize, line: Option<usize>, word: Option<usize>) -> RegionPath {
    RegionPath { block, line, word }
}

#[test]
fn an_id_gives_back_its_place_and_only_the_review_s_ids_do() {
    assert_eq!(path_of("r1"), Some(path(0, None, None)));
    assert_eq!(path_of("r2_l3"), Some(path(1, Some(2), None)));
    assert_eq!(path_of("r2_l3_w10"), Some(path(1, Some(2), Some(9))));
    for id in [
        "r0",
        "r01",
        "r1_w2",
        "r1_l0",
        "r1_l1_w1_x",
        "1",
        "r",
        "r1_l",
        "d-old",
    ] {
        assert_eq!(path_of(id), None, "{id}");
    }
}

#[test]
fn a_box_for_this_reading_replaces_the_model_s_and_any_other_is_the_researcher_s_own() {
    let model = vec![block(0.1), block(0.5)];
    let laid = lay_for_export(
        &model,
        Some("k1"),
        &[
            drawn("a", Some(("k1", "r2_l1_w2")), (0.3, 0.5, 0.1, 0.02)),
            drawn("b", None, (0.6, 0.6, 0.2, 0.1)),
            drawn("c", Some(("k0", "r1")), (0.1, 0.1, 0.2, 0.1)),
            drawn("d", Some(("k1", "r9")), (0.1, 0.1, 0.2, 0.1)),
        ],
        None,
    );
    let word = &laid.regions[1].lines[0].words[1];
    assert_eq!((word.x, word.width), (0.3, 0.1));
    assert_eq!(word.text, "mai", "the model's text kept");
    assert_eq!(laid.regions[0], model[0]);
    assert_eq!(laid.boxes.len(), 1);
    assert_eq!(laid.boxes[0].0, path(1, Some(0), Some(1)));
    let own: Vec<&str> = laid.own.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(own, ["b", "c", "d"]);
}

#[test]
fn a_block_moved_is_widened_to_hold_the_lines_the_model_measured_in_it() {
    let model = vec![block(0.1)];
    // Narrower than its line, which the researcher did not move.
    let mut laid = lay_for_export(
        &model,
        Some("k1"),
        &[drawn("a", Some(("k1", "r1")), (0.12, 0.09, 0.1, 0.04))],
        None,
    );
    let geometry = contain(&mut laid.regions, &laid.boxes, laid.estimated, &laid.own);
    let b = &laid.regions[0];
    assert!((b.x - 0.1).abs() < 1e-6 && (b.y - 0.09).abs() < 1e-6);
    assert!((b.x + b.width - 0.35).abs() < 1e-6 && (b.y + b.height - 0.13).abs() < 1e-6);
    assert_eq!(b.lines, model[0].lines, "the lines stay where measured");
    assert_eq!(geometry.adjusted.len(), 1);
    assert_eq!(geometry.adjusted[0].kind, BoxKind::ResearcherWidened);

    // A box that already holds its lines is written as drawn.
    let mut laid = lay_for_export(
        &model,
        Some("k1"),
        &[drawn("a", Some(("k1", "r1")), (0.05, 0.05, 0.4, 0.1))],
        None,
    );
    let geometry = contain(&mut laid.regions, &laid.boxes, laid.estimated, &laid.own);
    assert_eq!(laid.regions[0].x, 0.05);
    assert_eq!(geometry.adjusted[0].kind, BoxKind::Researcher);
    assert_eq!(geometry.adjusted[0].updated_at, Timestamp(2));
}

#[test]
fn a_line_moved_alone_widens_the_model_s_block_that_holds_it() {
    let model = vec![block(0.1)];
    let mut laid = lay_for_export(
        &model,
        Some("k1"),
        &[drawn("a", Some(("k1", "r1_l1")), (0.08, 0.1, 0.3, 0.03))],
        None,
    );
    let geometry = contain(&mut laid.regions, &laid.boxes, laid.estimated, &laid.own);
    let b = &laid.regions[0];
    assert!((b.x - 0.08).abs() < 1e-6 && (b.y + b.height - 0.13).abs() < 1e-6);
    let kinds: Vec<(RegionPath, BoxKind)> =
        geometry.adjusted.iter().map(|a| (a.at, a.kind)).collect();
    assert_eq!(
        kinds,
        [
            (path(0, None, None), BoxKind::ModelWidened),
            (path(0, Some(0), None), BoxKind::Researcher),
        ]
    );
    assert!(geometry.adjusted.iter().all(|a| a.id == "a"));
}

#[test]
fn a_line_estimated_in_a_block_placed_whole_is_written_out_to_keep_its_box() {
    let whole = TextRegion::block(0.1, 0.2, 0.4, 0.1, "Dakar, le\n3 mai");
    let aspect = 1.414;
    let mut laid = lay_for_export(
        std::slice::from_ref(&whole),
        Some("k1"),
        &[drawn("a", Some(("k1", "r1_l2")), (0.1, 0.26, 0.2, 0.04))],
        Some(aspect),
    );
    assert_eq!(laid.estimated, [0]);
    let lines = whole.estimated_lines(aspect);
    assert_eq!(laid.regions[0].lines.len(), lines.len());
    assert_eq!(
        laid.regions[0].lines[0], lines[0],
        "the review's own estimate"
    );
    assert_eq!(laid.regions[0].lines[1].y, 0.26);
    // Its estimated words came along, each with its share of the line.
    for word in &laid.regions[0].lines[1].words {
        assert_eq!(word.y, 0.26);
        assert!(
            word.x >= 0.1 - 1e-6 && word.x + word.width <= 0.3 + 1e-6,
            "{word:?}"
        );
    }
    let geometry = contain(&mut laid.regions, &laid.boxes, laid.estimated, &laid.own);
    assert_eq!(geometry.estimated, [0]);
    assert_eq!(geometry.adjusted[0].kind, BoxKind::Researcher);

    // Without the page's proportions the review drew no such line: the box adjusts nothing.
    let laid = lay_for_export(
        &[whole],
        Some("k1"),
        &[drawn("a", Some(("k1", "r1_l2")), (0.1, 0.26, 0.2, 0.04))],
        None,
    );
    assert!(laid.estimated.is_empty() && laid.boxes.is_empty());
    assert_eq!(laid.own.len(), 1);
}

#[test]
fn the_researcher_s_own_regions_keep_their_half_text_and_order() {
    let note = DrawnRegion {
        half: Some(PageHalf::Right),
        text: "Tampon".into(),
        ..drawn("n", None, (0.2, 0.3, 0.2, 0.1))
    };
    let mut laid = lay_for_export(
        &[],
        None,
        &[note, drawn("e", None, (0.1, 0.1, 0.1, 0.1))],
        None,
    );
    let geometry = contain(&mut laid.regions, &laid.boxes, laid.estimated, &laid.own);
    assert_eq!(geometry.drawn.len(), 2);
    assert_eq!(geometry.drawn[0].half, Some(PageHalf::Right));
    assert_eq!(geometry.drawn[0].region.text, "Tampon");
    assert!(geometry.drawn[0].region.lines.is_empty());
    assert_eq!(geometry.drawn[1].id, "e");
    assert!(!geometry.is_empty());
    // Nothing of theirs: nothing said.
    let mut regions = vec![block(0.1)];
    assert!(contain(&mut regions, &[], Vec::new(), &[]).is_empty());
    assert_eq!(regions, [block(0.1)]);
}
