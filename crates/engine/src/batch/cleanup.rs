//! Cleaning up: what an ended batch left in the provider's storage (the requests it uploaded,
//! the answers, the errors) deleted once nothing more can be read from it.
//!
//! The batch's record is marked as due for cleaning in the same write that marks it collected
//! or given up on, after its answers are stored: a crash before that write collects the batch
//! again, a crash after it finishes the deleting at the next collection or launch. The files
//! are named by the provider from the batch itself and recorded before any is deleted, so only
//! what Kalamu's own batches made is ever deleted, and never a file of a batch still waiting.
//! A failed round is logged and tried again later, [`BatchCleanup::MAX_FAILURES`] times, after
//! which the record says the files were left; no failure here touches a job or its results.

use kalamu_core::{BatchCleanup, BatchRecord, JobId};

use crate::control::lock;
use crate::{Engine, EngineError};

/// Where a batch's record is kept: with its job, or apart once the job was deleted.
#[derive(Clone, Copy)]
enum Kept {
    WithJob,
    Leftover,
}

impl Engine {
    /// Marks an ended batch as due for cleaning, unless it never reached the provider (no id)
    /// or was already marked.
    pub(super) fn due_for_cleanup(batch: &mut BatchRecord) {
        if batch.remote_id.is_some() && batch.cleanup.is_none() {
            batch.cleanup = Some(BatchCleanup::default());
        }
    }

    /// Deletes what the batch left at the provider, as far as it can now, and records how far
    /// it got. Never fails for the provider's sake: only a store that cannot be written is an
    /// error.
    pub(super) async fn clean_up(&self, batch: &mut BatchRecord) -> Result<(), EngineError> {
        self.clean_up_kept(batch, Kept::WithJob).await
    }

    async fn clean_up_kept(&self, batch: &mut BatchRecord, kept: Kept) -> Result<(), EngineError> {
        if !batch.cleanup_pending() {
            return Ok(());
        }
        let Some(remote) = batch.remote_id.clone() else {
            return Ok(());
        };
        let (Ok(provider), Some(key)) = (
            self.provider(&batch.provider),
            self.keys.key(&batch.provider),
        ) else {
            // No key or no such provider in this version: nothing can be asked, and nothing
            // counts as a failure; the next launch tries again.
            tracing::info!(batch = %remote, "batch files not deleted: no key for its provider");
            return Ok(());
        };
        let Some(mut cleanup) = batch.cleanup.clone() else {
            return Ok(());
        };
        let mut failed = None;
        if cleanup.files.is_none() {
            match provider.batch_files(&key, &remote).await {
                Ok(Some(files)) => {
                    // Recorded before any is deleted, so a crash in between still knows them.
                    cleanup.files = Some(files);
                    batch.cleanup = Some(cleanup.clone());
                    self.keep(batch, kept)?;
                }
                // Still settling (being cancelled): asked again later, not a failure.
                Ok(None) => return Ok(()),
                Err(err) => failed = Some(err),
            }
        }
        if let Some(files) = cleanup.files.as_mut() {
            let mut left = Vec::with_capacity(files.len());
            for file in files.drain(..) {
                match provider.delete_batch_file(&key, &file).await {
                    Ok(()) => {}
                    Err(err) => {
                        tracing::warn!(batch = %remote, file = %file, error = %err,
                            "batch file not deleted");
                        failed.get_or_insert(err);
                        left.push(file);
                    }
                }
            }
            *files = left;
        }
        if let Some(err) = failed {
            cleanup.failures += 1;
            if cleanup.failures >= BatchCleanup::MAX_FAILURES {
                cleanup.given_up = true;
                tracing::warn!(batch = %remote, error = %err, files = ?cleanup.files,
                    "batch files left at the provider after repeated failures");
            } else {
                tracing::info!(batch = %remote, error = %err, "batch files to delete later");
            }
        }
        batch.cleanup = Some(cleanup);
        self.keep(batch, kept)
    }

    fn keep(&self, batch: &BatchRecord, kept: Kept) -> Result<(), EngineError> {
        match kept {
            Kept::WithJob => self.store.put_batch(batch)?,
            Kept::Leftover => self.store.put_batch_leftover(batch)?,
        }
        Ok(())
    }

    /// Cleans up the ended batches of one job whose files are still to delete.
    pub(super) async fn clean_up_job(&self, job_id: JobId) -> Result<(), EngineError> {
        for mut batch in self.store.batches(job_id)? {
            if batch.cleanup_pending() {
                self.clean_up(&mut batch).await?;
            }
        }
        Ok(())
    }

    /// Finishes deleting what ended batches left at their providers: those of every job not
    /// running now (a running job cleans up its own as it collects), and those of deleted jobs.
    /// Run at launch, so a crash between collecting a batch and deleting its files, or files
    /// that could not be deleted then, are dealt with.
    pub async fn clean_up_batches(&self) -> Result<(), EngineError> {
        for job in self.store.list_jobs()? {
            if lock(&self.running).contains_key(&job.id) {
                continue;
            }
            self.clean_up_job(job.id).await?;
        }
        for mut batch in self.store.batch_leftovers()? {
            self.clean_up_kept(&mut batch, Kept::Leftover).await?;
        }
        Ok(())
    }
}
