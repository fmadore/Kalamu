//! Economy mode: a set-up's pieces gathered into batches for the provider's batch service, then
//! waited for and collected.
//!
//! A batch is recorded before it is sent and its pieces marked as submitted, so that closing the
//! app at any point loses nothing: a batch the provider accepted is collected when the job
//! resumes, and one never confirmed as sent (still `Sending`) puts its pieces back in the queue.
//! The one gap is a crash between the provider accepting a batch and its id being written,
//! which would send those pieces again; the window is one HTTP response.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use scriptorium_core::{
    ApiKey, BATCH_WINDOW_MS, BatchContext, BatchItem, BatchPhase, BatchProgress, BatchRecord,
    CacheKey, Job, JobId, PieceState, PieceStatus, PreparedInput, ProcessOutput, ProcessRequest,
    ProgressEvent, ProviderError, Setup, SetupId, SourceId, StopSignal, StoredResult, Timestamp,
};
use scriptorium_media::OpenSource;

use crate::{Control, Engine, EngineError};

/// Pieces prepared for one set-up and not yet sent.
#[derive(Default)]
pub(crate) struct Gathered {
    pieces: Vec<(PieceState, PreparedInput)>,
    bytes: usize,
}

/// How batches ended for a job, for its final status.
#[derive(Default)]
pub(crate) struct BatchOutcome {
    pub failed: bool,
    /// Batches still with the provider: the job was paused while waiting.
    pub waiting: bool,
    /// The key or the account stopped working.
    pub fatal: Option<String>,
}

impl BatchOutcome {
    pub(crate) fn merge(&mut self, other: BatchOutcome) {
        self.failed |= other.failed;
        self.waiting |= other.waiting;
        if self.fatal.is_none() {
            self.fatal = other.fatal;
        }
    }
}

impl Engine {
    /// Before a job runs: a batch never confirmed as sent is abandoned, and a piece marked as
    /// submitted that no waiting batch holds goes back in the queue.
    pub(crate) fn reopen_unsent(&self, job_id: JobId) -> Result<(), EngineError> {
        let mut held: HashSet<(SetupId, SourceId, u32)> = HashSet::new();
        for mut batch in self.store.batches(job_id)? {
            match batch.phase {
                BatchPhase::Sending => {
                    batch.phase = BatchPhase::Abandoned;
                    batch.updated_at = Timestamp::now();
                    self.store.put_batch(&batch)?;
                }
                BatchPhase::Waiting => held.extend(
                    batch
                        .items
                        .iter()
                        .map(|i| (batch.setup_id, i.source_id, i.index)),
                ),
                _ => {}
            }
        }
        for mut piece in self.store.pieces(job_id)? {
            if piece.status == PieceStatus::Submitted
                && !held.contains(&(piece.setup_id, piece.source_id, piece.index))
            {
                piece.status = PieceStatus::Pending;
                piece.updated_at = Timestamp::now();
                self.store.upsert_piece(&piece)?;
            }
        }
        Ok(())
    }

    /// Takes one piece for a batch: from the cache when it is there, otherwise prepared and
    /// gathered, sending the set-up's batch once it is as large as the provider takes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn gather(
        &self,
        job: &Job,
        source_name: &str,
        setup: &Setup,
        mut state: PieceState,
        key: &CacheKey,
        opened: &Arc<OpenSource>,
        gathered: &mut HashMap<SetupId, Gathered>,
    ) -> Result<BatchOutcome, EngineError> {
        if let Some(cached) = self.store.cached_result(key)? {
            state.status = PieceStatus::Cached;
            state.error = None;
            self.save(&mut state, Some(crate::run::preview(&cached.output.text)))?;
            return Ok(BatchOutcome::default());
        }
        let accepted = self.accepted(job, setup)?;
        let prepared = {
            let (opened, piece, stem) = (opened.clone(), state.piece.clone(), stem(source_name));
            tokio::task::spawn_blocking(move || opened.prepare(&piece, &accepted, &stem))
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?
        };
        let input = match prepared {
            Ok(input) => input,
            Err(e) => {
                state.status = PieceStatus::Failed;
                state.error = Some(e.to_string());
                self.save(&mut state, None)?;
                return Ok(BatchOutcome {
                    failed: true,
                    ..BatchOutcome::default()
                });
            }
        };
        let limits = self.provider(&setup.provider)?.batch_limits();
        let bucket = gathered.entry(setup.id).or_default();
        let mut outcome = BatchOutcome::default();
        // Send what is gathered first if this piece would take the batch over a limit.
        if !bucket.pieces.is_empty()
            && (bucket.pieces.len() >= limits.max_items
                || bucket.bytes + input.bytes.len() > limits.max_bytes)
        {
            outcome = self.send(job, setup, std::mem::take(bucket)).await?;
        }
        bucket.bytes += input.bytes.len();
        bucket.pieces.push((state, input));
        Ok(outcome)
    }

    /// Sends every set-up's gathered pieces.
    pub(crate) async fn send_gathered(
        &self,
        job: &Job,
        gathered: &mut HashMap<SetupId, Gathered>,
    ) -> Result<BatchOutcome, EngineError> {
        let mut outcome = BatchOutcome::default();
        for setup in &job.setups {
            if let Some(bucket) = gathered.remove(&setup.id)
                && !bucket.pieces.is_empty()
            {
                outcome.merge(self.send(job, setup, bucket).await?);
            }
        }
        Ok(outcome)
    }

    /// Records a batch, marks its pieces as submitted, then sends it.
    async fn send(
        &self,
        job: &Job,
        setup: &Setup,
        bucket: Gathered,
    ) -> Result<BatchOutcome, EngineError> {
        let provider = self.provider(&setup.provider)?;
        let api_key = self.api_key(setup)?;
        let now = Timestamp::now();
        let mut record = BatchRecord {
            id: BatchRecord::new_id(),
            job_id: job.id,
            setup_id: setup.id,
            provider: setup.provider.clone(),
            remote_id: None,
            phase: BatchPhase::Sending,
            items: bucket
                .pieces
                .iter()
                .enumerate()
                .map(|(i, (state, _))| BatchItem {
                    id: format!("p{i}"),
                    source_id: state.source_id,
                    index: state.index,
                })
                .collect(),
            submitted_at: now,
            updated_at: now,
        };
        self.store.put_batch(&record)?;
        let mut states: Vec<PieceState> = Vec::with_capacity(bucket.pieces.len());
        for (state, _) in &bucket.pieces {
            let mut state = state.clone();
            state.status = PieceStatus::Submitted;
            state.error = None;
            state.attempts += 1;
            self.save(&mut state, None)?;
            states.push(state);
        }

        let stop = StopSignal::default();
        let requests: Vec<(String, ProcessRequest<'_>)> = record
            .items
            .iter()
            .zip(&bucket.pieces)
            .map(|(item, (_, input))| {
                (
                    item.id.clone(),
                    ProcessRequest {
                        task: job.task,
                        model: &setup.model,
                        prompt: &setup.prompt.text,
                        user_prompt: &setup.prompt.user_prompt,
                        input,
                        languages: &job.settings.languages,
                        options: &setup.options,
                        analysis: setup.prompt.schema.as_ref(),
                        stop: &stop,
                    },
                )
            })
            .collect();
        // Only a refusal for the rate is retried: a timed-out request may have created the
        // batch, and sending it again would pay twice.
        let mut sent = provider.submit_batch(&api_key, &requests).await;
        let mut attempt = 1;
        while let Err(ProviderError::RateLimited { retry_after, .. }) = &sent
            && attempt < self.config.max_attempts
        {
            let delay = retry_after.unwrap_or(self.config.base_delay * 2u32.pow(attempt));
            tokio::time::sleep(delay).await;
            sent = provider.submit_batch(&api_key, &requests).await;
            attempt += 1;
        }
        drop(requests);

        record.updated_at = Timestamp::now();
        match sent {
            Ok(remote) => {
                record.remote_id = Some(remote);
                record.phase = BatchPhase::Waiting;
                self.store.put_batch(&record)?;
                self.emit(waiting(&record, 0));
                Ok(BatchOutcome::default())
            }
            Err(err) => {
                tracing::warn!(job = %job.id, error = %err, "batch refused");
                record.phase = BatchPhase::Abandoned;
                self.store.put_batch(&record)?;
                for mut state in states {
                    state.status = PieceStatus::Failed;
                    state.error = Some(err.to_string());
                    self.save(&mut state, None)?;
                }
                Ok(BatchOutcome {
                    failed: true,
                    fatal: err.is_fatal_for_job().then(|| err.to_string()),
                    ..BatchOutcome::default()
                })
            }
        }
    }

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
                    .ok_or(EngineError::Store(scriptorium_core::StoreError::NotFound))?;
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
            .ok_or(EngineError::Store(scriptorium_core::StoreError::NotFound))?;
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
                    let mut output = crate::run::clean(output);
                    output.usage.batch = true;
                    let cost = model.and_then(|m| catalog.actual_cost(m, &output.usage));
                    let preview = crate::run::preview(&output.text);
                    self.store.put_result(&StoredResult {
                        cache_key: state.cache_key.clone().unwrap_or_default(),
                        provider: setup.provider.clone(),
                        model: setup.model.clone(),
                        output,
                        cost_usd: cost,
                        duration_ms: waited,
                        created_at: Timestamp::now(),
                    })?;
                    state.status = PieceStatus::Done;
                    state.error = None;
                    self.save(&mut state, Some(preview))?;
                    if let Some(cost) = cost {
                        let total = {
                            let mut spent = spent.lock().unwrap_or_else(|p| p.into_inner());
                            *spent += cost;
                            *spent
                        };
                        self.emit(ProgressEvent::CostUpdated {
                            job_id: job.id,
                            spent_usd: total,
                        });
                    }
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
    async fn abandon(
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

    /// Jobs with batches still at the provider, which the app resumes at startup to collect
    /// them.
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

    fn api_key(&self, setup: &Setup) -> Result<ApiKey, EngineError> {
        self.keys
            .key(&setup.provider)
            .ok_or_else(|| EngineError::MissingKey(setup.provider.to_string()))
    }

    /// Stores a piece's new state and reports it.
    fn save(&self, state: &mut PieceState, preview: Option<String>) -> Result<(), EngineError> {
        state.updated_at = Timestamp::now();
        self.store.upsert_piece(state)?;
        self.emit(ProgressEvent::PieceUpdated {
            job_id: state.job_id,
            setup_id: state.setup_id,
            source_id: state.source_id,
            index: state.index,
            piece: state.piece.clone(),
            status: state.status,
            error: state.error.clone(),
            preview,
        });
        Ok(())
    }
}

fn waiting(batch: &BatchRecord, finished: u32) -> ProgressEvent {
    ProgressEvent::BatchWaiting {
        job_id: batch.job_id,
        setup_id: batch.setup_id,
        pieces: batch.items.len() as u32,
        finished,
        submitted_at: batch.submitted_at,
        due_by: Timestamp(batch.submitted_at.0 + BATCH_WINDOW_MS),
    }
}

fn stem(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".into())
}
