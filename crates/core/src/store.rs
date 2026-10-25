use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    CacheKey, Job, JobId, JobStatus, JobSummary, PieceState, SetupId, SourceId, SpeakerNames,
    StoredResult, Timestamp,
};

/// A user's edit of one piece, stored on top of the AI output, which is never overwritten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Correction {
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub index: u32,
    /// The corrected text, or `None` when the piece was only marked as reviewed.
    pub text: Option<String>,
    pub reviewed: bool,
    pub updated_at: Timestamp,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Job not found")]
    NotFound,
    #[error("Job database error: {0}")]
    Database(String),
    #[error("Stored data could not be read: {0}")]
    Corrupt(String),
}

/// Persistence used by the engine. Implemented by SQLite in production and in memory in tests.
pub trait JobStore: Send + Sync {
    fn create_job(&self, job: &Job) -> Result<(), StoreError>;
    fn job(&self, id: JobId) -> Result<Job, StoreError>;
    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError>;
    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError>;
    fn delete_job(&self, id: JobId) -> Result<(), StoreError>;

    /// Marks every running job as interrupted; called at startup after a crash or close.
    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError>;

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError>;
    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError>;

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError>;
    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError>;

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError>;
    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError>;

    /// Replaces the names given to one set-up's speakers in one recording.
    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError>;
    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError>;
}
