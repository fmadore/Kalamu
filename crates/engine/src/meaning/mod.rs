//! The search by meaning and the map, where they call a model: preparing a project's text
//! (embedding its chunks, batch by batch, each batch's vectors written as it returns) and
//! embedding each question. Both go through here, so every call is consented to, keyed,
//! retried, stoppable and recorded with its cost; nothing is embedded without the researcher
//! starting it.
//!
//! The embedders and the models offered are given as values by the app, so the engine never
//! matches on a provider's or a model's name.

mod error;
mod run;

pub use run::Ready;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use kalamu_core::{
    DocumentId, EmbedLimits, Embedder, EmbeddingLog, EmbeddingModel, EmbeddingRun, EmbeddingSpace,
    ProjectId, SpaceChoice, SpaceSize, StopSignal, Timestamp, VectorIndex,
};
use kalamu_search::{MapView, MeaningRequest, MeaningView, Neighbour, PreparationCounts, Search};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use ts_rs::TS;

use crate::Engine;
use crate::control::lock;
pub use error::MeaningError;

/// How far a project's preparation has got, sent after each batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningProgress {
    pub project_id: ProjectId,
    pub run: EmbeddingRun,
    /// Chunks embedded in this run, of `total` it set out to embed.
    pub done: u32,
    pub total: u32,
}

/// What the search by meaning's set-up shows for a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MeaningStatus {
    /// The models offered, as the catalogue lists them.
    pub models: Vec<EmbeddingModel>,
    /// The project's model, the last chosen; none before.
    pub choice: Option<SpaceChoice>,
    /// That model, when it is still offered.
    pub model: Option<EmbeddingModel>,
    /// Why the model cannot run now (a local model not downloaded, no key saved), as an id the
    /// interface translates; none when it can.
    pub blocked: Option<String>,
    /// What preparing would embed now, nothing sent yet; none without a model.
    pub counts: Option<PreparationCounts>,
    /// Its cost estimated from the model's price; none for a local model or an unknown price.
    pub estimated_cost_usd: Option<f64>,
    /// The run in progress.
    pub running: Option<MeaningProgress>,
    /// The project's runs, newest first, every model's.
    pub runs: Vec<EmbeddingRun>,
    /// When the project's text was last prepared for the model to the end.
    pub prepared_at: Option<Timestamp>,
    /// What the index holds for the project, model by model.
    pub spaces: Vec<SpaceSize>,
    /// Everything recorded as spent on the project's search by meaning: runs and questions.
    pub spent_usd: Option<f64>,
}

struct Running {
    stop: StopSignal,
    /// What a stop asked for: paused or cancelled.
    asked: Arc<Mutex<Option<kalamu_core::EmbeddingRunState>>>,
    progress: Arc<Mutex<Option<MeaningProgress>>>,
}

pub struct Meaning {
    pub(crate) engine: Arc<Engine>,
    pub(crate) search: Arc<Search>,
    pub(crate) vectors: Arc<dyn VectorIndex>,
    pub(crate) log: Arc<dyn EmbeddingLog>,
    pub(crate) embedders: Vec<Arc<dyn Embedder>>,
    pub(crate) models: Vec<EmbeddingModel>,
    running: Mutex<HashMap<ProjectId, Running>>,
    events: broadcast::Sender<MeaningProgress>,
}

/// Dollars for `tokens` at `price` per million.
pub(crate) fn cost(tokens: u64, price: Option<f64>) -> Option<f64> {
    price.map(|p| tokens as f64 * p / 1_000_000.0)
}

impl Meaning {
    pub fn new(
        engine: Arc<Engine>,
        search: Arc<Search>,
        vectors: Arc<dyn VectorIndex>,
        log: Arc<dyn EmbeddingLog>,
        embedders: Vec<Arc<dyn Embedder>>,
        models: Vec<EmbeddingModel>,
    ) -> Arc<Self> {
        let (events, _) = broadcast::channel(256);
        Arc::new(Self {
            engine,
            search,
            vectors,
            log,
            embedders,
            models,
            running: Mutex::new(HashMap::new()),
            events,
        })
    }

    /// Call once at startup: runs left running by a crash or a close become interrupted.
    pub fn recover(&self) -> Result<u32, MeaningError> {
        Ok(self.log.mark_running_runs_interrupted()?)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MeaningProgress> {
        self.events.subscribe()
    }

    pub(crate) fn emit(&self, progress: MeaningProgress) {
        if let Some(r) = lock(&self.running).get(&progress.project_id) {
            *lock(&r.progress) = Some(progress.clone());
        }
        let _ = self.events.send(progress);
    }

    /// The models offered whose embedder is there.
    pub fn models(&self) -> Vec<EmbeddingModel> {
        self.models
            .iter()
            .filter(|m| self.embedders.iter().any(|e| e.id() == m.provider))
            .cloned()
            .collect()
    }

    fn model(&self, space: &EmbeddingSpace) -> Option<&EmbeddingModel> {
        self.models.iter().find(|m| space.is_model(m))
    }

    fn embedder(&self, model: &EmbeddingModel) -> Result<Arc<dyn Embedder>, MeaningError> {
        self.embedders
            .iter()
            .find(|e| e.id() == model.provider)
            .cloned()
            .ok_or_else(|| MeaningError::UnknownModel(model.name.clone()))
    }

    /// The longest chunk sent to `model`: its own limit and its embedder's, whichever is lower.
    fn max_chars(&self, model: &EmbeddingModel) -> usize {
        let limits = self
            .embedder(model)
            .map(|e| e.limits(&model.id))
            .unwrap_or(EmbedLimits {
                texts: 1,
                chars: model.max_chars as usize,
            });
        (model.max_chars as usize).min(limits.chars).max(1)
    }

    /// The longest chunk for a space whose model may no longer be offered: chunks then keep
    /// the length they were cut at, so they are not taken for changed.
    fn max_chars_of(&self, space: &EmbeddingSpace) -> usize {
        self.model(space)
            .map_or(kalamu_search::meaning::chunks::TARGET, |m| {
                self.max_chars(m)
            })
    }

    /// The project's model, or [`MeaningError::NoModel`].
    pub(crate) fn current(&self, project: ProjectId) -> Result<SpaceChoice, MeaningError> {
        self.log
            .current_space(project)?
            .ok_or(MeaningError::NoModel)
    }

    /// Why `model` cannot run now, as an id the interface translates; none when it can.
    fn blocked(&self, model: &EmbeddingModel) -> Option<String> {
        let embedder = self.embedder(model).ok()?;
        if let Err(e) = embedder.check_ready(&model.id) {
            return Some(e.code().to_string());
        }
        if embedder.needs_key() && self.engine.keys.key(&model.provider).is_none() {
            return Some("provider.missing_key".into());
        }
        None
    }

    /// Makes the model the project's. Nothing is sent: preparing is the researcher's act.
    pub fn choose(
        &self,
        project: ProjectId,
        provider: &kalamu_core::ProviderId,
        model: &str,
    ) -> Result<SpaceChoice, MeaningError> {
        let found = self
            .models()
            .into_iter()
            .find(|m| &m.provider == provider && m.id == model)
            .ok_or_else(|| MeaningError::UnknownModel(model.to_string()))?;
        let Some(space) = EmbeddingSpace::of(&found) else {
            return Err(MeaningError::UnknownModel(found.name));
        };
        Ok(self.log.choose_space(project, &space, Timestamp::now())?)
    }

    /// Records the researcher's consent to send the project's text, and then its questions, to
    /// the provider of its model.
    pub fn consent(&self, project: ProjectId) -> Result<(), MeaningError> {
        let choice = self.current(project)?;
        Ok(self.log.consent(project, &choice.space, Timestamp::now())?)
    }

    /// What the set-up shows: the model, what preparing would embed and cost, the run in
    /// progress, the runs and what the index holds.
    pub fn status(&self, project: ProjectId) -> Result<MeaningStatus, MeaningError> {
        let choice = self.log.current_space(project)?;
        let model = choice.as_ref().and_then(|c| self.model(&c.space)).cloned();
        let counts = match &choice {
            Some(c) => Some(
                self.search
                    .meaning_preparation(project, &c.space, self.max_chars_of(&c.space), &|| false)?
                    .counts,
            ),
            None => None,
        };
        let estimated_cost_usd = match (&model, &counts) {
            (Some(m), Some(c)) if !m.local => cost(c.tokens, m.input_price),
            _ => None,
        };
        let runs = self.log.embedding_runs(project)?;
        let prepared_at = choice.as_ref().and_then(|c| {
            runs.iter()
                .find(|r| r.space == c.space && r.state == kalamu_core::EmbeddingRunState::Done)
                .and_then(|r| r.ended_at)
        });
        let spent: Vec<f64> = runs
            .iter()
            .filter_map(|r| r.cost_usd)
            .chain(choice.as_ref().and_then(|c| c.query_cost_usd))
            .collect();
        let running = lock(&self.running)
            .get(&project)
            .and_then(|r| lock(&r.progress).clone());
        Ok(MeaningStatus {
            models: self.models(),
            blocked: model.as_ref().and_then(|m| self.blocked(m)),
            choice,
            model,
            counts,
            estimated_cost_usd,
            running,
            runs,
            prepared_at,
            spaces: self.search.meaning_spaces(project)?,
            spent_usd: (!spent.is_empty()).then(|| spent.iter().sum()),
        })
    }

    /// Pauses the project's preparation after the batch in flight; what was embedded is kept.
    pub fn pause(&self, project: ProjectId) {
        self.stop(project, kalamu_core::EmbeddingRunState::Paused);
    }

    /// Cancels the project's preparation after the batch in flight; what was embedded is kept.
    pub fn cancel(&self, project: ProjectId) {
        self.stop(project, kalamu_core::EmbeddingRunState::Cancelled);
    }

    fn stop(&self, project: ProjectId, why: kalamu_core::EmbeddingRunState) {
        if let Some(r) = lock(&self.running).get(&project) {
            *lock(&r.asked) = Some(why);
            r.stop.stop();
        }
    }

    /// Removes the vectors `space`'s model made of the project's documents.
    pub fn remove_space(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
    ) -> Result<(), MeaningError> {
        Ok(self.search.remove_meaning_space(project, space)?)
    }

    /// The map of the project's documents with its model.
    pub fn map(&self, project: ProjectId) -> Result<MapView, MeaningError> {
        let choice = self.current(project)?;
        Ok(self.search.meaning_map(
            project,
            &choice.space,
            self.max_chars_of(&choice.space),
            &|| false,
        )?)
    }

    /// The documents most like `document`, with the project's model.
    pub fn neighbours(
        &self,
        project: ProjectId,
        document: DocumentId,
    ) -> Result<Vec<Neighbour>, MeaningError> {
        let choice = self.current(project)?;
        Ok(self.search.meaning_neighbours(
            project,
            &choice.space,
            self.max_chars_of(&choice.space),
            document,
            &|| false,
        )?)
    }

    /// Searches the project by meaning: the question embedded once by the project's model,
    /// consented to for a cloud model, its cost recorded; then compared with every chunk.
    pub async fn search(
        self: &Arc<Self>,
        project: ProjectId,
        request: MeaningRequest,
    ) -> Result<MeaningView, MeaningError> {
        let query = request.query.trim().to_string();
        if query.is_empty() {
            return Err(MeaningError::EmptyQuery);
        }
        let choice = self.current(project)?;
        let model = self
            .model(&choice.space)
            .cloned()
            .ok_or_else(|| MeaningError::UnknownModel(choice.space.model.clone()))?;
        let max_chars = self.max_chars(&model);
        let text: String = query.chars().take(max_chars).collect();
        let vector = self.embed_query(&choice, &model, text).await?;
        let search = self.search.clone();
        let space = choice.space.clone();
        tokio::task::spawn_blocking(move || {
            search.meaning_search(project, &request, &space, max_chars, &vector, &|| false)
        })
        .await
        .map_err(|e| MeaningError::Provider(kalamu_core::ProviderError::Other(e.to_string())))?
        .map_err(MeaningError::from)
    }
}

#[cfg(test)]
mod tests;
