//! Running a job: each source is opened once, split into pieces, and every piece × set-up is
//! either taken from the cache or sent to its provider, a few at a time.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use scriptorium_core::{
    ApiKey, CacheKey, Job, JobId, JobStatus, PieceRef, PieceState, PieceStatus, ProcessOutput,
    ProcessRequest, ProgressEvent, ProviderError, Setup, SetupId, SourceFile, SourceId, StopSignal,
    StoredResult, Timestamp,
};
use scriptorium_media::OpenSource;

use crate::{Control, Engine, EngineError};

/// How a piece ended, for the job's final status.
enum Outcome {
    Finished,
    Failed,
    /// Stopped part-way by a pause or cancel; the piece is pending again.
    Interrupted,
    /// The key or the account is unusable: stop the job.
    Fatal(String),
}

/// The text a model returned, tidied: a blank-page marker becomes an empty page.
fn clean(mut output: ProcessOutput) -> ProcessOutput {
    let trimmed = output.text.trim();
    if trimmed.eq_ignore_ascii_case("[BLANK PAGE]") {
        output.text = String::new();
    } else if trimmed.len() != output.text.len() {
        output.text = trimmed.to_string();
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
        for setup in &job.setups {
            let provider = self.provider(&setup.provider)?;
            if provider.needs_key() && self.keys.key(&setup.provider).is_none() {
                return Err(EngineError::MissingKey(setup.provider.to_string()));
            }
            provider
                .check_ready(&setup.model)
                .map_err(EngineError::NotReady)?;
        }
        let stop = control
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .stop
            .clone();
        self.store.set_job_status(job_id, JobStatus::Running)?;
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
            let pieces = if job.task == scriptorium_core::Task::Analysis {
                // An analysis reads the whole document at once.
                vec![PieceRef::Whole]
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
                            .process_piece(&job, &source, &setup, state, key, opened, spent, &stop)
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

            // Write outputs for every set-up whose pieces of this source are all done.
            let states = self.store.pieces(job_id)?;
            for setup in &job.setups {
                let mine: Vec<&PieceState> = states
                    .iter()
                    .filter(|p| p.setup_id == setup.id && p.source_id == source.id)
                    .collect();
                if !mine.is_empty() && mine.iter().all(|p| p.status.is_finished()) {
                    match self.write_outputs(&job, setup.id, source.id) {
                        Ok(files) => self.emit(ProgressEvent::SourceExported {
                            job_id,
                            setup_id: setup.id,
                            source_id: source.id,
                            files,
                        }),
                        Err(e) => {
                            failed = true;
                            self.emit(ProgressEvent::SourceFailed {
                                job_id,
                                source_id: source.id,
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(match stop_requested(&control) {
            Some(StopKind::Cancel) => JobStatus::Cancelled,
            Some(StopKind::Pause) => JobStatus::Paused,
            None if interrupted => JobStatus::Paused,
            None if failed => JobStatus::CompletedWithErrors,
            None => JobStatus::Completed,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_piece(
        &self,
        job: &Job,
        source: &SourceFile,
        setup: &Setup,
        mut state: PieceState,
        key: CacheKey,
        opened: Arc<OpenSource>,
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

        let provider = self.provider(&setup.provider)?;
        let api_key = if provider.needs_key() {
            self.keys
                .key(&setup.provider)
                .ok_or_else(|| EngineError::MissingKey(setup.provider.to_string()))?
        } else {
            ApiKey::new("")
        };
        // The provider's formats for the task, narrowed to those the model reads: a text-only
        // model behind a provider that takes PDFs and images is sent text alone.
        let accepted: Vec<&'static str> = {
            let catalog = self.catalog();
            let model = catalog.model(&setup.provider, &setup.model);
            provider
                .accepted_inputs(job.task)
                .iter()
                .copied()
                .filter(|mime| model.is_none_or(|m| m.inputs.iter().any(|i| i == mime)))
                .collect()
        };
        let stem = Path::new(&source.file_name)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "page".into());
        let prepared = {
            let (opened, piece) = (opened.clone(), state.piece.clone());
            tokio::task::spawn_blocking(move || opened.prepare(&piece, &accepted, &stem))
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?
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

        let started = Instant::now();
        let mut last_error: Option<ProviderError> = None;
        for attempt in 0..self.config.max_attempts {
            state.attempts += 1;
            let request = ProcessRequest {
                task: job.task,
                model: &setup.model,
                prompt: &setup.prompt.text,
                user_prompt: &setup.prompt.user_prompt,
                input: &input,
                languages: &job.settings.languages,
                options: &setup.options,
                analysis: setup.prompt.schema.as_ref(),
                stop,
            };
            match provider.process(&api_key, request).await {
                Ok(output) => {
                    let output = clean(output);
                    let catalog = self.catalog();
                    let cost = catalog
                        .model(&setup.provider, &setup.model)
                        .and_then(|m| catalog.actual_cost(m, &output.usage));
                    let result = StoredResult {
                        cache_key: key.0.clone(),
                        provider: setup.provider.clone(),
                        model: setup.model.clone(),
                        output,
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
        scriptorium_core::InputKind::Audio | scriptorium_core::InputKind::Video
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

enum StopKind {
    Pause,
    Cancel,
}

fn stop_requested(control: &Mutex<Control>) -> Option<StopKind> {
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

fn preview(text: &str) -> String {
    text.chars().take(400).collect()
}
