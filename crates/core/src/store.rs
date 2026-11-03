use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    BatchRecord, CacheKey, Charge, Job, JobId, JobStatus, JobSummary, PieceState, SetupId,
    SourceId, SpeakerNames, StoredResult, Timestamp,
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
    /// An analysis's corrected answers, replacing the model's when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fields: Option<Vec<crate::FieldAnswer>>,
    pub reviewed: bool,
    pub updated_at: Timestamp,
}

impl Correction {
    /// The correction made to `piece`, among its job's corrections.
    pub fn of<'a>(corrections: &'a [Correction], piece: &PieceState) -> Option<&'a Correction> {
        corrections.iter().find(|c| {
            c.setup_id == piece.setup_id && c.source_id == piece.source_id && c.index == piece.index
        })
    }
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
    /// Gives a job a new name; nothing else about it changes.
    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError>;
    fn delete_job(&self, id: JobId) -> Result<(), StoreError>;

    /// Marks every running job as interrupted; called at startup after a crash or close.
    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError>;

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError>;
    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError>;

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError>;
    /// The answer stored for a finished piece; none for one not finished.
    fn finished_result(&self, piece: &PieceState) -> Result<Option<StoredResult>, StoreError> {
        match &piece.cache_key {
            Some(key) if piece.status.is_finished() => self.cached_result(&CacheKey(key.clone())),
            _ => Ok(None),
        }
    }
    /// A finished analysis piece's answers as the researcher left them: the corrected ones,
    /// else the model's.
    fn answers(
        &self,
        piece: &PieceState,
        corrections: &[Correction],
    ) -> Result<Option<Vec<crate::FieldAnswer>>, StoreError> {
        if let Some(fields) = Correction::of(corrections, piece).and_then(|c| c.fields.clone()) {
            return Ok(Some(fields));
        }
        Ok(self.finished_result(piece)?.map(|r| r.output.fields))
    }
    /// What a job's own calls cost, in US dollars: the results its pieces stored, not those
    /// reused from the cache, and the answers billed but refused ([`Charge`]).
    fn spent(&self, job: JobId) -> Result<f64, StoreError>;
    /// Records a call billed without a result to keep.
    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError>;
    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError>;
    /// Stores a result unless one is already stored under its key. The first answer is kept,
    /// never replaced by a later one (two identical set-ups, a batch collected after a fast
    /// run), so the researcher's corrections stay over the text they corrected.
    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError>;

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError>;
    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError>;

    /// Replaces the names given to one set-up's speakers in one recording.
    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError>;
    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError>;

    /// Inserts or replaces a batch, by its own id.
    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError>;
    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError>;

    /// Keeps a snapshot of corrected text; one already kept under its id is left as it is.
    fn put_snapshot(&self, snapshot: &crate::Snapshot) -> Result<(), StoreError>;
    fn snapshot(&self, id: &str) -> Result<crate::Snapshot, StoreError>;

    /// Inserts or replaces a document unit, by its id; deleting its job deletes it.
    fn put_unit(&self, unit: &crate::DocumentUnit) -> Result<(), StoreError>;
    fn unit(&self, id: crate::UnitId) -> Result<crate::DocumentUnit, StoreError>;
    /// A job's units, in their order.
    fn units(&self, job: JobId) -> Result<Vec<crate::DocumentUnit>, StoreError>;
    fn delete_unit(&self, id: crate::UnitId) -> Result<(), StoreError>;
    /// Puts these units and deletes those at once, so a split or a join is never half made.
    fn replace_units(
        &self,
        put: &[crate::DocumentUnit],
        delete: &[crate::UnitId],
    ) -> Result<(), StoreError>;

    /// Keeps a workflow run as started: its frozen steps, sources and each step's progress.
    fn create_run(&self, run: &crate::WorkflowRun) -> Result<(), StoreError>;
    fn run(&self, id: crate::RunId) -> Result<crate::WorkflowRun, StoreError>;
    /// Every run, newest first.
    fn runs(&self) -> Result<Vec<crate::WorkflowRun>, StoreError>;
    /// Replaces what has happened to one step of a run.
    fn set_step_progress(
        &self,
        run: crate::RunId,
        index: u32,
        progress: &crate::StepProgress,
    ) -> Result<(), StoreError>;
    /// Records a step's job and the step's progress pointing at it together, so that a crash
    /// between the two never leaves a job its run does not know, made again at the next launch.
    fn create_step_job(
        &self,
        job: &Job,
        run: crate::RunId,
        index: u32,
        progress: &crate::StepProgress,
    ) -> Result<(), StoreError>;
    fn cancel_run(&self, id: crate::RunId) -> Result<(), StoreError>;
    /// Deletes the run's record; its jobs and their results stay.
    fn delete_run(&self, id: crate::RunId) -> Result<(), StoreError>;
    /// The run and step a job belongs to, if it was started by a workflow.
    fn run_of_job(&self, job: JobId) -> Result<Option<(crate::RunId, u32)>, StoreError>;
}
