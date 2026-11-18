//! Regions the researcher asked a model to read again from the review: each recorded before its
//! call, after it, and once the researcher decides, beside the job whose source it is on.

use kalamu_core::{JobId, ProposalStatus, ReadingEffect, RegionReading, SourceId, StoreError};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

/// The readings, their status and cost kept in columns of their own: what a job spent adds up
/// the costs, and the cache's pruning keeps the answers they name.
pub(crate) const MIGRATION: &str = r#"
CREATE TABLE region_readings (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL,
    page INTEGER NOT NULL,
    status TEXT NOT NULL,
    cache_key TEXT NOT NULL,
    cost_usd REAL,
    created_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX region_readings_by_source ON region_readings(job_id, source_id, page);
"#;

pub(crate) fn put(
    store: &SqliteStore,
    reading: &RegionReading,
    effect: Option<&ReadingEffect>,
) -> Result<(), StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    let ask = &reading.ask;
    tx.execute(
        "INSERT INTO region_readings
            (id, job_id, source_id, page, status, cache_key, cost_usd, created_at, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT (id) DO UPDATE SET
            status = excluded.status, cache_key = excluded.cache_key,
            cost_usd = excluded.cost_usd, data = excluded.data",
        params![
            ask.id,
            ask.job_id.to_string(),
            ask.source_id.to_string(),
            ask.page,
            reading.status.as_str(),
            reading.cache_key,
            reading.cost_usd,
            reading.asked_at.0,
            to_json(reading)?,
        ],
    )
    .map_err(db)?;
    match effect {
        Some(ReadingEffect::Correction(correction)) => {
            super::results::put_correction_on(&tx, correction)?;
        }
        Some(ReadingEffect::Region(region)) => super::regions::put_on(&tx, region)?,
        None => {}
    }
    tx.commit().map_err(db)
}

/// A reading as kept, its status read from its column: the one a restart changes.
fn read(data: String, status: String) -> Result<RegionReading, StoreError> {
    let mut reading: RegionReading = serde_json::from_str(&data).map_err(corrupt)?;
    reading.status = ProposalStatus::parse(&status)
        .ok_or_else(|| corrupt(format!("unknown region reading status {status}")))?;
    Ok(reading)
}

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<RegionReading, StoreError> {
    let row: Option<(String, String)> = store
        .conn()
        .query_row(
            "SELECT data, status FROM region_readings WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db)?;
    let (data, status) = row.ok_or(StoreError::NotFound)?;
    read(data, status)
}

pub(crate) fn of_source(
    store: &SqliteStore,
    job: JobId,
    source: SourceId,
) -> Result<Vec<RegionReading>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare(
            "SELECT data, status FROM region_readings WHERE job_id = ?1 AND source_id = ?2
             ORDER BY page, created_at, rowid",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map(params![job.to_string(), source.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db)?;
    rows.map(|r| r.map_err(db).and_then(|(data, status)| read(data, status)))
        .collect()
}
