//! The engine with a fake provider, an in-memory store and real PDFs in a temporary folder.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use scriptorium_core::{
    CacheKey, ExportFormat, FieldAnswer, FieldKind, FieldValue, OutputDestination, OutputSettings,
    PreparedInput, ProcessOutput, ProcessRequest, Task, Usage,
};
use scriptorium_store::SqliteStore;

use super::*;

mod batch;

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

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::Analysis => &["application/pdf", "text/plain"],
            Task::AudioTranscription => &[scriptorium_core::YOUTUBE_LINK],
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
        if mime == scriptorium_core::YOUTUBE_LINK {
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
            assert_eq!(piece, &scriptorium_core::PieceRef::Whole);
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
            ..EngineConfig::default()
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
    let mut catalog = scriptorium_catalog::Catalog::bundled();
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
    let pipeline = scriptorium_prompts::Pipeline {
        id: String::new(),
        name: "Places".into(),
        task: Task::Analysis,
        setups: vec![scriptorium_prompts::PipelineSetup {
            provider: ProviderId::new("fake"),
            model: "fake-model".into(),
            preset_id: Some("analysis.entities".into()),
            label: None,
            options: Default::default(),
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
        scriptorium_core::provenance::ProvenancePrompt::from_snapshot(&job.setups[0].prompt)
    );
    assert_eq!(file.presets.len(), 1);
    assert_eq!(file.presets[0].id, "analysis.entities");
    assert!(file.to_markdown().contains("## Model and prompt"));
}
