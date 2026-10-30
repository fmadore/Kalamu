//! One piece's call: taken from the cache, or prepared, sent to its provider with retries, and
//! stored; and what every path that stores an answer shares (the piece's new state, its cost).

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kalamu_core::{
    ApiKey, CacheKey, Job, JobId, PieceState, PieceStatus, PreparedInput, ProcessOutput,
    ProcessRequest, ProgressEvent, ProviderError, SourceFile, Step, StopSignal, StoredResult, Task,
    Timestamp,
};
use kalamu_media::OpenSource;

use crate::{Engine, EngineError};

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
        if let Some(cached) = self.store.cached_result(&key)? {
            state.status = PieceStatus::Cached;
            state.error = None;
            self.save(&mut state, Some(preview(&cached.output.text)))?;
            return Ok(Outcome::Finished);
        }

        state.status = PieceStatus::Running;
        self.save(&mut state, None)?;

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
                self.save(&mut state, None)?;
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
                    self.save(&mut state, Some(preview(&result.output.text)))?;
                    self.add_cost(job.id, &spent, cost);
                    return Ok(Outcome::Finished);
                }
                Err(ProviderError::Interrupted) => {
                    state.status = PieceStatus::Pending;
                    state.error = None;
                    self.save(&mut state, None)?;
                    return Ok(Outcome::Interrupted);
                }
                Err(err) if err.is_fatal_for_job() => {
                    state.status = PieceStatus::Failed;
                    state.error = Some(err.to_string());
                    self.save(&mut state, None)?;
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
                    // Paused or cancelled meanwhile: no further attempt, which would be paid
                    // for; the piece waits for the job to resume.
                    if wait_unless_stopped(delay, stop).await {
                        state.status = PieceStatus::Pending;
                        state.error = None;
                        self.save(&mut state, None)?;
                        return Ok(Outcome::Interrupted);
                    }
                }
                Err(err) => {
                    last_error = Some(err);
                    break;
                }
            }
        }
        state.status = PieceStatus::Failed;
        state.error = last_error.map(|e| e.to_string());
        self.save(&mut state, None)?;
        Ok(Outcome::Failed)
    }

    /// Stores a piece's new state and reports it.
    pub(crate) fn save(
        &self,
        state: &mut PieceState,
        preview: Option<String>,
    ) -> Result<(), EngineError> {
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

    /// Adds an answer's cost to what the job has spent, and reports the new total.
    pub(crate) fn add_cost(&self, job_id: JobId, spent: &Mutex<f64>, cost: Option<f64>) {
        let Some(cost) = cost else { return };
        let total = {
            let mut spent = spent.lock().unwrap_or_else(|p| p.into_inner());
            *spent += cost;
            *spent
        };
        self.emit(ProgressEvent::CostUpdated {
            job_id,
            spent_usd: total,
        });
    }
}

/// Waits `delay`, ending early if the job is paused or cancelled; true when it was.
async fn wait_unless_stopped(delay: Duration, stop: &StopSignal) -> bool {
    let step = Duration::from_millis(100);
    let mut left = delay;
    while !left.is_zero() && !stop.is_stopped() {
        let nap = left.min(step);
        tokio::time::sleep(nap).await;
        left -= nap;
    }
    stop.is_stopped()
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
