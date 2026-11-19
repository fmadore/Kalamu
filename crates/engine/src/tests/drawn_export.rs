//! The researcher's regions in the exports (#84): the document exporters read carries the
//! boxes they gave the model's regions and their own regions, under the set-up whose reading
//! they adjust, through a correction and after the page is read again; ALTO and PAGE XML write
//! them.

use kalamu_core::export::{BoxKind, RegionPath};
use kalamu_core::{AdjustedRegion, Correction, DrawnRegion, ExportDocument, RegionDraft};

use super::drawn_regions::{draft, ok, read_report, read_report_by};
use super::*;

fn path(block: usize, line: Option<usize>, word: Option<usize>) -> RegionPath {
    RegionPath { block, line, word }
}

/// A box for region `region` of reading `result` on page 1.
fn adjusting(
    job: &Job,
    source: &SourceFile,
    result: &str,
    region: &str,
    b: [f32; 4],
) -> RegionDraft {
    RegionDraft {
        x: b[0],
        y: b[1],
        width: b[2],
        height: b[3],
        text: String::new(),
        ..draft(
            job,
            source,
            1,
            Some(AdjustedRegion {
                result: result.into(),
                region: region.into(),
            }),
        )
    }
}

fn written(format: ExportFormat, doc: &ExportDocument) -> String {
    let files = kalamu_export::exporter(format)
        .unwrap_or_else(|| panic!("no {format:?} exporter"))
        .export_files(doc)
        .unwrap_or_else(|e| panic!("{e}"));
    files
        .iter()
        .map(|f| String::from_utf8_lossy(&f.bytes).into_owned())
        .collect()
}

#[tokio::test]
async fn the_export_carries_the_researcher_s_boxes_and_regions_through_a_correction() {
    let (_f, job, source, quiet) = read_report().await;
    let setup = job.setups[0].id;
    let result = ok(quiet.page_regions(job.id, source.id, setup, 0))
        .result
        .unwrap_or_default();
    let note = ok(quiet.save_region(draft(&job, &source, 1, None)));
    let second_page = ok(quiet.save_region(draft(&job, &source, 2, None)));
    // The second line, a tighter box that still holds its words; the last word, misread
    // "rnai", a box reaching above its line.
    let line = ok(quiet.save_region(adjusting(
        &job,
        &source,
        &result,
        "r1_l2",
        [0.1, 0.128, 0.45, 0.024],
    )));
    let word = ok(quiet.save_region(adjusting(
        &job,
        &source,
        &result,
        "r1_l3_w4",
        [0.4, 0.158, 0.1, 0.025],
    )));
    let doc = ok(quiet.export_document(&job, setup, source.id));
    let first = &doc.parts[0];
    let l2 = &first.regions[0].lines[1];
    assert!(
        (l2.y - 0.128).abs() < 1e-6 && (l2.width - 0.45).abs() < 1e-6,
        "{l2:?}"
    );
    assert_eq!(first.regions[0].lines[2].words[3].x, 0.4);
    let marks: Vec<(RegionPath, BoxKind, &str)> = first
        .researcher
        .adjusted
        .iter()
        .map(|a| (a.at, a.kind, a.id.as_str()))
        .collect();
    assert_eq!(
        marks,
        [
            (path(0, None, None), BoxKind::ModelWidened, word.id.as_str()),
            (
                path(0, Some(1), None),
                BoxKind::Researcher,
                line.id.as_str()
            ),
            (
                path(0, Some(2), None),
                BoxKind::ModelWidened,
                word.id.as_str()
            ),
            (
                path(0, Some(2), Some(3)),
                BoxKind::Researcher,
                word.id.as_str()
            ),
        ]
    );
    // The block holds the word moved above its line.
    let block = &first.regions[0];
    assert!(block.y + block.height >= 0.183 - 1e-6);
    let own: Vec<&str> = first
        .researcher
        .drawn
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(own, [note.id.as_str()]);
    assert_eq!(first.researcher.drawn[0].region.text, note.text);
    // Page 2, read without positions, carries the note drawn there.
    assert!(doc.parts[1].regions.is_empty());
    assert_eq!(doc.parts[1].researcher.drawn[0].id, second_page.id);
    // The formats say so; page 2's note is ALTO's too.
    let alto = written(ExportFormat::Alto, &doc);
    assert!(alto.contains("<Processing ID=\"processing_2\">"), "{alto}");
    assert_eq!(alto.matches("PROCESSINGREFS=\"processing_2\"").count(), 6);
    assert!(alto.contains(&format!("researcher_{}", second_page.id)));
    let page = written(ExportFormat::PageXml, &doc);
    assert!(page.contains(&format!(
        "position set by the researcher (region {})",
        line.id
    )));

    // A correction of the misread word: the corrected word takes the researcher's box, the
    // line keeps theirs.
    let correct = |text: &str| {
        ok(quiet.save_correction(Correction {
            job_id: job.id,
            setup_id: setup,
            source_id: source.id,
            index: 0,
            text: Some(text.into()),
            fields: None,
            reviewed: false,
            updated_at: Timestamp::now(),
        }))
    };
    correct("Le cercle de Kita\nLe cercle de Kita\nDakar, le 3 mai");
    let doc = ok(quiet.export_document(&job, setup, source.id));
    let first = &doc.parts[0];
    assert!(first.corrected);
    let mai = &first.regions[0].lines[2].words[3];
    assert_eq!((mai.text.as_str(), mai.x, mai.y), ("mai", 0.4, 0.158));
    assert!((first.regions[0].lines[1].y - 0.128).abs() < 1e-6);
    assert_eq!(first.researcher.adjusted.len(), 4);

    // A correction the model's boxes cannot hold is one block over the page: the boxes the
    // researcher gave its regions are then their own, without text, after the note.
    // Four times as many words as the model read: more than its boxes can honestly hold.
    correct(
        &["Une page entièrement réécrite, avec bien plus de mots que le modèle n'en a lus."; 4]
            .join(" "),
    );
    let doc = ok(quiet.export_document(&job, setup, source.id));
    let first = &doc.parts[0];
    assert_eq!(first.regions.len(), 1);
    assert!(first.regions[0].lines.is_empty());
    assert!(first.researcher.adjusted.is_empty());
    let own: Vec<(&str, &str)> = first
        .researcher
        .drawn
        .iter()
        .map(|d| (d.id.as_str(), d.region.text.as_str()))
        .collect();
    assert_eq!(
        own,
        [
            (note.id.as_str(), note.text.as_str()),
            (line.id.as_str(), ""),
            (word.id.as_str(), "")
        ]
    );
    let page = written(ExportFormat::PageXml, &doc);
    assert!(page.contains(&format!("<TextRegion id=\"researcher_{}\"", word.id)));
    let alto = written(ExportFormat::Alto, &doc);
    assert!(!alto.contains(&word.id), "ALTO writes text only");
}

/// Comparison: set-up B's files never carry the box the researcher gave set-up A's reading.
#[tokio::test]
async fn a_box_for_one_set_up_s_reading_stays_out_of_another_s_export() {
    let (_f, job, source, quiet) = read_report_by(&["fake-model", "pro"]).await;
    let (a, b) = (job.setups[0].id, job.setups[1].id);
    let reading = |setup| {
        ok(quiet.page_regions(job.id, source.id, setup, 0))
            .result
            .unwrap_or_default()
    };
    assert_ne!(reading(a), reading(b));
    let note = ok(quiet.save_region(draft(&job, &source, 1, None)));
    let moved = ok(quiet.save_region(adjusting(
        &job,
        &source,
        &reading(a),
        "r1",
        [0.05, 0.05, 0.6, 0.2],
    )));
    let doc_a = ok(quiet.export_document(&job, a, source.id));
    let doc_b = ok(quiet.export_document(&job, b, source.id));
    let ids = |doc: &ExportDocument| -> Vec<String> {
        let r = &doc.parts[0].researcher;
        r.adjusted
            .iter()
            .map(|x| x.id.clone())
            .chain(r.drawn.iter().map(|d| d.id.clone()))
            .collect()
    };
    assert_eq!(ids(&doc_a), [moved.id.clone(), note.id.clone()]);
    assert_eq!(ids(&doc_b), std::slice::from_ref(&note.id));
    assert_eq!(doc_a.parts[0].regions[0].x, 0.05);
    assert_eq!(
        doc_b.parts[0].regions[0].x, 0.1,
        "B's block keeps the model's box"
    );
    assert!(!written(ExportFormat::PageXml, &doc_b).contains(&moved.id));
}

/// A box made on a reading no set-up shows any more (the page read again) is the researcher's
/// own region in the files, as in the review.
#[tokio::test]
async fn after_the_page_is_read_again_an_adjustment_is_exported_as_the_researcher_s_own() {
    let (f, job, source, quiet) = read_report().await;
    let old = DrawnRegion {
        id: "d-old".into(),
        job_id: job.id,
        source_id: source.id,
        page: 1,
        half: None,
        x: 0.1,
        y: 0.1,
        width: 0.5,
        height: 0.02,
        text: String::new(),
        adjusts: Some(AdjustedRegion {
            result: "a-reading-replaced".into(),
            region: "r1_l1".into(),
        }),
        source_hash: source.sha256.clone(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    };
    f.store.put_region(&old).unwrap_or_else(|e| panic!("{e}"));
    let doc = ok(quiet.export_document(&job, job.setups[0].id, source.id));
    let first = &doc.parts[0];
    assert!(first.researcher.adjusted.is_empty());
    assert_eq!(first.researcher.drawn.len(), 1);
    assert_eq!(first.researcher.drawn[0].id, "d-old");
    assert_eq!(first.regions[0].lines[0].y, 0.1, "the model's own box");
    let page = written(ExportFormat::PageXml, &doc);
    assert!(
        page.contains("<TextRegion id=\"researcher_d-old\""),
        "{page}"
    );
    // And nothing of theirs, nothing in the files.
    let plain = ok(quiet.remove_region("d-old"));
    assert_eq!(plain.id, "d-old");
    let doc = ok(quiet.export_document(&job, job.setups[0].id, source.id));
    assert!(doc.parts.iter().all(|p| p.researcher.is_empty()));
}
