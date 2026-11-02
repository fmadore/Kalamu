//! Workflow runs: the frozen definition and sources in one row, each step's progress in another.
//!
//! A step's job is kept without a foreign key, as snapshots are: a job deleted leaves the step
//! pointing at it, so the run reads it as gone instead of starting the step again.

use kalamu_core::{Approval, JobId, RunId, StepProgress, StoreError, Timestamp, WorkflowRun};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE runs (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    cancelled INTEGER NOT NULL DEFAULT 0,
    data TEXT NOT NULL
);
CREATE TABLE run_steps (
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    job_id TEXT,
    approved_at INTEGER,
    partial INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    PRIMARY KEY (run_id, idx)
);
CREATE INDEX run_steps_by_job ON run_steps(job_id);
"#;

pub(crate) fn create(store: &SqliteStore, run: &WorkflowRun) -> Result<(), StoreError> {
    let mut frozen = run.clone();
    for step in &mut frozen.steps {
        step.progress = StepProgress::default();
    }
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    tx.execute(
        "INSERT INTO runs (id, created_at, cancelled, data) VALUES (?1, ?2, ?3, ?4)",
        params![
            run.id.to_string(),
            run.created_at.0,
            run.cancelled,
            to_json(&frozen)?
        ],
    )
    .map_err(db)?;
    for (i, step) in run.steps.iter().enumerate() {
        write_progress(&tx, run.id, i as u32, &step.progress)?;
    }
    tx.commit().map_err(db)
}

fn write_progress(
    conn: &rusqlite::Connection,
    run: RunId,
    index: u32,
    progress: &StepProgress,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO run_steps (run_id, idx, job_id, approved_at, partial, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (run_id, idx) DO UPDATE SET job_id = ?3, approved_at = ?4, partial = ?5,
         error = ?6",
        params![
            run.to_string(),
            index,
            progress.job_id.map(|j| j.to_string()),
            progress.approval.map(|a| a.at.0),
            progress.approval.is_some_and(|a| a.partial),
            progress.error,
        ],
    )
    .map_err(db)?;
    Ok(())
}

pub(crate) fn set_progress(
    store: &SqliteStore,
    run: RunId,
    index: u32,
    progress: &StepProgress,
) -> Result<(), StoreError> {
    write_known(&store.conn(), run, index, progress)
}

/// Writes the progress of a step the run has, on `conn`, which may be a transaction's.
pub(crate) fn write_known(
    conn: &rusqlite::Connection,
    run: RunId,
    index: u32,
    progress: &StepProgress,
) -> Result<(), StoreError> {
    let known: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM run_steps WHERE run_id = ?1 AND idx = ?2",
            params![run.to_string(), index],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    if known.is_none() {
        return Err(StoreError::NotFound);
    }
    write_progress(conn, run, index, progress)
}

pub(crate) fn get(store: &SqliteStore, id: RunId) -> Result<WorkflowRun, StoreError> {
    let conn = store.conn();
    let row: Option<(String, bool)> = conn
        .query_row(
            "SELECT data, cancelled FROM runs WHERE id = ?1",
            [id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db)?;
    let (data, cancelled) = row.ok_or(StoreError::NotFound)?;
    let mut run: WorkflowRun = serde_json::from_str(&data).map_err(corrupt)?;
    run.cancelled = cancelled;
    let mut stmt = conn
        .prepare("SELECT idx, job_id, approved_at, partial, error FROM run_steps WHERE run_id = ?1")
        .map_err(db)?;
    let rows = stmt
        .query_map([id.to_string()], |row| {
            Ok((
                row.get::<_, u32>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(db)?;
    for row in rows {
        let (index, job, approved_at, partial, error) = row.map_err(db)?;
        let Some(step) = run.steps.get_mut(index as usize) else {
            continue;
        };
        step.progress = StepProgress {
            job_id: job
                .map(|j| j.parse::<JobId>().map_err(corrupt))
                .transpose()?,
            approval: approved_at.map(|at| Approval {
                at: Timestamp(at),
                partial,
            }),
            error,
        };
    }
    Ok(run)
}

/// Newest first.
pub(crate) fn list(store: &SqliteStore) -> Result<Vec<WorkflowRun>, StoreError> {
    let ids: Vec<String> = {
        let conn = store.conn();
        let mut stmt = conn
            .prepare("SELECT id FROM runs ORDER BY created_at DESC, rowid DESC")
            .map_err(db)?;
        stmt.query_map([], |row| row.get(0))
            .map_err(db)?
            .collect::<Result<_, _>>()
            .map_err(db)?
    };
    ids.iter()
        .map(|id| get(store, id.parse().map_err(corrupt)?))
        .collect()
}

pub(crate) fn cancel(store: &SqliteStore, id: RunId) -> Result<(), StoreError> {
    let changed = store
        .conn()
        .execute(
            "UPDATE runs SET cancelled = 1 WHERE id = ?1",
            [id.to_string()],
        )
        .map_err(db)?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// The run's record only: its jobs and their results stay.
pub(crate) fn delete(store: &SqliteStore, id: RunId) -> Result<(), StoreError> {
    store
        .conn()
        .execute("DELETE FROM runs WHERE id = ?1", [id.to_string()])
        .map_err(db)?;
    Ok(())
}

pub(crate) fn of_job(store: &SqliteStore, job: JobId) -> Result<Option<(RunId, u32)>, StoreError> {
    let row: Option<(String, u32)> = store
        .conn()
        .query_row(
            "SELECT run_id, idx FROM run_steps WHERE job_id = ?1",
            [job.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db)?;
    row.map(|(run, idx)| Ok((run.parse().map_err(corrupt)?, idx)))
        .transpose()
}
