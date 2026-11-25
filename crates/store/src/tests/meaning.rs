//! The vector index and the record of the search by meaning: stamps and contents, spaces kept
//! apart, removal with a job or a space, and each project's model, consent, questions and runs.

#![allow(clippy::unwrap_used)]

use kalamu_core::{
    ChunkVector, EmbeddingLog, EmbeddingRun, EmbeddingRunState, EmbeddingSpace, PassageKey,
    Project, ProjectId, ProjectStore, VectorIndex,
};

use super::*;
use crate::SqliteVectors;
use crate::vectors::{from_blob, to_blob};

fn space(model: &str, dimensions: u32) -> EmbeddingSpace {
    EmbeddingSpace {
        provider: ProviderId::new("gemini"),
        model: model.into(),
        dimensions,
    }
}

fn key(job: JobId, index: u32) -> PassageKey {
    PassageKey {
        job_id: job,
        setup_id: SetupId::new(),
        source_id: SourceId::new(),
        index,
    }
}

fn chunk(key: PassageKey, chunk: u32, of: u32, content: &str, vector: Vec<f32>) -> ChunkVector {
    ChunkVector {
        key,
        chunk,
        of,
        stamp: format!("stamp-{content}"),
        content: content.into(),
        start: chunk * 10,
        end: chunk * 10 + 12,
        vector,
    }
}

#[test]
fn vectors_round_trip_as_little_endian_floats() {
    let v = vec![0.5f32, -1.25, 3.0e-7];
    let blob = to_blob(&v);
    assert_eq!(blob.len(), 12);
    assert_eq!(&blob[..4], &0.5f32.to_le_bytes());
    assert_eq!(from_blob(&blob).unwrap(), v);
    assert!(from_blob(&[1, 2, 3]).is_err());
}

#[test]
fn a_new_text_replaces_a_passages_chunks_and_the_same_text_keeps_them() {
    let index = SqliteVectors::in_memory().unwrap();
    let s = space("embed", 3);
    let job = JobId::new();
    let k = key(job, 0);
    let before = index.generation();
    index
        .put(
            &s,
            &[
                chunk(k, 0, 2, "a", vec![1.0, 0.0, 0.0]),
                chunk(k, 1, 2, "a", vec![0.0, 1.0, 0.0]),
            ],
        )
        .unwrap();
    assert!(index.generation() > before);
    assert_eq!(index.stamps(&s, job).unwrap().len(), 2);

    // Marked reviewed: a new stamp, the same text, kept.
    index.restamp(&s, &k, "a", "stamp-reviewed").unwrap();
    let stamps = index.stamps(&s, job).unwrap();
    assert!(stamps.iter().all(|c| c.stamp == "stamp-reviewed"));

    // Corrected: one chunk of the new text, the old ones gone.
    index
        .put(&s, &[chunk(k, 0, 1, "b", vec![0.0, 0.0, 1.0])])
        .unwrap();
    let vectors = index.vectors(&s, &[job]).unwrap();
    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].content, "b");
    assert_eq!(vectors[0].vector, vec![0.0, 0.0, 1.0]);
}

#[test]
fn spaces_are_kept_apart_and_removed_one_at_a_time() {
    let index = SqliteVectors::in_memory().unwrap();
    let small = space("embed", 2);
    let large = space("embed", 4);
    let job = JobId::new();
    let other = JobId::new();
    let k = key(job, 0);
    index
        .put(&small, &[chunk(k, 0, 1, "a", vec![1.0, 0.0])])
        .unwrap();
    index
        .put(&large, &[chunk(k, 0, 1, "a", vec![0.0, 1.0, 0.0, 0.0])])
        .unwrap();
    index
        .put(&large, &[chunk(key(other, 0), 0, 1, "c", vec![1.0; 4])])
        .unwrap();
    // A passage with nothing worth embedding: kept, but neither a vector nor counted.
    index
        .put(&small, &[chunk(key(job, 1), 0, 0, "blank", Vec::new())])
        .unwrap();
    assert_eq!(index.vectors(&small, &[job]).unwrap().len(), 1);
    assert_eq!(index.vectors(&large, &[job]).unwrap()[0].vector.len(), 4);
    assert_eq!(index.stamps(&small, job).unwrap().len(), 2);

    let sizes = index.spaces(&[job]).unwrap();
    assert_eq!(sizes.len(), 2);
    assert_eq!(sizes[0].space, small);
    assert_eq!((sizes[0].chunks, sizes[0].bytes), (1, 8));
    assert_eq!((sizes[1].chunks, sizes[1].bytes), (1, 16));

    index.remove_space(&small, &[job]).unwrap();
    assert!(index.stamps(&small, job).unwrap().is_empty());
    assert_eq!(index.vectors(&large, &[job, other]).unwrap().len(), 2);

    index.forget_job(job).unwrap();
    assert_eq!(index.indexed_jobs().unwrap(), vec![other]);
    index.remove_passages(&[key(other, 0)]).unwrap();
    // Another key of the same job: the source and set-up differ, nothing removed.
    assert_eq!(index.indexed_jobs().unwrap(), vec![other]);
}

#[test]
fn an_index_file_of_another_version_is_emptied() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vectors.sqlite");
    {
        let index = SqliteVectors::open(&path).unwrap();
        index
            .put(
                &space("m", 1),
                &[chunk(key(JobId::new(), 0), 0, 1, "a", vec![1.0])],
            )
            .unwrap();
    }
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
    }
    let index = SqliteVectors::open(&path).unwrap();
    assert!(index.indexed_jobs().unwrap().is_empty());
}

#[test]
fn an_index_file_that_is_no_database_is_set_aside_and_made_anew() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vectors.sqlite");
    let aside = dir.path().join("vectors.sqlite.unreadable");
    std::fs::write(&aside, b"set aside before").unwrap();
    let bytes = b"not a database at all, only bytes that look like nothing";
    std::fs::write(&path, bytes).unwrap();
    let index = SqliteVectors::open(&path).unwrap();
    assert!(index.indexed_jobs().unwrap().is_empty());
    index
        .put(
            &space("m", 1),
            &[chunk(key(JobId::new(), 0), 0, 1, "a", vec![1.0])],
        )
        .unwrap();
    drop(index);
    // The unreadable file is kept beside the new one, in place of the one set aside before.
    assert_eq!(std::fs::read(&aside).unwrap(), bytes);
    assert_eq!(
        SqliteVectors::open(&path)
            .unwrap()
            .indexed_jobs()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn an_index_file_that_fails_to_open_for_another_reason_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vectors.sqlite");
    {
        let index = SqliteVectors::open(&path).unwrap();
        index
            .put(
                &space("m", 1),
                &[chunk(key(JobId::new(), 0), 0, 1, "a", vec![1.0])],
            )
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    // Its log cannot be opened: a passing fault, not a damaged index.
    let log = dir.path().join("vectors.sqlite-wal");
    let _ = std::fs::remove_file(&log);
    std::fs::create_dir(&log).unwrap();
    assert!(SqliteVectors::open(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!dir.path().join("vectors.sqlite.unreadable").exists());
    // Once the fault is gone, the vectors are there.
    std::fs::remove_dir(&log).unwrap();
    assert_eq!(
        SqliteVectors::open(&path)
            .unwrap()
            .indexed_jobs()
            .unwrap()
            .len(),
        1
    );
}

fn project(store: &SqliteStore) -> ProjectId {
    let project = Project {
        id: ProjectId::new(),
        name: "Presse".into(),
        description: String::new(),
        archived: false,
        source_defaults: Default::default(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    };
    store.put_project(&project).unwrap();
    project.id
}

#[test]
fn a_projects_model_consent_questions_and_runs_are_recorded() {
    let store = SqliteStore::in_memory().unwrap();
    let p = project(&store);
    assert!(store.current_space(p).unwrap().is_none());
    let cloud = space("embed", 768);
    let local = EmbeddingSpace {
        provider: ProviderId::new("offline"),
        model: "gemma".into(),
        dimensions: 768,
    };

    // Consent needs the model chosen first.
    assert!(matches!(
        store.consent(p, &cloud, Timestamp(5)),
        Err(StoreError::NotFound)
    ));
    let chosen = store.choose_space(p, &cloud, Timestamp(10)).unwrap();
    assert_eq!(chosen.space, cloud);
    assert!(chosen.consented_at.is_none());
    store.consent(p, &cloud, Timestamp(11)).unwrap();
    store.add_query(p, &cloud, 7, Some(0.001)).unwrap();
    store.add_query(p, &cloud, 3, Some(0.0005)).unwrap();

    // Another model chosen in the same millisecond is the current one; going back keeps the
    // first one's consent and cost.
    assert_eq!(
        store.choose_space(p, &local, Timestamp(10)).unwrap().space,
        local
    );
    let back = store.choose_space(p, &cloud, Timestamp(10)).unwrap();
    assert_eq!(back.consented_at, Some(Timestamp(11)));
    assert_eq!((back.queries, back.query_tokens), (2, 10));
    assert!((back.query_cost_usd.unwrap() - 0.0015).abs() < 1e-12);

    let mut run = EmbeddingRun {
        id: 0,
        project_id: p,
        space: cloud.clone(),
        state: EmbeddingRunState::Running,
        chunks: 0,
        tokens: 0,
        cost_usd: None,
        started_at: Timestamp(20),
        ended_at: None,
        error: None,
    };
    run.id = store.start_run(&run).unwrap();
    run.chunks = 40;
    run.tokens = 9000;
    run.cost_usd = Some(0.0135);
    store.update_run(&run).unwrap();
    let mut second = run.clone();
    second.id = store.start_run(&second).unwrap();
    assert_eq!(store.mark_running_runs_interrupted().unwrap(), 2);
    let runs = store.embedding_runs(p).unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].id, second.id);
    assert_eq!(runs[1].chunks, 40);
    assert_eq!(runs[1].tokens, 9000);
    assert!(
        runs.iter()
            .all(|r| r.state == EmbeddingRunState::Interrupted)
    );

    // Deleting the project deletes its record.
    store.delete_project(p).unwrap();
    assert!(store.embedding_runs(p).unwrap().is_empty());
    assert!(store.current_space(p).unwrap().is_none());
}
