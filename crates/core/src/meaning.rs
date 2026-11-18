//! What the search by meaning and the map of a project's documents keep: each passage's text cut
//! into chunks, each chunk's vector from one embedding model, in a local index that is only ever
//! derived from the store; and, in the job database, which model each project uses, whether the
//! researcher agreed to send its text, and what each preparation cost.
//!
//! The vector index is a cache like the word search's [`SearchIndex`](crate::SearchIndex): every
//! chunk carries the stamp of the passage it was cut from and a hash of its text, and a chunk
//! whose passage changed is never searched nor mapped. Losing the file loses only what embedding
//! the text again costs. Vectors of two models, or of two lengths of one model, are never
//! compared: each lives in its own [`EmbeddingSpace`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{EmbeddingModel, JobId, PassageKey, ProjectId, ProviderId, StoreError, Timestamp};

/// One model at one length of vector: the vectors that can be compared with each other.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingSpace {
    pub provider: ProviderId,
    pub model: String,
    pub dimensions: u32,
}

impl EmbeddingSpace {
    /// The model's space at the first length it lists; none for a model listing no length.
    pub fn of(model: &EmbeddingModel) -> Option<Self> {
        Some(Self {
            provider: model.provider.clone(),
            model: model.id.clone(),
            dimensions: *model.dimensions.first()?,
        })
    }

    /// Whether `model` is this space's model.
    pub fn is_model(&self, model: &EmbeddingModel) -> bool {
        self.provider == model.provider && self.model == model.id
    }
}

/// One chunk of a passage's text with its vector.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkVector {
    pub key: PassageKey,
    /// Its place among the passage's chunks, from 0.
    pub chunk: u32,
    /// How many chunks the passage has; 0 for a passage with nothing worth embedding, kept as
    /// one row without a vector so that it is not taken for one never seen.
    pub of: u32,
    /// The passage's stamp when it was cut, as the word search stamps it.
    pub stamp: String,
    /// A hash of the passage's text and of how it was cut: a passage whose stamp changed but
    /// whose text did not (a page marked reviewed) keeps its vectors.
    pub content: String,
    /// Where the chunk stands in the passage's text, in characters: `[start, end)`.
    pub start: u32,
    pub end: u32,
    /// Of length 1; empty for a passage with nothing worth embedding.
    pub vector: Vec<f32>,
}

/// What the index holds of one chunk, without its vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStamp {
    pub key: PassageKey,
    pub chunk: u32,
    pub of: u32,
    pub stamp: String,
    pub content: String,
}

/// How much of a space the index holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpaceSize {
    pub space: EmbeddingSpace,
    /// Chunks with a vector.
    pub chunks: u32,
    /// What their vectors take on disk, about.
    #[ts(type = "number")]
    pub bytes: u64,
}

/// The vector index, a file of its own beside the job database and the word search's index:
/// losing it loses only what embedding again costs.
pub trait VectorIndex: Send + Sync {
    /// What the index holds of a job's passages in `space`.
    fn stamps(&self, space: &EmbeddingSpace, job: JobId) -> Result<Vec<ChunkStamp>, StoreError>;
    /// Writes chunks, all at once. A passage's chunks of another content are removed first, and
    /// those kept take the stamp written.
    fn put(&self, space: &EmbeddingSpace, chunks: &[ChunkVector]) -> Result<(), StoreError>;
    /// Gives a passage's chunks of `content` a new stamp: its text did not change.
    fn restamp(
        &self,
        space: &EmbeddingSpace,
        key: &PassageKey,
        content: &str,
        stamp: &str,
    ) -> Result<(), StoreError>;
    /// Removes passages' chunks in every space.
    fn remove_passages(&self, keys: &[PassageKey]) -> Result<(), StoreError>;
    /// Every chunk of `jobs` in `space` that has a vector.
    fn vectors(
        &self,
        space: &EmbeddingSpace,
        jobs: &[JobId],
    ) -> Result<Vec<ChunkVector>, StoreError>;
    /// The jobs that have chunks, so that those of deleted jobs can be forgotten.
    fn indexed_jobs(&self) -> Result<Vec<JobId>, StoreError>;
    fn forget_job(&self, job: JobId) -> Result<(), StoreError>;
    /// Each space holding chunks of `jobs`, with their size.
    fn spaces(&self, jobs: &[JobId]) -> Result<Vec<SpaceSize>, StoreError>;
    /// Removes the chunks of `jobs` in `space`.
    fn remove_space(&self, space: &EmbeddingSpace, jobs: &[JobId]) -> Result<(), StoreError>;
    /// A number that changes whenever the index is written, for what keeps its vectors in
    /// memory between two searches.
    fn generation(&self) -> u64;
}

/// The model a project's search by meaning and map use, as the researcher chose it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpaceChoice {
    pub project_id: ProjectId,
    pub space: EmbeddingSpace,
    pub chosen_at: Timestamp,
    /// When the researcher agreed to send the project's text, and then its questions, to the
    /// model's provider; none until they did, and always for a model run on this computer.
    pub consented_at: Option<Timestamp>,
    /// Questions embedded with it so far, and what they were billed.
    pub queries: u32,
    #[ts(type = "number")]
    pub query_tokens: u64,
    pub query_cost_usd: Option<f64>,
}

/// Where a preparation stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EmbeddingRunState {
    Running,
    /// Paused by the researcher: what was embedded is kept, the rest waits.
    Paused,
    /// Cancelled by the researcher: what was embedded is kept.
    Cancelled,
    /// The app closed or stopped while it ran: what was embedded is kept.
    Interrupted,
    Failed,
    Done,
}

/// One preparation of a project's text for the search by meaning, and what it cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EmbeddingRun {
    /// Given by the store; 0 before it is recorded.
    #[ts(type = "number")]
    pub id: i64,
    pub project_id: ProjectId,
    pub space: EmbeddingSpace,
    pub state: EmbeddingRunState,
    /// Chunks embedded so far.
    pub chunks: u32,
    /// Tokens billed so far, as the provider said.
    #[ts(type = "number")]
    pub tokens: u64,
    /// US dollars, from the model's price; none for a model run on this computer or a price
    /// the catalogue does not know.
    pub cost_usd: Option<f64>,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    /// Why it failed, as an id the interface translates.
    pub error: Option<String>,
}

/// What the job database keeps of the search by meaning: each project's model, consent and
/// preparations. Kept apart from [`JobStore`](crate::JobStore) so that the engine's jobs and this
/// work stay separate, and in the same database so the cost outlives the vector index.
pub trait EmbeddingLog: Send + Sync {
    /// The project's current model, the last one chosen.
    fn current_space(&self, project: ProjectId) -> Result<Option<SpaceChoice>, StoreError>;
    /// Makes `space` the project's model; its consent, if given before, still holds.
    fn choose_space(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        at: Timestamp,
    ) -> Result<SpaceChoice, StoreError>;
    /// Records the researcher's consent to send the project's text to `space`'s provider.
    fn consent(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        at: Timestamp,
    ) -> Result<(), StoreError>;
    /// Adds a question embedded with `space` to what it cost.
    fn add_query(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        tokens: u64,
        cost_usd: Option<f64>,
    ) -> Result<(), StoreError>;
    /// Records a new run and returns its id.
    fn start_run(&self, run: &EmbeddingRun) -> Result<i64, StoreError>;
    /// Writes a run's counts, state and end.
    fn update_run(&self, run: &EmbeddingRun) -> Result<(), StoreError>;
    /// A project's runs, newest first.
    fn embedding_runs(&self, project: ProjectId) -> Result<Vec<EmbeddingRun>, StoreError>;
    /// Marks the runs left running by a crash or a close as interrupted; returns how many.
    fn mark_running_runs_interrupted(&self) -> Result<u32, StoreError>;
}
