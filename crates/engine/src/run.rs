//! Running a job: each source is opened once, split into pieces, and every piece × set-up is
//! either taken from the cache or sent to its provider, a few at a time. In economy mode, the
//! pieces of set-ups whose provider has a batch service are gathered into batches instead
//! ([`crate::batch`]) and collected once every source has been read.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use kalamu_core::{
    ApiKey, CacheKey, Job, JobId, JobStatus, Pace, PieceRef, PieceState, PieceStatus,
    PreparedInput, ProcessOutput, ProcessRequest, ProgressEvent, ProviderError, Setup, SetupId,
    SourceFile, SourceId, Step, StopSignal, StoredResult, Task, Timestamp,
};
use kalamu_media::OpenSource;

use crate::{Control, Engine, EngineError};

/// What a call reads: a piece still to cut from its source, or an input already made (a table
/// of contents' pages, gathered for its second step).
pub(crate) enum Input {
    Open(Arc<OpenSource>),
    Ready(PreparedInput),
}

/// How a piece ended, for the job's final status.
pub(crate) enum Outcome {
    Finished,
    Failed,
    /// Stopped part-way by a pause or cancel; the piece is pending again.
    Interrupted,
    /// The key or the account is unusable: stop the job.
    Fatal(String),
}

/// The text a model returned, tidied: a blank-page marker becomes an empty page.
pub(crate) fn clean(mut output: ProcessOutput) -> ProcessOutput {
    let trimmed = output.text.trim();
    if trimmed.eq_ignore_ascii_case("[BLANK PAGE]") {
        output.text = String::new();
    } else if trimmed.len() != output.text.len() {
        output.text = trimmed.to_string();
    }
    output
}

/// A provider's answer as the task stores it: a correction's one field becomes the page's text.
pub(crate) fn finish(task: Task, output: ProcessOutput) -> ProcessOutput {
    let mut output = clean(output);
    if task == Task::TextCorrection {
        output.text = kalamu_core::correction::text(&output.fields);
        output.fields = Vec::new();
    }
    output
}

impl Engine {
    pub(crate) async fn run(
        self: &Arc<Self>,
        job_id: JobId,
        control: Arc<Mutex<Control>>,
    ) -> Result<JobStatus, EngineError> {
        let job = self.store.job(job_id)?;
        // Check keys and offline models before touching anything, so a missing key or an
        // undownloaded model is reported at once.
        for step in job
            .setups
            .iter()
            .flat_map(|s| [Some(s.step()), s.merge_step()])
        {
            let Some(step) = step else { continue };
            let provider = self.provider(step.provider)?;
            if provider.needs_key() && self.keys.key(step.provider).is_none() {
                return Err(EngineError::MissingKey(step.provider.to_string()));
            }
            provider
                .check_ready(step.model)
                .map_err(EngineError::NotReady)?;
        }
        let stop = control
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stop
            .clone();
        self.store.set_job_status(job_id, JobStatus::Running)?;
        self.reopen_unsent(job_id)?;
        // Set-ups sent as batches; the others, and every set-up of a fast job, go one by one.
        let batched: HashSet<SetupId> = if job.settings.pace == Pace::Economy {
            let mut batched = HashSet::new();
            for setup in &job.setups {
                if self
                    .provider(&setup.provider)?
                    .batches(job.task, &setup.model)
                {
                    batched.insert(setup.id);
                }
            }
            batched
        } else {
            HashSet::new()
        };
        let mut gathered = HashMap::new();
        let mut batch_outcome = crate::batch::BatchOutcome::default();
        let existing: HashMap<(SetupId, SourceId, u32), PieceState> = self
            .store
            .pieces(job_id)?
            .into_iter()
            .map(|p| ((p.setup_id, p.source_id, p.index), p))
            .collect();
        let finished_before = existing.values().filter(|p| p.status.is_finished()).count();
        self.emit(ProgressEvent::JobStarted {
            job_id,
            pieces_total: existing.len() as u32,
            pieces_finished: finished_before as u32,
        });

        let mut failed = false;
        // A piece stopped part-way is pending again; the job must not read as finished.
        let mut interrupted = false;
        let spent = Arc::new(Mutex::new(0.0f64));
        for source in &job.sources {
            if stop_requested(&control).is_some() {
                break;
            }
            let opened = {
                let source = source.clone();
                let ffmpeg = self.config.ffmpeg.clone();
                tokio::task::spawn_blocking(move || OpenSource::open(&source, ffmpeg.as_ref()))
                    .await
                    .map_err(|e| EngineError::Output(e.to_string()))?
            };
            let opened = match opened {
                Ok(o) => Arc::new(o),
                Err(e) => {
                    failed = true;
                    self.emit(ProgressEvent::SourceFailed {
                        job_id,
                        source_id: source.id,
                        error: e.to_string(),
                    });
                    continue;
                }
            };
            // A recording resumed keeps the pieces planned the first time: planning decodes the
            // whole file, and the cuts must not move under results already stored.
            let stored = stored_pieces(&existing, &job, source);
            let pieces = if job.task == Task::Analysis {
                // An analysis reads the whole document at once.
                vec![PieceRef::Whole]
            } else if job.task == Task::TextCorrection {
                // A correction reads the text there is: a PDF's pages, a text file's passages.
                let (opened, sample) = (opened.clone(), job.settings.sample_pages);
                let planned = tokio::task::spawn_blocking(move || opened.text_pieces(sample))
                    .await
                    .map_err(|e| EngineError::Output(e.to_string()))?;
                match planned {
                    Ok(pieces) => pieces,
                    Err(e) => {
                        failed = true;
                        self.emit(ProgressEvent::SourceFailed {
                            job_id,
                            source_id: source.id,
                            error: e.to_string(),
                        });
                        continue;
                    }
                }
            } else if stored.is_empty() {
                let (opened, split, sample, length) = (
                    opened.clone(),
                    source.split_spreads,
                    job.settings.sample_pages,
                    self.piece_length(&job),
                );
                tokio::task::spawn_blocking(move || opened.pieces(split, sample, length))
                    .await
                    .map_err(|e| EngineError::Output(e.to_string()))?
            } else {
                stored
            };
            // A table of contents ends with the step that combines its pages.
            let mut pieces = pieces;
            if job.task == Task::Contents && pieces.last() != Some(&PieceRef::Whole) {
                pieces.push(PieceRef::Whole);
            }
            self.emit(ProgressEvent::SourcePrepared {
                job_id,
                source_id: source.id,
                pieces: pieces.len() as u32,
            });

            // Record every piece first, so totals and the resume screen are right at once.
            let mut work = Vec::new();
            for setup in &job.setups {
                for (index, piece) in pieces.iter().enumerate() {
                    let index = index as u32;
                    // The combining step's key depends on the pages' results: it is worked out
                    // once they are read (`merge_pages`).
                    if job.task == Task::Contents && *piece == PieceRef::Whole {
                        if !existing.contains_key(&(setup.id, source.id, index)) {
                            self.store.upsert_piece(&PieceState {
                                job_id,
                                setup_id: setup.id,
                                source_id: source.id,
                                index,
                                piece: piece.clone(),
                                status: PieceStatus::Pending,
                                cache_key: None,
                                error: None,
                                attempts: 0,
                                updated_at: Timestamp::now(),
                            })?;
                        }
                        continue;
                    }
                    let key = CacheKey::compute(
                        job.task,
                        &source.sha256,
                        piece,
                        setup,
                        &job.settings.languages,
                        &crate::output::preprocessing(source),
                    );
                    let state = match existing.get(&(setup.id, source.id, index)) {
                        Some(s)
                            if s.status.is_finished()
                                && s.cache_key.as_deref() == Some(key.as_str()) =>
                        {
                            continue;
                        }
                        // In a batch the provider still holds: collected below.
                        Some(s) if s.status == PieceStatus::Submitted => continue,
                        Some(s) => PieceState {
                            status: PieceStatus::Pending,
                            cache_key: Some(key.0.clone()),
                            piece: piece.clone(),
                            ..s.clone()
                        },
                        None => PieceState {
                            job_id,
                            setup_id: setup.id,
                            source_id: source.id,
                            index,
                            piece: piece.clone(),
                            status: PieceStatus::Pending,
                            cache_key: Some(key.0.clone()),
                            error: None,
                            attempts: 0,
                            updated_at: Timestamp::now(),
                        },
                    };
                    self.store.upsert_piece(&state)?;
                    work.push((setup.clone(), state, key));
                }
            }

            let (to_batch, work): (Vec<_>, Vec<_>) = work
                .into_iter()
                .partition(|(setup, _, _)| batched.contains(&setup.id));
            for (setup, state, key) in to_batch {
                if stop_requested(&control).is_some() {
                    break;
                }
                let outcome = self
                    .gather(
                        &job,
                        &source.file_name,
                        &setup,
                        state,
                        &key,
                        &opened,
                        &mut gathered,
                    )
                    .await?;
                if let Some(message) = &outcome.fatal {
                    tracing::warn!(job = %job_id, %message, "stopping job");
                    control.lock().unwrap_or_else(|p| p.into_inner()).pause = true;
                }
                batch_outcome.merge(outcome);
            }

            let mut in_flight = FuturesUnordered::new();
            let mut queue = work.into_iter();
            let mut stopped = false;
            loop {
                while !stopped && in_flight.len() < self.config.concurrency.max(1) {
                    if stop_requested(&control).is_some() {
                        stopped = true;
                        break;
                    }
                    let Some((setup, state, key)) = queue.next() else {
                        break;
                    };
                    let engine = self.clone();
                    let (job, source, opened, spent, stop) = (
                        job.clone(),
                        source.clone(),
                        opened.clone(),
                        spent.clone(),
                        stop.clone(),
                    );
                    in_flight.push(tokio::spawn(async move {
                        engine
                            .process_piece(
                                &job,
                                &source,
                                setup.step(),
                                state,
                                key,
                                Input::Open(opened),
                                spent,
                                &stop,
                            )
                            .await
                    }));
                }
                let Some(done) = in_flight.next().await else {
                    break;
                };
                match done {
                    Ok(Ok(Outcome::Finished)) => {}
                    Ok(Ok(Outcome::Interrupted)) => interrupted = true,
                    Ok(Ok(Outcome::Failed)) | Err(_) => failed = true,
                    Ok(Ok(Outcome::Fatal(message))) => {
                        tracing::warn!(job = %job_id, %message, "stopping job");
                        control.lock().unwrap_or_else(|p| p.into_inner()).pause = true;
                        stopped = true;
                        failed = true;
                    }
                    Ok(Err(e)) => return Err(e),
                }
            }

            // A table of contents: combine the pages of each set-up that read them all.
            if job.task == Task::Contents && !stopped {
                for setup in job.setups.iter().filter(|s| !batched.contains(&s.id)) {
                    match self.merge_pages(&job, source, setup, &spent, &stop).await? {
                        Outcome::Finished => {}
                        Outcome::Interrupted => interrupted = true,
                        Outcome::Failed => failed = true,
                        Outcome::Fatal(message) => {
                            tracing::warn!(job = %job_id, %message, "stopping job");
                            control.lock().unwrap_or_else(|p| p.into_inner()).pause = true;
                            failed = true;
                        }
                    }
                }
            }

            // Write outputs for every set-up whose pieces of this source are all done.
            let setups: Vec<&Setup> = job.setups.iter().collect();
            failed |= self.export_finished(&job, source.id, &setups)?;
        }

        // Economy mode: send what is gathered, then wait for every batch of the job, including
        // those sent before the app was last closed.
        if !batched.is_empty() {
            if stop_requested(&control).is_none() {
                let outcome = self.send_gathered(&job, &mut gathered).await?;
                batch_outcome.merge(outcome);
            }
            if batch_outcome.fatal.is_none() {
                let outcome = self.wait_for_batches(&job, &control, &spent).await?;
                batch_outcome.merge(outcome);
            }
            let setups: Vec<&Setup> = job
                .setups
                .iter()
                .filter(|s| batched.contains(&s.id))
                .collect();
            for source in &job.sources {
                // The pages came back from the batch: combine them, at once.
                if job.task == Task::Contents && stop_requested(&control).is_none() {
                    for setup in &setups {
                        match self.merge_pages(&job, source, setup, &spent, &stop).await? {
                            Outcome::Finished => {}
                            Outcome::Interrupted => interrupted = true,
                            Outcome::Failed | Outcome::Fatal(_) => failed = true,
                        }
                    }
                }
                failed |= self.export_finished(&job, source.id, &setups)?;
            }
            failed |= batch_outcome.failed;
            interrupted |= batch_outcome.waiting;
        }

        Ok(match stop_requested(&control) {
            Some(StopKind::Cancel) => JobStatus::Cancelled,
            Some(StopKind::Pause) => JobStatus::Paused,
            None if interrupted => JobStatus::Paused,
            None if failed => JobStatus::CompletedWithErrors,
            None => JobStatus::Completed,
        })
    }

    /// The provider's formats for the task, narrowed to those the model reads: a text-only
    /// model behind a provider that takes PDFs and images is sent text alone. A correction
    /// sends text whatever the model reads.
    pub(crate) fn accepted(
        &self,
        job: &Job,
        step: Step<'_>,
    ) -> Result<Vec<&'static str>, EngineError> {
        let provider = self.provider(step.provider)?;
        let catalog = self.catalog();
        let model = catalog.model(step.provider, step.model);
        Ok(provider
            .accepted_inputs(job.task)
            .iter()
            .copied()
            .filter(|mime| model.is_none_or(|m| m.inputs.iter().any(|i| i == mime)))
            .filter(|mime| job.task != Task::TextCorrection || *mime == "text/plain")
            .collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn process_piece(
        &self,
        job: &Job,
        source: &SourceFile,
        step: Step<'_>,
        mut state: PieceState,
        key: CacheKey,
        input: Input,
        spent: Arc<Mutex<f64>>,
        stop: &StopSignal,
    ) -> Result<Outcome, EngineError> {
        let update = |engine: &Engine, state: &PieceState, preview: Option<String>| {
            engine.emit(ProgressEvent::PieceUpdated {
                job_id: state.job_id,
                setup_id: state.setup_id,
                source_id: state.source_id,
                index: state.index,
                piece: state.piece.clone(),
                status: state.status,
                error: state.error.clone(),
                preview,
            });
        };

        if let Some(cached) = self.store.cached_result(&key)? {
            state.status = PieceStatus::Cached;
            state.error = None;
            state.updated_at = Timestamp::now();
            self.store.upsert_piece(&state)?;
            update(self, &state, Some(preview(&cached.output.text)));
            return Ok(Outcome::Finished);
        }

        state.status = PieceStatus::Running;
        state.updated_at = Timestamp::now();
        self.store.upsert_piece(&state)?;
        update(self, &state, None);

        let provider = self.provider(step.provider)?;
        let api_key = if provider.needs_key() {
            self.keys
                .key(step.provider)
                .ok_or_else(|| EngineError::MissingKey(step.provider.to_string()))?
        } else {
            ApiKey::new("")
        };
        let prepared = match input {
            Input::Ready(input) => Ok(input),
            Input::Open(opened) => {
                let accepted = self.accepted(job, step)?;
                let stem = Path::new(&source.file_name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "page".into());
                let piece = state.piece.clone();
                tokio::task::spawn_blocking(move || opened.prepare(&piece, &accepted, &stem))
                    .await
                    .map_err(|e| EngineError::Output(e.to_string()))?
            }
        };
        let input = match prepared {
            Ok(input) => input,
            Err(e) => {
                state.status = PieceStatus::Failed;
                state.error = Some(e.to_string());
                state.updated_at = Timestamp::now();
                self.store.upsert_piece(&state)?;
                update(self, &state, None);
                return Ok(Outcome::Failed);
            }
        };

        // The set-up's options (speakers, timestamps), which the combining step shares.
        let options = job
            .setups
            .iter()
            .find(|s| s.id == state.setup_id)
            .map(|s| s.options.clone())
            .unwrap_or_default();
        let started = Instant::now();
        let mut last_error: Option<ProviderError> = None;
        for attempt in 0..self.config.max_attempts {
            state.attempts += 1;
            let request = ProcessRequest {
                task: job.task,
                model: step.model,
                prompt: &step.prompt.text,
                user_prompt: &step.prompt.user_prompt,
                input: &input,
                languages: &job.settings.languages,
                options: &options,
                analysis: step.prompt.schema.as_ref(),
                stop,
            };
            let sent = provider.sent_settings(&request, false);
            match provider.process(&api_key, request).await {
                Ok(output) => {
                    let output = finish(job.task, output);
                    let catalog = self.catalog();
                    let cost = catalog
                        .model(step.provider, step.model)
                        .and_then(|m| catalog.actual_cost(m, &output.usage));
                    let result = StoredResult {
                        cache_key: key.0.clone(),
                        provider: step.provider.clone(),
                        model: step.model.to_string(),
                        output,
                        request: sent,
                        cost_usd: cost,
                        duration_ms: started.elapsed().as_millis() as u64,
                        created_at: Timestamp::now(),
                    };
                    self.store.put_result(&result)?;
                    state.status = PieceStatus::Done;
                    state.error = None;
                    state.updated_at = Timestamp::now();
                    self.store.upsert_piece(&state)?;
                    update(self, &state, Some(preview(&result.output.text)));
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
                    return Ok(Outcome::Finished);
                }
                Err(ProviderError::Interrupted) => {
                    state.status = PieceStatus::Pending;
                    state.error = None;
                    state.updated_at = Timestamp::now();
                    self.store.upsert_piece(&state)?;
                    update(self, &state, None);
                    return Ok(Outcome::Interrupted);
                }
                Err(err) if err.is_fatal_for_job() => {
                    state.status = PieceStatus::Failed;
                    state.error = Some(err.to_string());
                    state.updated_at = Timestamp::now();
                    self.store.upsert_piece(&state)?;
                    update(self, &state, None);
                    return Ok(Outcome::Fatal(err.to_string()));
                }
                Err(err) if crate::is_retryable(&err) && attempt + 1 < self.config.max_attempts => {
                    let delay = match &err {
                        ProviderError::RateLimited {
                            retry_after: Some(d),
                            ..
                        } => *d + Duration::from_secs(1),
                        _ => backoff(self.config.base_delay, attempt, state.index),
                    };
                    tracing::info!(piece = state.index, ?delay, error = %err, "retrying");
                    last_error = Some(err);
                    tokio::time::sleep(delay).await;
                }
                Err(err) => {
                    last_error = Some(err);
                    break;
                }
            }
        }
        state.status = PieceStatus::Failed;
        state.error = last_error.map(|e| e.to_string());
        state.updated_at = Timestamp::now();
        self.store.upsert_piece(&state)?;
        update(self, &state, None);
        Ok(Outcome::Failed)
    }
}

/// The pieces already recorded for a recording, in order; empty for documents, whose pages are
/// cheap to list again, and for a recording not started yet.
fn stored_pieces(
    existing: &HashMap<(SetupId, SourceId, u32), PieceState>,
    job: &Job,
    source: &SourceFile,
) -> Vec<PieceRef> {
    let audio = matches!(
        source.kind,
        kalamu_core::InputKind::Audio | kalamu_core::InputKind::Video
    );
    let Some(first) = job.setups.first() else {
        return Vec::new();
    };
    if !audio {
        return Vec::new();
    }
    let mut mine: Vec<&PieceState> = existing
        .values()
        .filter(|p| p.setup_id == first.id && p.source_id == source.id)
        .collect();
    mine.sort_by_key(|p| p.index);
    mine.into_iter().map(|p| p.piece.clone()).collect()
}

pub(crate) enum StopKind {
    Pause,
    Cancel,
}

pub(crate) fn stop_requested(control: &Mutex<Control>) -> Option<StopKind> {
    let c = control.lock().unwrap_or_else(|p| p.into_inner());
    if c.cancel {
        Some(StopKind::Cancel)
    } else if c.pause {
        Some(StopKind::Pause)
    } else {
        None
    }
}

/// `base × 2^attempt`, plus up to two seconds spread by piece so parallel retries do not land
/// together.
fn backoff(base: Duration, attempt: u32, index: u32) -> Duration {
    let spread = Duration::from_millis(u64::from(index % 8) * 250);
    base * 2u32.saturating_pow(attempt).min(32) + spread
}

pub(crate) fn preview(text: &str) -> String {
    text.chars().take(400).collect()
}
