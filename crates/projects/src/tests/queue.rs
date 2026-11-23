//! The review queue's result policy: the preferred result where there is one, else every result
//! the researcher has not set aside, and documents counted once whatever reads them.

use kalamu_core::DocumentId;

use super::*;

/// A comparison job of `task` over `sources` with two set-ups, `flash` and `pro`; nothing read.
fn comparison(f: &Fixture, task: Task, sources: Vec<SourceFile>) -> Job {
    let mut job = f.job(task, Vec::new());
    f.store.delete_job(job.id).unwrap();
    let mut pro = job.setups[0].clone();
    pro.id = SetupId::new();
    pro.label = "pro".into();
    pro.model = "gemini-pro".into();
    job.setups[0].label = "flash".into();
    job.setups.push(pro);
    job.id = kalamu_core::JobId::new();
    job.sources = sources;
    f.store.create_job(&job).unwrap();
    job
}

fn keys(plan: &QueuePlan) -> Vec<(DocumentId, String)> {
    plan.scope
        .targets
        .iter()
        .map(|t| {
            let label = if t.preferred {
                "preferred".to_string()
            } else {
                plan.choices
                    .iter()
                    .find(|c| c.key.setup_id == t.setup_id)
                    .map(|c| c.setup_label.clone())
                    .unwrap_or_default()
            };
            (t.document_id, label)
        })
        .collect()
}

#[test]
fn a_preferred_result_stands_alone_and_alternatives_are_listed_otherwise() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let (a, b) = (f.file("a.pdf", "a"), f.file("b.pdf", "b"));
    let docs = f
        .projects
        .add_documents(p.id, vec![a.clone(), b.clone()], None)
        .unwrap();
    let job = comparison(&f, Task::TextExtraction, vec![a, b]);
    f.projects.link_job(p.id, job.id, None).unwrap();

    // No preference: both set-ups read both documents, which stay two documents.
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project)
        .unwrap();
    assert_eq!(plan.scope.documents, 2);
    assert_eq!(plan.scope.targets.len(), 4);
    assert_eq!(plan.preferred, 0);
    assert_eq!(plan.choices.len(), 2);
    assert!(plan.choices.iter().all(|c| c.documents == 2 && c.included));

    // The researcher prefers `pro` for the first document: it alone is read there.
    f.projects
        .set_preferred(
            p.id,
            docs[0],
            Task::TextExtraction,
            Some((job.id, job.setups[1].id)),
        )
        .unwrap();
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project)
        .unwrap();
    assert_eq!(plan.preferred, 1);
    let first: Vec<_> = plan
        .scope
        .targets
        .iter()
        .filter(|t| t.document_id == docs[0])
        .collect();
    assert_eq!(first.len(), 1);
    assert!(first[0].preferred);
    assert_eq!(first[0].setup_id, job.setups[1].id);
    // The alternatives now stand for the second document only.
    assert!(plan.choices.iter().all(|c| c.documents == 1));

    // Setting `flash` aside leaves the second document with `pro` alone; a preferred result is
    // never set aside.
    let flash = ResultKey {
        job_id: job.id,
        setup_id: job.setups[0].id,
    };
    let pro = ResultKey {
        job_id: job.id,
        setup_id: job.setups[1].id,
    };
    for key in [flash, pro] {
        f.projects
            .set_result_aside(p.id, &QueueSelection::Project, key, true)
            .unwrap();
    }
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project)
        .unwrap();
    assert_eq!(
        keys(&plan),
        vec![(docs[0], "preferred".to_string())],
        "the preferred result stays; the second document has nothing left to read"
    );
    assert!(plan.choices.iter().all(|c| !c.included));
    assert_eq!(plan.scope.documents, 2, "coverage still counts it");
}

#[test]
fn results_set_aside_are_kept_per_scope_after_a_restart_and_can_be_brought_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kalamu.db");
    let open = |path: &std::path::Path| {
        let store = Arc::new(SqliteStore::open(path).unwrap());
        Fixture {
            projects: Projects::new(store.clone(), store.clone()),
            store,
            dir: tempfile::tempdir().unwrap(),
        }
    };
    let (project, letters, job, docs) = {
        let f = open(&path);
        let p = f.projects.create("Letters", "").unwrap();
        let letters = f.projects.create_collection(p.id, "Letters").unwrap();
        let (a, b) = (f.file("a.pdf", "a"), f.file("b.pdf", "b"));
        let docs = f
            .projects
            .add_documents(p.id, vec![a.clone(), b.clone()], None)
            .unwrap();
        f.projects
            .add_to_collection(p.id, letters.id, &docs)
            .unwrap();
        let job = comparison(&f, Task::TextExtraction, vec![a, b]);
        f.projects.link_job(p.id, job.id, None).unwrap();
        let flash = ResultKey {
            job_id: job.id,
            setup_id: job.setups[0].id,
        };
        f.projects
            .set_result_aside(p.id, &QueueSelection::Project, flash, true)
            .unwrap();
        // A job that does not exist is not set aside.
        let gone = ResultKey {
            job_id: kalamu_core::JobId::new(),
            setup_id: job.setups[0].id,
        };
        assert!(matches!(
            f.projects
                .set_result_aside(p.id, &QueueSelection::Project, gone, true),
            Err(ProjectError::NotFound)
        ));
        (p.id, letters.id, job, docs)
    };

    // Reopened, the project's queue still reads `pro` alone; the collection's, both.
    let f = open(&path);
    let plan = f
        .projects
        .queue_plan(project, &QueueSelection::Project)
        .unwrap();
    assert_eq!(
        keys(&plan),
        vec![(docs[0], "pro".to_string()), (docs[1], "pro".to_string())]
    );
    let flash = plan
        .choices
        .iter()
        .find(|c| c.setup_label == "flash")
        .unwrap();
    assert!(!flash.included, "listed, to be brought back");
    let collection = QueueSelection::Collection { id: letters };
    let theirs = f.projects.queue_plan(project, &collection).unwrap();
    assert_eq!(theirs.scope.targets.len(), 4);
    assert!(theirs.choices.iter().all(|c| c.included));

    // Brought back, it is read again.
    f.projects
        .set_result_aside(project, &QueueSelection::Project, flash.key, false)
        .unwrap();
    let plan = f
        .projects
        .queue_plan(project, &QueueSelection::Project)
        .unwrap();
    assert_eq!(plan.scope.targets.len(), 4);

    // Deleting the job leaves nothing set aside behind it.
    f.projects
        .set_result_aside(project, &collection, flash.key, true)
        .unwrap();
    f.store.delete_job(job.id).unwrap();
    assert!(
        f.store
            .results_set_aside(project, &collection.key())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_queue_covers_a_collection_or_a_selection() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let letters = f.projects.create_collection(p.id, "Letters").unwrap();
    let files = vec![
        f.file("a.pdf", "a"),
        f.file("b.pdf", "b"),
        f.file("c.pdf", "c"),
    ];
    let docs = f.projects.add_documents(p.id, files.clone(), None).unwrap();
    f.projects
        .add_to_collection(p.id, letters.id, &docs[..2])
        .unwrap();
    let job = f.job(Task::TextExtraction, files);
    f.projects.link_job(p.id, job.id, None).unwrap();

    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Collection { id: letters.id })
        .unwrap();
    assert_eq!(plan.scope.documents, 2);
    assert_eq!(plan.scope.targets.len(), 2);

    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Documents { ids: vec![docs[2]] })
        .unwrap();
    assert_eq!(plan.scope.documents, 1);
    assert_eq!(plan.scope.targets.len(), 1);
    assert_eq!(plan.scope.targets[0].document_id, docs[2]);
    assert_eq!(plan.scope.targets[0].document, "c");

    // A document of another project is refused, not quietly left out.
    let other = f.projects.create("Other", "").unwrap();
    let stray = f
        .projects
        .add_documents(other.id, vec![f.file("d.pdf", "d")], None)
        .unwrap();
    assert!(matches!(
        f.projects
            .queue_plan(p.id, &QueueSelection::Documents { ids: stray }),
        Err(ProjectError::Elsewhere(_))
    ));
}

#[test]
fn a_document_not_yet_read_is_counted_but_reads_nothing() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    f.projects
        .add_documents(p.id, vec![f.file("a.pdf", "a")], None)
        .unwrap();
    let plan = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project)
        .unwrap();
    assert_eq!(plan.scope.documents, 1);
    assert!(plan.scope.targets.is_empty());
    assert!(plan.choices.is_empty());
}

#[test]
fn each_scope_keeps_its_own_sample_until_drawn_again_or_removed() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let letters = f.projects.create_collection(p.id, "Letters").unwrap();
    let docs = f
        .projects
        .add_documents(p.id, vec![f.file("a.pdf", "a"), f.file("b.pdf", "b")], None)
        .unwrap();
    let plan = |s: &QueueSelection| f.projects.queue_plan(p.id, s).unwrap().scope;

    let whole = f
        .projects
        .draw_sample(p.id, &QueueSelection::Project, 20, Some(1957))
        .unwrap();
    assert_eq!((whole.seed, whole.size), (1957, 20));
    assert_eq!(plan(&QueueSelection::Project).sample, Some(whole));
    // A seed typed by hand is six digits at most; a longer one is refused and the sample kept.
    let typed = f
        .projects
        .draw_sample(p.id, &QueueSelection::Project, 20, Some(999_999))
        .unwrap();
    assert_eq!(typed.seed, 999_999);
    assert!(matches!(
        f.projects
            .draw_sample(p.id, &QueueSelection::Project, 20, Some(1_000_000)),
        Err(ProjectError::Seed)
    ));
    assert_eq!(plan(&QueueSelection::Project).sample, Some(typed));
    let collection = QueueSelection::Collection { id: letters.id };
    assert_eq!(plan(&collection).sample, None);

    // The same documents chosen again, in another order, find the same sample.
    let chosen = QueueSelection::Documents {
        ids: vec![docs[0], docs[1]],
    };
    let again = QueueSelection::Documents {
        ids: vec![docs[1], docs[0]],
    };
    assert_eq!(chosen.key(), again.key());
    let drawn = f.projects.draw_sample(p.id, &chosen, 5, None).unwrap();
    assert_eq!(plan(&again).sample, Some(drawn));

    // A new draw is a new seed unless one is given.
    let redrawn = f
        .projects
        .draw_sample(p.id, &QueueSelection::Project, 20, None)
        .unwrap();
    assert_eq!(plan(&QueueSelection::Project).sample, Some(redrawn));
    f.projects
        .clear_sample(p.id, &QueueSelection::Project)
        .unwrap();
    assert_eq!(plan(&QueueSelection::Project).sample, None);
    assert_eq!(plan(&again).sample, Some(drawn));
}

#[test]
fn an_item_kept_aside_keeps_its_note_for_the_answer_it_held() {
    let f = fixture();
    let p = f.projects.create("Letters", "").unwrap();
    let file = f.file("a.pdf", "a");
    f.projects
        .add_documents(p.id, vec![file.clone()], None)
        .unwrap();
    let job = f.job(Task::TextExtraction, vec![file]);
    f.projects.link_job(p.id, job.id, None).unwrap();
    let key = QueueItemKey {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: Some(1),
    };
    f.projects
        .keep_aside(
            p.id,
            &key,
            Some("3f9a1c07be52".into()),
            Some("  Ask about the stamp "),
        )
        .unwrap();
    let kept = f
        .projects
        .queue_plan(p.id, &QueueSelection::Project)
        .unwrap()
        .scope
        .kept;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].note, "Ask about the stamp");
    assert_eq!(kept[0].revision.as_deref(), Some("3f9a1c07be52"));

    // Matched to the queue's items: kept with the answer it held, not with a newer one.
    let mut queue = kalamu_core::ReviewQueue {
        items: vec![
            manifest_item(&job, Some("3f9a1c07be52")),
            manifest_item(&job, Some("0a1b2c3d4e5f")),
        ],
        ..Default::default()
    };
    queue.mark_kept(&kept);
    assert!(queue.items[0].kept.as_ref().is_some_and(|k| k.current));
    assert!(queue.items[1].kept.as_ref().is_some_and(|k| !k.current));

    f.projects.keep_aside(p.id, &key, None, None).unwrap();
    assert!(
        f.projects
            .queue_plan(p.id, &QueueSelection::Project)
            .unwrap()
            .scope
            .kept
            .is_empty()
    );
}

/// An item of `job`'s first set-up, page 2, holding the answer `revision`.
fn manifest_item(job: &Job, revision: Option<&str>) -> kalamu_core::QueueItem {
    kalamu_core::QueueItem {
        document_id: fixed(1),
        document: "Letter of 3 May 1957".into(),
        job_id: job.id,
        job_name: job.name.clone(),
        task: job.task,
        source_id: job.sources[0].id,
        setup_id: job.setups[0].id,
        setup_label: job.setups[0].label.clone(),
        model: job.setups[0].model.clone(),
        preferred: false,
        piece: Some(PieceRef::page(2)),
        index: Some(1),
        revision: revision.map(String::from),
        answered_at: Some(Timestamp(1_778_000_000_000)),
        signals: vec![kalamu_core::Signal::NotReviewed],
        reviewed: false,
        corrected: false,
        processing: false,
        open: true,
        decided_at: None,
        sample: false,
        kept: None,
    }
}

/// An id of any kind with a fixed value, for golden files.
pub(super) fn fixed<T: std::str::FromStr>(n: u8) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("00000000-0000-0000-0000-{n:012}").parse().unwrap()
}

#[test]
fn the_review_manifest_matches_its_golden_files() {
    use kalamu_core::{CutReason, KeptNote, SampleCounts, Signal};

    let job = Job {
        id: fixed(2),
        name: "Bulletins, May 1957".into(),
        sources: vec![SourceFile {
            id: fixed(3),
            ..f_source()
        }],
        setups: vec![Setup {
            id: fixed(4),
            label: "flash".into(),
            model: "gemini-flash".into(),
            ..fixture().job(Task::TextExtraction, Vec::new()).setups[0].clone()
        }],
        ..fixture().job(Task::TextExtraction, Vec::new())
    };
    let base = manifest_item(&job, Some("3f9a1c07be52"));
    let items = vec![
        // A failed page, nothing read.
        kalamu_core::QueueItem {
            piece: Some(PieceRef::page(1)),
            index: Some(0),
            revision: None,
            answered_at: None,
            signals: vec![Signal::Failed {
                error: Some("The provider was busy (503)".into()),
            }],
            processing: true,
            ..base.clone()
        },
        // Cut off and read otherwise by `pro`, reviewed and corrected, kept aside.
        kalamu_core::QueueItem {
            signals: vec![
                Signal::Truncated {
                    reason: CutReason::Limit,
                },
                Signal::Disagrees {
                    with: vec!["pro".into()],
                    words: 3,
                    fields: Vec::new(),
                },
            ],
            reviewed: true,
            corrected: true,
            open: false,
            decided_at: Some(Timestamp(1_778_100_000_000)),
            kept: Some(KeptNote {
                note: "=Check the stamp, with the archivist".into(),
                kept_at: Timestamp(1_778_200_000_000),
                current: true,
            }),
            ..base.clone()
        },
        // Drawn at random, nothing flags it, reviewed.
        kalamu_core::QueueItem {
            piece: Some(PieceRef::page(3)),
            index: Some(2),
            signals: Vec::new(),
            reviewed: true,
            open: false,
            sample: true,
            decided_at: Some(Timestamp(1_778_300_000_000)),
            ..base.clone()
        },
        // Kept aside with an earlier answer: listed with the others again.
        kalamu_core::QueueItem {
            piece: Some(PieceRef::page(4)),
            index: Some(3),
            kept: Some(KeptNote {
                note: "Faded ink".into(),
                kept_at: Timestamp(1_777_000_000_000),
                current: false,
            }),
            ..base.clone()
        },
    ];
    let queue = kalamu_core::ReviewQueue {
        counts: kalamu_core::QueueCounts {
            documents: 1,
            documents_read: 1,
            results: 1,
            pieces: 4,
            processed: 3,
            failed: 1,
            reviewed: 2,
            flagged: 2,
            unresolved: 1,
            ..Default::default()
        },
        items,
        sample: Some(SampleCounts {
            seed: 1957,
            size: 1,
            drawn_at: Timestamp(1_778_250_000_000),
            candidates: 2,
            drawn: 1,
            reviewed: 1,
        }),
    };
    // `pro` set aside: named in the manifest, as the sample was drawn without it; `flash` read.
    let choice = |setup: u8, label: &str, included: bool| ResultChoice {
        key: ResultKey {
            job_id: job.id,
            setup_id: fixed(setup),
        },
        job_name: job.name.clone(),
        task: job.task,
        setup_label: label.into(),
        model: format!("gemini-{label}"),
        documents: 1,
        included,
    };
    let manifest = ReviewManifest::new(
        fixed(5),
        "Mission archives",
        "project".into(),
        &queue,
        &[choice(4, "flash", true), choice(6, "pro", false)],
        Timestamp(1_778_400_000_000),
    );
    golden("review-manifest.csv", &csv_body(manifest.table()));
    golden("review-manifest.json", &manifest.json().unwrap());
}

/// A source file with fixed values, for golden files.
fn f_source() -> SourceFile {
    SourceFile {
        id: fixed(3),
        path: "C:/archive/bulletin.pdf".into(),
        file_name: "bulletin.pdf".into(),
        sha256: "abc".into(),
        size_bytes: 10,
        modified_ms: None,
        kind: kalamu_core::InputKind::Pdf,
        page_count: Some(4),
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        dense_pages: None,
        title: None,
        derived_from: None,
        pages: Vec::new(),
    }
}

/// Compares `actual` with `golden/<name>`; `UPDATE_GOLDEN=1` rewrites the file after an intended
/// change, for the diff to be reviewed.
pub(super) fn golden(name: &str, actual: &str) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("golden")
        .join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        actual, expected,
        "{name} differs from its golden file; rerun with UPDATE_GOLDEN=1 if intended"
    );
}
