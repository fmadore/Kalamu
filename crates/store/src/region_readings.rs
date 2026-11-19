//! Regions the researcher asked a model to read again from the review: each recorded before its
//! call, after it, and once the researcher decides, beside the job whose source it is on.

use kalamu_core::{JobId, ReadingEffect, RegionReading, SourceId, StoreError};
use rusqlite::params;

use super::{SqliteStore, db, proposals, to_json};

pub(crate) const TABLE: &str = "region_readings";
/// The kind, as an unreadable record's error names it.
const WHAT: &str = "region reading";

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

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<RegionReading, StoreError> {
    proposals::get(store, TABLE, WHAT, id)
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
    proposals::all(
        &mut stmt,
        params![job.to_string(), source.to_string()],
        WHAT,
    )
}
