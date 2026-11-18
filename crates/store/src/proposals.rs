//! What the tables of requests made from the review keep alike (the passages corrected, the
//! regions read again): each row's record as JSON in `data` with its status in a column of its
//! own, the one a restart changes. Each table keeps the columns its kind is found by.

use kalamu_core::{Proposal, ProposalStatus, StoreError};
use rusqlite::{Connection, OptionalExtension, Params, Statement};
use serde::de::DeserializeOwned;

use super::{SqliteStore, corrupt, db};

/// Every table of requests made from the review. What a job spent adds up their costs
/// ([`crate::jobs::SPENT`], [`crate::jobs::LIST_JOBS`]), the cache's pruning keeps the answers
/// they name ([`crate::jobs::PRUNE_RESULTS`]), and a restart cancels what still waited.
pub(crate) const TABLES: [&str; 2] = [super::passages::TABLE, super::region_readings::TABLE];

/// A record as kept, its status read from its column; `what` names the kind in the error.
fn read<T: DeserializeOwned + Proposal>(
    data: String,
    status: String,
    what: &str,
) -> Result<T, StoreError> {
    let mut proposal: T = serde_json::from_str(&data).map_err(corrupt)?;
    *proposal.lifecycle().status = ProposalStatus::parse(&status)
        .ok_or_else(|| corrupt(format!("unknown {what} status {status}")))?;
    Ok(proposal)
}

/// One record of `table`, one of [`TABLES`], by its id.
pub(crate) fn get<T: DeserializeOwned + Proposal>(
    store: &SqliteStore,
    table: &'static str,
    what: &str,
    id: &str,
) -> Result<T, StoreError> {
    let row: Option<(String, String)> = store
        .conn()
        .query_row(
            &format!("SELECT data, status FROM {table} WHERE id = ?1"),
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db)?;
    let (data, status) = row.ok_or(StoreError::NotFound)?;
    read(data, status, what)
}

/// The records a statement selecting `data, status` finds.
pub(crate) fn all<T: DeserializeOwned + Proposal>(
    stmt: &mut Statement<'_>,
    params: impl Params,
    what: &str,
) -> Result<Vec<T>, StoreError> {
    let rows = stmt
        .query_map(params, |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(db)?;
    rows.map(|r| {
        r.map_err(db)
            .and_then(|(data, status)| read(data, status, what))
    })
    .collect()
}

/// At a start: every request still waiting for its model when Kalamu closed waits no longer.
pub(crate) fn cancel_waiting(conn: &Connection) -> Result<(), StoreError> {
    for table in TABLES {
        conn.execute(
            &format!("UPDATE {table} SET status = 'cancelled' WHERE status = 'asked'"),
            [],
        )
        .map_err(db)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TABLES;
    use crate::jobs::{LIST_JOBS, PRUNE_RESULTS, SPENT};

    #[test]
    fn every_table_of_requests_is_costed_and_its_answers_kept() {
        for table in TABLES {
            let named = format!("FROM {table} WHERE job_id = ");
            assert!(LIST_JOBS.contains(&named), "{table} missing from LIST_JOBS");
            assert!(SPENT.contains(&named), "{table} missing from SPENT");
            let kept = format!("NOT IN (SELECT cache_key FROM {table})");
            assert!(PRUNE_RESULTS.contains(&kept), "{table} not kept by pruning");
        }
    }
}
