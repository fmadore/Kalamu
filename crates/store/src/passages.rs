//! Passages the researcher asked a model to correct from the review: each recorded before its
//! call, after it, and once the researcher decides, beside the job whose text it was.

use kalamu_core::{Correction, JobId, PassageProposal, SetupId, SourceId, StoreError};
use rusqlite::params;

use super::{SqliteStore, db, proposals, to_json};

pub(crate) const TABLE: &str = "passages";
/// The kind, as an unreadable record's error names it.
const WHAT: &str = "passage";

/// The passages, their status and cost kept in columns of their own: what a job spent adds
/// up the costs, and the cache's pruning keeps the answers they name.
pub(crate) const MIGRATION: &str = r#"
CREATE TABLE passages (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    status TEXT NOT NULL,
    cache_key TEXT NOT NULL,
    cost_usd REAL,
    created_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX passages_by_piece ON passages(job_id, setup_id, source_id, idx);
"#;

pub(crate) fn put(
    store: &SqliteStore,
    proposal: &PassageProposal,
    correction: Option<&Correction>,
) -> Result<(), StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    let ask = &proposal.ask;
    tx.execute(
        "INSERT INTO passages
            (id, job_id, setup_id, source_id, idx, status, cache_key, cost_usd, created_at, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT (id) DO UPDATE SET
            status = excluded.status, cache_key = excluded.cache_key,
            cost_usd = excluded.cost_usd, data = excluded.data",
        params![
            ask.id,
            ask.job_id.to_string(),
            ask.setup_id.to_string(),
            ask.source_id.to_string(),
            ask.index,
            proposal.status.as_str(),
            proposal.cache_key,
            proposal.cost_usd,
            proposal.asked_at.0,
            to_json(proposal)?,
        ],
    )
    .map_err(db)?;
    if let Some(correction) = correction {
        super::results::put_correction_on(&tx, correction)?;
    }
    tx.commit().map_err(db)
}

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<PassageProposal, StoreError> {
    proposals::get(store, TABLE, WHAT, id)
}

pub(crate) fn of_source(
    store: &SqliteStore,
    job: JobId,
    setup: SetupId,
    source: SourceId,
) -> Result<Vec<PassageProposal>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare(
            "SELECT data, status FROM passages
             WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3
             ORDER BY created_at, rowid",
        )
        .map_err(db)?;
    proposals::all(
        &mut stmt,
        params![job.to_string(), setup.to_string(), source.to_string()],
        WHAT,
    )
}
