//! A source as a whole: one that cannot be opened keeps its reason across a restart, and a
//! recording's pass over its pauses ends on a pause.

use kalamu_core::PieceRef;

use super::*;

impl Fixture {
    /// The engine started again on the same store, as after the app was closed and opened.
    fn restarted(&self) -> Arc<Engine> {
        Engine::new(
            self.store.clone(),
            vec![self.fake.clone()],
            kalamu_catalog::Catalog::bundled(),
            PromptLibrary::in_memory(),
            Arc::new(Keys),
            EngineConfig {
                base_delay: Duration::from_millis(5),
                app_version: "test".into(),
                listening_cache: self.dir.path().join("listening"),
                snapshots: self.dir.path().join("snapshots"),
                ..EngineConfig::default()
            },
        )
    }
}

/// Waits for the job's run to end, and says how.
async fn finished(
    events: &mut tokio::sync::broadcast::Receiver<ProgressEvent>,
    id: JobId,
) -> JobStatus {
    loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == id => {
                return status;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    }
}

#[tokio::test]
async fn a_source_that_cannot_be_opened_keeps_its_reason_after_a_restart() {
    let f = fixture();
    let source = f.pdf("registre.pdf", 2);
    let kept = f.pdf("lettre.pdf", 1);
    let bytes = std::fs::read(&source.path).unwrap_or_default();
    let job = f.new_job(vec![source.clone(), kept], &["fake-model"]);
    std::fs::remove_file(&source.path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);

    // After a restart the reason is still there, on one failed piece standing for the file.
    let engine = f.restarted();
    let pieces: Vec<_> = engine
        .pieces(job.id)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.source_id == source.id)
        .collect();
    assert_eq!(pieces.len(), 1, "{pieces:?}");
    assert_eq!(pieces[0].status, PieceStatus::Failed);
    assert_eq!(pieces[0].piece, PieceRef::Whole);
    assert!(
        pieces[0]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("registre.pdf")),
        "{pieces:?}"
    );
    // Nothing was written for it, and nothing was charged.
    assert!(!f.dir.path().join("registre.txt").exists());
    assert_eq!(f.fake.calls(), 1, "only the other file was read");
    // The job's summary counts it with the failures, no cost added.
    let summary = engine
        .jobs()
        .unwrap_or_default()
        .into_iter()
        .find(|j| j.id == job.id)
        .unwrap_or_else(|| panic!("listed"));
    assert_eq!(
        (
            summary.pieces_total,
            summary.pieces_finished,
            summary.pieces_failed
        ),
        (2, 1, 1)
    );
    assert_eq!(summary.status, JobStatus::CompletedWithErrors);

    // The file is back: retrying reads it, its pages take the failed piece's place, and the
    // job is complete.
    std::fs::write(&source.path, &bytes).unwrap_or_else(|e| panic!("{e}"));
    let mut events = engine.subscribe();
    engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("retry: {e}"));
    assert_eq!(finished(&mut events, job.id).await, JobStatus::Completed);
    let mut pieces: Vec<_> = engine
        .pieces(job.id)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.source_id == source.id)
        .collect();
    pieces.sort_by_key(|p| p.index);
    assert_eq!(
        pieces.iter().map(|p| p.piece.clone()).collect::<Vec<_>>(),
        [PieceRef::page(1), PieceRef::page(2)]
    );
    assert!(
        pieces
            .iter()
            .all(|p| p.status == PieceStatus::Done && p.error.is_none())
    );
    let txt = std::fs::read_to_string(f.dir.path().join("registre.txt")).unwrap_or_default();
    assert!(txt.contains("fake-model page 2"), "{txt}");
    assert!(!txt.contains("Not read"), "{txt}");
}

#[tokio::test]
async fn a_source_that_fails_part_way_fails_its_pieces_still_to_read() {
    let f = fixture();
    let source = f.pdf("registre.pdf", 3);
    let job = f.new_job(vec![source.clone()], &["fake-model"]);
    // Read once with its last page failing, then changed on disk before the resume.
    f.fake
        .fail(3, vec![ProviderError::Transient("busy".into()); 3]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    std::fs::write(
        &source.path,
        kalamu_media::sample_pdf(&[(595.0, 842.0); 4], false),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let mut pieces = f.store.pieces(job.id).unwrap_or_default();
    pieces.sort_by_key(|p| p.index);
    assert_eq!(pieces.len(), 3, "no piece added: {pieces:?}");
    assert_eq!(count(&pieces, PieceStatus::Done), 2, "pages read are kept");
    assert!(
        pieces[2]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("has changed")),
        "{pieces:?}"
    );
}

#[tokio::test]
async fn a_recording_never_planned_is_planned_afresh_once_it_opens() {
    let f = fixture();
    let path = f.dir.path().join("entretien.wav");
    std::fs::write(&path, tone_wav()).unwrap_or_else(|e| panic!("{e}"));
    let source = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = f.new_job_for(
        Task::AudioTranscription,
        vec![source.clone()],
        &["fake-model"],
    );
    std::fs::remove_file(&path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].piece, PieceRef::Whole);

    // The failed piece is not taken for the recording's stored plan: it is cut again.
    std::fs::write(&path, tone_wav()).unwrap_or_else(|e| panic!("{e}"));
    let mut events = f.engine.subscribe();
    f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("retry: {e}"));
    // Without FFmpeg, and with a model that takes no recording as it is, the piece fails.
    assert_eq!(
        finished(&mut events, job.id).await,
        JobStatus::CompletedWithErrors
    );
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces.len(), 1, "{pieces:?}");
    assert!(
        // Its length unknown without FFmpeg: one segment to the end.
        matches!(pieces[0].piece, PieceRef::Segment { index: 0, .. }),
        "{pieces:?}"
    );
}

/// A recording long enough that FFmpeg's pass over its pauses takes many seconds: a WAV whose
/// header announces a gigabyte of silence, left as a hole in the file where the system allows.
fn long_recording(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("long.wav");
    let data: u32 = 1 << 30;
    let mut header = Vec::new();
    header.extend(b"RIFF");
    header.extend((36 + data).to_le_bytes());
    header.extend(b"WAVEfmt ");
    header.extend(16u32.to_le_bytes());
    header.extend(1u16.to_le_bytes());
    header.extend(1u16.to_le_bytes());
    header.extend(16_000u32.to_le_bytes());
    header.extend(32_000u32.to_le_bytes());
    header.extend(2u16.to_le_bytes());
    header.extend(16u16.to_le_bytes());
    header.extend(b"data");
    header.extend(data.to_le_bytes());
    let file = std::fs::File::create(&path).unwrap_or_else(|e| panic!("{e}"));
    std::io::Write::write_all(&mut &file, &header).unwrap_or_else(|e| panic!("{e}"));
    file.set_len(header.len() as u64 + u64::from(data))
        .unwrap_or_else(|e| panic!("{e}"));
    path
}

/// With a real FFmpeg (skipped where there is none): a pause while a nine-hour recording's
/// pauses are being found ends FFmpeg at once, plans nothing, fails nothing, and leaves the
/// job paused, to resume.
#[tokio::test]
async fn a_pause_during_a_recording_s_analysis_ends_it_and_leaves_the_job_resumable() {
    let Some(ffmpeg) = kalamu_media::Ffmpeg::locate(&[]) else {
        eprintln!("skipped: no FFmpeg on PATH");
        return;
    };
    let f = fixture_with(Some(ffmpeg.clone()));
    let path = long_recording(f.dir.path());
    let source = kalamu_media::inspect(&path, Some(&ffmpeg)).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        source.duration_ms.is_some_and(|d| d > 8 * 3_600_000),
        "{source:?}"
    );
    let job = f.new_job_for(Task::AudioTranscription, vec![source], &["fake-model"]);
    let mut events = f.engine.subscribe();
    f.engine.start(job.id).unwrap_or_else(|e| panic!("{e}"));
    tokio::time::sleep(Duration::from_millis(700)).await;
    let paused = std::time::Instant::now();
    f.engine.pause(job.id);
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(5), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { status, .. })) => break status,
            Ok(Ok(ProgressEvent::SourceFailed { error, .. })) => panic!("failed: {error}"),
            Ok(Ok(ProgressEvent::SourcePrepared { .. })) => {
                panic!("the analysis finished before the pause")
            }
            Ok(Ok(_)) => {}
            other => panic!("the pause did not end the analysis: {other:?}"),
        }
    };
    assert!(
        paused.elapsed() < Duration::from_secs(2),
        "{:?}",
        paused.elapsed()
    );
    assert_eq!(status, JobStatus::Paused);
    assert_eq!(
        f.store.job(job.id).map(|j| j.status).ok(),
        Some(JobStatus::Paused)
    );
    assert!(
        f.store.pieces(job.id).unwrap_or_default().is_empty(),
        "no plan was recorded from a pass cut short"
    );
}
