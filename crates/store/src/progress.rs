//! A job's progress counted where its pieces are kept (#143): a running job's page reads it
//! twice a second, so it costs a few grouped queries over an index, never every piece of the
//! job decoded and sent up to be counted.

use std::collections::HashMap;

use kalamu_core::{ERRORS_LISTED, FileProgress, JobId, JobProgress, SetupId, SourceId, StoreError};
use rusqlite::params;

use super::{SqliteStore, corrupt, db, to_json};

/// The index the counts are read from: a job's pieces by file, with the set-up and status
/// alongside, so counting the files a page shows seeks to each of them and reads nothing else,
/// and the pieces held in batches are counted from the index alone.
pub(crate) const MIGRATION: &str = r#"
CREATE INDEX IF NOT EXISTS pieces_progress ON pieces(job_id, source_id, setup_id, status);
"#;

/// The pieces of the job a provider's batch service holds.
pub(crate) const HELD: &str =
    "SELECT COUNT(*) FROM pieces WHERE job_id = ?1 AND status = 'submitted'";

/// Every file of the job with a failed piece, under any set-up.
pub(crate) const FAILED: &str = "SELECT DISTINCT source_id FROM pieces
    WHERE job_id = ?1 AND status = 'failed'
    ORDER BY source_id";

/// Each file asked for (`?2`, a JSON list), counted under each set-up, in the order the store
/// gives a job's pieces.
pub(crate) const COUNTS: &str = "SELECT source_id, setup_id, COUNT(*),
        SUM(status IN ('done', 'cached')), SUM(status = 'failed')
    FROM pieces
    WHERE job_id = ?1 AND source_id IN (SELECT value FROM json_each(?2))
    GROUP BY source_id, setup_id
    ORDER BY source_id, setup_id";

/// Each file's first few different reasons under each set-up, in the order of the first piece
/// that gave each.
pub(crate) const ERRORS: &str = "SELECT source_id, setup_id, error FROM (
        SELECT source_id, setup_id, error,
            ROW_NUMBER() OVER (PARTITION BY source_id, setup_id ORDER BY first) AS n
        FROM (
            SELECT source_id, setup_id, error, MIN(idx) AS first
            FROM pieces
            WHERE job_id = ?1 AND source_id IN (SELECT value FROM json_each(?2))
                AND error IS NOT NULL
            GROUP BY source_id, setup_id, error
        )
    )
    WHERE n <= ?3
    ORDER BY source_id, setup_id, n";

pub(crate) fn job_progress(
    store: &SqliteStore,
    job: JobId,
    sources: &[SourceId],
) -> Result<JobProgress, StoreError> {
    let conn = store.conn();
    let held: u32 = conn
        .query_row(HELD, [job.to_string()], |row| row.get(0))
        .map_err(db)?;
    let failed_sources = {
        let mut stmt = conn.prepare(FAILED).map_err(db)?;
        let rows = stmt
            .query_map([job.to_string()], |row| row.get::<_, String>(0))
            .map_err(db)?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(db)?.parse::<SourceId>().map_err(corrupt)?);
        }
        ids
    };
    if sources.is_empty() {
        return Ok(JobProgress {
            held,
            failed_sources,
            files: Vec::new(),
        });
    }
    // One JSON list rather than a parameter per file, so a page of any length is one query.
    let wanted = to_json(&sources.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
    let ids = |source: String, setup: String| -> Result<(SourceId, SetupId), StoreError> {
        Ok((
            source.parse::<SourceId>().map_err(corrupt)?,
            setup.parse::<SetupId>().map_err(corrupt)?,
        ))
    };

    let mut files = Vec::new();
    let mut at: HashMap<(SourceId, SetupId), usize> = HashMap::new();
    let mut stmt = conn.prepare(COUNTS).map_err(db)?;
    let rows = stmt
        .query_map(params![job.to_string(), wanted], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, u32>(4)?,
            ))
        })
        .map_err(db)?;
    for row in rows {
        let (source, setup, total, done, failed) = row.map_err(db)?;
        let (source_id, setup_id) = ids(source, setup)?;
        at.insert((source_id, setup_id), files.len());
        files.push(FileProgress {
            source_id,
            setup_id,
            total,
            done,
            failed,
            errors: Vec::new(),
        });
    }

    let mut stmt = conn.prepare(ERRORS).map_err(db)?;
    let rows = stmt
        .query_map(
            params![job.to_string(), wanted, ERRORS_LISTED as i64],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .map_err(db)?;
    for row in rows {
        let (source, setup, error) = row.map_err(db)?;
        if let Some(&i) = at.get(&ids(source, setup)?) {
            files[i].errors.push(error);
        }
    }
    Ok(JobProgress {
        held,
        failed_sources,
        files,
    })
}
