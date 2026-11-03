//! Document units (#50): made once from the corrected entries, then the researcher's; their
//! pages frozen as a PDF a later job reads, which a change to the unit marks as out of date.

use kalamu_core::{Correction, DocumentUnit, UnitError, UnitId};

use super::*;
use crate::UnitEdit;

/// A table of contents of the three-page issue, run and its entries corrected: an entry for
/// a page the issue does not have is added.
async fn contents(t: &Two) -> Job {
    let job = t.job(
        Task::Contents,
        vec![issue(t)],
        "contents.periodical",
        Some("tiny"),
    );
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    let mut entries = t
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .find(|p| p.piece == PieceRef::Whole)
        .map(|p| kalamu_core::contents::entries(&p.ai_fields).to_vec())
        .unwrap_or_default();
    entries.push(vec![
        text_answer("title", "Avis"),
        text_answer("pages", "9"),
    ]);
    t.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            index: 3,
            text: None,
            fields: Some(kalamu_core::contents::answers(entries)),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    job
}

fn units(t: &Two, job: &Job) -> Vec<DocumentUnit> {
    t.engine
        .units_view(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"))
        .units
        .into_iter()
        .map(|v| v.unit)
        .collect()
}

fn edit(job: &Job, id: Option<UnitId>, title: &str, pages: &str) -> UnitEdit {
    UnitEdit {
        id,
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        title: title.into(),
        pages: pages.into(),
        fields: None,
    }
}

fn code<T: std::fmt::Debug>(result: Result<T, EngineError>) -> &'static str {
    match result {
        Err(e) => e.code(),
        Ok(v) => panic!("expected an error, got {v:?}"),
    }
}

#[tokio::test]
async fn units_are_made_once_from_the_corrected_entries() {
    let t = two();
    let job = contents(&t).await;
    let (setup, source) = (job.setups[0].id, job.sources[0].id);
    let view = t
        .engine
        .units_view(job.id, setup, source)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        (view.entries, view.units.len(), view.readable),
        (3, 0, true)
    );

    let report = t
        .engine
        .seed_units(job.id, setup, source)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(report.made, 3);
    assert_eq!(report.problems.len(), 1);
    assert_eq!(report.problems[0].unknown, ["9"]);
    let made = units(&t, &job);
    assert_eq!(
        made.iter().map(|u| u.title.as_str()).collect::<Vec<_>>(),
        ["Le Hajj", "Fatwa", "Avis"]
    );
    assert_eq!(made[0].pages, [PieceRef::page(1), PieceRef::page(2)]);
    assert!(made[2].pages.is_empty(), "page 9 is not in the issue");
    assert!(
        made[0].fields.iter().any(|a| a.key == "summary"),
        "the entry's fields come with it"
    );
    assert_eq!(
        code(t.engine.seed_units(job.id, setup, source)),
        UnitError::AlreadyMade.code()
    );

    // The researcher's changes: pages in another order, a page that is not there refused.
    let fatwa = t
        .engine
        .save_unit(edit(&job, Some(made[1].id), "Fatwa sur le jeûne", "3, 1"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(fatwa.pages, [PieceRef::page(3), PieceRef::page(1)]);
    assert_eq!(
        (fatwa.revision, fatwa.fields.len()),
        (2, made[1].fields.len())
    );
    assert_eq!(
        code(
            t.engine
                .save_unit(edit(&job, Some(made[2].id), "Avis", "4"))
        ),
        "unit.no_such_page"
    );
    let added = t
        .engine
        .save_unit(edit(&job, None, "Publicité", "3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(added.position, 3);
    let view = t
        .engine
        .units_view(job.id, setup, source)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(view.units[1].pages, "3, 1");
    assert_eq!(
        view.units[3].shared[0].with,
        [fatwa.id],
        "page 3 is shared whole, and said to be"
    );

    // Only a table of contents has units.
    let extraction = t.job(
        Task::TextExtraction,
        vec![issue(&t)],
        "text_extraction.printed",
        None,
    );
    assert_eq!(
        code(t.engine.seed_units(
            extraction.id,
            extraction.setups[0].id,
            extraction.sources[0].id
        )),
        UnitError::NotContents.code()
    );
}

#[tokio::test]
async fn units_are_joined_and_split() {
    let t = two();
    let job = contents(&t).await;
    t.engine
        .seed_units(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let made = units(&t, &job);
    let joined = t
        .engine
        .join_units(made[0].id, made[1].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        joined.pages,
        [PieceRef::page(1), PieceRef::page(2), PieceRef::page(3)]
    );
    assert_eq!(units(&t, &job).len(), 2);

    let [first, second] = t
        .engine
        .split_unit(joined.id, 2)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(first.pages, [PieceRef::page(1), PieceRef::page(2)]);
    assert_eq!(second.pages, [PieceRef::page(3)]);
    let now = units(&t, &job);
    assert_eq!(
        now.iter().map(|u| u.id).collect::<Vec<_>>(),
        [first.id, second.id, made[2].id],
        "the second part follows the first"
    );
    assert_eq!(
        code(t.engine.split_unit(second.id, 1)),
        "unit.bad_split",
        "one page"
    );
    assert_eq!(
        code(t.engine.join_units(first.id, first.id)),
        "unit.not_joinable"
    );
    // Another table of contents' item is neither joined nor saved through this one.
    let other = contents(&t).await;
    t.engine
        .seed_units(other.id, other.setups[0].id, other.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let theirs = units(&t, &other);
    assert_eq!(
        code(t.engine.join_units(first.id, theirs[0].id)),
        "unit.not_joinable"
    );
    assert_eq!(
        code(
            t.engine
                .save_unit(edit(&other, Some(first.id), "Le Hajj", "1"))
        ),
        "unit.other_result"
    );
    assert_eq!(units(&t, &job).len(), 3, "nothing was half made");
    assert_eq!(code(t.engine.unit_sources(&[])), "unit.none_chosen");
}

/// An item's pages frozen twice are one file, so a second job that reads them finds every page
/// in the cache: nothing is sent again.
#[tokio::test]
async fn the_same_item_frozen_twice_is_one_file_and_read_from_the_cache() {
    let t = two();
    let job = contents(&t).await;
    t.engine
        .seed_units(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let hajj = units(&t, &job)[0].id;
    let freeze = || {
        t.engine
            .unit_sources(&[hajj])
            .unwrap_or_else(|e| panic!("{e}"))
            .remove(0)
    };
    let (once, twice) = (freeze(), freeze());
    assert_eq!(
        (once.path.as_str(), once.sha256.as_str()),
        (twice.path.as_str(), twice.sha256.as_str())
    );

    let first = t.job(Task::Contents, vec![once], "contents.periodical", None);
    assert_eq!(t.run(first.id).await, JobStatus::Completed);
    let calls = t.reader.calls();
    let second = t.job(Task::Contents, vec![twice], "contents.periodical", None);
    assert_eq!(t.run(second.id).await, JobStatus::Completed);
    assert_eq!(t.reader.calls(), calls, "every page from the cache");
}

#[tokio::test]
async fn a_units_pages_are_read_on_their_own_and_a_change_marks_the_job() {
    let t = two();
    let job = contents(&t).await;
    t.engine
        .seed_units(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let made = units(&t, &job);
    let fatwa = t
        .engine
        .save_unit(edit(&job, Some(made[1].id), "Fatwa", "3, 1"))
        .unwrap_or_else(|e| panic!("{e}"));

    let sources = t
        .engine
        .unit_sources(&[fatwa.id, made[0].id])
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        sources
            .iter()
            .map(|s| s.file_name.as_str())
            .collect::<Vec<_>>(),
        ["issue 01 Le Hajj.pdf", "issue 02 Fatwa.pdf"],
        "in the units' order"
    );
    let frozen = &sources[1];
    assert_eq!((frozen.kind, frozen.page_count), (InputKind::Pdf, Some(2)));
    let bytes = std::fs::read(&frozen.path).unwrap_or_default();
    assert_eq!(kalamu_core::fingerprint::sha256_hex(&bytes), frozen.sha256);
    let pdf = kalamu_media::PdfSource::from_bytes(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(pdf.page_count(), 2);
    let from = frozen
        .derived_from
        .clone()
        .unwrap_or_else(|| panic!("no derived_from"));
    assert_eq!((from.job_id, from.source_id), (job.id, job.sources[0].id));
    assert_eq!(
        from.unit.as_ref().map(|u| u.pages.clone()),
        Some(vec![PieceRef::page(3), PieceRef::page(1)])
    );
    assert_eq!(
        code(t.engine.unit_sources(&[made[2].id])),
        "unit.no_pages",
        "a unit without pages cannot be read"
    );

    // A later job reads the frozen pages; the unit changed afterwards marks it, without
    // changing what it read.
    let later = t.job(
        Task::Contents,
        vec![frozen.clone()],
        "contents.periodical",
        None,
    );
    let state = |t: &Two| {
        t.engine
            .derived_states(later.id)
            .unwrap_or_else(|e| panic!("{e}"))[0]
            .freshness
    };
    assert_eq!(state(&t), crate::Freshness::Current);
    assert_eq!(t.run(later.id).await, JobStatus::Completed);
    let provenance: serde_json::Value =
        serde_json::from_str(&t.read("issue 02 Fatwa.contents.provenance.json"))
            .unwrap_or_default();
    let derived = &provenance["source"]["derived_from"];
    assert_eq!(derived["task"], "contents");
    assert_eq!(derived["pieces"], 2);
    assert_eq!(derived["unit"]["title"], "Fatwa");
    assert_eq!(
        derived["unit"]["pages"],
        serde_json::json!([{ "kind": "page", "number": 3 }, { "kind": "page", "number": 1 }])
    );
    t.engine
        .save_unit(edit(&job, Some(fatwa.id), "Fatwa (titre corrigé)", "3, 1"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&t), crate::Freshness::Current, "a title is not read");
    t.engine
        .save_unit(edit(&job, Some(fatwa.id), "Fatwa", "3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&t), crate::Freshness::Changed);
    // The list of what to check says so, for the item's pages, not for a corrected text.
    let queue = t
        .engine
        .review_queue(&kalamu_core::ReviewScope {
            documents: 1,
            targets: vec![kalamu_core::ReviewTarget {
                document_id: kalamu_core::DocumentId::new(),
                document: "Fatwa".into(),
                job_id: later.id,
                source_id: later.sources[0].id,
                setup_id: later.setups[0].id,
                preferred: false,
            }],
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        queue.items.iter().any(|i| i.piece.is_none()
            && i.signals
                == vec![kalamu_core::Signal::Stale {
                    gone: false,
                    unit: true
                }]),
        "{:?}",
        queue.items
    );
    assert!(
        std::path::Path::new(&frozen.path).is_file(),
        "the copy stays"
    );
    t.engine
        .delete_unit(fatwa.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(state(&t), crate::Freshness::Gone);
}

#[tokio::test]
async fn selected_units_are_exported_with_their_list() {
    let t = two();
    let job = contents(&t).await;
    t.engine
        .seed_units(job.id, job.setups[0].id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let made = units(&t, &job);
    let folder = t.dir.path().join("items");
    let written = t
        .engine
        .export_units(&[made[1].id, made[0].id], &folder)
        .unwrap_or_else(|e| panic!("{e}"));
    let names: Vec<String> = written
        .iter()
        .map(|p| {
            std::path::Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(
        names,
        [
            "issue 01 Le Hajj.pdf",
            "issue 02 Fatwa.pdf",
            "issue units.csv",
            "issue units.json"
        ]
    );
    let csv = std::fs::read_to_string(folder.join("issue units.csv")).unwrap_or_default();
    let mut lines = csv.lines();
    assert_eq!(
        lines.next().map(|l| l.trim_start_matches('\u{feff}')),
        Some("id,number,title,pages,file,Authors,Summary,original,original_sha256")
    );
    assert!(
        lines
            .next()
            .is_some_and(|l| l.contains(",1,Le Hajj,1-2,issue 01 Le Hajj.pdf,La Rédaction,")),
        "{csv}"
    );
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("issue units.json")).unwrap_or_default())
            .unwrap_or_default();
    assert_eq!(json["units"][1]["id"], made[1].id.to_string());
    assert_eq!(
        json["units"][1]["pages"],
        serde_json::json!([{ "kind": "page", "number": 3 }])
    );
    assert_eq!(json["units"][1]["original"]["file_name"], "issue.pdf");

    // Exporting again never replaces what is there.
    let again = t
        .engine
        .export_units(&[made[0].id], &folder)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(again[0].ends_with("issue 01 Le Hajj (2).pdf"), "{again:?}");
}
