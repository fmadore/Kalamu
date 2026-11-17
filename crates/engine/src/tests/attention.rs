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
        .review_queue(&ReviewScope {
            documents,
            targets,
            ..Default::default()
        })
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
            Signal::Truncated {
                reason: kalamu_core::CutReason::Limit,
            },
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
            q.counts.reasons.truncated,
            q.counts.reasons.disagrees,
            q.counts.reasons.not_reviewed
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
    assert_eq!((q.counts.reasons.stale, q.counts.unresolved), (1, 1));

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

fn sampled(f: &Fixture, targets: &[ReviewTarget], seed: u32, size: u32) -> ReviewQueue {
    let calls = f.fake.calls();
    let queue = f
        .engine
        .review_queue(&ReviewScope {
            documents: 1,
            targets: targets.to_vec(),
            sample: Some(kalamu_core::SampleSpec {
                seed,
                size,
                drawn_at: Timestamp(0),
            }),
            kept: Vec::new(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.fake.calls(), calls, "drawing the sample calls no model");
    queue
}

/// The pages drawn, by their index, in the order the queue lists them.
fn drawn(q: &ReviewQueue) -> Vec<u32> {
    q.items
        .iter()
        .filter(|i| i.sample)
        .filter_map(|i| i.index)
        .collect()
}

#[tokio::test]
async fn a_sample_draws_pages_nothing_flags_once_whatever_reads_them() {
    let f = fixture();
    // Page 3 is cut off for both set-ups, which otherwise read alike.
    f.fake.cut_off.store(3, Ordering::SeqCst);
    let scan = f.pdf("registre.pdf", 10);
    let job = f.new_job(vec![scan.clone()], &["fake-model", "fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    mark_reviewed(&f, &job, 0, 0);
    let doc = DocumentId::new();
    let targets = vec![target(&job, 0, &scan, doc), target(&job, 1, &scan, doc)];
    let plain = queue(&f, 1, targets.clone());

    let q = sampled(&f, &targets, 1957, 4);
    let sample = q.sample.unwrap_or_else(|| panic!("a sample"));
    // Ten pages read twice are ten pages; the one cut off is never drawn.
    assert_eq!((sample.candidates, sample.drawn), (9, 4));
    let pages = drawn(&q);
    assert_eq!(pages.len(), 4, "each page drawn stands once: {pages:?}");
    assert!(!pages.contains(&2), "the page cut off is flagged");
    assert!(
        q.items
            .iter()
            .filter(|i| i.sample)
            .all(|i| !i.flagged() && i.setup_id == job.setups[0].id)
    );
    // The pages of lowest rank for the seed, as the draw orders them anywhere.
    let mut expected: Vec<u32> = kalamu_core::sample::draw(
        1957,
        4,
        (0..10u32)
            .filter(|i| *i != 2)
            .map(|i| {
                let piece = kalamu_core::PieceRef::page(i + 1);
                (
                    kalamu_core::sample::page_identity(doc, Task::TextExtraction, &piece),
                    i,
                )
            })
            .collect(),
    );
    let mut got = pages.clone();
    expected.sort_unstable();
    got.sort_unstable();
    assert_eq!(got, expected);
    // The coverage and the reasons are the queue's, sample or not.
    assert_eq!(q.counts, plain.counts);

    // The same seed, the same pages; another seed, others; the targets' order does not matter.
    let mut reversed = targets.clone();
    reversed.reverse();
    let mut again = drawn(&sampled(&f, &reversed, 1957, 4));
    again.sort_unstable();
    assert_eq!(again, got);
    let others: Vec<Vec<u32>> = (1958..1962)
        .map(|seed| {
            let mut d = drawn(&sampled(&f, &targets, seed, 4));
            d.sort_unstable();
            d
        })
        .collect();
    assert!(others.iter().any(|d| *d != got));

    // Asked for more than there are, the sample is every page nothing flags; the page reviewed
    // under the first set-up, which stands for it, is counted reviewed.
    let all = sampled(&f, &targets, 1957, 100);
    let sample = all.sample.unwrap_or_else(|| panic!("a sample"));
    assert_eq!((sample.drawn, sample.reviewed), (9, 1));
    let first = all
        .items
        .iter()
        .find(|i| i.sample && i.index == Some(0))
        .unwrap_or_else(|| panic!("page 1"));
    assert!(first.reviewed && !first.open && first.signals.is_empty());
}

#[tokio::test]
async fn an_item_kept_aside_is_marked_for_the_answer_it_was_kept_with() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 2);
    let job = f.new_job(vec![scan.clone()], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let targets = vec![target(&job, 0, &scan, DocumentId::new())];
    let q = queue(&f, 1, targets.clone());
    let page = |q: &ReviewQueue, index: u32| {
        q.items
            .iter()
            .find(|i| i.index == Some(index))
            .cloned()
            .unwrap_or_else(|| panic!("page {index}"))
    };
    let kept = |index: u32, revision: Option<String>| kalamu_core::KeptAside {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: scan.id,
        index: Some(index),
        revision,
        note: "Ask the archivist".into(),
        kept_at: Timestamp(1),
    };
    let scope = ReviewScope {
        documents: 1,
        targets,
        sample: None,
        kept: vec![
            kept(0, page(&q, 0).revision),
            kept(1, Some("an earlier one".into())),
        ],
    };
    let q = f
        .engine
        .review_queue(&scope)
        .unwrap_or_else(|e| panic!("{e}"));
    let first = page(&q, 0).kept.unwrap_or_else(|| panic!("kept"));
    assert!(first.current && first.note == "Ask the archivist");
    // Kept with another answer: the note is shown, the item is not kept aside with it.
    assert!(page(&q, 1).kept.is_some_and(|k| !k.current));
    // Keeping aside resolves nothing.
    assert!(page(&q, 0).open && !page(&q, 0).reviewed);
}
