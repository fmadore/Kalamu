//! Document units (#50): a table of contents' articles and items as pages of its source, kept
//! beside the job's results and never in them. They go with their job; the copies of their
//! pages that later jobs read are snapshots, which stay.

use kalamu_core::{DocumentUnit, JobId, StoreError, UnitId};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE units (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX units_by_job ON units(job_id);
"#;

pub(crate) fn put(store: &SqliteStore, unit: &DocumentUnit) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "INSERT INTO units (id, job_id, position, data) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET position = excluded.position, data = excluded.data",
            params![
                unit.id.to_string(),
                unit.job_id.to_string(),
                unit.position,
                to_json(unit)?,
            ],
        )
        .map_err(db)?;
    Ok(())
}

pub(crate) fn get(store: &SqliteStore, id: UnitId) -> Result<DocumentUnit, StoreError> {
    let data: Option<String> = store
        .conn()
        .query_row(
            "SELECT data FROM units WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    serde_json::from_str(&data.ok_or(StoreError::NotFound)?).map_err(corrupt)
}

pub(crate) fn of_job(store: &SqliteStore, job: JobId) -> Result<Vec<DocumentUnit>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare("SELECT data FROM units WHERE job_id = ?1 ORDER BY position, rowid")
        .map_err(db)?;
    let rows = stmt
        .query_map([job.to_string()], |row| row.get::<_, String>(0))
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    rows.iter()
        .map(|data| serde_json::from_str(data).map_err(corrupt))
        .collect()
}

pub(crate) fn delete(store: &SqliteStore, id: UnitId) -> Result<(), StoreError> {
    store
        .conn()
        .execute("DELETE FROM units WHERE id = ?1", [id.to_string()])
        .map_err(db)?;
    Ok(())
}
