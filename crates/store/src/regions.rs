//! Regions the researcher drew over a scan, or whose box they adjusted: a layer of its own
//! beside the job whose source they are on, never written into a stored result. Saving or
//! removing one marks the source's files out of date, since `provenance.json` lists them.

use kalamu_core::{DrawnRegion, JobId, SourceId, StoreError};
use rusqlite::{Connection, OptionalExtension, params};

use super::{SqliteStore, corrupt, db, outdated, to_json};

/// The regions, by job and source, their page in a column of its own to read one source's
/// pages in order.
pub(crate) const MIGRATION: &str = r#"
CREATE TABLE drawn_regions (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL,
    page INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX drawn_regions_by_source ON drawn_regions(job_id, source_id, page);
"#;

pub(crate) fn put(store: &SqliteStore, region: &DrawnRegion) -> Result<(), StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    tx.execute(
        "INSERT INTO drawn_regions (id, job_id, source_id, page, created_at, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (id) DO UPDATE SET
            job_id = excluded.job_id, source_id = excluded.source_id, page = excluded.page,
            created_at = excluded.created_at, data = excluded.data",
        params![
            region.id,
            region.job_id.to_string(),
            region.source_id.to_string(),
            region.page,
            region.created_at.0,
            to_json(region)?,
        ],
    )
    .map_err(db)?;
    mark(&tx, region.job_id, region.source_id)?;
    tx.commit().map_err(db)
}

pub(crate) fn remove(store: &SqliteStore, id: &str) -> Result<DrawnRegion, StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    let region = read_one(&tx, id)?;
    tx.execute("DELETE FROM drawn_regions WHERE id = ?1", [id])
        .map_err(db)?;
    mark(&tx, region.job_id, region.source_id)?;
    tx.commit().map_err(db)?;
    Ok(region)
}

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<DrawnRegion, StoreError> {
    read_one(&store.conn(), id)
}

pub(crate) fn of_source(
    store: &SqliteStore,
    job: JobId,
    source: SourceId,
) -> Result<Vec<DrawnRegion>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare(
            "SELECT data FROM drawn_regions WHERE job_id = ?1 AND source_id = ?2
             ORDER BY page, created_at, rowid",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map(params![job.to_string(), source.to_string()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(db)?;
    rows.map(|r| {
        r.map_err(db)
            .and_then(|data| serde_json::from_str(&data).map_err(corrupt))
    })
    .collect()
}

fn read_one(conn: &Connection, id: &str) -> Result<DrawnRegion, StoreError> {
    let data: Option<String> = conn
        .query_row(
            "SELECT data FROM drawn_regions WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    serde_json::from_str(&data.ok_or(StoreError::NotFound)?).map_err(corrupt)
}

/// Marks the source's files out of date under every set-up that has pieces of it: a region is
/// the source's, shown in each set-up's review, and listed in each one's `provenance.json`.
fn mark(conn: &Connection, job: JobId, source: SourceId) -> Result<(), StoreError> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT setup_id FROM pieces WHERE job_id = ?1 AND source_id = ?2")
        .map_err(db)?;
    let setups = stmt
        .query_map(params![job.to_string(), source.to_string()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    for setup in setups {
        outdated::mark(conn, job, setup.parse().map_err(corrupt)?, source)?;
    }
    Ok(())
}
