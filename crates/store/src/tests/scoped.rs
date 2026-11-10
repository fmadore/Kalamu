//! What a large job needs from the store (#143): one source's rows without the rest of the
//! job, costs read without the answers, and a result cache that can be pruned.

use kalamu_core::{Charge, SpeakerNames, Usage};

use super::*;

/// A job of two sources read by two set-ups.
fn two_by_two() -> Job {
    let mut job = job();
    let mut second = job.sources[0].clone();
    second.id = SourceId::new();
    second.file_name = "letter 2.pdf".into();
    job.sources.push(second);
    let mut other = job.setups[0].clone();
    other.id = SetupId::new();
    other.label = "pro".into();
    job.setups.push(other);
    job
}

fn at(job: &Job, setup: usize, source: usize, index: u32) -> PieceState {
    PieceState {
        setup_id: job.setups[setup].id,
        source_id: job.sources[source].id,
        cache_key: Some(format!("key-{setup}-{source}-{index}")),
        ..piece(job, index, PieceStatus::Done)
    }
}

fn result(key: &str) -> StoredResult {
    StoredResult {
        cache_key: key.into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput {
            text: "Bamako, le 3 mai 1957. ".repeat(400),
            ..ProcessOutput::default()
        },
        request: Vec::new(),
        cost_usd: Some(0.001),
        duration_ms: 1200,
        created_at: Timestamp::now(),
    }
}

#[test]
fn one_source_s_rows_are_read_without_the_rest_of_the_job() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = two_by_two();
    store.create_job(&job)?;
    let mut pieces = Vec::new();
    for setup in 0..2 {
        for source in 0..2 {
            // Recorded out of order: they come back in their order.
            for index in [2, 0, 1] {
                pieces.push(at(&job, setup, source, index));
            }
        }
    }
    store.upsert_pieces(&pieces)?;
    for (setup, source, index) in [(0, 1, 0), (1, 1, 2), (1, 0, 0)] {
        store.put_correction(&Correction {
            job_id: job.id,
            setup_id: job.setups[setup].id,
            source_id: job.sources[source].id,
            index,
            text: Some(format!("corrected {setup} {source} {index}")),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })?;
    }
    for (cost, source) in [(0.1, 1), (0.2, 0), (0.3, 1)] {
        store.add_charge(&Charge {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[source].id,
            index: 0,
            usage: Usage::default(),
            cost_usd: Some(cost),
            created_at: Timestamp::now(),
        })?;
    }
    for source in 0..2 {
        store.put_speaker_names(
            job.id,
            &SpeakerNames {
                setup_id: job.setups[1].id,
                source_id: job.sources[source].id,
                names: [("Speaker 1".to_string(), format!("Imam {source}"))].into(),
            },
        )?;
    }

    let (setup, source) = (job.setups[1].id, job.sources[1].id);
    let mine = store.source_pieces(job.id, setup, source)?;
    assert_eq!(
        mine.iter().map(|p| p.index).collect::<Vec<_>>(),
        [0, 1, 2],
        "in their order"
    );
    assert!(
        mine.iter()
            .all(|p| p.setup_id == setup && p.source_id == source)
    );
    let whole: Vec<_> = store
        .pieces(job.id)?
        .into_iter()
        .filter(|p| p.setup_id == setup && p.source_id == source)
        .collect();
    assert_eq!(mine, whole, "the same rows the whole job's list holds");

    let corrections = store.source_corrections(job.id, setup, source)?;
    assert_eq!(corrections.len(), 1);
    assert_eq!(corrections[0].text.as_deref(), Some("corrected 1 1 2"));

    let charges = store.source_charges(job.id, job.setups[0].id, source)?;
    assert_eq!(
        charges.iter().map(|c| c.cost_usd).collect::<Vec<_>>(),
        [Some(0.1), Some(0.3)],
        "in the order they were billed"
    );
    assert!(store.source_charges(job.id, setup, source)?.is_empty());

    let names = store.source_speaker_names(job.id, setup, source)?;
    assert_eq!(names.len(), 1);
    assert_eq!(names[0].names["Speaker 1"], "Imam 1");
    assert!(
        store
            .source_speaker_names(job.id, job.setups[0].id, source)?
            .is_empty()
    );
    Ok(())
}

/// The job list and a job's cost read each result's cost and cut-off mark from an index,
/// never through the answer stored before them in the row.
#[test]
fn costs_are_read_without_the_answers() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let conn = store.conn();
    for query in [crate::jobs::LIST_JOBS, crate::jobs::SPENT] {
        let mut stmt = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {query}"))
            .map_err(db)?;
        let job = vec!["j"; stmt.parameter_count()];
        let plan: Vec<String> = stmt
            .query_map(rusqlite::params_from_iter(job), |row| row.get(3))
            .map_err(db)?
            .collect::<Result<_, _>>()
            .map_err(db)?;
        assert!(
            plan.iter()
                .any(|step| step.contains("COVERING INDEX results_costs")),
            "{plan:?}"
        );
    }
    Ok(())
}

#[test]
fn pruning_deletes_only_results_no_piece_refers_to() -> Result<(), StoreError> {
    let dir = tempfile::tempdir().map_err(|e| StoreError::Database(e.to_string()))?;
    let store = SqliteStore::open(&dir.path().join("kalamu.db"))?;
    let kept = two_by_two();
    let deleted = two_by_two();
    store.create_job(&kept)?;
    store.create_job(&deleted)?;
    // A piece done, one still waiting for its answer, one with no key yet, and the deleted
    // job's pieces, one of them under a key the kept job shares.
    let done = at(&kept, 0, 0, 0);
    let waiting = PieceState {
        status: PieceStatus::Submitted,
        ..at(&kept, 0, 1, 0)
    };
    let unkeyed = PieceState {
        cache_key: None,
        status: PieceStatus::Pending,
        ..at(&kept, 1, 0, 0)
    };
    store.upsert_pieces(&[done.clone(), waiting.clone(), unkeyed])?;
    let shared = PieceState {
        job_id: deleted.id,
        ..at(&deleted, 0, 0, 0)
    };
    let shared = PieceState {
        cache_key: done.cache_key.clone(),
        ..shared
    };
    let own = PieceState {
        job_id: deleted.id,
        ..at(&deleted, 1, 1, 0)
    };
    store.upsert_pieces(&[shared, own.clone()])?;
    let keys = |p: &PieceState| p.cache_key.clone().unwrap_or_default();
    for key in [keys(&done), keys(&waiting), keys(&own), "stray".into()] {
        store.put_result(&result(&key))?;
    }

    // While the job is there its results stay; only the stray one goes.
    assert_eq!(store.prune_results()?.results, 1);
    store.delete_job(deleted.id)?;
    let pruned = store.prune_results()?;
    assert_eq!(pruned.results, 1, "the deleted job's own result");
    assert!(pruned.bytes > 0, "{pruned:?}");
    assert!(store.cached_result(&CacheKey(keys(&own)))?.is_none());
    assert!(store.cached_result(&CacheKey(keys(&done)))?.is_some());
    assert!(store.cached_result(&CacheKey(keys(&waiting)))?.is_some());

    // Nothing left to prune: nothing is deleted again.
    assert_eq!(store.prune_results()?.results, 0);
    Ok(())
}
