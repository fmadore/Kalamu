//! What a batch leaves in the provider's storage is deleted once its answers are stored, or
//! once it is given up on and has settled; a crash in between finishes it at the next launch;
//! failures are tried again a few times and never touch the job.

use kalamu_core::BatchCleanup;

use super::*;

/// An economy job whose pages all go in one batch, the fake able to see the store.
fn one_batch() -> Economy {
    let e = economy();
    e.batchy.max_items.store(1_000, Ordering::SeqCst);
    *e.batchy.store.lock().unwrap_or_else(|p| p.into_inner()) = Some(e.f.store.clone());
    e
}

fn cleanup_of(e: &Economy, job: JobId) -> Vec<Option<BatchCleanup>> {
    e.f.store
        .batches(job)
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.cleanup)
        .collect()
}

fn done() -> Option<BatchCleanup> {
    Some(BatchCleanup {
        files: Some(Vec::new()),
        ..BatchCleanup::default()
    })
}

#[tokio::test]
async fn a_collected_batch_s_files_are_deleted_after_its_answers_are_stored() {
    let e = one_batch();
    let job = e.job(3, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    assert_eq!(e.batchy.deleted(), vec!["remote-0-in", "remote-0-out"]);
    assert_eq!(
        *e.batchy
            .submitted_at_delete
            .lock()
            .unwrap_or_else(|p| p.into_inner()),
        vec![0, 0],
        "every answer was stored before a file was deleted"
    );
    assert_eq!(cleanup_of(&e, job.id), vec![done()]);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 3);

    // Nothing is asked again once done.
    e.f.engine
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.batchy.listed.load(Ordering::SeqCst), 1);
    assert_eq!(e.batchy.deletes.load(Ordering::SeqCst), 2);
}

/// Deletions that fail are tried again at later launches, a bounded number of times, then the
/// files are left and the batch's record says so; the job and its results are untouched.
#[tokio::test]
async fn files_that_cannot_be_deleted_are_tried_again_then_left() {
    let e = one_batch();
    *e.batchy
        .delete_error
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = Some(ProviderError::Transient("503".into()));
    let job = e.job(2, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    let files = Some(vec!["remote-0-in".to_string(), "remote-0-out".to_string()]);
    assert_eq!(
        cleanup_of(&e, job.id),
        vec![Some(BatchCleanup {
            files: files.clone(),
            failures: 1,
            given_up: false,
        })]
    );
    for _ in 1..BatchCleanup::MAX_FAILURES {
        e.f.engine
            .clean_up_batches()
            .await
            .unwrap_or_else(|e| panic!("{e}"));
    }
    assert_eq!(
        cleanup_of(&e, job.id),
        vec![Some(BatchCleanup {
            files,
            failures: BatchCleanup::MAX_FAILURES,
            given_up: true,
        })]
    );
    assert_eq!(
        e.batchy.listed.load(Ordering::SeqCst),
        1,
        "the files are named once, then deleted by their recorded ids"
    );
    let attempts = e.batchy.deletes.load(Ordering::SeqCst);
    assert_eq!(attempts, 2 * BatchCleanup::MAX_FAILURES as usize);

    // Left: not asked about again, even once the provider would answer.
    *e.batchy
        .delete_error
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = None;
    e.f.engine
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.batchy.deletes.load(Ordering::SeqCst), attempts);
    assert_eq!(
        e.f.store.job(job.id).map(|j| j.status).ok(),
        Some(JobStatus::Completed)
    );
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 2);
}

/// The app stopped between collecting a batch and deleting its files: the next launch deletes
/// them, whether the provider had named them yet or not, and never touches the files of a
/// batch still waiting.
#[tokio::test]
async fn files_left_by_a_stop_after_collection_are_deleted_at_the_next_launch() {
    let e = economy();
    let job = e.job(3, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    let batches = e.f.store.batches(job.id).unwrap_or_default();
    assert_eq!(batches.len(), 2, "two pages a batch");

    // As a crash leaves them: one just marked collected, the other with its files named but
    // none deleted; and a third batch, still waiting, which a launch must leave alone.
    for mut batch in batches {
        let remote = batch.remote_id.clone().unwrap_or_default();
        batch.cleanup = Some(if remote == "remote-0" {
            BatchCleanup::default()
        } else {
            BatchCleanup {
                files: Some(vec![format!("{remote}-in"), format!("{remote}-out")]),
                ..BatchCleanup::default()
            }
        });
        e.f.store
            .put_batch(&batch)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    let mut waiting = e.f.store.batches(job.id).unwrap_or_default().remove(0);
    waiting.id = "still-waiting".into();
    waiting.remote_id = Some("remote-waiting".into());
    waiting.phase = BatchPhase::Waiting;
    waiting.cleanup = Some(BatchCleanup::default());
    e.f.store
        .put_batch(&waiting)
        .unwrap_or_else(|e| panic!("{e}"));
    e.batchy
        .deleted
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clear();
    let listed = e.batchy.listed.load(Ordering::SeqCst);

    let relaunched = engine(e.f.store.clone(), e.batchy.clone());
    relaunched.recover().unwrap_or_else(|e| panic!("{e}"));
    relaunched
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        e.batchy.deleted(),
        vec!["remote-0-in", "remote-0-out", "remote-1-in", "remote-1-out"]
    );
    assert_eq!(
        e.batchy.listed.load(Ordering::SeqCst),
        listed + 1,
        "only the batch not yet named is asked about"
    );
    for batch in e.f.store.batches(job.id).unwrap_or_default() {
        if batch.id == "still-waiting" {
            assert_eq!(batch.cleanup, Some(BatchCleanup::default()), "untouched");
        } else {
            assert_eq!(batch.cleanup, done(), "{batch:?}");
        }
    }
}

/// A cancelled batch may still be writing for a few minutes: its files are deleted once the
/// provider says it has settled, at a later launch, even if its job was deleted meanwhile.
#[tokio::test]
async fn a_cancelled_batch_s_files_are_deleted_once_it_settles_even_after_its_job_is_deleted() {
    let e = one_batch();
    e.batchy.working_polls.store(usize::MAX, Ordering::SeqCst);
    e.batchy.settling.store(true, Ordering::SeqCst);
    let kept = e.job(1, Pace::Economy);
    let events = e.start_until_waiting(kept.id, 1).await;
    e.f.engine.cancel(kept.id);
    assert_eq!(e.finish(kept.id, events).await, JobStatus::Cancelled);
    assert_eq!(cleanup_of(&e, kept.id), vec![Some(BatchCleanup::default())]);

    // Another, deleted while its batch waits, as the app does it.
    let deleted = e.job(1, Pace::Economy);
    let events = e.start_until_waiting(deleted.id, 1).await;
    e.f.engine.cancel(deleted.id);
    assert_eq!(e.finish(deleted.id, events).await, JobStatus::Cancelled);
    e.f.engine
        .cancel_batches(deleted.id)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    e.f.engine
        .delete_job(deleted.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.f.store.batch_leftovers().unwrap_or_default().len(), 1);
    assert!(
        e.batchy.deleted().is_empty(),
        "nothing deleted while settling"
    );

    // Settled: the next launch deletes both batches' files and forgets the deleted job's.
    e.batchy.settling.store(false, Ordering::SeqCst);
    e.f.engine
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        e.batchy.deleted(),
        vec!["remote-0-in", "remote-0-out", "remote-1-in", "remote-1-out"]
    );
    assert_eq!(cleanup_of(&e, kept.id), vec![done()]);
    assert!(e.f.store.batch_leftovers().unwrap_or_default().is_empty());
}

/// A provider that keeps the ended batch itself (Claude's) has it deleted through the same
/// path: once, after its answers are stored; a deletion that failed is tried again at the next
/// launch, and nothing is asked once it is deleted.
#[tokio::test]
async fn a_batch_the_provider_keeps_is_deleted_once_and_again_after_a_restart_if_it_failed() {
    let e = one_batch();
    e.batchy.keeps_batch.store(true, Ordering::SeqCst);
    *e.batchy
        .delete_error
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = Some(ProviderError::Transient("529".into()));
    let job = e.job(2, Pace::Economy);
    assert_eq!(e.f.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        cleanup_of(&e, job.id),
        vec![Some(BatchCleanup {
            files: Some(vec!["remote-0".to_string()]),
            failures: 1,
            given_up: false,
        })],
        "named and recorded before the deletion was tried"
    );
    assert!(e.batchy.deleted().is_empty());

    // The provider answers again; the app is started anew.
    *e.batchy
        .delete_error
        .lock()
        .unwrap_or_else(|p| p.into_inner()) = None;
    let relaunched = engine(e.f.store.clone(), e.batchy.clone());
    relaunched.recover().unwrap_or_else(|e| panic!("{e}"));
    relaunched
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.batchy.deleted(), vec!["remote-0"]);
    assert_eq!(
        e.batchy
            .submitted_at_delete
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone(),
        vec![0],
        "every answer was stored before the batch was deleted"
    );
    assert_eq!(
        cleanup_of(&e, job.id),
        vec![Some(BatchCleanup {
            files: Some(Vec::new()),
            failures: 1,
            given_up: false,
        })],
        "nothing left to delete; the failed round stays counted"
    );

    // Deleted once: a later launch asks nothing more.
    relaunched
        .clean_up_batches()
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(e.batchy.listed.load(Ordering::SeqCst), 1);
    assert_eq!(e.batchy.deletes.load(Ordering::SeqCst), 2);
    assert_eq!(e.batchy.deleted(), vec!["remote-0"]);
    let pieces = e.f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 2);
}
