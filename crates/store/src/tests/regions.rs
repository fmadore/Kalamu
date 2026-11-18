//! Regions drawn over a scan: kept, changed, removed and put back by their id, each change
//! marking the source's files out of date, and deleted with their job.

use kalamu_core::{AdjustedRegion, DrawnRegion, PageHalf};

use super::*;

fn region(job: &Job, id: &str, page: u32, created: i64) -> DrawnRegion {
    DrawnRegion {
        id: id.into(),
        job_id: job.id,
        source_id: job.sources[0].id,
        page,
        half: None,
        x: 0.1,
        y: 0.2,
        width: 0.3,
        height: 0.05,
        text: "note en marge".into(),
        adjusts: None,
        source_hash: "abc".into(),
        created_at: Timestamp(created),
        updated_at: Timestamp(created),
    }
}

#[test]
fn a_region_is_kept_changed_and_listed_by_page() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let second_page = region(&job, "d2", 2, 1);
    let mut first = region(&job, "d1", 1, 5);
    store.put_region(&second_page)?;
    store.put_region(&first)?;
    assert_eq!(store.region("d1")?, first);
    // Changed in place: still one region, its new box and text kept.
    first.x = 0.4;
    first.text = String::new();
    first.half = Some(PageHalf::Right);
    first.adjusts = Some(AdjustedRegion {
        result: "key-0".into(),
        region: "r1_l2".into(),
    });
    first.updated_at = Timestamp(9);
    store.put_region(&first)?;
    let listed = store.source_regions(job.id, job.sources[0].id)?;
    assert_eq!(listed, vec![first.clone(), second_page]);
    assert!(store.source_regions(job.id, SourceId::new())?.is_empty());
    assert!(matches!(store.region("other"), Err(StoreError::NotFound)));
    Ok(())
}

#[test]
fn a_region_removed_is_given_back_and_can_be_put_back() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let drawn = region(&job, "d1", 1, 1);
    store.put_region(&drawn)?;
    let removed = store.remove_region("d1")?;
    assert_eq!(removed, drawn);
    assert!(matches!(store.region("d1"), Err(StoreError::NotFound)));
    assert!(matches!(
        store.remove_region("d1"),
        Err(StoreError::NotFound)
    ));
    // Undo: the same region, its id and dates, back again.
    store.put_region(&removed)?;
    assert_eq!(store.region("d1")?, drawn);
    Ok(())
}

#[test]
fn saving_or_removing_a_region_marks_the_source_s_files_out_of_date() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    // No piece read yet: no files to be out of date.
    store.put_region(&region(&job, "d0", 1, 1))?;
    assert!(store.outdated_files(Some(job.id))?.is_empty());
    store.upsert_piece(&piece(&job, 0, PieceStatus::Done))?;
    store.put_region(&region(&job, "d1", 1, 1))?;
    let marked = store.outdated_files(Some(job.id))?;
    assert_eq!(marked.len(), 1);
    assert_eq!(
        (marked[0].setup_id, marked[0].source_id),
        (job.setups[0].id, job.sources[0].id)
    );
    store.clear_outdated(&marked[0])?;
    assert!(store.outdated_files(Some(job.id))?.is_empty());
    store.remove_region("d1")?;
    assert_eq!(store.outdated_files(Some(job.id))?.len(), 1);
    Ok(())
}

#[test]
fn deleting_a_job_deletes_its_regions() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.put_region(&region(&job, "d1", 1, 1))?;
    store.delete_job(job.id)?;
    assert!(matches!(store.region("d1"), Err(StoreError::NotFound)));
    assert!(store.source_regions(job.id, job.sources[0].id)?.is_empty());
    Ok(())
}
