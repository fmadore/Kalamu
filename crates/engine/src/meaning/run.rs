//! A preparation run and a question's embedding: the calls themselves, retried as the engine
//! retries a piece, each batch's vectors written as it returns and its cost recorded with the
//! run, so a run stopped by a pause, a quit or a crash goes on later without embedding anything
//! twice.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use kalamu_core::{
    ApiKey, ChunkVector, EmbedRequest, EmbedRole, Embedded, Embedder, EmbeddingModel, EmbeddingRun,
    EmbeddingRunState, ProjectId, ProviderError, SpaceChoice, StopSignal, Timestamp,
};

use super::{Meaning, MeaningError, MeaningProgress, Running, cost};
use crate::control::lock;
use crate::piece::wait_unless_stopped;

/// What a run needs, checked before it starts: the model, its embedder and key, the
/// researcher's consent.
pub struct Ready {
    project: ProjectId,
    choice: SpaceChoice,
    model: EmbeddingModel,
    embedder: Arc<dyn Embedder>,
    key: Option<ApiKey>,
    max_chars: usize,
}

impl Ready {
    pub fn project(&self) -> ProjectId {
        self.project
    }
}

/// How a call ended.
enum Called {
    Answer(Embedded),
    Stopped,
    Failed(ProviderError),
}

impl Meaning {
    /// What calling the project's model needs, or why it cannot be called: no model chosen,
    /// one no longer offered, a local one not downloaded, no key, or no consent for a cloud one.
    fn ready(&self, project: ProjectId) -> Result<Ready, MeaningError> {
        let choice = self.current(project)?;
        let model = self
            .model(&choice.space)
            .cloned()
            .ok_or_else(|| MeaningError::UnknownModel(choice.space.model.clone()))?;
        let embedder = self.embedder(&model)?;
        embedder
            .check_ready(&model.id)
            .map_err(MeaningError::NotReady)?;
        let key = if embedder.needs_key() {
            Some(
                self.engine
                    .keys
                    .key(&model.provider)
                    .ok_or_else(|| MeaningError::MissingKey(model.provider.to_string()))?,
            )
        } else {
            None
        };
        if !model.local && choice.consented_at.is_none() {
            return Err(MeaningError::NoConsent(model.provider.to_string()));
        }
        let max_chars = self.max_chars(&model);
        Ok(Ready {
            project,
            choice,
            model,
            embedder,
            key,
            max_chars,
        })
    }

    /// Checks that the project's text can be prepared now, and claims the project so a second
    /// run does not start beside it. The run itself is [`Meaning::run`].
    pub fn check(&self, project: ProjectId) -> Result<Ready, MeaningError> {
        let ready = self.ready(project)?;
        let mut running = lock(&self.running);
        if running.contains_key(&project) {
            return Err(MeaningError::AlreadyRunning);
        }
        running.insert(
            project,
            Running {
                stop: StopSignal::default(),
                asked: Arc::new(Mutex::new(None)),
                progress: Arc::new(Mutex::new(None)),
            },
        );
        Ok(ready)
    }

    /// Embeds what the project's text needs with its model, batch by batch, until it is done,
    /// paused, cancelled or fails. Returns the run as recorded.
    pub async fn run(self: Arc<Self>, ready: Ready) -> Result<EmbeddingRun, MeaningError> {
        let project = ready.project;
        let outcome = self.run_claimed(ready).await;
        lock(&self.running).remove(&project);
        outcome
    }

    async fn run_claimed(&self, ready: Ready) -> Result<EmbeddingRun, MeaningError> {
        let (stop, asked) = match lock(&self.running).get(&ready.project) {
            Some(r) => (r.stop.clone(), r.asked.clone()),
            None => (StopSignal::default(), Arc::new(Mutex::new(None))),
        };
        let space = ready.choice.space.clone();
        let search = self.search.clone();
        let (project, max_chars) = (ready.project, ready.max_chars);
        let planning = stop.clone();
        let preparation = tokio::task::spawn_blocking({
            let space = space.clone();
            move || {
                search.meaning_preparation(project, &space, max_chars, &|| planning.is_stopped())
            }
        })
        .await
        .map_err(|e| MeaningError::Provider(ProviderError::Other(e.to_string())))??;
        self.vectors.put(&space, &preparation.empty)?;

        let mut run = EmbeddingRun {
            id: 0,
            project_id: project,
            space: space.clone(),
            state: EmbeddingRunState::Running,
            chunks: 0,
            tokens: 0,
            cost_usd: (!ready.model.local && ready.model.input_price.is_some()).then_some(0.0),
            started_at: Timestamp::now(),
            ended_at: None,
            error: None,
        };
        run.id = self.log.start_run(&run)?;
        let total = preparation.pending.len() as u32;
        let tell = |run: &EmbeddingRun| {
            self.emit(MeaningProgress {
                project_id: project,
                run: run.clone(),
                done: run.chunks,
                total,
            });
        };
        tell(&run);

        let limits = ready.embedder.limits(&ready.model.id);
        let per_call = limits.texts.max(1);
        let mut failure = None;
        for batch in preparation.pending.chunks(per_call) {
            if stop.is_stopped() {
                break;
            }
            let texts: Vec<String> = batch.iter().map(|c| c.text.clone()).collect();
            match self.call(&ready, EmbedRole::Passage, &texts, &stop).await {
                Called::Answer(answer) => {
                    let dimensions = space.dimensions as usize;
                    if answer.vectors.len() != batch.len()
                        || answer.vectors.iter().any(|v| v.len() != dimensions)
                    {
                        failure = Some(MeaningError::BadAnswer);
                        break;
                    }
                    let chunks: Vec<ChunkVector> = batch
                        .iter()
                        .zip(answer.vectors)
                        .map(|(c, vector)| ChunkVector {
                            key: c.key,
                            chunk: c.chunk,
                            of: c.of,
                            stamp: c.stamp.clone(),
                            content: c.content.clone(),
                            start: c.start,
                            end: c.end,
                            vector,
                        })
                        .collect();
                    // Written before anything else, so a crash after a call loses only it.
                    self.vectors.put(&space, &chunks)?;
                    run.chunks += batch.len() as u32;
                    run.tokens += answer.input_tokens;
                    if !ready.model.local {
                        run.cost_usd = cost(run.tokens, ready.model.input_price);
                    }
                    self.log.update_run(&run)?;
                    tell(&run);
                }
                Called::Stopped => break,
                Called::Failed(e) => {
                    failure = Some(MeaningError::Provider(e));
                    break;
                }
            }
        }
        run.ended_at = Some(Timestamp::now());
        run.state = match &failure {
            Some(e) => {
                run.error = Some(e.code().to_string());
                EmbeddingRunState::Failed
            }
            None if stop.is_stopped() => (*lock(&asked)).unwrap_or(EmbeddingRunState::Paused),
            None => EmbeddingRunState::Done,
        };
        self.log.update_run(&run)?;
        tell(&run);
        match failure {
            Some(e) => Err(e),
            None => Ok(run),
        }
    }

    /// One call, tried again after a rate limit or a passing failure as the engine tries a
    /// piece again, the wait cut short by a stop.
    async fn call(
        &self,
        ready: &Ready,
        role: EmbedRole,
        texts: &[String],
        stop: &StopSignal,
    ) -> Called {
        let config = &self.engine.config;
        let mut last = ProviderError::Other("no attempt".into());
        for attempt in 0..config.max_attempts.max(1) {
            let request = EmbedRequest {
                model: &ready.model.id,
                role,
                texts,
                dimensions: Some(ready.choice.space.dimensions),
                stop,
            };
            match ready.embedder.embed(ready.key.as_ref(), request).await {
                Ok(answer) => return Called::Answer(answer),
                Err(ProviderError::Interrupted) => return Called::Stopped,
                Err(e) if e.is_retryable() && attempt + 1 < config.max_attempts => {
                    let delay = match &e {
                        ProviderError::RateLimited {
                            retry_after: Some(d),
                            ..
                        } => *d + Duration::from_secs(1),
                        _ => config.base_delay * 2u32.saturating_pow(attempt).min(32),
                    };
                    tracing::info!(?delay, error = %e, "embedding retried");
                    last = e;
                    if wait_unless_stopped(delay, stop).await {
                        return Called::Stopped;
                    }
                }
                Err(e) => return Called::Failed(e),
            }
        }
        Called::Failed(last)
    }

    /// The question's vector, with the project's model: refused without consent for a cloud
    /// model, its tokens added to what the project's questions cost.
    pub(crate) async fn embed_query(
        &self,
        choice: &SpaceChoice,
        model: &EmbeddingModel,
        text: String,
    ) -> Result<Vec<f32>, MeaningError> {
        let ready = self.ready(choice.project_id)?;
        let stop = StopSignal::default();
        let answer = match self
            .call(&ready, EmbedRole::Query, std::slice::from_ref(&text), &stop)
            .await
        {
            Called::Answer(a) => a,
            Called::Stopped => return Err(MeaningError::Provider(ProviderError::Interrupted)),
            Called::Failed(e) => return Err(MeaningError::Provider(e)),
        };
        let price = (!model.local).then_some(model.input_price).flatten();
        self.log.add_query(
            choice.project_id,
            &choice.space,
            answer.input_tokens,
            cost(answer.input_tokens, price),
        )?;
        let vector = answer
            .vectors
            .into_iter()
            .next()
            .filter(|v| v.len() == choice.space.dimensions as usize)
            .ok_or(MeaningError::BadAnswer)?;
        Ok(vector)
    }
}
