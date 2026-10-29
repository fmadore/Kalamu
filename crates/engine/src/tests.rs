//! The engine with a fake provider, an in-memory store and real PDFs in a temporary folder.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use kalamu_core::{
    CacheKey, ExportFormat, FieldAnswer, FieldKind, FieldValue, OutputDestination, OutputSettings,
    PreparedInput, ProcessOutput, ProcessRequest, Task, Usage,
};
use kalamu_store::SqliteStore;

use super::*;

mod batch;
mod contents;

/// Answers with the page label; can be told to fail some pages a number of times.
struct Fake {
    calls: AtomicUsize,
    /// Page number → errors to return before succeeding.
    failures: Mutex<HashMap<u32, Vec<ProviderError>>>,
    /// A page whose answer stops at the output limit (0: none).
    cut_off: std::sync::atomic::AtomicU32,
}

impl Fake {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            failures: Mutex::new(HashMap::new()),
            cut_off: std::sync::atomic::AtomicU32::new(0),
        })
    }

    fn fail(&self, page: u32, errors: Vec<ProviderError>) {
        self.failures
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(page, errors);
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Provider for Fake {
    fn id(&self) -> ProviderId {
        ProviderId::new("fake")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::Analysis => &["application/pdf", "text/plain"],
            Task::AudioTranscription => &[kalamu_core::YOUTUBE_LINK],
            _ => &["application/pdf"],
        }
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let PreparedInput {
            piece, bytes, mime, ..
        } = request.input;
        // A YouTube video arrives as its link, and is transcribed with times in the text.
        if mime == kalamu_core::YOUTUBE_LINK {
            assert_eq!(bytes, b"https://www.youtube.com/watch?v=dQw4w9WgXcQ");
            return Ok(ProcessOutput {
                text: "[00:00:05] Speaker 1: Bonjour.
[00:01:10] Speaker 2: Merci."
                    .into(),
                ..ProcessOutput::default()
            });
        }
        assert!(bytes.starts_with(b"%PDF"), "a page arrives as a PDF");
        // An analysis reads the whole document and answers each field.
        if let Some(schema) = request.analysis {
            assert_eq!(piece, &kalamu_core::PieceRef::Whole);
            let fields: Vec<FieldAnswer> = schema
                .fields
                .iter()
                .map(|f| FieldAnswer {
                    key: f.key.clone(),
                    value: match &f.kind {
                        FieldKind::List { .. } => FieldValue::List(vec!["Bamako".into()]),
                        _ => FieldValue::Text(format!("{} answer", request.model)),
                    },
                    reason: None,
                })
                .collect();
            return Ok(ProcessOutput {
                text: schema.render(&fields),
                fields,
                ..ProcessOutput::default()
            });
        }
        let kalamu_core::PieceRef::Page { number, .. } = piece else {
            return Err(ProviderError::Other("not a page".into()));
        };
        let next_error = {
            let mut failures = self.failures.lock().unwrap_or_else(|p| p.into_inner());
            failures.get_mut(number).and_then(|errs| {
                if errs.is_empty() {
                    None
                } else {
                    Some(errs.remove(0))
                }
            })
        };
        if let Some(err) = next_error {
            return Err(err);
        }
        Ok(ProcessOutput {
            text: format!("{} page {number}", request.model),
            usage: Usage {
                input_tokens: 1000,
                output_tokens: 100,
                ..Usage::default()
            },
            model_version: Some(format!("{}-001", request.model)),
            truncated: self.cut_off.load(Ordering::SeqCst) == *number,
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["fake-model".into()])
    }
}

struct Keys;

impl KeySource for Keys {
    fn key(&self, _provider: &ProviderId) -> Option<ApiKey> {
        Some(ApiKey::new("test"))
    }
}

struct Fixture {
    engine: Arc<Engine>,
    fake: Arc<Fake>,
    store: Arc<SqliteStore>,
    dir: tempfile::TempDir,
}

fn fixture() -> Fixture {
    fixture_with(None)
}

fn fixture_with(ffmpeg: Option<kalamu_media::Ffmpeg>) -> Fixture {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let fake = Fake::new();
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![fake.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 3,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            ffmpeg,
            listening_cache: dir.path().join("listening"),
            ..EngineConfig::default()
        },
    );
    Fixture {
        engine,
        fake,
        store,
        dir,
    }
}

impl Fixture {
    fn pdf(&self, name: &str, pages: usize) -> SourceFile {
        let path: PathBuf = self.dir.path().join(name);
        let bytes = kalamu_media::sample_pdf(&vec![(595.0, 842.0); pages], false);
        std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("write pdf"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"))
    }

    fn new_job(&self, sources: Vec<SourceFile>, models: &[&str]) -> Job {
        let setups = models
            .iter()
            .map(|m| SetupRequest {
                provider: ProviderId::new("fake"),
                model: (*m).into(),
                preset_id: None,
                label: None,
                options: Default::default(),
                merge: None,
            })
            .collect();
        self.engine
            .create_job(NewJob {
                name: "Test".into(),
                task: Task::TextExtraction,
                sources,
                setups,
                settings: Default::default(),
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt, ExportFormat::Json],
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("create job: {e}"))
    }

    async fn run(&self, id: JobId) -> JobStatus {
        let mut events = self.engine.subscribe();
        self.engine
            .start(id)
            .unwrap_or_else(|e| panic!("start: {e}"));
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
}

#[tokio::test]
async fn a_job_processes_every_page_and_writes_outputs_with_provenance() {
    let f = fixture();
    let source = f.pdf("bulletin.pdf", 3);
    let job = f.new_job(vec![source], &["fake-model"]);
    assert!(f.engine.outputs(job.id).unwrap_or_default().is_empty());
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3);

    // The written files are listed from disk, so the list outlives the run.
    let outputs = f.engine.outputs(job.id).unwrap_or_default();
    let txt_path = f.dir.path().join("bulletin.txt").display().to_string();
    let provenance_path = f
        .dir
        .path()
        .join("bulletin.provenance.json")
        .display()
        .to_string();
    assert!(outputs.contains(&txt_path), "{outputs:?}");
    assert_eq!(outputs.last(), Some(&provenance_path));
    assert!(outputs.iter().all(|p| std::path::Path::new(p).is_file()));

    let txt = std::fs::read_to_string(f.dir.path().join("bulletin.txt")).unwrap_or_default();
    assert!(txt.contains("--- Page 2 ---\n\nfake-model page 2"));
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("bulletin.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(provenance["model"], "fake-model");
    assert_eq!(provenance["model_versions"][0], "fake-model-001");
    assert_eq!(provenance["pieces"], 3);
    assert!(
        provenance["prompt"]["text"]
            .as_str()
            .is_some_and(|t| !t.is_empty())
    );
    assert_eq!(
        f.store.job(job.id).map(|j| j.status).ok(),
        Some(JobStatus::Completed)
    );

    // A file deleted since leaves the list.
    std::fs::remove_file(&provenance_path).unwrap_or_else(|e| panic!("{e}"));
    let outputs = f.engine.outputs(job.id).unwrap_or_default();
    assert!(outputs.contains(&txt_path));
    assert!(!outputs.contains(&provenance_path));
}

#[tokio::test]
async fn the_same_files_and_settings_are_never_sent_twice() {
    let f = fixture();
    let source = f.pdf("bulletin.pdf", 2);
    let first = f.new_job(vec![source.clone()], &["fake-model"]);
    f.run(first.id).await;
    let second = f.new_job(vec![source], &["fake-model"]);
    assert_eq!(f.run(second.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 2, "the second job came from the cache");
    let pieces = f.store.pieces(second.id).unwrap_or_default();
    assert!(pieces.iter().all(|p| p.status == PieceStatus::Cached));
}

#[tokio::test]
async fn transient_errors_are_retried_and_failures_can_be_retried_later() {
    let f = fixture();
    f.fake.fail(1, vec![ProviderError::Transient("503".into())]);
    f.fake
        .fail(2, vec![ProviderError::Transient("busy".into()); 3]);
    let job = f.new_job(vec![f.pdf("letters.pdf", 3)], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 2);
    assert_eq!(count(&pieces, PieceStatus::Failed), 1);
    assert!(
        !f.dir.path().join("letters.txt").exists(),
        "no output until every page is done"
    );

    // Retrying calls only the failed page.
    let before = f.fake.calls();
    let mut events = f.engine.subscribe();
    f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    while !matches!(events.recv().await, Ok(ProgressEvent::JobFinished { .. })) {}
    assert_eq!(f.fake.calls(), before + 1);
    assert!(f.dir.path().join("letters.txt").exists());
}

#[tokio::test]
async fn a_piece_stopped_part_way_is_done_again_on_resume_not_failed() {
    let f = fixture();
    f.fake.fail(2, vec![ProviderError::Interrupted]);
    let job = f.new_job(vec![f.pdf("long.pdf", 3)], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Paused);
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Failed), 0);
    assert_eq!(count(&pieces, PieceStatus::Pending), 1);
    let before = f.fake.calls();
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        f.fake.calls(),
        before + 1,
        "only the stopped piece is sent again"
    );
}

#[tokio::test]
async fn a_rejected_key_pauses_the_job_instead_of_failing_every_page() {
    let f = fixture();
    // A rejected key fails every page, so which page fails first does not matter: pages are
    // prepared on blocking threads and may reach the provider in any order.
    for page in 1..=6 {
        f.fake
            .fail(page, vec![ProviderError::Auth("invalid key".into())]);
    }
    let job = f.new_job(vec![f.pdf("a.pdf", 6)], &["fake-model"]);
    let status = f.run(job.id).await;
    assert_eq!(status, JobStatus::Paused);
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(
        count(&pieces, PieceStatus::Pending),
        4,
        "only the pages already in flight are tried; the rest waits for a new key"
    );
}

#[tokio::test]
async fn an_interrupted_job_resumes_without_repeating_finished_pages() {
    let f = fixture();
    let job = f.new_job(vec![f.pdf("long.pdf", 4)], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    // Simulate a crash during a second run: the job was running and one page was in flight.
    let mut piece = f.store.pieces(job.id).unwrap_or_default()[3].clone();
    piece.status = PieceStatus::Running;
    f.store
        .upsert_piece(&piece)
        .unwrap_or_else(|e| panic!("{e}"));
    f.store
        .set_job_status(job.id, JobStatus::Running)
        .unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(f.engine.recover().unwrap_or_default(), vec![job.id]);
    assert_eq!(
        f.store.job(job.id).map(|j| j.status).ok(),
        Some(JobStatus::Interrupted)
    );
    let calls = f.fake.calls();
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        f.fake.calls(),
        calls,
        "the interrupted page's result was already cached"
    );
}

#[tokio::test]
async fn comparison_jobs_write_one_set_of_files_per_setup_and_score_them() {
    let f = fixture();
    let source = f.pdf("lettre.pdf", 2);
    let job = f.new_job(vec![source.clone()], &["flash", "pro"]);
    assert!(job.is_comparison());
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert!(f.dir.path().join("lettre.flash.txt").exists());
    assert!(f.dir.path().join("lettre.pro.txt").exists());

    // The user corrects page 1 to what "pro" read.
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: source.id,
            index: 0,
            text: Some("pro page 1".into()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let flash = std::fs::read_to_string(f.dir.path().join("lettre.flash.txt")).unwrap_or_default();
    assert!(
        flash.contains("pro page 1"),
        "outputs follow the correction"
    );

    let report = f
        .engine
        .compare(job.id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let score = |label: &str| {
        report
            .scores
            .iter()
            .find(|s| s.label == label)
            .and_then(|s| s.cer)
    };
    assert_eq!(score("pro"), Some(0.0));
    assert!(score("flash").is_some_and(|c| c > 0.0));
    assert_eq!(report.pages.len(), 2);

    let views = f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default();
    let corrected = views
        .iter()
        .find(|v| v.setup_id == job.setups[0].id && v.index == 0);
    assert_eq!(
        corrected.and_then(|v| v.ai_text.clone()).as_deref(),
        Some("flash page 1"),
        "the AI output is kept under the correction"
    );
}

#[tokio::test]
async fn a_job_is_scored_against_every_page_reviewed_or_corrected() {
    let f = fixture();
    let (letter, register) = (f.pdf("lettre.pdf", 3), f.pdf("registre.pdf", 1));
    let job = f.new_job(vec![letter.clone(), register.clone()], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let review = |source: &SourceFile, index: u32, text: Option<&str>| {
        f.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: source.id,
                index,
                text: text.map(Into::into),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
    };
    let scores = || {
        f.engine
            .job_scores(job.id)
            .unwrap_or_else(|e| panic!("{e}"))
    };
    assert_eq!(scores().total[0].cer, None, "nothing reviewed yet");
    assert!(scores().sources.is_empty());

    // Page 1 checked and found right; page 2 corrected; page 3 left alone.
    review(&letter, 0, None);
    review(&letter, 1, Some("fake-model page two"));
    let s = scores();
    assert_eq!(s.sources.len(), 1, "only the letter has a reference");
    let total = &s.total[0];
    assert_eq!(total.scored_pages, 2);
    // "2" for "two": three edits, over the 17 and 19 characters of both references, not the
    // mean of each page's rate.
    assert!(total.cer.is_some_and(|c| (c - 3.0 / 36.0).abs() < 1e-9));
    assert!(total.wer.is_some_and(|w| (w - 1.0 / 6.0).abs() < 1e-9));

    // The register's page is reviewed too: the job's rate covers both documents.
    review(&register, 0, None);
    let s = scores();
    assert_eq!(s.sources.len(), 2);
    assert_eq!(s.total[0].scored_pages, 3);
    assert!(
        s.total[0]
            .cer
            .is_some_and(|c| (c - 3.0 / 53.0).abs() < 1e-9)
    );
    let report = f
        .engine
        .compare(job.id, letter.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        report.scores[0].scored_pages, 2,
        "the review scores it the same"
    );
}

#[tokio::test]
async fn a_pause_during_a_retry_wait_sends_no_further_attempt() {
    let f = fixture();
    let source = f.pdf("lettre.pdf", 1);
    // Refused once, with half a minute to wait before the next, paid, attempt.
    f.fake.fail(
        1,
        vec![ProviderError::RateLimited {
            retry_after: Some(Duration::from_secs(30)),
            message: "slow down".into(),
        }],
    );
    let job = f.new_job(vec![source], &["fake-model"]);
    let mut events = f.engine.subscribe();
    f.engine.start(job.id).unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..250 {
        if f.fake.calls() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    f.engine.pause(job.id);
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(5), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { status, .. })) => break status,
            Ok(Ok(_)) => {}
            other => panic!("the pause did not end the wait: {other:?}"),
        }
    };
    assert_eq!(status, JobStatus::Paused);
    assert_eq!(f.fake.calls(), 1, "no attempt after the pause");
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(
        pieces[0].status,
        PieceStatus::Pending,
        "to be read on resuming"
    );
}

#[tokio::test]
async fn a_source_changed_since_it_was_added_is_not_read() {
    let f = fixture();
    let source = f.pdf("lettre.pdf", 2);
    let job = f.new_job(vec![source.clone()], &["fake-model"]);
    // Another scan saved under the same name after the job was set up.
    std::fs::write(
        &source.path,
        kalamu_media::sample_pdf(&[(595.0, 842.0); 3], false),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(
        f.fake.calls(),
        0,
        "nothing is sent or cached under the old fingerprint"
    );
}

#[tokio::test]
async fn an_answer_cut_off_at_the_output_limit_is_marked_everywhere() {
    let f = fixture();
    f.fake.cut_off.store(2, Ordering::SeqCst);
    let source = f.pdf("registre.pdf", 3);
    let job = f.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);

    let summary = f
        .engine
        .jobs()
        .unwrap_or_default()
        .into_iter()
        .find(|j| j.id == job.id);
    assert_eq!(summary.map(|j| j.pieces_truncated), Some(1));
    let views = f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default();
    let cut: Vec<u32> = views
        .iter()
        .filter(|v| v.truncated)
        .map(|v| v.index)
        .collect();
    assert_eq!(cut, vec![1], "page 2, kept as read");
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("registre.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(provenance["truncated"], serde_json::json!([2]));
}

#[tokio::test]
async fn sample_runs_only_the_first_pages() {
    let f = fixture();
    let source = f.pdf("thick.pdf", 10);
    let mut new = NewJob {
        name: "Sample".into(),
        task: Task::TextExtraction,
        sources: vec![source],
        setups: vec![SetupRequest {
            provider: ProviderId::new("fake"),
            model: "fake-model".into(),
            preset_id: Some("text_extraction.newspaper".into()),
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        output: OutputSettings {
            formats: vec![ExportFormat::Txt],
            ..Default::default()
        },
    };
    new.settings.sample_pages = Some(2);
    let job = f.engine.create_job(new).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        job.setups[0]
            .prompt
            .text
            .contains("Paragraph Consolidation")
    );
    f.run(job.id).await;
    assert_eq!(f.fake.calls(), 2);
}

#[tokio::test]
async fn a_youtube_video_is_read_from_its_link_once_and_its_address_recorded() {
    let f = fixture();
    let new = |link: &str, destination: OutputDestination| NewJob {
        name: "Video".into(),
        task: Task::AudioTranscription,
        sources: f
            .engine
            .inspect_links(&[link])
            .into_iter()
            .flatten()
            .collect(),
        setups: vec![SetupRequest {
            provider: ProviderId::new("fake"),
            model: "fake-model".into(),
            preset_id: None,
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        output: OutputSettings {
            formats: vec![ExportFormat::Txt, ExportFormat::Srt],
            destination,
            ..Default::default()
        },
    };
    assert!(matches!(
        f.engine.inspect_links(&["https://vimeo.com/1"]).pop(),
        Some(Err(EngineError::NotYouTube(_)))
    ));
    // A video has no folder of its own to write beside.
    assert!(matches!(
        f.engine.create_job(new(
            "https://youtu.be/dQw4w9WgXcQ",
            OutputDestination::BesideSources
        )),
        Err(EngineError::LinkNeedsFolder)
    ));
    let folder = OutputDestination::Folder(f.dir.path().display().to_string());
    let job = f
        .engine
        .create_job(new("https://youtu.be/dQw4w9WgXcQ", folder.clone()))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let srt =
        std::fs::read_to_string(f.dir.path().join("youtube-dQw4w9WgXcQ.srt")).unwrap_or_default();
    assert!(srt.contains("00:01:10,000"), "{srt}");
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("youtube-dQw4w9WgXcQ.provenance.json"))
            .unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(
        provenance["source"]["url"],
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    );
    assert!(provenance["source"]["read_at"].is_number());
    // The review times each line from the text, for the links that open the video there.
    let views = f
        .engine
        .source_pieces(job.id, job.sources[0].id)
        .unwrap_or_default();
    let starts: Vec<u64> = views[0].segments.iter().map(|s| s.start_ms).collect();
    assert_eq!(starts, vec![5_000, 70_000]);

    // The same video under another form of link comes from the cache.
    let again = f
        .engine
        .create_job(new("https://www.youtube.com/shorts/dQw4w9WgXcQ", folder))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(again.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 1);
}

#[tokio::test]
async fn speaker_names_rename_the_review_and_the_files_but_not_the_output() {
    let f = fixture();
    let source = f.pdf("entretien.pdf", 1);
    // The fake model writes "<model> page 1", so this model's text opens with a speaker label.
    let job = f.new_job(vec![source.clone()], &["Speaker 1:"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    f.engine
        .save_speaker_names(
            job.id,
            SpeakerNames {
                setup_id: job.setups[0].id,
                source_id: source.id,
                names: [("Speaker 1".to_string(), "Imam Traoré".to_string())]
                    .into_iter()
                    .collect(),
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let txt = std::fs::read_to_string(f.dir.path().join("entretien.txt")).unwrap_or_default();
    assert!(txt.contains("Imam Traoré: page 1"), "{txt}");
    let views = f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default();
    assert_eq!(views[0].ai_text.as_deref(), Some("Imam Traoré: page 1"));
    assert_eq!(
        views[0].speaker_names.get("Speaker 1").map(String::as_str),
        Some("Imam Traoré")
    );
    let key = f
        .store
        .pieces(job.id)
        .unwrap_or_default()
        .into_iter()
        .find_map(|p| p.cache_key)
        .unwrap_or_default();
    let stored = f.store.cached_result(&CacheKey(key)).ok().flatten();
    assert_eq!(
        stored.map(|r| r.output.text).as_deref(),
        Some("Speaker 1: page 1"),
        "the model's output keeps its own labels"
    );
}

#[tokio::test]
async fn an_analysis_reads_each_document_once_and_corrections_replace_its_answers() {
    let f = fixture();
    let source = f.pdf("bulletin.pdf", 3);
    let job = f
        .engine
        .create_job(NewJob {
            name: "Places".into(),
            task: Task::Analysis,
            sources: vec![source.clone()],
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Csv],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        job.setups[0].prompt.schema.is_some(),
        "the fields are frozen into the job"
    );
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 1, "one call for the whole document");
    let txt =
        std::fs::read_to_string(f.dir.path().join("bulletin.analysis.txt")).unwrap_or_default();
    assert!(txt.contains("Places: Bamako"), "{txt}");

    let mut answers = f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default()[0]
        .ai_fields
        .clone();
    let places = answers.iter_mut().find(|a| a.key == "places");
    if let Some(places) = places {
        places.value = FieldValue::List(vec!["Bamako".into(), "Ségou".into()]);
    }
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: source.id,
            index: 0,
            text: None,
            fields: Some(answers),
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let csv =
        std::fs::read_to_string(f.dir.path().join("bulletin.analysis.csv")).unwrap_or_default();
    assert!(csv.contains("Bamako; Ségou"), "{csv}");
    let view = &f
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default()[0];
    assert!(
        view.ai_fields
            .iter()
            .any(|a| a.value == FieldValue::List(vec!["Bamako".into()])),
        "the model's answers are kept under the correction"
    );
    assert!(view.corrected_fields.is_some());
}

#[tokio::test]
async fn a_text_only_model_is_not_sent_the_formats_its_provider_takes() {
    let f = fixture();
    let mut catalog = kalamu_catalog::Catalog::bundled();
    let mut model = catalog.models[0].clone();
    model.provider = ProviderId::new("fake");
    model.id = "fake-text".into();
    model.inputs = vec!["text/plain".into()];
    catalog.models.push(model);
    f.engine.set_catalog(catalog);
    let source = f.pdf("scan.pdf", 1);
    let job = f
        .engine
        .create_job(NewJob {
            name: "Text only".into(),
            task: Task::Analysis,
            sources: vec![source],
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-text".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    f.run(job.id).await;
    assert_eq!(
        f.fake.calls(),
        0,
        "the fake provider takes PDFs, the model does not"
    );
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces[0].status, PieceStatus::Failed);
    assert!(
        pieces[0]
            .error
            .as_deref()
            .is_some_and(|e| e.contains("reads text only")),
        "{:?}",
        pieces[0].error
    );
}

#[test]
fn a_pipeline_record_carries_the_prompt_a_job_would_send() {
    let f = fixture();
    let pipeline = kalamu_prompts::Pipeline {
        id: String::new(),
        name: "Places".into(),
        task: Task::Analysis,
        setups: vec![kalamu_prompts::PipelineSetup {
            provider: ProviderId::new("fake"),
            model: "fake-model".into(),
            preset_id: Some("analysis.entities".into()),
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        formats: vec![ExportFormat::Csv],
        destination: Default::default(),
        updated_at: Timestamp(0),
    };
    let file = f
        .engine
        .pipeline_file(pipeline)
        .unwrap_or_else(|e| panic!("{e}"));
    let job = f
        .engine
        .create_job(NewJob {
            name: "Places".into(),
            task: Task::Analysis,
            sources: vec![f.pdf("a.pdf", 1)],
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: Default::default(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let recorded = file.setups[0]
        .prompt
        .clone()
        .unwrap_or_else(|| panic!("no prompt"));
    assert_eq!(
        recorded,
        kalamu_core::provenance::ProvenancePrompt::from_snapshot(&job.setups[0].prompt)
    );
    assert_eq!(file.presets.len(), 1);
    assert_eq!(file.presets[0].id, "analysis.entities");
    assert!(file.to_markdown().contains("## Model and prompt"));
}

/// One second of a 440 Hz tone as 16 kHz mono WAV.
fn tone_wav() -> Vec<u8> {
    let samples: Vec<i16> = (0..16_000)
        .map(|i| ((i as f32 * 440.0 * std::f32::consts::TAU / 16_000.0).sin() * 8_000.0) as i16)
        .collect();
    let data = (samples.len() * 2) as u32;
    let mut wav = b"RIFF".to_vec();
    wav.extend((36 + data).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(16_000u32.to_le_bytes());
    wav.extend(32_000u32.to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(data.to_le_bytes());
    wav.extend(samples.iter().flat_map(|s| s.to_le_bytes()));
    wav
}

impl Fixture {
    /// A job over one recording named `name`, with one piece from `start_ms` to `end_ms`.
    fn recording_piece(&self, name: &str, start_ms: u64, end_ms: u64) -> (Job, SourceFile) {
        let path = self.dir.path().join(name);
        std::fs::write(&path, tone_wav()).unwrap_or_else(|_| panic!("write recording"));
        let source = kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"));
        let job = self.new_job(vec![source.clone()], &["fake-model"]);
        self.store
            .upsert_piece(&kalamu_core::PieceState {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: source.id,
                index: 1,
                piece: kalamu_core::PieceRef::Segment {
                    index: 1,
                    start_ms,
                    end_ms,
                },
                status: PieceStatus::Done,
                cache_key: None,
                error: None,
                attempts: 1,
                updated_at: Timestamp(0),
            })
            .unwrap_or_else(|_| panic!("piece"));
        (job, source)
    }
}

#[test]
fn a_recording_the_player_plays_is_heard_from_the_file_itself() {
    let f = fixture();
    let (job, source) = f.recording_piece("interview.mp3", 1_200_000, 2_400_000);
    let audio = f
        .engine
        .piece_audio(job.id, source.id, 1, false)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        audio,
        PieceAudio {
            path: source.path,
            start_ms: 1_200_000,
            end_ms: 2_400_000,
            original: true,
        }
    );
    assert!(
        !f.dir.path().join("listening").exists(),
        "nothing converted"
    );
}

#[test]
fn a_recording_the_player_cannot_play_is_never_offered_as_it_is() {
    // Without FFmpeg it cannot be converted either, so it says so.
    let f = fixture();
    let (job, source) = f.recording_piece("interview.wma", 0, 1_000);
    let err = f.engine.piece_audio(job.id, source.id, 1, false);
    assert!(matches!(err, Err(EngineError::Media(_))), "{err:?}");
}

#[test]
fn a_converted_piece_is_kept_and_found_again() {
    let Some(ffmpeg) = kalamu_media::Ffmpeg::locate(&[]) else {
        eprintln!("skipped: no FFmpeg on PATH");
        return;
    };
    let f = fixture_with(Some(ffmpeg));
    // As if the player had failed on a WAV it should have played.
    let (job, source) = f.recording_piece("interview.wav", 250, 750);
    let first = f
        .engine
        .piece_audio(job.id, source.id, 1, true)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!first.original);
    assert_eq!(
        (first.start_ms, first.end_ms),
        (0, 0),
        "the copy is the piece"
    );
    assert!(
        first
            .path
            .starts_with(&*f.dir.path().join("listening").to_string_lossy())
    );
    let bytes = std::fs::read(&first.path).unwrap_or_default();
    assert!(bytes.starts_with(b"OggS") || bytes.starts_with(b"RIFF"));
    std::fs::write(&first.path, b"kept").unwrap_or_else(|_| panic!("mark"));
    let again = f
        .engine
        .piece_audio(job.id, source.id, 1, true)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(again, first);
    assert_eq!(
        std::fs::read(&again.path).unwrap_or_default(),
        b"kept",
        "not converted twice"
    );
}
