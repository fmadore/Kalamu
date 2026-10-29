use kalamu_core::{
    BatchItem, BatchPhase, FieldAnswer, FieldValue, InputKind, JobSettings, OutputSettings,
    ProcessOutput, PromptSnapshot, RequestSetting, Setup, SetupOptions, SourceFile, Task,
};

use super::*;

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
