//! The researcher's regions in ALTO, PAGE XML and the searchable PDF's text layer (#84): a block,
//! a line and a word of the model's given another box, regions of their own with and without
//! text, a split spread, a corrected page and the lines of a block placed whole, as golden
//! files. The parts are made as the engine makes them, with core's `lay_for_export` and
//! `contain`.

use kalamu_core::export::{BoxKind, ResearcherGeometry};
use kalamu_core::regions::{contain, lay_for_export};
use kalamu_core::{
    AdjustedRegion, DrawnRegion, JobId, PageHalf, PieceRef, TextLine, TextRegion, TextWord,
};

use super::*;

/// When the pages were read, and when the researcher placed their boxes, a day later.
const READ: Timestamp = Timestamp(1_793_096_727_000);
const PLACED: Timestamp = Timestamp(1_793_183_127_000);

/// A line of words as Tesseract places them, each 0.07 wide, from `x`.
fn line(x: f32, y: f32, words: &[&str]) -> TextLine {
    let words: Vec<TextWord> = words
        .iter()
        .enumerate()
        .map(|(i, text)| word(x + i as f32 * 0.08, y, 0.07, text, Some(0.9)))
        .collect();
    TextLine {
        x,
        y,
        width: words.len() as f32 * 0.08 - 0.01,
        height: 0.02,
        words,
    }
}

/// A block of Tesseract's lines.
fn placed(lines: Vec<TextLine>) -> TextRegion {
    let (x, y) = (lines[0].x, lines[0].y);
    let right = lines.iter().map(|l| l.x + l.width).fold(0.0, f32::max);
    let bottom = lines.iter().map(|l| l.y + l.height).fold(0.0, f32::max);
    let text = lines
        .iter()
        .map(TextLine::text)
        .collect::<Vec<_>>()
        .join("\n");
    TextRegion {
        lines,
        ..TextRegion::block(x, y, right - x, bottom - y, text)
    }
}

/// A region of the researcher's: drawn (`adjusts` none) or giving a box to region `adjusts` of
/// reading `k1`.
fn theirs(
    n: u32,
    half: Option<PageHalf>,
    adjusts: Option<&str>,
    b: [f32; 4],
    text: &str,
) -> DrawnRegion {
    DrawnRegion {
        id: format!("6f1c2a3b-0000-4000-8000-00000000000{n}"),
        job_id: JobId(uuid_nil()),
        source_id: SourceId(uuid_nil()),
        page: 1,
        half,
        x: b[0],
        y: b[1],
        width: b[2],
        height: b[3],
        text: text.into(),
        adjusts: adjusts.map(|region| AdjustedRegion {
            result: "k1".into(),
            region: region.into(),
        }),
        source_hash: "0f".repeat(32),
        created_at: PLACED,
        updated_at: PLACED,
    }
}

/// A part of page `piece` read as `model`, with the researcher's regions laid on it as the
/// engine lays them; `aspect` the piece's proportions as the review shows it.
fn part(
    piece: PieceRef,
    model: &[TextRegion],
    drawn: &[DrawnRegion],
    aspect: Option<f32>,
) -> ExportPart {
    let laid = lay_for_export(model, Some("k1"), drawn, aspect);
    let mut regions = laid.regions;
    let researcher = contain(&mut regions, &laid.boxes, laid.estimated, &laid.own);
    let text = regions
        .iter()
        .map(|r| r.text.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    ExportPart {
        piece,
        text,
        segments: vec![],
        corrected: false,
        reviewed: false,
        rows_corrected: Vec::new(),
        fields: vec![],
        regions,
        failed: None,
        drawn_dpi: None,
        read_at: Some(READ),
        corrected_at: None,
        researcher,
    }
}

/// A notebook of three A4 pages:
///
/// 1. Tesseract's two blocks: the first given a box of the researcher's that holds its line,
///    the second's line moved and its second word given a box; a note in the margin with text,
///    and a stamp marked without any.
/// 2. A spread read in halves: on the left, Mistral OCR's paragraph given a wider box; on the
///    right, Tesseract's line, and a note of the researcher's on that half.
/// 3. A corrected page (the words already laid on Tesseract's boxes), one corrected word given a
///    box; then Mistral OCR's block whose second estimated line was given one.
fn notebook(dir: &tempfile::TempDir) -> ExportDocument {
    let path = dir.path().join("carnet.pdf");
    pdf_of_pages(&path, &[(595, 842), (595, 842), (595, 842)]);
    let mut src = source("carnet.pdf", InputKind::Pdf);
    src.path = path.display().to_string();
    src.split_spreads = true;

    let first = [
        placed(vec![line(0.1, 0.1, &["Dakar,", "le", "3", "mai"])]),
        placed(vec![line(0.1, 0.2, &["Le", "Gouverneur"])]),
    ];
    let page_1 = part(
        PieceRef::page(1),
        &first,
        &[
            theirs(1, None, Some("r1"), [0.08, 0.09, 0.36, 0.04], ""),
            theirs(2, None, Some("r2_l1"), [0.1, 0.205, 0.2, 0.025], ""),
            theirs(3, None, Some("r2_l1_w2"), [0.18, 0.205, 0.12, 0.025], ""),
            theirs(
                4,
                None,
                None,
                [0.7, 0.1, 0.2, 0.05],
                "Reçu le 4 mai & classé",
            ),
            theirs(5, None, None, [0.7, 0.8, 0.15, 0.1], ""),
        ],
        None,
    );

    let half = |side| PieceRef::Page {
        number: 2,
        half: Some(side),
    };
    // A half of an A4 page alone, as the review shows it: twice as tall as wide, near enough.
    let half_aspect = 842.0 / (595.0 / 2.0);
    let left = part(
        half(PageHalf::Left),
        &[TextRegion::block(
            0.1,
            0.1,
            0.8,
            0.2,
            "Le cercle de Kita a reçu les notables",
        )],
        &[DrawnRegion {
            page: 2,
            half: Some(PageHalf::Left),
            ..theirs(
                6,
                Some(PageHalf::Left),
                Some("r1"),
                [0.05, 0.08, 0.9, 0.25],
                "",
            )
        }],
        Some(half_aspect),
    );
    let right = part(
        half(PageHalf::Right),
        &[placed(vec![line(0.1, 0.1, &["Bamako"])])],
        &[DrawnRegion {
            page: 2,
            ..theirs(
                7,
                Some(PageHalf::Right),
                None,
                [0.5, 0.5, 0.4, 0.1],
                "Tampon du cercle",
            )
        }],
        Some(half_aspect),
    );

    let corrected = [
        placed(vec![line(0.1, 0.1, &["Le", "cercle", "de", "Kita"])]),
        TextRegion::block(
            0.1,
            0.3,
            0.5,
            0.1,
            "Le commandant a réuni\nles notables du quartier",
        ),
    ];
    let mut page_3 = part(
        PieceRef::page(3),
        &corrected,
        &[
            DrawnRegion {
                page: 3,
                ..theirs(8, None, Some("r1_l1_w4"), [0.34, 0.1, 0.09, 0.02], "")
            },
            DrawnRegion {
                page: 3,
                ..theirs(9, None, Some("r2_l2"), [0.1, 0.36, 0.45, 0.03], "")
            },
        ],
        Some(842.0 / 595.0),
    );
    page_3.corrected = true;
    page_3.corrected_at = Some(PLACED);
    document(Task::TextExtraction, src, vec![page_1, left, right, page_3])
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
}

#[test]
fn the_researcher_s_geometry_is_what_core_lays() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let doc = notebook(&dir);
    let kinds =
        |g: &ResearcherGeometry| -> Vec<BoxKind> { g.adjusted.iter().map(|a| a.kind).collect() };
    let first = &doc.parts[0].researcher;
    // r1 (theirs, holding its line), r2 widened for its line, r2_l1 theirs widened to hold its
    // first word, r2_l1_w2 theirs.
    assert_eq!(
        kinds(first),
        [
            BoxKind::Researcher,
            BoxKind::ModelWidened,
            BoxKind::ResearcherWidened,
            BoxKind::Researcher,
        ]
    );
    assert_eq!(first.drawn.len(), 2);
    assert_eq!(doc.parts[3].researcher.estimated, [1]);
}

#[test]
fn page_xml_says_which_positions_the_researcher_set_as_golden_files() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let doc = notebook(&dir);
    let files = exporter(ExportFormat::PageXml)
        .unwrap_or_else(|| panic!("no PAGE XML exporter"))
        .export_files(&doc)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(files.len(), 3);
    for file in &files {
        check(&format!("researcher.{}", file.suffix), &file.bytes);
        page_xml::valid(&text(&file.bytes));
    }
    let first = text(&files[0].bytes);
    // The model's ids stay; the researcher's own come after, as valid XML ids.
    assert!(first.contains("<TextRegion id=\"r1\" custom=\"readingOrder {index:0;}\" comments=\"position set by the researcher (region 6f1c2a3b-0000-4000-8000-000000000001)\">"), "{first}");
    assert!(first.contains("regionRef=\"researcher_6f1c2a3b-0000-4000-8000-000000000004\""));
    // A region without text is its box alone.
    assert!(first.contains(
        "<TextRegion id=\"researcher_6f1c2a3b-0000-4000-8000-000000000005\" custom=\"readingOrder {index:3;}\" comments=\"region placed by the researcher (region 6f1c2a3b-0000-4000-8000-000000000005)\">\n      <Coords points=\"1735,2806 2107,2806 2107,3157 1735,3157\"/>\n    </TextRegion>"
    ), "{first}");
    assert!(first.contains("<MetadataItem type=\"processingStep\" name=\"review\""));
    assert!(first.contains("<LastChange>2026-10-28T"), "{first}");
    // Page 3: a corrected word given a box says so, in its line placed by the model.
    let third = text(&files[2].bytes);
    assert!(third.contains("text corrected by the researcher"));
    assert!(third.contains("<Word id=\"r1_l1_w4\" comments=\"position set by the researcher"));
}

#[test]
fn alto_refers_the_researcher_s_positions_to_a_processing_step_as_a_golden_file() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let doc = notebook(&dir);
    let xml = run(ExportFormat::Alto, &doc);
    check("researcher.alto.xml", &xml);
    let xml = text(&xml);
    assert_eq!(xml.matches("<Processing ID=").count(), 2);
    assert!(xml.contains("<processingCategory>contentModification</processingCategory>"));
    assert!(xml.contains("<TextBlock ID=\"p1_b1\" PROCESSINGREFS=\"processing_2\""));
    assert!(xml.contains(
        "<String ID=\"p1_b2_l1_w2\" CONTENT=\"Gouverneur\" PROCESSINGREFS=\"processing_2\""
    ));
    // Text of the researcher's own, findable; a region without text is not ALTO's.
    assert!(xml.contains("<TextBlock ID=\"researcher_6f1c2a3b-0000-4000-8000-000000000004\" PROCESSINGREFS=\"processing_2\""));
    assert!(xml.contains("CONTENT=\"&amp;\""));
    assert!(!xml.contains("000000000005"));
    // The note on the right half lands on the right of the whole page: 0.5 + 0.5 / 2 of 2479.
    assert!(xml.contains("researcher_6f1c2a3b-0000-4000-8000-000000000007\" PROCESSINGREFS=\"processing_2\" HPOS=\"1859\""), "{xml}");
}

#[test]
fn the_text_layer_sets_the_researcher_s_text_over_its_box_as_golden_files() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let doc = notebook(&dir);
    let pages = crate::searchable_pdf::texts_by_page(&doc);
    for (number, regions) in &pages {
        let layer = crate::searchable_pdf::text_layer(regions, 595.0, 842.0);
        check(
            &format!("researcher.p{number:04}.layer.txt"),
            layer.as_bytes(),
        );
    }
    // Page 1: the model's two lines, then the note; the stamp without text sets nothing.
    let first = crate::searchable_pdf::text_layer(&pages[&1], 595.0, 842.0);
    assert_eq!(first.matches(" Tf\n").count(), 3, "{first}");
    // The PDF itself holds the note, findable.
    let bytes = run(ExportFormat::SearchablePdf, &doc);
    let out = lopdf::Document::load_mem(&bytes).unwrap_or_else(|e| panic!("{e}"));
    let found = out.extract_text(&[1, 2]).unwrap_or_default();
    assert!(
        found.contains("classé") && found.contains("Tampon"),
        "{found:?}"
    );
}

#[test]
fn a_document_without_the_researcher_s_regions_is_written_as_before() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut doc = notebook(&dir);
    for part in &mut doc.parts {
        part.researcher = ResearcherGeometry::default();
    }
    let xml = text(&run(ExportFormat::Alto, &doc));
    assert!(!xml.contains("processing_2") && !xml.contains("researcher_"));
    let files = exporter(ExportFormat::PageXml)
        .unwrap_or_else(|| panic!("no PAGE XML exporter"))
        .export_files(&doc)
        .unwrap_or_else(|e| panic!("{e}"));
    let first = text(&files[0].bytes);
    assert!(
        !first.contains("MetadataItem") && !first.contains("researcher"),
        "{first}"
    );
}

/// A page the model read without positions, where the researcher wrote a note: ALTO and PAGE XML
/// hold the note; nothing at all, and there are none.
#[test]
fn a_note_of_the_researcher_s_alone_is_written_and_nothing_at_all_is_refused() {
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let mut doc = notebook(&dir);
    doc.parts = vec![part(
        PieceRef::page(1),
        &[],
        &[theirs(
            4,
            None,
            None,
            [0.7, 0.1, 0.2, 0.05],
            "Reçu le 4 mai",
        )],
        None,
    )];
    let xml = text(&run(ExportFormat::Alto, &doc));
    assert!(xml.contains("CONTENT=\"Reçu\""), "{xml}");
    doc.parts[0].researcher = ResearcherGeometry::default();
    assert!(exporter(ExportFormat::Alto).is_some_and(|e| e.export(&doc).is_err()));
}
