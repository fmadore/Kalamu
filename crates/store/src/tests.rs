use kalamu_core::{
    BatchItem, BatchPhase, FieldAnswer, FieldValue, InputKind, JobSettings, OutputSettings,
    ProcessOutput, PromptSnapshot, RequestSetting, Setup, SetupOptions, SourceFile, Task,
};

use super::*;
use kalamu_core::{PieceRef, PieceStatus, ProviderId, SetupId, SourceId, Timestamp};

mod progress;
mod projects;
mod runs;
mod scoped;
mod search;

fn job() -> Job {
    Job {
        id: JobId::new(),
        name: "Letters".into(),
        task: Task::TextExtraction,
        created_at: Timestamp::now(),
        sources: vec![SourceFile {
            id: SourceId::new(),
            path: r"C:\archive\letter.pdf".into(),
            file_name: "letter.pdf".into(),
            sha256: "abc".into(),
            size_bytes: 10,
            modified_ms: None,
            kind: InputKind::Pdf,
            page_count: Some(2),
            duration_ms: None,
            split_spreads: false,
            even_out_volume: false,
            title: None,
            derived_from: None,
        }],
        setups: vec![Setup {
            id: SetupId::new(),
            label: "gemini".into(),
            provider: ProviderId::new("gemini"),
            model: "gemini-3.8-flash".into(),
            prompt: PromptSnapshot {
                preset_id: None,
                preset_name: "Printed".into(),
                template: "Transcribe.".into(),
                text: "Transcribe.".into(),
                user_prompt: "Transcribe this page.".into(),
                schema: None,
            },
            options: SetupOptions::default(),
            merge: None,
            pages_cap: None,
        }],
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        status: JobStatus::Pending,
    }
}

fn piece(job: &Job, index: u32, status: PieceStatus) -> PieceState {
    PieceState {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index,
        piece: PieceRef::page(index + 1),
        status,
        cache_key: Some(format!("key-{index}")),
        error: None,
        attempts: 1,
        updated_at: Timestamp::now(),
    }
}

#[test]
fn jobs_round_trip_and_are_listed_with_progress() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    assert_eq!(store.job(job.id)?, job);

    store.upsert_piece(&piece(&job, 0, PieceStatus::Done))?;
    store.upsert_piece(&piece(&job, 1, PieceStatus::Failed))?;
    let listed = store.list_jobs()?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].pieces_total, 2);
    assert_eq!(listed[0].pieces_finished, 1);
    assert_eq!(listed[0].pieces_failed, 1);
    Ok(())
}

/// Pieces recorded together are recorded all or none: one that cannot be written (here, of a
/// job the database does not hold) leaves none of the others behind.
#[test]
fn pieces_recorded_together_are_all_or_none() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut stray = piece(&job, 2, PieceStatus::Pending);
    stray.job_id = JobId::new();
    let refused = store.upsert_pieces(&[
        piece(&job, 0, PieceStatus::Pending),
        piece(&job, 1, PieceStatus::Pending),
        stray,
    ]);
    assert!(refused.is_err());
    assert!(store.pieces(job.id)?.is_empty(), "nothing kept");

    store.upsert_pieces(&[
        piece(&job, 0, PieceStatus::Pending),
        piece(&job, 1, PieceStatus::Done),
    ])?;
    let pieces = store.pieces(job.id)?;
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[1].status, PieceStatus::Done);
    Ok(())
}

/// The Jobs page finds a job by a file it read: each source's name, and a video's title.
#[test]
fn job_sources_name_every_file_and_title() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let older = job();
    let mut newer = job();
    newer.created_at = Timestamp(older.created_at.0 + 1000);
    let mut video = newer.sources[0].clone();
    video.id = SourceId::new();
    video.file_name = "abc123".into();
    video.title = Some("Sermon du vendredi".into());
    newer.sources.push(video);
    store.create_job(&older)?;
    store.create_job(&newer)?;

    let listed = store.job_sources()?;
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].job_id, newer.id);
    assert_eq!(
        listed[0].names,
        ["letter.pdf", "abc123", "Sermon du vendredi"]
    );
    assert_eq!(listed[1].job_id, older.id);
    assert_eq!(listed[1].names, ["letter.pdf"]);
    Ok(())
}

#[test]
fn a_renamed_job_keeps_everything_else() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.upsert_piece(&piece(&job, 0, PieceStatus::Done))?;
    store.rename_job(job.id, "Lettres du gouverneur, 1957")?;

    let renamed = store.job(job.id)?;
    assert_eq!(renamed.name, "Lettres du gouverneur, 1957");
    assert_eq!(renamed.sources, job.sources);
    assert_eq!(renamed.setups, job.setups);
    let listed = store.list_jobs()?;
    assert_eq!(listed[0].name, "Lettres du gouverneur, 1957");
    assert_eq!(listed[0].pieces_finished, 1);
    assert!(matches!(
        store.rename_job(JobId::new(), "x"),
        Err(StoreError::NotFound)
    ));
    Ok(())
}

/// A job has spent what its own pieces cost over all its runs, not what it reused.
#[test]
fn a_job_has_spent_what_its_own_pieces_cost() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    assert_eq!(store.spent(job.id)?, 0.0);
    for (index, status, cost) in [
        (0, PieceStatus::Done, Some(0.01)),
        (1, PieceStatus::Cached, Some(0.02)),
        (2, PieceStatus::Done, Some(0.04)),
        (3, PieceStatus::Done, None),
    ] {
        store.put_result(&StoredResult {
            cache_key: format!("key-{index}"),
            provider: ProviderId::new("gemini"),
            model: "gemini-3.8-flash".into(),
            output: ProcessOutput::default(),
            request: Vec::new(),
            cost_usd: cost,
            duration_ms: 10,
            created_at: Timestamp::now(),
        })?;
        store.upsert_piece(&piece(&job, index, status))?;
    }
    store.upsert_piece(&piece(&job, 4, PieceStatus::Failed))?;
    assert!((store.spent(job.id)? - 0.05).abs() < 1e-12);
    let listed = store.list_jobs()?;
    assert!(listed[0].cost_usd.is_some_and(|c| (c - 0.05).abs() < 1e-12));
    Ok(())
}

/// An answer billed but refused for breaking its fields counts in what the job spent, in the
/// job list too, and goes with the job when it is deleted.
#[test]
fn answers_billed_but_refused_count_in_what_a_job_spent() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    assert_eq!(store.list_jobs()?[0].cost_usd, None);
    let charge = Charge {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: 0,
        usage: kalamu_core::Usage {
            input_tokens: 900,
            output_tokens: 40,
            ..kalamu_core::Usage::default()
        },
        cost_usd: Some(0.003),
        created_at: Timestamp::now(),
    };
    store.add_charge(&charge)?;
    store.add_charge(&Charge {
        cost_usd: None,
        ..charge.clone()
    })?;
    assert!((store.spent(job.id)? - 0.003).abs() < 1e-12);
    assert!(
        store.list_jobs()?[0]
            .cost_usd
            .is_some_and(|c| (c - 0.003).abs() < 1e-12)
    );
    let charges = store.charges(job.id)?;
    assert_eq!(charges.len(), 2);
    assert_eq!(charges[0], charge);

    // Kept results add to it.
    store.put_result(&StoredResult {
        cache_key: "key-1".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput::default(),
        request: Vec::new(),
        cost_usd: Some(0.01),
        duration_ms: 10,
        created_at: Timestamp::now(),
    })?;
    store.upsert_piece(&piece(&job, 1, PieceStatus::Done))?;
    assert!((store.spent(job.id)? - 0.013).abs() < 1e-12);
    assert!(
        store.list_jobs()?[0]
            .cost_usd
            .is_some_and(|c| (c - 0.013).abs() < 1e-12)
    );

    store.delete_job(job.id)?;
    assert!(store.charges(job.id)?.is_empty());
    Ok(())
}

#[test]
fn piece_updates_replace_the_previous_state() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.upsert_piece(&piece(&job, 0, PieceStatus::Running))?;
    store.upsert_piece(&piece(&job, 0, PieceStatus::Done))?;
    let pieces = store.pieces(job.id)?;
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].status, PieceStatus::Done);
    Ok(())
}

#[test]
fn startup_marks_running_jobs_and_pieces_as_interrupted() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.set_job_status(job.id, JobStatus::Running)?;
    store.upsert_piece(&piece(&job, 0, PieceStatus::Running))?;

    assert_eq!(store.mark_running_as_interrupted()?, vec![job.id]);
    assert_eq!(store.job(job.id)?.status, JobStatus::Interrupted);
    assert_eq!(store.pieces(job.id)?[0].status, PieceStatus::Pending);
    Ok(())
}

#[test]
fn results_outlive_their_job() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let result = StoredResult {
        cache_key: "k".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput {
            text: "Bamako, le 3 mai 1957".into(),
            ..ProcessOutput::default()
        },
        request: vec![RequestSetting::new(
            "generationConfig.thinkingConfig.thinkingLevel",
            "low",
        )],
        cost_usd: Some(0.001),
        duration_ms: 1200,
        created_at: Timestamp::now(),
    };
    store.put_result(&result)?;
    store.delete_job(job.id)?;
    assert!(matches!(store.job(job.id), Err(StoreError::NotFound)));
    assert_eq!(
        store.cached_result(&CacheKey("k".into()))?,
        Some(result.clone())
    );
    assert_eq!(store.cached_result(&CacheKey("other".into()))?, None);
    Ok(())
}

/// A second answer under the same key never replaces the first: corrections made over it stay
/// over the text they corrected.
#[test]
fn the_first_answer_under_a_key_is_kept() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let answer = |text: &str, truncated: bool| StoredResult {
        cache_key: "k".into(),
        provider: ProviderId::new("gemini"),
        model: "gemini-3.8-flash".into(),
        output: ProcessOutput {
            text: text.into(),
            truncated,
            ..ProcessOutput::default()
        },
        request: Vec::new(),
        cost_usd: None,
        duration_ms: 10,
        created_at: Timestamp::now(),
    };
    store.put_result(&answer("Bamako, le 3 mai", true))?;
    store.put_result(&answer("Bamako, le 5 mai", false))?;
    let kept = store
        .cached_result(&CacheKey("k".into()))?
        .ok_or(StoreError::NotFound)?;
    assert_eq!(kept.output.text, "Bamako, le 3 mai");
    assert!(kept.output.truncated);
    Ok(())
}

/// A result stored before request settings were kept reads back with none, not as corrupt.
#[test]
fn older_results_have_no_request_settings() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let output = to_json(&ProcessOutput {
        text: "Ségou".into(),
        ..ProcessOutput::default()
    })?;
    store
        .conn()
        .execute(
            "INSERT INTO results (cache_key, provider, model, output, duration_ms, created_at)
             VALUES ('old', 'claude', 'claude-sonnet-5-5', ?1, 10, 0)",
            [output],
        )
        .map_err(db)?;
    let result = store
        .cached_result(&CacheKey("old".into()))?
        .ok_or(StoreError::NotFound)?;
    assert_eq!(result.output.text, "Ségou");
    assert!(result.request.is_empty());
    Ok(())
}

#[test]
fn corrections_are_kept_per_piece() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut correction = Correction {
        job_id: job.id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index: 0,
        text: None,
        fields: None,
        reviewed: true,
        updated_at: Timestamp::now(),
    };
    store.put_correction(&correction)?;
    correction.text = Some("Corrected".into());
    correction.fields = Some(vec![FieldAnswer {
        key: "places".into(),
        value: FieldValue::List(vec!["Bamako".into()]),
        reason: None,
    }]);
    store.put_correction(&correction)?;
    assert_eq!(store.corrections(job.id)?, vec![correction]);
    Ok(())
}

#[test]
fn corrections_saved_together_are_all_kept_or_none() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let correction = |job_id: JobId, index: u32| Correction {
        job_id,
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        index,
        text: Some(format!("Page {}", index + 1)),
        fields: None,
        reviewed: true,
        updated_at: Timestamp::now(),
    };
    let both = [correction(job.id, 0), correction(job.id, 1)];
    store.put_corrections(&both)?;
    assert_eq!(store.corrections(job.id)?, both.to_vec());
    assert_eq!(store.outdated_files(Some(job.id))?.len(), 1);

    // The second belongs to no job: the first, written before it, is not kept either.
    let mut changed = correction(job.id, 0);
    changed.text = Some("Changed".into());
    assert!(
        store
            .put_corrections(&[changed, correction(JobId::new(), 0)])
            .is_err()
    );
    assert_eq!(store.corrections(job.id)?, both.to_vec());
    Ok(())
}

#[test]
fn jobs_deleted_together_go_with_everything_they_hold() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let (first, second, kept) = (job(), job(), job());
    for job in [&first, &second, &kept] {
        store.create_job(job)?;
        store.upsert_piece(&piece(job, 0, PieceStatus::Done))?;
    }
    store.delete_jobs(&[first.id, second.id])?;
    let left: Vec<JobId> = store.list_jobs()?.iter().map(|j| j.id).collect();
    assert_eq!(left, vec![kept.id]);
    assert!(store.pieces(first.id)?.is_empty());
    assert_eq!(store.pieces(kept.id)?.len(), 1);
    Ok(())
}

#[test]
fn speaker_names_replace_each_other() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let mut names = SpeakerNames {
        setup_id: job.setups[0].id,
        source_id: job.sources[0].id,
        names: [
            ("Speaker 1".to_string(), "Imam Traoré".to_string()),
            ("Speaker 2".to_string(), "Interviewer".to_string()),
        ]
        .into_iter()
        .collect(),
    };
    store.put_speaker_names(job.id, &names)?;
    names.names.remove("Speaker 2");
    names.names.insert("Speaker 3".into(), "  ".into());
    store.put_speaker_names(job.id, &names)?;
    names.names.remove("Speaker 3");
    assert_eq!(store.speaker_names(job.id)?, vec![names]);
    Ok(())
}

/// A save marks its source's files out of date with it; a mark is cleared only at the
/// generation the files were written from, and goes with its job.
#[test]
fn saves_mark_files_out_of_date_until_written() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let (setup, source) = (job.setups[0].id, job.sources[0].id);
    assert!(store.outdated_files(None)?.is_empty());
    let correction = Correction {
        job_id: job.id,
        setup_id: setup,
        source_id: source,
        index: 0,
        text: Some("Corrected".into()),
        fields: None,
        reviewed: true,
        updated_at: Timestamp::now(),
    };
    store.put_correction(&correction)?;
    let marked = store.outdated_files(Some(job.id))?;
    assert_eq!(marked.len(), 1);
    assert_eq!((marked[0].setup_id, marked[0].source_id), (setup, source));

    // Names saved after the files were read: clearing the earlier generation keeps the mark.
    let names = SpeakerNames {
        setup_id: setup,
        source_id: source,
        names: [("Speaker 1".to_string(), "Imam".to_string())]
            .into_iter()
            .collect(),
    };
    store.put_speaker_names(job.id, &names)?;
    store.clear_outdated(&marked[0])?;
    let later = store.outdated_files(None)?;
    assert_eq!(later.len(), 1);
    assert!(later[0].generation > marked[0].generation);
    store.clear_outdated(&later[0])?;
    assert!(store.outdated_files(None)?.is_empty());

    store.put_correction(&correction)?;
    assert_eq!(store.outdated_files(Some(job.id))?.len(), 1);
    assert!(store.outdated_files(Some(JobId::new()))?.is_empty());
    store.delete_job(job.id)?;
    assert!(
        store.outdated_files(None)?.is_empty(),
        "deleted with the job"
    );
    Ok(())
}

#[test]
fn batches_are_kept_with_their_job_and_survive_startup() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    store.set_job_status(job.id, JobStatus::Running)?;
    store.upsert_piece(&piece(&job, 0, PieceStatus::Submitted))?;
    let mut batch = BatchRecord {
        id: "b1".into(),
        job_id: job.id,
        setup_id: job.setups[0].id,
        provider: ProviderId::new("gemini"),
        remote_id: None,
        phase: BatchPhase::Sending,
        items: vec![BatchItem {
            id: "p0".into(),
            source_id: job.sources[0].id,
            index: 0,
            request: vec![RequestSetting::new("max_tokens", 32000)],
        }],
        submitted_at: Timestamp::now(),
        updated_at: Timestamp::now(),
    };
    store.put_batch(&batch)?;
    batch.remote_id = Some("batches/123".into());
    batch.phase = BatchPhase::Waiting;
    store.put_batch(&batch)?;
    assert_eq!(store.batches(job.id)?, vec![batch]);

    // A piece waiting at the provider is not lost work: it stays submitted.
    store.mark_running_as_interrupted()?;
    assert_eq!(store.pieces(job.id)?[0].status, PieceStatus::Submitted);

    store.delete_job(job.id)?;
    assert!(store.batches(job.id)?.is_empty());
    Ok(())
}

#[test]
fn a_database_file_survives_reopening() -> Result<(), StoreError> {
    let dir = tempfile::tempdir().map_err(corrupt)?;
    let path = dir.path().join("jobs.sqlite");
    let job = job();
    {
        let store = SqliteStore::open(&path)?;
        store.create_job(&job)?;
        store.upsert_piece(&piece(&job, 0, PieceStatus::Done))?;
    }
    let store = SqliteStore::open(&path)?;
    assert_eq!(store.job(job.id)?.name, "Letters");
    assert_eq!(store.pieces(job.id)?.len(), 1);
    Ok(())
}

/// A database a newer Kalamu migrated further is refused, and left as it was, rather than run
/// on by a version that does not know its tables.
#[test]
fn a_database_from_a_newer_version_is_refused_and_left_alone() -> Result<(), StoreError> {
    let dir = tempfile::tempdir().map_err(corrupt)?;
    let path = dir.path().join("jobs.sqlite");
    let job = job();
    let known = migrations::MIGRATIONS.len() as i64;
    {
        let store = SqliteStore::open(&path)?;
        store.create_job(&job)?;
        store
            .conn()
            .pragma_update(None, "user_version", known + 1)
            .map_err(db)?;
    }
    match SqliteStore::open(&path) {
        Err(StoreError::Newer { found, known: k }) => {
            assert_eq!((found, k), (known as u64 + 1, known as u64));
        }
        Err(e) => panic!("refused for another reason: {e}"),
        Ok(_) => panic!("a newer database was opened"),
    }
    let version: i64 = Connection::open(&path)
        .map_err(db)?
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db)?;
    assert_eq!(version, known + 1);
    assert_eq!(
        schema_known(known, migrations::MIGRATIONS.len())?,
        known as usize
    );
    assert_eq!(schema_known(0, migrations::MIGRATIONS.len())?, 0);
    Ok(())
}

#[test]
fn units_are_kept_in_order_and_go_with_their_job() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let ids = (job.id, job.setups[0].id, job.sources[0].id);
    let mut second = kalamu_core::DocumentUnit::new(ids.0, ids.1, ids.2, 1);
    second.title = "Lettre".into();
    second.pages = vec![PieceRef::page(2)];
    let first = kalamu_core::DocumentUnit::new(ids.0, ids.1, ids.2, 0);
    store.put_unit(&second)?;
    store.put_unit(&first)?;
    assert_eq!(store.units(job.id)?, [first.clone(), second.clone()]);

    second.revision += 1;
    second.pages.push(PieceRef::page(1));
    store.put_unit(&second)?;
    assert_eq!(store.unit(second.id)?, second);
    store.delete_unit(first.id)?;
    assert_eq!(store.units(job.id)?, [second.clone()]);
    assert!(matches!(store.unit(first.id), Err(StoreError::NotFound)));

    store.delete_job(job.id)?;
    assert!(store.units(job.id)?.is_empty());
    Ok(())
}

/// Lookups by job and by document have indexes of their own.
#[test]
fn lookups_by_job_and_by_document_are_indexed() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let conn = store.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'index'")
        .map_err(db)?;
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(db)?
        .collect::<Result<_, _>>()
        .map_err(db)?;
    for index in [
        "document_links_by_job",
        "preferred_results_by_job",
        "collection_documents_by_document",
    ] {
        assert!(names.iter().any(|n| n == index), "{index}: {names:?}");
    }
    Ok(())
}
