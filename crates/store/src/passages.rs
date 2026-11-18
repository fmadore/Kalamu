//! Passages the researcher asked a model to correct from the review: each recorded before its
//! call, after it, and once the researcher decides, beside the job whose text it was.

use kalamu_core::{
    Correction, JobId, PassageProposal, ProposalStatus, SetupId, SourceId, StoreError,
};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db, to_json};

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

/// A proposal as kept, its status read from its column: the one a restart changes.
fn read(data: String, status: String) -> Result<PassageProposal, StoreError> {
    let mut proposal: PassageProposal = serde_json::from_str(&data).map_err(corrupt)?;
    proposal.status = ProposalStatus::parse(&status)
        .ok_or_else(|| corrupt(format!("unknown passage status {status}")))?;
    Ok(proposal)
}

pub(crate) fn get(store: &SqliteStore, id: &str) -> Result<PassageProposal, StoreError> {
    let row: Option<(String, String)> = store
        .conn()
        .query_row(
            "SELECT data, status FROM passages WHERE id = ?1",
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
    let rows = stmt
        .query_map(
            params![job.to_string(), setup.to_string(), source.to_string()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(db)?;
    rows.map(|r| r.map_err(db).and_then(|(data, status)| read(data, status)))
        .collect()
}
