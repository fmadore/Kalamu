//! One piece's call: taken from the cache, or prepared, sent to its provider with retries, and
//! stored; and what every path that stores an answer shares (the piece's new state, its cost).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kalamu_core::{
    ApiKey, CacheKey, Charge, Job, JobId, PieceState, PieceStatus, PreparedInput, ProcessOutput,
    ProcessRequest, ProgressEvent, Provider, ProviderError, ProviderId, RequestSetting,
    SetupOptions, SourceFile, Step, StopSignal, StoredResult, Task, Timestamp, Tuning, Usage,
};
use kalamu_media::{MediaError, OpenSource};

use crate::control::lock;
use crate::{Engine, EngineError};

/// What a call reads: a piece still to cut from its source, or an input already made (a table
/// of contents' pages, gathered for its second step).
pub(crate) enum Input {
    Open(Arc<OpenSource>),
    Ready(PreparedInput),
}

/// How a call ended, after its retries.
pub(crate) enum Called {
    Answer {
        output: ProcessOutput,
        /// What the request carried, for provenance.
        sent: Vec<RequestSetting>,
        duration_ms: u64,
    },
    /// Stopped by a pause or cancel.
    Interrupted,
    /// The key or the account is unusable.
    Fatal(ProviderError),
    Failed(Option<ProviderError>),
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
    /// What a request to `model` asks beyond the prompt: the reasoning level chosen for the
    /// set-up (the suggestion when none was, the nearest the model has when it lacks it) and
    /// the model's whole output limit, from the catalogue, with the abilities that shape its
    /// request. A model the catalogue lacks is sent the level chosen and its provider's own
    /// limit, and is taken to have none of those abilities.
    pub(crate) fn tuning(
        &self,
        task: Task,
        provider: &ProviderId,
        model: &str,
        chosen: Option<&str>,
    ) -> Tuning {
        self.current_catalog()
            .model(provider, model)
            .map(|m| m.tuning(task, chosen))
            .unwrap_or_else(|| Tuning {
                reasoning: chosen.map(str::to_string),
                ..Tuning::default()
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
        let catalog = self.current_catalog();
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
                let stem = crate::source_stem(&source.file_name, "page");
                let piece = state.piece.clone();
                tokio::task::spawn_blocking(move || opened.prepare(&piece, &accepted, &stem))
                    .await
                    .map_err(|e| EngineError::Output(e.to_string()))?
            }
        };
        let input = match prepared {
            Ok(input) => input,
            // Paused or cancelled while FFmpeg cut the piece: it is read on resuming.
            Err(MediaError::Stopped) => {
                state.status = PieceStatus::Pending;
                state.error = None;
                self.save(&mut state, None)?;
                return Ok(Outcome::Interrupted);
            }
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
        let called = self
            .call(
                job,
                step,
                &provider,
                &api_key,
                &mut state,
                &input,
                &step.prompt.user_prompt,
                &options,
                &spent,
                stop,
            )
            .await?;
        match called {
            Called::Answer {
                output,
                sent,
                duration_ms,
            } => {
                let output = finish(job.task, output);
                let cost = self.store_answer(&key, step, output.clone(), sent, duration_ms)?;
                state.status = PieceStatus::Done;
                state.error = None;
                self.save(&mut state, Some(preview(&output.text)))?;
                self.add_cost(job.id, &spent, cost);
                Ok(Outcome::Finished)
            }
            Called::Interrupted => {
                state.status = PieceStatus::Pending;
                state.error = None;
                self.save(&mut state, None)?;
                Ok(Outcome::Interrupted)
            }
            Called::Fatal(err) => {
                state.status = PieceStatus::Failed;
                state.error = Some(err.to_string());
                self.save(&mut state, None)?;
                Ok(Outcome::Fatal(err.to_string()))
            }
            Called::Failed(err) => {
                state.status = PieceStatus::Failed;
                state.error = err.map(|e| e.to_string());
                self.save(&mut state, None)?;
                Ok(Outcome::Failed)
            }
        }
    }

    /// Stores an answer under its key with what it cost, and returns the cost.
    pub(crate) fn store_answer(
        &self,
        key: &CacheKey,
        step: Step<'_>,
        output: ProcessOutput,
        sent: Vec<RequestSetting>,
        duration_ms: u64,
    ) -> Result<Option<f64>, EngineError> {
        let catalog = self.current_catalog();
        let cost = catalog
            .model(step.provider, step.model)
            .and_then(|m| catalog.actual_cost(m, &output.usage));
        self.store.put_result(&StoredResult {
            cache_key: key.0.clone(),
            provider: step.provider.clone(),
            model: step.model.to_string(),
            output,
            request: sent,
            cost_usd: cost,
            duration_ms,
            created_at: Timestamp::now(),
        })?;
        Ok(cost)
    }

    /// One call with its retries: a rate limit waits as long as asked, a passing failure is
    /// tried again after a growing delay, a pause or cancel ends the wait. Answers billed but
    /// refused are charged to `state`'s piece, whose attempts are counted.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn call(
        &self,
        job: &Job,
        step: Step<'_>,
        provider: &Arc<dyn Provider>,
        api_key: &ApiKey,
        state: &mut PieceState,
        input: &PreparedInput,
        user_prompt: &str,
        options: &SetupOptions,
        spent: &Arc<Mutex<f64>>,
        stop: &StopSignal,
    ) -> Result<Called, EngineError> {
        let tuning = self.tuning(
            job.task,
            step.provider,
            step.model,
            options.reasoning.as_deref(),
        );
        let started = Instant::now();
        let mut last_error: Option<ProviderError> = None;
        for attempt in 0..self.config.max_attempts {
            state.attempts += 1;
            let request = ProcessRequest {
                task: job.task,
                model: step.model,
                prompt: &step.prompt.text,
                user_prompt,
                input,
                languages: &job.settings.languages,
                options,
                tuning: &tuning,
                analysis: step.prompt.schema.as_ref(),
                stop,
            };
            let sent = provider.sent_settings(&request, false);
            let answer = provider.process(api_key, request).await;
            if let Err(err) = &answer {
                self.charge(state, step.provider, step.model, err, spent)?;
            }
            match answer {
                Ok(output) => {
                    return Ok(Called::Answer {
                        output,
                        sent,
                        duration_ms: started.elapsed().as_millis() as u64,
                    });
                }
                Err(ProviderError::Interrupted) => return Ok(Called::Interrupted),
                Err(err) if err.is_fatal_for_job() => return Ok(Called::Fatal(err)),
                Err(err) if err.is_retryable() && attempt + 1 < self.config.max_attempts => {
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
                        return Ok(Called::Interrupted);
                    }
                }
                Err(err) => {
                    last_error = Some(err);
                    break;
                }
            }
        }
        Ok(Called::Failed(last_error))
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

    /// Records what a failed call was billed for (an answer refused for breaking its fields),
    /// and adds it to what the job has spent, so the cost shown is the cost charged. Failures
    /// the provider did not bill record nothing.
    pub(crate) fn charge(
        &self,
        state: &PieceState,
        provider: &ProviderId,
        model: &str,
        err: &ProviderError,
        spent: &Mutex<f64>,
    ) -> Result<(), EngineError> {
        let Some(usage) = err.billed() else {
            return Ok(());
        };
        self.bill(state, provider, model, usage, spent)
    }

    /// Records a call billed though nothing was kept from it, and adds it to what the job has
    /// spent.
    pub(crate) fn bill(
        &self,
        state: &PieceState,
        provider: &ProviderId,
        model: &str,
        usage: &Usage,
        spent: &Mutex<f64>,
    ) -> Result<(), EngineError> {
        let catalog = self.current_catalog();
        let cost = catalog
            .model(provider, model)
            .and_then(|m| catalog.actual_cost(m, usage));
        self.store.add_charge(&Charge {
            job_id: state.job_id,
            setup_id: state.setup_id,
            source_id: state.source_id,
            index: state.index,
            usage: usage.clone(),
            cost_usd: cost,
            created_at: Timestamp::now(),
        })?;
        self.add_cost(state.job_id, spent, cost);
        Ok(())
    }

    /// Adds an answer's cost to what the job has spent, and reports the new total.
    pub(crate) fn add_cost(&self, job_id: JobId, spent: &Mutex<f64>, cost: Option<f64>) {
        let Some(cost) = cost else { return };
        let total = {
            let mut spent = lock(spent);
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
pub(crate) async fn wait_unless_stopped(delay: Duration, stop: &StopSignal) -> bool {
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
