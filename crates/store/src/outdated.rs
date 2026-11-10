//! Files older than what is stored: a source's files under one set-up, marked when a correction
//! or speakers' names are saved and cleared once they are written again. Deleting the job
//! deletes its marks.

use kalamu_core::{JobId, OutdatedFiles, SetupId, SourceId, StoreError};
use rusqlite::{Connection, params};

use super::{SqliteStore, corrupt, db};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE outdated_files (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    generation INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (job_id, setup_id, source_id)
);
"#;

/// Marks a source's files under one set-up out of date, on the connection or transaction that
/// stores what made them so; a mark already there has its generation raised.
pub(crate) fn mark(
    conn: &Connection,
    job: JobId,
    setup: SetupId,
    source: SourceId,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO outdated_files (job_id, setup_id, source_id) VALUES (?1, ?2, ?3)
         ON CONFLICT (job_id, setup_id, source_id) DO UPDATE SET generation = generation + 1",
        params![job.to_string(), setup.to_string(), source.to_string()],
    )
    .map_err(db)?;
    Ok(())
}

pub(crate) fn of(
    store: &SqliteStore,
    job: Option<JobId>,
) -> Result<Vec<OutdatedFiles>, StoreError> {
    let conn = store.conn();
    let select = "SELECT job_id, setup_id, source_id, generation FROM outdated_files";
    let mut stmt = match job {
        Some(_) => conn.prepare(&format!("{select} WHERE job_id = ?1 ORDER BY rowid")),
        None => conn.prepare(&format!("{select} ORDER BY rowid")),
    }
    .map_err(db)?;
    let read = |row: &rusqlite::Row<'_>| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
        ))
    };
    let rows = match job {
        Some(job) => stmt.query_map([job.to_string()], read),
        None => stmt.query_map([], read),
    }
    .map_err(db)?;
    let mut out = Vec::new();
    for row in rows {
        let (job, setup, source, generation) = row.map_err(db)?;
        out.push(OutdatedFiles {
            job_id: job.parse().map_err(corrupt)?,
            setup_id: setup.parse().map_err(corrupt)?,
            source_id: source.parse().map_err(corrupt)?,
            generation,
        });
    }
    Ok(out)
}

pub(crate) fn clear(store: &SqliteStore, files: &OutdatedFiles) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "DELETE FROM outdated_files
             WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3 AND generation = ?4",
            params![
                files.job_id.to_string(),
                files.setup_id.to_string(),
                files.source_id.to_string(),
                files.generation,
            ],
        )
        .map_err(db)?;
    Ok(())
}
