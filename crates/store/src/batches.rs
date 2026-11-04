//! Batches sent in economy mode, kept so their answers are collected after a restart.

use kalamu_core::{BatchRecord, JobId, StoreError};
use rusqlite::params;

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) fn put_batch(store: &SqliteStore, batch: &BatchRecord) -> Result<(), StoreError> {
    store
        .conn()
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

pub(crate) fn batches(store: &SqliteStore, job: JobId) -> Result<Vec<BatchRecord>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare("SELECT data FROM batches WHERE job_id = ?1 ORDER BY submitted_at, id")
        .map_err(db)?;
    let rows = stmt
        .query_map([job.to_string()], |row| row.get::<_, String>(0))
        .map_err(db)?;
    rows.map(|row| serde_json::from_str(&row.map_err(db)?).map_err(corrupt))
        .collect()
}
