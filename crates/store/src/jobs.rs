//! Jobs, their pieces and the result cache: what the engine records before and after each
//! call.

use kalamu_core::{
    CacheKey, Job, JobId, JobSources, JobStatus, JobSummary, PieceRef, PieceState, PieceStatus,
    ProviderId, SetupId, SourceId, StoreError, StoredResult, Timestamp,
};
use rusqlite::{Connection, OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) fn parse_status(s: &str) -> Result<JobStatus, StoreError> {
    JobStatus::parse(s).ok_or_else(|| corrupt(format!("unknown job status {s}")))
}

/// Records a new job on `conn`, which may be a transaction's.
pub(crate) fn insert_job(conn: &Connection, job: &Job) -> Result<(), StoreError> {
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

pub(crate) fn create_job(store: &SqliteStore, job: &Job) -> Result<(), StoreError> {
    insert_job(&store.conn(), job)
}

pub(crate) fn job(store: &SqliteStore, id: JobId) -> Result<Job, StoreError> {
    let row: Option<(String, String)> = store
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

pub(crate) fn list_jobs(store: &SqliteStore) -> Result<Vec<JobSummary>, StoreError> {
    let conn = store.conn();
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
        let (data, status, updated_at, total, finished, failed, truncated, kept, refused, workflow) =
            row.map_err(db)?;
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

pub(crate) fn job_sources(store: &SqliteStore) -> Result<Vec<JobSources>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare(
            "SELECT j.id, json_extract(s.value, '$.file_name'), json_extract(s.value, '$.title')
             FROM jobs j, json_each(j.data, '$.sources') s
             ORDER BY j.created_at DESC, j.id, s.key",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(db)?;
    let mut out: Vec<JobSources> = Vec::new();
    for row in rows {
        let (id, file_name, title) = row.map_err(db)?;
        let job_id: JobId = id.parse().map_err(corrupt)?;
        if out.last().is_none_or(|j| j.job_id != job_id) {
            out.push(JobSources {
                job_id,
                names: Vec::new(),
            });
        }
        if let Some(job) = out.last_mut() {
            job.names.extend(file_name.into_iter().chain(title));
        }
    }
    Ok(out)
}

pub(crate) fn set_job_status(
    store: &SqliteStore,
    id: JobId,
    status: JobStatus,
) -> Result<(), StoreError> {
    let changed = store
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

pub(crate) fn rename_job(store: &SqliteStore, id: JobId, name: &str) -> Result<(), StoreError> {
    // The name is kept twice, as a column and inside the job's data; both change together.
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    let data: Option<String> = tx
        .query_row(
            "SELECT data FROM jobs WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    let mut job: Job = serde_json::from_str(&data.ok_or(StoreError::NotFound)?).map_err(corrupt)?;
    job.name = name.to_string();
    tx.execute(
        "UPDATE jobs SET name = ?1, data = ?2 WHERE id = ?3",
        params![name, to_json(&job)?, id.to_string()],
    )
    .map_err(db)?;
    tx.commit().map_err(db)
}

pub(crate) fn delete_job(store: &SqliteStore, id: JobId) -> Result<(), StoreError> {
    store
        .conn()
        .execute("DELETE FROM jobs WHERE id = ?1", [id.to_string()])
        .map_err(db)?;
    Ok(())
}

pub(crate) fn mark_running_as_interrupted(store: &SqliteStore) -> Result<Vec<JobId>, StoreError> {
    let conn = store.conn();
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

pub(crate) fn upsert_piece(store: &SqliteStore, piece: &PieceState) -> Result<(), StoreError> {
    upsert_piece_on(&store.conn(), piece)
}

/// Records several pieces in one transaction: all of them, or none if one fails.
pub(crate) fn upsert_pieces(store: &SqliteStore, pieces: &[PieceState]) -> Result<(), StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    for piece in pieces {
        upsert_piece_on(&tx, piece)?;
    }
    tx.commit().map_err(db)
}

/// Records a piece on `conn`, which may be a transaction's.
fn upsert_piece_on(conn: &Connection, piece: &PieceState) -> Result<(), StoreError> {
    conn.execute(
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

pub(crate) fn pieces(store: &SqliteStore, job: JobId) -> Result<Vec<PieceState>, StoreError> {
    let conn = store.conn();
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

pub(crate) fn cached_result(
    store: &SqliteStore,
    key: &CacheKey,
) -> Result<Option<StoredResult>, StoreError> {
    let row = store
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

pub(crate) fn spent(store: &SqliteStore, job: JobId) -> Result<f64, StoreError> {
    // A piece done by this job paid for its result; a cached one reused another's. Answers
    // refused after being billed were paid for too.
    store
        .conn()
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

pub(crate) fn put_result(store: &SqliteStore, result: &StoredResult) -> Result<(), StoreError> {
    store
        .conn()
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
