//! Sending: pieces gathered per set-up, recorded as a batch, then sent.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kalamu_core::{
    BatchItem, BatchPhase, BatchRecord, CacheKey, Job, JobId, PieceState, PieceStatus,
    ProcessRequest, ProviderError, Setup, SetupId, SourceId, StopSignal, Timestamp,
};
use kalamu_media::OpenSource;

use super::{BatchOutcome, Gathered, stem, waiting};
use crate::{Engine, EngineError};

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
            self.save(&mut state, Some(crate::piece::preview(&cached.output.text)))?;
            return Ok(BatchOutcome::default());
        }
        let accepted = self.accepted(job, setup.step())?;
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
    pub(super) async fn send(
        &self,
        job: &Job,
        setup: &Setup,
        bucket: Gathered,
    ) -> Result<BatchOutcome, EngineError> {
        let provider = self.provider(&setup.provider)?;
        let api_key = self.api_key(setup)?;
        let now = Timestamp::now();
        let stop = StopSignal::default();
        let requests: Vec<(String, ProcessRequest<'_>)> = bucket
            .pieces
            .iter()
            .enumerate()
            .map(|(i, (_, input))| {
                (
                    format!("p{i}"),
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
                .zip(&requests)
                .map(|((state, _), (id, request))| BatchItem {
                    id: id.clone(),
                    source_id: state.source_id,
                    index: state.index,
                    request: provider.sent_settings(request, true),
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
}
