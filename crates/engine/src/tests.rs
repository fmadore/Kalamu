//! The engine with a fake provider, an in-memory store and real PDFs in a temporary folder.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use scriptorium_core::{
    CacheKey, ExportFormat, OutputSettings, PreparedInput, ProcessOutput, ProcessRequest, Task,
    Usage,
};
use scriptorium_store::SqliteStore;

use super::*;

/// Answers with the page label; can be told to fail some pages a number of times.
struct Fake {
    calls: AtomicUsize,
    /// Page number → errors to return before succeeding.
    failures: Mutex<HashMap<u32, Vec<ProviderError>>>,
}

impl Fake {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            failures: Mutex::new(HashMap::new()),
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

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let PreparedInput { piece, bytes, .. } = request.input;
        assert!(bytes.starts_with(b"%PDF"), "a page arrives as a PDF");
        let scriptorium_core::PieceRef::Page { number, .. } = piece else {
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
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let fake = Fake::new();
    let engine = Engine::new(
        store.clone(),
        vec![fake.clone()],
        scriptorium_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 3,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            ffmpeg: None,
        },
    );
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
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
        let bytes = scriptorium_media::sample_pdf(&vec![(595.0, 842.0); pages], false);
        std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("write pdf"));
        scriptorium_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"))
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
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3);

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
    f.fake
        .fail(1, vec![ProviderError::Auth("invalid key".into())]);
    let job = f.new_job(vec![f.pdf("a.pdf", 6)], &["fake-model"]);
    let status = f.run(job.id).await;
    assert_eq!(status, JobStatus::Paused);
    let pieces = f.store.pieces(job.id).unwrap_or_default();
    assert!(
        count(&pieces, PieceStatus::Pending) >= 3,
        "the rest waits for a new key"
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
