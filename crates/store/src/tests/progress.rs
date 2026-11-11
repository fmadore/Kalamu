//! A job's progress counted by the store (#143): the same counts as reading every piece and
//! counting them, from an index rather than the pieces.

use std::collections::{HashMap, HashSet};

use kalamu_core::{ERRORS_LISTED, FileProgress, JobProgress};

use super::*;

/// The counts made the way the engine made them before the store did: every piece of the job
/// read, those of `sources` counted in the order the store gives them.
fn counted(
    store: &SqliteStore,
    job: JobId,
    sources: &[SourceId],
) -> Result<JobProgress, StoreError> {
    let wanted: HashSet<SourceId> = sources.iter().copied().collect();
    let mut progress = JobProgress::default();
    let mut at: HashMap<(SourceId, SetupId), usize> = HashMap::new();
    for p in store.pieces(job)? {
        if p.status == PieceStatus::Submitted {
            progress.held += 1;
        }
        if p.status == PieceStatus::Failed && !progress.failed_sources.contains(&p.source_id) {
            progress.failed_sources.push(p.source_id);
        }
        if !wanted.contains(&p.source_id) {
            continue;
        }
        let i = *at.entry((p.source_id, p.setup_id)).or_insert_with(|| {
            progress.files.push(FileProgress {
                source_id: p.source_id,
                setup_id: p.setup_id,
                total: 0,
                done: 0,
                failed: 0,
                errors: Vec::new(),
            });
            progress.files.len() - 1
        });
        let row = &mut progress.files[i];
        row.total += 1;
        match p.status {
            PieceStatus::Done | PieceStatus::Cached => row.done += 1,
            PieceStatus::Failed => row.failed += 1,
            _ => {}
        }
        if let Some(e) = &p.error
            && row.errors.len() < ERRORS_LISTED
            && !row.errors.contains(e)
        {
            row.errors.push(e.clone());
        }
    }
    Ok(progress)
}

/// Three files under two set-ups, with every status and some errors repeated.
fn three_by_two(store: &SqliteStore) -> Result<Job, StoreError> {
    let mut job = job();
    for name in ["letter 2.pdf", "letter 3.pdf"] {
        let mut more = job.sources[0].clone();
        more.id = SourceId::new();
        more.file_name = name.into();
        job.sources.push(more);
    }
    let mut other = job.setups[0].clone();
    other.id = SetupId::new();
    other.label = "pro".into();
    job.setups.push(other);
    store.create_job(&job)?;
    let statuses = [
        (PieceStatus::Done, None),
        (PieceStatus::Cached, None),
        (PieceStatus::Failed, Some("refused")),
        (PieceStatus::Failed, Some("refused")),
        (PieceStatus::Failed, Some("timed out")),
        (PieceStatus::Submitted, None),
        (PieceStatus::Pending, Some("rate limited, tried again")),
        (PieceStatus::Running, None),
    ];
    let mut pieces = Vec::new();
    for (s, source) in job.sources.iter().enumerate() {
        for (u, setup) in job.setups.iter().enumerate() {
            // Each file and set-up a different number of pieces, recorded out of order.
            let count = (statuses.len() - s - u) as u32;
            for index in (0..count).rev() {
                let (status, error) = statuses[index as usize];
                pieces.push(PieceState {
                    setup_id: setup.id,
                    source_id: source.id,
                    error: error.map(str::to_string),
                    ..piece(&job, index, status)
                });
            }
        }
    }
    store.upsert_pieces(&pieces)?;
    Ok(job)
}

#[test]
fn the_store_counts_what_reading_every_piece_counted() -> Result<(), StoreError> {
    let dir = tempfile::tempdir().map_err(|e| StoreError::Database(e.to_string()))?;
    let on_disk = SqliteStore::open(&dir.path().join("kalamu.db"))?;
    for store in [SqliteStore::in_memory()?, on_disk] {
        let job = three_by_two(&store)?;
        // A fourth file read without a failure.
        let clean = SourceId::new();
        store.upsert_pieces(&[PieceState {
            source_id: clean,
            ..piece(&job, 0, PieceStatus::Done)
        }])?;
        let mut ids: Vec<SourceId> = job.sources.iter().map(|s| s.id).collect();
        let mut failing = ids.clone();
        failing.sort_by_key(|id| id.to_string());
        ids.push(clean);
        for asked in [&ids[..], &ids[..1], &ids[1..], &[]] {
            assert_eq!(
                store.job_progress(job.id, asked)?,
                counted(&store, job.id, asked)?,
                "{} files asked for",
                asked.len()
            );
        }
        let progress = store.job_progress(job.id, &ids)?;
        assert_eq!(progress.held, 5, "held over the whole job");
        assert_eq!(progress.files.len(), 7);
        assert_eq!(progress.failed_sources, failing, "the clean file left out");
        assert_eq!(
            store.job_progress(job.id, &[])?.failed_sources,
            failing,
            "over the whole job, whatever is asked for"
        );
        let first = progress
            .files
            .iter()
            .find(|f| f.source_id == ids[0] && f.setup_id == job.setups[0].id)
            .ok_or(StoreError::NotFound)?;
        assert_eq!((first.total, first.done, first.failed), (8, 2, 3));
        assert_eq!(
            first.errors,
            ["refused", "timed out", "rate limited, tried again"]
        );
        // A file not in the job, and another job's files, are not counted.
        let other = three_by_two(&store)?;
        let progress = store.job_progress(job.id, &[other.sources[0].id, SourceId::new()])?;
        assert!(progress.files.is_empty());
        assert_eq!(progress.held, 5);
        assert_eq!(progress.failed_sources, failing);
    }
    Ok(())
}

#[test]
fn a_file_lists_at_most_a_few_different_errors() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let pieces: Vec<PieceState> = (0..6)
        .rev()
        .map(|i| PieceState {
            error: Some(format!("error {i}")),
            ..piece(&job, i, PieceStatus::Failed)
        })
        .collect();
    store.upsert_pieces(&pieces)?;
    let progress = store.job_progress(job.id, &[job.sources[0].id])?;
    assert_eq!(progress.files[0].failed, 6);
    assert_eq!(
        progress.files[0].errors,
        ["error 0", "error 1", "error 2"],
        "the first ones, in the pieces' order"
    );
    assert_eq!(progress, counted(&store, job.id, &[job.sources[0].id])?);
    Ok(())
}

/// The counts and the pieces held are read from the index alone, never from the pieces' rows.
#[test]
fn progress_is_counted_from_an_index() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let conn = store.conn();
    for query in [
        crate::progress::HELD,
        crate::progress::FAILED,
        crate::progress::COUNTS,
    ] {
        let mut stmt = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {query}"))
            .map_err(db)?;
        let args = vec!["[]"; stmt.parameter_count()];
        let plan: Vec<String> = stmt
            .query_map(rusqlite::params_from_iter(args), |row| row.get(3))
            .map_err(db)?
            .collect::<Result<_, _>>()
            .map_err(db)?;
        assert!(
            plan.iter()
                .any(|step| step.contains("COVERING INDEX pieces_progress")),
            "{plan:?}"
        );
    }
    Ok(())
}
