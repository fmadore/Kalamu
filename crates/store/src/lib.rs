//! SQLite implementation of [`JobStore`]: the job database and the result cache.
//!
//! Every finished piece is written as soon as it arrives, so closing the app or a crash loses at
//! most the piece in progress. Results are keyed by [`CacheKey`] and outlive the jobs that
//! produced them, so re-running a folder with the same settings costs nothing.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use kalamu_core::{
    BatchRecord, CacheKey, Correction, Job, JobId, JobStatus, JobStore, JobSummary, PieceRef,
    PieceState, PieceStatus, ProviderId, SetupId, SourceId, SpeakerNames, StoreError, StoredResult,
    Timestamp,
};
use rusqlite::{Connection, OptionalExtension, params};

/// Schema migrations, applied in order; `PRAGMA user_version` records how many have run.
const MIGRATIONS: &[&str] = &[
    r#"
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE TABLE pieces (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    piece TEXT NOT NULL,
    status TEXT NOT NULL,
    cache_key TEXT,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, idx)
);
CREATE TABLE results (
    cache_key TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    output TEXT NOT NULL,
    cost_usd REAL,
    duration_ms INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE corrections (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    text TEXT,
    reviewed INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, idx)
);
"#,
    r#"
CREATE TABLE speaker_names (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    label TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, label)
);
"#,
    r#"
ALTER TABLE corrections ADD COLUMN fields TEXT;
"#,
    r#"
CREATE TABLE batches (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    phase TEXT NOT NULL,
    submitted_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX batches_by_job ON batches(job_id);
"#,
    r#"
ALTER TABLE results ADD COLUMN request TEXT;
"#,
];

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
        for (i, migration) in MIGRATIONS.iter().enumerate().skip(applied) {
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

impl JobStore for SqliteStore {
    fn create_job(&self, job: &Job) -> Result<(), StoreError> {
        let now = Timestamp::now().0;
        self.conn()
            .execute(
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
                        COALESCE(SUM(p.status = 'failed'), 0)
                 FROM jobs j LEFT JOIN pieces p ON p.job_id = j.id
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
                ))
            })
            .map_err(db)?;
        let mut jobs = Vec::new();
        for row in rows {
            let (data, status, updated_at, total, finished, failed) = row.map_err(db)?;
            let job: Job = serde_json::from_str(&data).map_err(corrupt)?;
            jobs.push(JobSummary {
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

    fn put_result(&self, result: &StoredResult) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR REPLACE INTO results
                    (cache_key, provider, model, output, cost_usd, duration_ms, created_at,
                     request)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    result.cache_key,
                    result.provider.as_str(),
                    result.model,
                    to_json(&result.output)?,
                    result.cost_usd,
                    i64::try_from(result.duration_ms).unwrap_or(i64::MAX),
                    result.created_at.0,
                    to_json(&result.request)?,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn put_correction(&self, correction: &Correction) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO corrections
                    (job_id, setup_id, source_id, idx, text, reviewed, updated_at, fields)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT (job_id, setup_id, source_id, idx) DO UPDATE SET
                    text = excluded.text, reviewed = excluded.reviewed,
                    updated_at = excluded.updated_at, fields = excluded.fields",
                params![
                    correction.job_id.to_string(),
                    correction.setup_id.to_string(),
                    correction.source_id.to_string(),
                    correction.index,
                    correction.text,
                    correction.reviewed,
                    correction.updated_at.0,
                    correction.fields.as_ref().map(to_json).transpose()?,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn corrections(&self, job: JobId) -> Result<Vec<Correction>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT setup_id, source_id, idx, text, reviewed, updated_at, fields
                 FROM corrections WHERE job_id = ?1",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([job.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, bool>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            let (setup, source, index, text, reviewed, updated, fields) = row.map_err(db)?;
            out.push(Correction {
                job_id: job,
                setup_id: setup.parse().map_err(corrupt)?,
                source_id: source.parse().map_err(corrupt)?,
                index,
                text,
                fields: fields
                    .map(|f| serde_json::from_str(&f))
                    .transpose()
                    .map_err(corrupt)?,
                reviewed,
                updated_at: Timestamp(updated),
            });
        }
        Ok(out)
    }

    fn put_speaker_names(&self, job: JobId, names: &SpeakerNames) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        let key = [
            job.to_string(),
            names.setup_id.to_string(),
            names.source_id.to_string(),
        ];
        tx.execute(
            "DELETE FROM speaker_names WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3",
            params![key[0], key[1], key[2]],
        )
        .map_err(db)?;
        for (label, name) in &names.names {
            if name.trim().is_empty() {
                continue;
            }
            tx.execute(
                "INSERT INTO speaker_names (job_id, setup_id, source_id, label, name)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![key[0], key[1], key[2], label, name.trim()],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)
    }

    fn speaker_names(&self, job: JobId) -> Result<Vec<SpeakerNames>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT setup_id, source_id, label, name FROM speaker_names
                 WHERE job_id = ?1 ORDER BY setup_id, source_id",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([job.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(db)?;
        let mut out: Vec<SpeakerNames> = Vec::new();
        for row in rows {
            let (setup, source, label, name) = row.map_err(db)?;
            let setup_id: SetupId = setup.parse().map_err(corrupt)?;
            let source_id: SourceId = source.parse().map_err(corrupt)?;
            match out.last_mut() {
                Some(last) if last.setup_id == setup_id && last.source_id == source_id => {
                    last.names.insert(label, name);
                }
                _ => out.push(SpeakerNames {
                    setup_id,
                    source_id,
                    names: [(label, name)].into_iter().collect(),
                }),
            }
        }
        Ok(out)
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
}

#[cfg(test)]
mod tests;
