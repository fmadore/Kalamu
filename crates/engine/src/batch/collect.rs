//! Collecting: waiting batches polled until they end, their answers stored as single calls'
//! would be, and batches given up on.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kalamu_core::{
    BatchContext, BatchPhase, BatchProgress, BatchRecord, Job, JobId, JobStatus, PieceState,
    PieceStatus, ProcessOutput, ProviderError, Setup, SourceId, StoredResult, Timestamp,
};

use super::{BatchOutcome, waiting};
use crate::{Control, Engine, EngineError};

impl Engine {
    /// Polls every waiting batch of a job until all are collected, or the job is paused or
    /// cancelled. Asks at once, so a resumed job collects what finished while the app was closed.
    pub(crate) async fn wait_for_batches(
        &self,
        job: &Job,
        control: &Mutex<Control>,
        spent: &Mutex<f64>,
    ) -> Result<BatchOutcome, EngineError> {
        let mut outcome = BatchOutcome::default();
        loop {
            let open: Vec<BatchRecord> = self
                .store
                .batches(job.id)?
                .into_iter()
                .filter(|b| b.phase == BatchPhase::Waiting)
                .collect();
            if open.is_empty() {
                return Ok(outcome);
            }
            match crate::run::stop_requested(control) {
                Some(crate::run::StopKind::Cancel) => {
                    for batch in open {
                        self.abandon(job, batch, None).await?;
                    }
                    return Ok(outcome);
                }
                Some(crate::run::StopKind::Pause) => {
                    outcome.waiting = true;
                    return Ok(outcome);
                }
                None => {}
            }
            for batch in open {
                let setup = job
                    .setups
                    .iter()
                    .find(|s| s.id == batch.setup_id)
                    .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
                let provider = self.provider(&setup.provider)?;
                let api_key = self.api_key(setup)?;
                let context = BatchContext {
                    task: job.task,
                    model: &setup.model,
                    analysis: setup.prompt.schema.as_ref(),
                };
                let ids: Vec<String> = batch.items.iter().map(|i| i.id.clone()).collect();
                let remote = batch.remote_id.clone().unwrap_or_default();
                match provider.poll_batch(&api_key, &remote, context, &ids).await {
                    Ok(BatchProgress::Working { finished, .. }) => {
                        self.emit(waiting(&batch, finished));
                    }
                    Ok(BatchProgress::Ended(answers)) => {
                        outcome.merge(self.collect(job, batch, answers, spent)?);
                    }
                    Ok(BatchProgress::Abandoned(reason)) => {
                        outcome.failed = true;
                        self.abandon(job, batch, Some(reason)).await?;
                    }
                    Err(err) if err.is_fatal_for_job() => {
                        control.lock().unwrap_or_else(|p| p.into_inner()).pause = true;
                        outcome.fatal = Some(err.to_string());
                        outcome.waiting = true;
                        return Ok(outcome);
                    }
                    // A network hiccup or a busy service: ask again next time.
                    Err(err) => tracing::info!(batch = %remote, error = %err, "batch not read"),
                }
            }
            let next = Instant::now() + self.config.batch_poll;
            while Instant::now() < next && crate::run::stop_requested(control).is_none() {
                tokio::time::sleep(self.config.batch_poll.min(Duration::from_millis(500))).await;
            }
        }
    }

    /// Stores a finished batch's answers, each as a single call's would be.
    fn collect(
        &self,
        job: &Job,
        mut batch: BatchRecord,
        answers: Vec<(String, Result<ProcessOutput, ProviderError>)>,
        spent: &Mutex<f64>,
    ) -> Result<BatchOutcome, EngineError> {
        let setup = job
            .setups
            .iter()
            .find(|s| s.id == batch.setup_id)
            .ok_or(EngineError::Store(kalamu_core::StoreError::NotFound))?;
        let mut answers: HashMap<String, Result<ProcessOutput, ProviderError>> =
            answers.into_iter().collect();
        let mut pieces: HashMap<(SourceId, u32), PieceState> = self
            .store
            .pieces(job.id)?
            .into_iter()
            .filter(|p| p.setup_id == batch.setup_id)
            .map(|p| ((p.source_id, p.index), p))
            .collect();
        let catalog = self.catalog();
        let model = catalog.model(&setup.provider, &setup.model);
        let waited = (Timestamp::now().0 - batch.submitted_at.0).max(0) as u64;
        let mut outcome = BatchOutcome::default();
        for item in &batch.items {
            let Some(mut state) = pieces.remove(&(item.source_id, item.index)) else {
                continue;
            };
            if state.status != PieceStatus::Submitted {
                continue;
            }
            let answer = answers.remove(&item.id).unwrap_or_else(|| {
                Err(ProviderError::Other(
                    "the provider returned no answer for this piece".into(),
                ))
            });
            match answer {
                Ok(output) => {
                    let mut output = crate::piece::finish(job.task, output);
                    output.usage.batch = true;
                    let cost = model.and_then(|m| catalog.actual_cost(m, &output.usage));
                    let preview = crate::piece::preview(&output.text);
                    self.store.put_result(&StoredResult {
                        cache_key: state.cache_key.clone().unwrap_or_default(),
                        provider: setup.provider.clone(),
                        model: setup.model.clone(),
                        output,
                        request: item.request.clone(),
                        cost_usd: cost,
                        duration_ms: waited,
                        created_at: Timestamp::now(),
                    })?;
                    state.status = PieceStatus::Done;
                    state.error = None;
                    self.save(&mut state, Some(preview))?;
                    self.add_cost(job.id, spent, cost);
                }
                // Cancelled before the provider reached it: still to do.
                Err(ProviderError::Interrupted) => {
                    state.status = PieceStatus::Pending;
                    self.save(&mut state, None)?;
                }
                Err(err) => {
                    outcome.failed = true;
                    state.status = PieceStatus::Failed;
                    state.error = Some(err.to_string());
                    self.save(&mut state, None)?;
                }
            }
        }
        batch.phase = BatchPhase::Collected;
        batch.updated_at = Timestamp::now();
        self.store.put_batch(&batch)?;
        Ok(outcome)
    }

    /// Gives up on a batch: cancelled by the user (`reason` is `None`; the provider is asked to
    /// stop and the pieces are left to do) or ended by the provider without answers.
    pub(super) async fn abandon(
        &self,
        job: &Job,
        mut batch: BatchRecord,
        reason: Option<String>,
    ) -> Result<(), EngineError> {
        if reason.is_none()
            && let (Some(remote), Some(setup)) = (
                batch.remote_id.as_deref(),
                job.setups.iter().find(|s| s.id == batch.setup_id),
            )
            && let Err(err) = self
                .provider(&setup.provider)?
                .cancel_batch(&self.api_key(setup)?, remote)
                .await
        {
            tracing::warn!(batch = remote, error = %err, "batch not cancelled");
        }
        let held: HashSet<(SourceId, u32)> =
            batch.items.iter().map(|i| (i.source_id, i.index)).collect();
        for mut state in self.store.pieces(job.id)? {
            if state.setup_id == batch.setup_id
                && state.status == PieceStatus::Submitted
                && held.contains(&(state.source_id, state.index))
            {
                state.status = if reason.is_some() {
                    PieceStatus::Failed
                } else {
                    PieceStatus::Pending
                };
                state.error = reason.clone();
                self.save(&mut state, None)?;
            }
        }
        batch.phase = BatchPhase::Abandoned;
        batch.updated_at = Timestamp::now();
        self.store.put_batch(&batch)?;
        Ok(())
    }

    /// Asks the provider to stop a job's waiting batches, before the job is deleted.
    pub async fn cancel_batches(&self, job_id: JobId) -> Result<(), EngineError> {
        let job = self.store.job(job_id)?;
        for batch in self.store.batches(job_id)? {
            if batch.phase == BatchPhase::Waiting {
                self.abandon(&job, batch, None).await?;
            }
        }
        Ok(())
    }

    /// Waits for a job's batches and collects their answers, then writes the documents they
    /// complete. Nothing is sent: a batch recorded but never confirmed goes back to the queue
    /// for the next run, and a table of contents is not combined. The job keeps the status it
    /// had unless the answers finish it.
    pub(crate) async fn collect_batches(
        self: &Arc<Self>,
        job_id: JobId,
        control: Arc<Mutex<Control>>,
    ) -> Result<JobStatus, EngineError> {
        let job = self.store.job(job_id)?;
        self.store.set_job_status(job_id, JobStatus::Running)?;
        self.reopen_unsent(job_id)?;
        let spent = Mutex::new(self.store.spent(job_id)?);
        let outcome = self.wait_for_batches(&job, &control, &spent).await?;
        let setups: Vec<&Setup> = job.setups.iter().collect();
        let mut failed = outcome.failed;
        for source in &job.sources {
            failed |= self.export_finished(&job, source.id, &setups)?;
        }
        let pieces = self.store.pieces(job_id)?;
        let left = pieces
            .iter()
            .any(|p| !p.status.is_finished() && p.status != PieceStatus::Failed);
        Ok(match crate::run::stop_requested(&control) {
            Some(crate::run::StopKind::Cancel) => JobStatus::Cancelled,
            Some(crate::run::StopKind::Pause) => JobStatus::Paused,
            // Work remains: the job waits for the researcher, paused as they left it or
            // interrupted by the last close.
            None if left || outcome.waiting => match job.status {
                JobStatus::Paused => JobStatus::Paused,
                _ => JobStatus::Interrupted,
            },
            None if failed || pieces.iter().any(|p| p.status == PieceStatus::Failed) => {
                JobStatus::CompletedWithErrors
            }
            None => JobStatus::Completed,
        })
    }

    /// Jobs with batches still at the provider, which the app collects at startup.
    pub fn jobs_waiting_for_batches(&self) -> Result<Vec<JobId>, EngineError> {
        let mut waiting = Vec::new();
        for job in self.store.list_jobs()? {
            if job.status.is_unfinished()
                && self
                    .store
                    .batches(job.id)?
                    .iter()
                    .any(|b| b.phase == BatchPhase::Waiting)
            {
                waiting.push(job.id);
            }
        }
        Ok(waiting)
    }
}
