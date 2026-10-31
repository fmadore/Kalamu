//! Snapshots of corrected text, kept apart from the jobs they were taken from: a later analysis
//! still knows what it read after the earlier job is deleted.

use kalamu_core::{Snapshot, StoreError};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    data TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
"#;

pub(crate) fn put(store: &SqliteStore, snapshot: &Snapshot) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "INSERT OR IGNORE INTO snapshots (id, job_id, data, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                snapshot.id,
                snapshot.job_id.to_string(),
                to_json(snapshot)?,
                snapshot.created_at.0,
            ],
        )
        .map_err(db)?;
    Ok(())
}

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<Snapshot, StoreError> {
    let data: Option<String> = store
        .conn()
        .query_row("SELECT data FROM snapshots WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(db)?;
    serde_json::from_str(&data.ok_or(StoreError::NotFound)?).map_err(corrupt)
}
