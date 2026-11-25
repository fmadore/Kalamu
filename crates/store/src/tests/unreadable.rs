//! One job record this version cannot read costs that row alone: the list shows the others,
//! and the one left out is named.

use super::*;

#[test]
fn a_job_that_cannot_be_read_is_named_and_the_others_listed() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let (mut kept, mut newer, mut odd) = (job(), job(), job());
    kept.name = "Kept".into();
    newer.name = "From a newer version".into();
    odd.name = "Odd status".into();
    for job in [&kept, &newer, &odd] {
        store.create_job(job)?;
    }
    // A record a newer version wrote, with a task this one does not know; a status it does not
    // know either.
    store
        .conn()
        .execute(
            "UPDATE jobs SET data = json_set(data, '$.task', 'divination') WHERE id = ?1",
            [newer.id.to_string()],
        )
        .map_err(db)?;
    store
        .conn()
        .execute(
            "UPDATE jobs SET status = 'dreaming' WHERE id = ?1",
            [odd.id.to_string()],
        )
        .map_err(db)?;

    let listed: Vec<String> = store.list_jobs()?.into_iter().map(|j| j.name).collect();
    assert_eq!(listed, ["Kept"]);
    let mut unreadable = store.unreadable_jobs()?;
    unreadable.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(
        unreadable
            .iter()
            .map(|u| (u.id.clone(), u.name.as_str()))
            .collect::<Vec<_>>(),
        [
            (newer.id.to_string(), "From a newer version"),
            (odd.id.to_string(), "Odd status"),
        ]
    );
    assert!(unreadable.iter().all(|u| !u.reason.is_empty()));
    // The record is kept as it is.
    let data: String = store
        .conn()
        .query_row(
            "SELECT data FROM jobs WHERE id = ?1",
            [newer.id.to_string()],
            |row| row.get(0),
        )
        .map_err(db)?;
    assert!(data.contains("divination"));
    Ok(())
}
