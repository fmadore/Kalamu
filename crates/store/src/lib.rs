//! SQLite implementation of [`JobStore`]: the job database and the result cache.
//!
//! Every finished piece is written as soon as it arrives, so closing the app or a crash loses at
//! most the piece in progress. Results are keyed by [`CacheKey`] and outlive the jobs that
//! produced them, so re-running a folder with the same settings costs nothing.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use kalamu_core::{
    BatchRecord, CacheKey, Charge, Correction, Job, JobId, JobSources, JobStatus, JobStore,
    JobSummary, OutdatedFiles, PieceState, SetupId, SourceId, SpeakerNames, StoreError,
    StoredResult,
};
use rusqlite::Connection;

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Opens (or creates) the database file and brings its schema up to date.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path).map_err(db)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(db)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(db)?;
        Self::init(conn)
    }

    /// A private database in memory, for tests.
    pub fn in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory().map_err(db)?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "foreign_keys", "ON").map_err(db)?;
        let applied: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(db)?;
        let applied = schema_known(applied, migrations::MIGRATIONS.len())?;
        for (i, migration) in migrations::MIGRATIONS.iter().enumerate().skip(applied) {
            conn.execute_batch(&format!(
                "BEGIN; {migration} PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))
            .map_err(db)?;
        }
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        // A panic while holding the lock leaves SQLite consistent (each statement is atomic), so
        // a poisoned mutex is safe to take over.
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// The migrations a database has had (`PRAGMA user_version`), if this version knows them all.
/// More than it knows means a newer Kalamu wrote it: its tables may hold what this version would
/// lose or misread, so it is refused rather than run on.
fn schema_known(applied: i64, known: usize) -> Result<usize, StoreError> {
    let applied = usize::try_from(applied).unwrap_or_default();
    if applied > known {
        return Err(StoreError::Newer {
            found: applied as u64,
            known: known as u64,
        });
    }
    Ok(applied)
}

fn db(err: rusqlite::Error) -> StoreError {
    StoreError::Database(err.to_string())
}

fn corrupt(err: impl std::fmt::Display) -> StoreError {
    StoreError::Corrupt(err.to_string())
}

/// A query over a job's rows (`select` ends with `WHERE job_id = ?1`) sorted by `order`, or,
/// with `scope`, over one source's rows under one set-up sorted by `order_one`, and its
/// arguments. The tables kept per piece have keys that begin with `(job_id, setup_id,
/// source_id)`, so the narrowed query reads only that source's rows, however large the job.
fn scoped(
    select: &str,
    order: &str,
    order_one: &str,
    job: JobId,
    scope: Option<(SetupId, SourceId)>,
) -> (String, Vec<String>) {
    match scope {
        None => (format!("{select} {order}"), vec![job.to_string()]),
        Some((setup, source)) => (
            format!("{select} AND setup_id = ?2 AND source_id = ?3 {order_one}"),
            vec![job.to_string(), setup.to_string(), source.to_string()],
        ),
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_string(value).map_err(corrupt)
}

impl JobStore for SqliteStore {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        jobs::create_job(self, job)
    }

    fn job(&self, id: JobId) -> Result<Job, StoreError> {
        jobs::job(self, id)
    }

    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError> {
        jobs::list_jobs(self)
    }

    fn job_sources(&self) -> Result<Vec<JobSources>, StoreError> {
        jobs::job_sources(self)
    }

    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError> {
        jobs::set_job_status(self, id, status)
    }

    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError> {
        jobs::rename_job(self, id, name)
    }

    fn delete_job(&self, id: JobId) -> Result<(), StoreError> {
        jobs::delete_job(self, id)
    }

    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError> {
        jobs::mark_running_as_interrupted(self)
    }

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError> {
        jobs::upsert_piece(self, piece)
    }

    fn upsert_pieces(&self, pieces: &[PieceState]) -> Result<(), StoreError> {
        jobs::upsert_pieces(self, pieces)
    }

    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError> {
        jobs::pieces(self, job, None)
    }

    fn source_pieces(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<PieceState>, StoreError> {
        jobs::pieces(self, job, Some((setup, source)))
    }

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError> {
        jobs::cached_result(self, key)
    }

    fn spent(&self, job: JobId) -> Result<f64, StoreError> {
        jobs::spent(self, job)
    }

    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError> {
        results::add_charge(self, charge)
    }

    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError> {
        results::charges(self, job, None)
    }

    fn source_charges(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Charge>, StoreError> {
        results::charges(self, job, Some((setup, source)))
    }

    fn prune_results(&self) -> Result<kalamu_core::PrunedResults, StoreError> {
        jobs::prune_results(self)
    }

    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        jobs::put_result(self, result)
    }

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        results::put_correction(self, correction)
    }

    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        results::corrections(self, job, None)
    }

    fn source_corrections(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<Correction>, StoreError> {
        results::corrections(self, job, Some((setup, source)))
    }

    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        results::put_speaker_names(self, job, names)
    }

    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        results::speaker_names(self, job, None)
    }

    fn source_speaker_names(
        &self,
        job: JobId,
        setup: SetupId,
        source: SourceId,
    ) -> Result<Vec<SpeakerNames>, StoreError> {
        results::speaker_names(self, job, Some((setup, source)))
    }

    fn outdated_files(&self, job: Option<JobId>) -> Result<Vec<OutdatedFiles>, StoreError> {
        outdated::of(self, job)
    }

    fn clear_outdated(&self, files: &OutdatedFiles) -> Result<(), StoreError> {
        outdated::clear(self, files)
    }

    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        batches::put_batch(self, batch)
    }

    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
        batches::batches(self, job)
    }

    fn put_snapshot(&self, snapshot: &kalamu_core::Snapshot) -> Result<(), StoreError> {
        snapshots::put(self, snapshot)
    }

    fn snapshot(&self, id: &str) -> Result<kalamu_core::Snapshot, StoreError> {
        snapshots::get(self, id)
    }

    fn put_unit(&self, unit: &kalamu_core::DocumentUnit) -> Result<(), StoreError> {
        units::put(self, unit)
    }

    fn unit(&self, id: kalamu_core::UnitId) -> Result<kalamu_core::DocumentUnit, StoreError> {
        units::get(self, id)
    }

    fn units(&self, job: JobId) -> Result<Vec<kalamu_core::DocumentUnit>, StoreError> {
        units::of_job(self, job)
    }

    fn delete_unit(&self, id: kalamu_core::UnitId) -> Result<(), StoreError> {
        units::delete(self, id)
    }

    fn replace_units(
        &self,
        put: &[kalamu_core::DocumentUnit],
        delete: &[kalamu_core::UnitId],
    ) -> Result<(), StoreError> {
        units::replace(self, put, delete)
    }

    fn create_run(&self, run: &kalamu_core::WorkflowRun) -> Result<(), StoreError> {
        runs::create(self, run)
    }

    fn run(&self, id: kalamu_core::RunId) -> Result<kalamu_core::WorkflowRun, StoreError> {
        runs::get(self, id)
    }

    fn runs(&self) -> Result<Vec<kalamu_core::WorkflowRun>, StoreError> {
        runs::list(self)
    }

    fn set_step_progress(
        &self,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        runs::set_progress(self, run, index, progress)
    }

    fn create_step_job(
        &self,
        job: &Job,
        run: kalamu_core::RunId,
        index: u32,
        progress: &kalamu_core::StepProgress,
    ) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        insert_job(&tx, job)?;
        runs::write_known(&tx, run, index, progress)?;
        tx.commit().map_err(db)
    }

    fn cancel_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        runs::cancel(self, id)
    }

    fn delete_run(&self, id: kalamu_core::RunId) -> Result<(), StoreError> {
        runs::delete(self, id)
    }

    fn run_of_job(&self, job: JobId) -> Result<Option<(kalamu_core::RunId, u32)>, StoreError> {
        runs::of_job(self, job)
    }
}

mod batches;
mod dictionary;
mod jobs;
use jobs::insert_job;
mod migrations;
mod outdated;
mod projects;
mod results;
mod runs;
mod search;
mod snapshots;
mod units;

pub use search::SqliteIndex;

#[cfg(test)]
mod tests;
