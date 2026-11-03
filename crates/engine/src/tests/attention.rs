//! The review queue from stored results: failures and work not started kept apart from
//! answers, cut-off answers, set-ups that disagree, inputs changed since, and reviews that
//! resolve an item without hiding it. Nothing in it calls the provider.

use kalamu_core::{
    Correction, DocumentId, ReviewQueue, ReviewScope, ReviewTarget, Signal, SourceFile,
};

use super::*;

fn target(job: &Job, setup: usize, source: &SourceFile, document: DocumentId) -> ReviewTarget {
    ReviewTarget {
        document_id: document,
        document: source.file_name.clone(),
        job_id: job.id,
        source_id: source.id,
        setup_id: job.setups[setup].id,
        preferred: false,
    }
}

fn queue(f: &Fixture, documents: u32, targets: Vec<ReviewTarget>) -> ReviewQueue {
    let calls = f.fake.calls();
    let queue = f
        .engine
        .review_queue(&ReviewScope { documents, targets })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.fake.calls(), calls, "the queue calls no model");
    queue
}

fn mark_reviewed(f: &Fixture, job: &Job, setup: usize, index: u32) {
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[setup].id,
            source_id: job.sources[0].id,
            index,
            text: None,
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

#[tokio::test]
async fn a_comparison_lists_where_its_set_ups_disagree_and_what_was_cut_off() {
    let f = fixture();
    f.fake.cut_off.store(2, Ordering::SeqCst);
    let source = f.pdf("lettre.pdf", 3);
    let job = f.new_job(vec![source.clone()], &["flash", "pro"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let doc = DocumentId::new();
    let targets = vec![target(&job, 0, &source, doc), target(&job, 1, &source, doc)];

    let q = queue(&f, 1, targets.clone());
    // Two results of one document: six pages to review, one document.
    assert_eq!(q.items.len(), 6);
    assert_eq!(
        (
            q.counts.documents,
            q.counts.documents_read,
            q.counts.results
        ),
        (1, 1, 2)
    );
    assert_eq!((q.counts.pieces, q.counts.processed), (6, 6));
    let flash_2 = q
        .items
        .iter()
        .find(|i| i.setup_label == "flash" && i.index == Some(1))
        .unwrap_or_else(|| panic!("page 2"));
    assert_eq!(flash_2.piece, Some(kalamu_core::PieceRef::page(2)));
    assert!(flash_2.revision.as_ref().is_some_and(|r| r.len() == 12));
    assert!(!flash_2.processing && flash_2.open);
    assert_eq!(
        flash_2.signals,
        vec![
            Signal::Truncated,
            Signal::Disagrees {
                with: vec!["pro".into()],
                words: 2,
                fields: Vec::new(),
            },
            Signal::NotReviewed,
        ]
    );
    assert_eq!(
        (
            q.counts.truncated,
            q.counts.disagrees,
            q.counts.not_reviewed
        ),
        (2, 6, 6)
    );
    assert_eq!((q.counts.flagged, q.counts.unresolved), (6, 6));

    // Reviewed, page 1 of `flash` is resolved, and stays listed for what was found.
    mark_reviewed(&f, &job, 0, 0);
    let q = queue(&f, 1, targets);
    let flash_1 = q
        .items
        .iter()
        .find(|i| i.setup_label == "flash" && i.index == Some(0))
        .unwrap_or_else(|| panic!("page 1"));
    assert!(flash_1.reviewed && !flash_1.open);
    assert!(
        !flash_1.signals.contains(&Signal::NotReviewed)
            && matches!(flash_1.signals[..], [Signal::Disagrees { .. }])
    );
    assert_eq!(q.counts.reviewed, 1);
    assert_eq!((q.counts.flagged, q.counts.unresolved), (6, 5));
    // Reviewing one set-up never reviews the other.
    let pro_1 = q
        .items
        .iter()
        .find(|i| i.setup_label == "pro" && i.index == Some(0));
    assert!(pro_1.is_some_and(|i| !i.reviewed && i.open));
}

#[tokio::test]
async fn failures_and_work_not_started_are_kept_apart_from_answers() {
    let f = fixture();
    f.fake
        .fail(2, vec![ProviderError::Transient("busy".into()); 3]);
    let read = f.pdf("letters.pdf", 3);
    let job = f.new_job(vec![read.clone()], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let waiting = f.pdf("register.pdf", 2);
    let not_started = f.new_job(vec![waiting.clone()], &["fake-model"]);
    // Every page reviewed but the failed one.
    mark_reviewed(&f, &job, 0, 0);
    mark_reviewed(&f, &job, 0, 2);

    let q = queue(
        &f,
        2,
        vec![
            target(&job, 0, &read, DocumentId::new()),
            target(&not_started, 0, &waiting, DocumentId::new()),
        ],
    );
    assert_eq!(q.items.len(), 2, "{:?}", q.items);
    let failed = &q.items[0];
    assert_eq!(failed.index, Some(1));
    assert!(failed.processing && failed.open);
    assert!(matches!(
        &failed.signals[..],
        [Signal::Failed { error: Some(e) }] if !e.is_empty()
    ));
    let never = &q.items[1];
    assert_eq!((never.piece.clone(), never.index), (None, None));
    assert_eq!(never.signals, vec![Signal::NotRead]);
    assert!(never.processing);
    assert_eq!(
        (q.counts.documents, q.counts.documents_read),
        (2, 1),
        "a document with nothing read is counted, not read"
    );
    assert_eq!(
        (
            q.counts.pieces,
            q.counts.processed,
            q.counts.failed,
            q.counts.reviewed
        ),
        (3, 2, 1, 2)
    );
    assert_eq!(q.counts.flagged, 2);
}

#[tokio::test]
async fn an_answer_read_from_a_text_corrected_since_is_stale_even_once_reviewed() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    let extraction = f.new_job(vec![scan], &["fake-model"]);
    assert_eq!(f.run(extraction.id).await, JobStatus::Completed);
    let frozen = f
        .engine
        .snapshot(extraction.id, extraction.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let analysis = f.new_job_for(Task::Analysis, frozen.clone(), &["fake-model"]);
    assert_eq!(f.run(analysis.id).await, JobStatus::Completed);
    let doc = DocumentId::new();
    let targets = vec![target(&analysis, 0, &frozen[0], doc)];

    let q = queue(&f, 1, targets.clone());
    assert_eq!(q.items.len(), 1);
    assert_eq!(q.items[0].piece, Some(kalamu_core::PieceRef::Whole));
    assert_eq!(q.items[0].signals, vec![Signal::NotReviewed]);
    mark_reviewed(&f, &analysis, 0, 0);
    assert!(queue(&f, 1, targets.clone()).items.is_empty());

    // The text it read is corrected: the reviewed answer read an earlier version.
    f.engine
        .save_correction(Correction {
            job_id: extraction.id,
            setup_id: extraction.setups[0].id,
            source_id: extraction.sources[0].id,
            index: 1,
            text: Some("Le Père Éric écrit à l'imam.".into()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let q = queue(&f, 1, targets.clone());
    assert_eq!(q.items.len(), 1);
    assert_eq!(
        q.items[0].signals,
        vec![Signal::Stale {
            gone: false,
            unit: false
        }]
    );
    assert!(q.items[0].reviewed && q.items[0].open);
    assert_eq!((q.counts.stale, q.counts.unresolved), (1, 1));

    f.engine
        .delete_job(extraction.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let q = queue(&f, 1, targets);
    assert_eq!(
        q.items[0].signals,
        vec![Signal::Stale {
            gone: true,
            unit: false
        }]
    );
}

#[tokio::test]
async fn analyses_that_list_the_same_names_do_not_disagree() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 1);
    // Two models answer with different prose and the same names.
    let job = f.new_job_for(Task::Analysis, vec![scan.clone()], &["flash", "pro"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let q = queue(&f, 1, vec![target(&job, 0, &scan, DocumentId::new())]);
    assert_eq!(q.items.len(), 1);
    assert_eq!(q.items[0].signals, vec![Signal::NotReviewed]);
}
