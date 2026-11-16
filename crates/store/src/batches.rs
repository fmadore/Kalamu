//! Batches sent in economy mode, kept so their answers are collected after a restart, and so
//! what they left in the provider's storage is deleted, even once their job is deleted.

use kalamu_core::{BatchRecord, JobId, StoreError};
use rusqlite::{Transaction, params};

use super::{SqliteStore, corrupt, db, to_json};

/// Batches whose job was deleted while their files at the provider were still to delete: kept
/// apart from the jobs, which `batches` follows when they are deleted, until the cleanup ends.
pub(crate) const LEFTOVERS: &str = r#"
CREATE TABLE batch_leftovers (
    id TEXT PRIMARY KEY,
    data TEXT NOT NULL
);
"#;

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

pub(crate) fn batch_leftovers(store: &SqliteStore) -> Result<Vec<BatchRecord>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare("SELECT data FROM batch_leftovers ORDER BY id")
        .map_err(db)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(db)?;
    rows.map(|row| serde_json::from_str(&row.map_err(db)?).map_err(corrupt))
        .collect()
}

pub(crate) fn put_batch_leftover(
    store: &SqliteStore,
    batch: &BatchRecord,
) -> Result<(), StoreError> {
    let conn = store.conn();
    if batch.cleanup_pending() {
        conn.execute(
            "INSERT OR REPLACE INTO batch_leftovers (id, data) VALUES (?1, ?2)",
            params![batch.id, to_json(batch)?],
        )
    } else {
        conn.execute("DELETE FROM batch_leftovers WHERE id = ?1", [&batch.id])
    }
    .map_err(db)?;
    Ok(())
}

/// Within a job's deletion, keeps the batches whose files are still to delete at the provider.
pub(crate) fn keep_leftovers(tx: &Transaction<'_>, job: JobId) -> Result<(), StoreError> {
    let datas = {
        let mut stmt = tx
            .prepare("SELECT data FROM batches WHERE job_id = ?1")
            .map_err(db)?;
        stmt.query_map([job.to_string()], |row| row.get::<_, String>(0))
            .map_err(db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db)?
    };
    for data in datas {
        let batch: BatchRecord = serde_json::from_str(&data).map_err(corrupt)?;
        if batch.cleanup_pending() {
            tx.execute(
                "INSERT OR REPLACE INTO batch_leftovers (id, data) VALUES (?1, ?2)",
                params![batch.id, data],
            )
            .map_err(db)?;
        }
    }
    Ok(())
}
