use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    BatchRecord, CacheKey, Charge, Job, JobId, JobSources, JobStatus, JobSummary, PieceState,
    SetupId, SourceId, SpeakerNames, StoredResult, Timestamp,
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

/// A source's files under one set-up that are older than what is stored: a correction or
/// speakers' names were saved since they were written. Kept in the store until they are written
/// again, so neither a crash nor a write that failed leaves them behind for good.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutdatedFiles {
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    /// Raised by each save, so files written from what was stored before a later save never
    /// clear its mark.
    pub generation: i64,
}

/// What pruning the result cache freed ([`JobStore::prune_results`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrunedResults {
    /// Results deleted.
    pub results: u64,
    /// The database's size before less its size after, in bytes.
    pub bytes: u64,
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
    /// The names of every job's sources, newest job first; a job without sources is left out.
    fn job_sources(&self) -> Result<Vec<JobSources>, StoreError>;
    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError>;
    /// Gives a job a new name; nothing else about it changes.
    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError>;
    fn delete_job(&self, id: JobId) -> Result<(), StoreError>;

    /// Marks every running job as interrupted; called at startup after a crash or close.
    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError>;

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError>;
    /// Records several pieces at once, all or none: a source's plan is never left recorded in
    /// part, to be taken for the whole plan when the job resumes.
    fn upsert_pieces(&self, pieces: &[PieceState]) -> Result<(), StoreError>;
    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError>;
    /// One source's pieces under one set-up, in their order: what writing its files reads,
    /// without the rest of a job of thousands of sources.
    fn source_pieces(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<PieceState>, StoreError>;

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
    /// A finished piece's text as the researcher left it: the corrected text, else the
    /// model's, with the speakers' names applied ([`crate::speakers::text_as_left`]). The
    /// model's answer is not read when a correction replaces it; none for a piece not finished
    /// and not corrected.
    fn text(
        &self,
        piece: &PieceState,
        corrections: &[Correction],
        speakers: &crate::SpeakerIndex,
    ) -> Result<Option<String>, StoreError> {
        let names = speakers.of(piece.setup_id, piece.source_id);
        let corrected = Correction::of(corrections, piece).and_then(|c| c.text.as_deref());
        if let Some(text) = corrected {
            return Ok(Some(crate::speakers::rename_text(text, names)));
        }
        Ok(self
            .finished_result(piece)?
            .map(|r| crate::speakers::rename_text(&r.output.text, names)))
    }
    /// What a job's own calls cost, in US dollars: the results its pieces stored, not those
    /// reused from the cache, and the answers billed but refused ([`Charge`]).
    fn spent(&self, job: JobId) -> Result<f64, StoreError>;
    /// Records a call billed without a result to keep.
    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError>;
    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError>;
    /// The charges of one source's pieces under one set-up, in the order they were billed.
    fn source_charges(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Charge>, StoreError>;
    /// Deletes the stored results that no piece of any job refers to (those of deleted jobs,
    /// and answers a piece read again under another key no longer uses), then gives the space
    /// back to the disk. A piece records its key before its answer is stored, so a result a
    /// job is waiting for or will read is never one of them; a result deleted is paid for
    /// again if the same work is asked for later.
    fn prune_results(&self) -> Result<PrunedResults, StoreError>;
    /// Stores a result unless one is already stored under its key. The first answer is kept,
    /// never replaced by a later one (two identical set-ups, a batch collected after a fast
    /// run), so the researcher's corrections stay over the text they corrected.
    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError>;

    /// Stores a correction and marks its source's files under its set-up out of date, in one
    /// transaction.
    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError>;
    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError>;
    /// The corrections of one source's pieces under one set-up.
    fn source_corrections(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Correction>, StoreError>;

    /// Replaces the names given to one set-up's speakers in one recording and marks its files
    /// under that set-up out of date, in one transaction.
    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError>;
    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError>;
    /// The names given to one set-up's speakers in one recording: none, or one entry.
    fn source_speaker_names(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<SpeakerNames>, StoreError>;
    /// The names given to every recording's speakers in a job, ready to look up piece by piece.
    fn speakers(&self, job: JobId) -> Result<crate::SpeakerIndex, StoreError> {
        Ok(crate::SpeakerIndex::new(self.speaker_names(job)?))
    }

    /// The files marked out of date ([`OutdatedFiles`]), of one job or of every job.
    fn outdated_files(&self, job: Option<JobId>) -> Result<Vec<OutdatedFiles>, StoreError>;
    /// Clears a mark once its files are written, unless a save raised it since: those files
    /// may not hold what that save stored.
    fn clear_outdated(&self, files: &OutdatedFiles) -> Result<(), StoreError>;

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
