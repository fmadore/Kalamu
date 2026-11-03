//! SQLite implementation of [`JobStore`]: the job database and the result cache.
//!
//! Every finished piece is written as soon as it arrives, so closing the app or a crash loses at
//! most the piece in progress. Results are keyed by [`CacheKey`] and outlive the jobs that
//! produced them, so re-running a folder with the same settings costs nothing.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use kalamu_core::{
    BatchRecord, CacheKey, Charge, Correction, Job, JobId, JobStatus, JobStore, JobSummary,
    PieceRef, PieceState, PieceStatus, ProviderId, SetupId, SourceId, SpeakerNames, StoreError,
    StoredResult, Timestamp,
};
use rusqlite::{Connection, OptionalExtension, params};

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
        let applied = usize::try_from(applied).unwrap_or_default();
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

fn db(err: rusqlite::Error) -> StoreError {
    StoreError::Database(err.to_string())
}

fn corrupt(err: impl std::fmt::Display) -> StoreError {
    StoreError::Corrupt(err.to_string())
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_string(value).map_err(corrupt)
}

fn parse_status(s: &str) -> Result<JobStatus, StoreError> {
    JobStatus::parse(s).ok_or_else(|| corrupt(format!("unknown job status {s}")))
}

/// Records a new job on `conn`, which may be a transaction's.
fn insert_job(conn: &Connection, job: &Job) -> Result<(), StoreError> {
    let now = Timestamp::now().0;
    conn.execute(
        "INSERT INTO jobs (id, name, task, status, created_at, updated_at, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            job.id.to_string(),
            job.name,
            job.task.as_str(),
            job.status.as_str(),
            job.created_at.0,
            now,
            to_json(job)?,
        ],
    )
    .map_err(db)?;
    Ok(())
}

impl JobStore for SqliteStore {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        insert_job(&self.conn(), job)
    }

    fn job(&self, id: JobId) -> Result<Job, StoreError> {
        let row: Option<(String, String)> = self
            .conn()
            .query_row(
                "SELECT data, status FROM jobs WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(db)?;
        let (data, status) = row.ok_or(StoreError::NotFound)?;
        let mut job: Job = serde_json::from_str(&data).map_err(corrupt)?;
        job.status = parse_status(&status)?;
        Ok(job)
    }

    fn list_jobs(&self) -> Result<Vec<JobSummary>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT j.data, j.status, j.updated_at,
                        COUNT(p.idx),
                        COALESCE(SUM(p.status IN ('done', 'cached')), 0),
                        COALESCE(SUM(p.status = 'failed'), 0),
                        COALESCE(SUM(p.status IN ('done', 'cached') AND r.truncated), 0),
                        SUM(CASE WHEN p.status = 'done' THEN r.cost_usd END),
                        (SELECT SUM(c.cost_usd) FROM charges c WHERE c.job_id = j.id),
                        (SELECT json_extract(w.data, '$.workflow_name')
                         FROM run_steps s JOIN runs w ON w.id = s.run_id
                         WHERE s.job_id = j.id LIMIT 1)
                 FROM jobs j LEFT JOIN pieces p ON p.job_id = j.id
                 LEFT JOIN results r ON r.cache_key = p.cache_key
                 GROUP BY j.id
                 ORDER BY j.created_at DESC",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, u32>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, u32>(6)?,
                    row.get::<_, Option<f64>>(7)?,
                    row.get::<_, Option<f64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            })
            .map_err(db)?;
        let mut jobs = Vec::new();
        for row in rows {
            let (
                data,
                status,
                updated_at,
                total,
                finished,
                failed,
                truncated,
                kept,
                refused,
                workflow,
            ) = row.map_err(db)?;
            // What its results cost, and the answers billed but refused.
            let cost = match (kept, refused) {
                (None, None) => None,
                (kept, refused) => Some(kept.unwrap_or(0.0) + refused.unwrap_or(0.0)),
            };
            let job: Job = serde_json::from_str(&data).map_err(corrupt)?;
            jobs.push(JobSummary {
                models: job.models(),
                workflow,
                id: job.id,
                name: job.name,
                task: job.task,
                status: parse_status(&status)?,
                created_at: job.created_at,
                updated_at: Timestamp(updated_at),
                source_count: job.sources.len() as u32,
                setup_count: job.setups.len() as u32,
                pieces_total: total,
                pieces_finished: finished,
                pieces_failed: failed,
                pieces_truncated: truncated,
                cost_usd: cost,
            });
        }
        Ok(jobs)
    }

    fn set_job_status(&self, id: JobId, status: JobStatus) -> Result<(), StoreError> {
        let changed = self
            .conn()
            .execute(
                "UPDATE jobs SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![status.as_str(), Timestamp::now().0, id.to_string()],
            )
            .map_err(db)?;
        if changed == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    fn rename_job(&self, id: JobId, name: &str) -> Result<(), StoreError> {
        // The name is kept twice, as a column and inside the job's data; both change together.
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        let data: Option<String> = tx
            .query_row(
                "SELECT data FROM jobs WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(db)?;
        let mut job: Job =
            serde_json::from_str(&data.ok_or(StoreError::NotFound)?).map_err(corrupt)?;
        job.name = name.to_string();
        tx.execute(
            "UPDATE jobs SET name = ?1, data = ?2 WHERE id = ?3",
            params![name, to_json(&job)?, id.to_string()],
        )
        .map_err(db)?;
        tx.commit().map_err(db)
    }

    fn delete_job(&self, id: JobId) -> Result<(), StoreError> {
        self.conn()
            .execute("DELETE FROM jobs WHERE id = ?1", [id.to_string()])
            .map_err(db)?;
        Ok(())
    }

    fn mark_running_as_interrupted(&self) -> Result<Vec<JobId>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM jobs WHERE status = 'running'")
            .map_err(db)?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?;
        drop(stmt);
        conn.execute(
            "UPDATE jobs SET status = 'interrupted' WHERE status = 'running'",
            [],
        )
        .map_err(db)?;
        // A piece caught mid-call was never finished: it goes back to the queue.
        conn.execute(
            "UPDATE pieces SET status = 'pending' WHERE status = 'running'",
            [],
        )
        .map_err(db)?;
        ids.iter().map(|id| id.parse().map_err(corrupt)).collect()
    }

    fn upsert_piece(&self, piece: &PieceState) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO pieces
                    (job_id, setup_id, source_id, idx, piece, status, cache_key, error, attempts,
                     updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (job_id, setup_id, source_id, idx) DO UPDATE SET
                    piece = excluded.piece, status = excluded.status,
                    cache_key = excluded.cache_key, error = excluded.error,
                    attempts = excluded.attempts, updated_at = excluded.updated_at",
                params![
                    piece.job_id.to_string(),
                    piece.setup_id.to_string(),
                    piece.source_id.to_string(),
                    piece.index,
                    to_json(&piece.piece)?,
                    piece.status.as_str(),
                    piece.cache_key,
                    piece.error,
                    piece.attempts,
                    piece.updated_at.0,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn pieces(&self, job: JobId) -> Result<Vec<PieceState>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT setup_id, source_id, idx, piece, status, cache_key, error, attempts,
                        updated_at
                 FROM pieces WHERE job_id = ?1 ORDER BY source_id, setup_id, idx",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([job.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, u32>(7)?,
                    row.get::<_, i64>(8)?,
                ))
            })
            .map_err(db)?;
        let mut pieces = Vec::new();
        for row in rows {
            let (setup, source, index, piece, status, cache_key, error, attempts, updated) =
                row.map_err(db)?;
            pieces.push(PieceState {
                job_id: job,
                setup_id: setup.parse::<SetupId>().map_err(corrupt)?,
                source_id: source.parse::<SourceId>().map_err(corrupt)?,
                index,
                piece: serde_json::from_str::<PieceRef>(&piece).map_err(corrupt)?,
                status: PieceStatus::parse(&status)
                    .ok_or_else(|| corrupt(format!("unknown piece status {status}")))?,
                cache_key,
                error,
                attempts,
                updated_at: Timestamp(updated),
            });
        }
        Ok(pieces)
    }

    fn cached_result(&self, key: &CacheKey) -> Result<Option<StoredResult>, StoreError> {
        let row = self
            .conn()
            .query_row(
                "SELECT provider, model, output, cost_usd, duration_ms, created_at, request
                 FROM results WHERE cache_key = ?1",
                [key.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<f64>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(db)?;
        row.map(
            |(provider, model, output, cost_usd, duration_ms, created_at, request)| {
                Ok(StoredResult {
                    cache_key: key.0.clone(),
                    provider: ProviderId(provider),
                    model,
                    output: serde_json::from_str(&output).map_err(corrupt)?,
                    // Results stored before the column existed have none.
                    request: match request {
                        Some(json) => serde_json::from_str(&json).map_err(corrupt)?,
                        None => Vec::new(),
                    },
                    cost_usd,
                    duration_ms: u64::try_from(duration_ms).unwrap_or_default(),
                    created_at: Timestamp(created_at),
                })
            },
        )
        .transpose()
    }

    fn spent(&self, job: JobId) -> Result<f64, StoreError> {
        // A piece done by this job paid for its result; a cached one reused another's. Answers
        // refused after being billed were paid for too.
        self.conn()
            .query_row(
                "SELECT (SELECT COALESCE(SUM(r.cost_usd), 0) FROM pieces p
                         JOIN results r ON r.cache_key = p.cache_key
                         WHERE p.job_id = ?1 AND p.status = 'done')
                      + (SELECT COALESCE(SUM(cost_usd), 0) FROM charges WHERE job_id = ?1)",
                [job.to_string()],
                |row| row.get(0),
            )
            .map_err(db)
    }

    fn add_charge(&self, charge: &Charge) -> Result<(), StoreError> {
        results::add_charge(self, charge)
    }

    fn charges(&self, job: JobId) -> Result<Vec<Charge>, StoreError> {
        results::charges(self, job)
    }

    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO results
                    (cache_key, provider, model, output, cost_usd, duration_ms, created_at,
                     request, truncated)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    result.cache_key,
                    result.provider.as_str(),
                    result.model,
                    to_json(&result.output)?,
                    result.cost_usd,
                    i64::try_from(result.duration_ms).unwrap_or(i64::MAX),
                    result.created_at.0,
                    to_json(&result.request)?,
                    result.output.truncated,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        results::put_correction(self, correction)
    }

    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        results::corrections(self, job)
    }

    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        results::put_speaker_names(self, job, names)
    }

    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        results::speaker_names(self, job)
    }

    fn put_batch(&self, batch: &BatchRecord) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR REPLACE INTO batches (id, job_id, phase, submitted_at, data)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    batch.id,
                    batch.job_id.to_string(),
                    batch.phase.as_str(),
                    batch.submitted_at.0,
                    to_json(batch)?
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn batches(&self, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT data FROM batches WHERE job_id = ?1 ORDER BY submitted_at, id")
            .map_err(db)?;
        let rows = stmt
            .query_map([job.to_string()], |row| row.get::<_, String>(0))
            .map_err(db)?;
        rows.map(|row| serde_json::from_str(&row.map_err(db)?).map_err(corrupt))
            .collect()
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

mod dictionary;
mod migrations;
mod projects;
mod results;
mod runs;
mod snapshots;
mod units;

#[cfg(test)]
mod tests;
