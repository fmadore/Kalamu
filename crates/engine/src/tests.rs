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
use kalamu_core::{JobStatus, PieceStatus, ProviderError, StoreError};

/// Pieces of a job by status.
fn count(pieces: &[kalamu_core::PieceState], status: PieceStatus) -> usize {
    pieces.iter().filter(|p| p.status == status).count()
}

mod answers;
mod attention;
mod batch;
mod contents;
mod large;
mod later;
mod names;
mod pages;
mod reading;
mod snapshot;
mod sources;
mod workflow;

/// Answers with the page label; can be told to fail some pages a number of times.
struct Fake {
    calls: AtomicUsize,
    /// Page number → errors to return before succeeding.
    failures: Mutex<HashMap<u32, Vec<ProviderError>>>,
    /// A page whose answer stops at the output limit (0: none).
    cut_off: std::sync::atomic::AtomicU32,
    /// The texts analyses were sent, in order.
    read: Mutex<Vec<String>>,
    /// What each call asked beyond the prompt.
    tunings: Mutex<Vec<kalamu_core::Tuning>>,
    /// The page ranges sent together, in order.
    groups: Mutex<Vec<(u32, u32)>>,
    /// A page a multi-page answer leaves unmarked (0: none).
    unmarked: std::sync::atomic::AtomicU32,
    /// Multi-page answers stop at the output limit.
    cut_groups: std::sync::atomic::AtomicBool,
}

impl Fake {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            failures: Mutex::new(HashMap::new()),
            cut_off: std::sync::atomic::AtomicU32::new(0),
            read: Mutex::new(Vec::new()),
            tunings: Mutex::new(Vec::new()),
            groups: Mutex::new(Vec::new()),
            unmarked: std::sync::atomic::AtomicU32::new(0),
            cut_groups: std::sync::atomic::AtomicBool::new(false),
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
        self.tunings
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(request.tuning.clone());
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
        // An analysis reads the whole document, a PDF or a text, and answers each field.
        if let Some(schema) = request.analysis {
            assert_eq!(piece, &kalamu_core::PieceRef::Whole);
            if mime == "text/plain" {
                self.read
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push(String::from_utf8_lossy(bytes).into_owned());
            } else {
                assert!(bytes.starts_with(b"%PDF"), "a document arrives as a PDF");
            }
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
        assert!(bytes.starts_with(b"%PDF"), "a page arrives as a PDF");
        // Several pages in one file: one text, each page after its mark.
        if let Some(range) = request
            .user_prompt
            .split("This file holds pages ")
            .nth(1)
            .and_then(|rest| rest.split(" of the document").next())
        {
            let mut bounds = range.split(" to ").filter_map(|n| n.parse::<u32>().ok());
            let (Some(first), Some(last)) = (bounds.next(), bounds.next()) else {
                return Err(ProviderError::Other("no range".into()));
            };
            self.groups
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .push((first, last));
            let unmarked = self.unmarked.load(Ordering::SeqCst);
            let text: String = (first..=last)
                .filter(|n| *n != unmarked)
                .map(|n| {
                    format!(
                        "{}\n{} page {n}\n",
                        kalamu_core::pages::marker(n),
                        request.model
                    )
                })
                .collect();
            return Ok(ProcessOutput {
                text,
                usage: Usage {
                    input_tokens: 1000,
                    output_tokens: 100,
                    ..Usage::default()
                },
                truncated: self.cut_groups.load(Ordering::SeqCst),
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
            snapshots: dir.path().join("snapshots"),
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
        self.new_job_for(Task::TextExtraction, sources, models)
    }

    fn new_job_for(&self, task: Task, sources: Vec<SourceFile>, models: &[&str]) -> Job {
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
                task,
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

    // A job tied to no project names no document.
    assert!(provenance.get("documents").is_none());

    // A file deleted since leaves the list.
    std::fs::remove_file(&provenance_path).unwrap_or_else(|e| panic!("{e}"));
    let outputs = f.engine.outputs(job.id).unwrap_or_default();
    assert!(outputs.contains(&txt_path));
    assert!(!outputs.contains(&provenance_path));
}

/// One document for every source, its shelfmark as the researcher last typed it.
struct Described(Mutex<String>);

impl SourceDocuments for Described {
    fn documents(
        &self,
        _job: JobId,
        _source: SourceId,
    ) -> Result<Vec<kalamu_core::provenance::ProvenanceDocument>, StoreError> {
        use kalamu_core::provenance::{ProvenanceDescribed, ProvenanceDocument};
        let shelfmark = self.0.lock().unwrap_or_else(|p| p.into_inner()).clone();
        Ok(vec![ProvenanceDocument {
            project_id: kalamu_core::ProjectId::new(),
            project_name: "Archives".into(),
            document_id: kalamu_core::DocumentId::new(),
            title: "Bulletin".into(),
            description: [(
                kalamu_core::DescriptionField::Shelfmark,
                ProvenanceDescribed {
                    value: shelfmark,
                    from: kalamu_core::DescribedBy::Document,
                },
            )]
            .into(),
        }])
    }
}

#[tokio::test]
async fn provenance_names_the_project_document_as_it_stands_when_written() {
    let f = fixture();
    let described = Arc::new(Described(Mutex::new("19G 1".into())));
    f.engine.set_source_documents(described.clone());
    let source = f.pdf("bulletin.pdf", 1);
    let job = f.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let read = || -> serde_json::Value {
        serde_json::from_slice(
            &std::fs::read(f.dir.path().join("bulletin.provenance.json")).unwrap_or_default(),
        )
        .unwrap_or_default()
    };
    let document = &read()["documents"][0];
    assert_eq!(document["project_name"], "Archives");
    assert_eq!(document["title"], "Bulletin");
    assert_eq!(document["description"]["shelfmark"]["value"], "19G 1");
    assert_eq!(document["description"]["shelfmark"]["from"], "document");

    // The description changed since: its provenance and the `.json` that holds it are
    // written again, saying what it says now; the text is left as it is, and the provider is
    // not asked again.
    *described.0.lock().unwrap_or_else(|p| p.into_inner()) = "19G 2".into();
    let calls = f.fake.calls();
    let txt = f.dir.path().join("bulletin.txt");
    std::fs::write(&txt, "kept as the researcher left it").unwrap_or_else(|e| panic!("{e}"));
    let written = f
        .engine
        .rewrite_provenance(job.id, job.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(written.len(), 2, "{written:?}");
    assert_eq!(
        read()["documents"][0]["description"]["shelfmark"]["value"],
        "19G 2"
    );
    let json: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("bulletin.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(
        json["provenance"]["documents"][0]["description"]["shelfmark"]["value"],
        "19G 2"
    );
    assert_eq!(
        std::fs::read_to_string(&txt).unwrap_or_default(),
        "kept as the researcher left it"
    );

    // A source with nothing written yet is left alone.
    let unread = f.new_job(vec![f.pdf("vide.pdf", 1)], &["fake-model"]);
    let none = f
        .engine
        .rewrite_provenance(unread.id, unread.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(none.is_empty());
    assert!(!f.dir.path().join("vide.provenance.json").exists());
    assert_eq!(f.fake.calls(), calls);
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
    let txt = std::fs::read_to_string(f.dir.path().join("letters.txt")).unwrap_or_default();
    assert!(
        txt.contains("[Not read: "),
        "written, the failed page marked: {txt}"
    );

    // Retrying calls only the failed page.
    let before = f.fake.calls();
    let mut events = f.engine.subscribe();
    f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    while !matches!(events.recv().await, Ok(ProgressEvent::JobFinished { .. })) {}
    assert_eq!(f.fake.calls(), before + 1);
    let txt = std::fs::read_to_string(f.dir.path().join("letters.txt")).unwrap_or_default();
    assert!(!txt.contains("Not read"), "written again, whole: {txt}");
}

/// An answer the provider billed but the engine refused, for breaking its fields, is paid for:
/// it counts in what the job spent, with the answers kept, and so does one a filter stopped
/// after the provider counted it; a failure the provider did not bill adds nothing.
#[tokio::test]
async fn answers_billed_but_refused_count_in_the_cost() {
    let f = fixture();
    // The fake model, priced as Gemini 3.8 Flash: $0.75 and $3.75 per million tokens.
    let mut catalog = f.engine.catalog();
    let mut priced = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("a priced model"));
    priced.provider = ProviderId::new("fake");
    priced.id = "priced-model".into();
    priced.pricing.batch_discount = None;
    catalog.models.push(priced);
    f.engine.set_catalog(catalog);
    let usage = Usage {
        input_tokens: 1000,
        output_tokens: 100,
        ..Usage::default()
    };
    f.fake.fail(
        2,
        vec![ProviderError::InvalidAnswer {
            reason: "not valid JSON".into(),
            usage: usage.clone(),
        }],
    );
    f.fake.fail(3, vec![ProviderError::Transient("503".into())]);
    // A filter stopped page 4 after the model wrote; refusals are not tried again.
    f.fake.fail(
        4,
        vec![ProviderError::Blocked {
            reason: "RECITATION".into(),
            usage: Some(usage),
        }],
    );
    let job = f.new_job(vec![f.pdf("letters.pdf", 4)], &["priced-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(f.fake.calls(), 6);

    // Each call read 1,000 tokens and wrote 100: $0.001125. Three kept, two refused.
    let call = 0.001_125;
    let charges = f.store.charges(job.id).unwrap_or_default();
    assert_eq!(charges.len(), 2, "only the billed failures: {charges:?}");
    let mut charged: Vec<u32> = charges.iter().map(|c| c.index).collect();
    charged.sort_unstable();
    assert_eq!(charged, [1, 3]);
    assert!(
        charges
            .iter()
            .all(|c| c.cost_usd.is_some_and(|c| (c - call).abs() < 1e-12))
    );
    let spent = f.store.spent(job.id).unwrap_or_default();
    assert!((spent - 5.0 * call).abs() < 1e-12, "{spent}");
    let listed = f.store.list_jobs().unwrap_or_default();
    assert!(
        listed[0]
            .cost_usd
            .is_some_and(|c| (c - 5.0 * call).abs() < 1e-12)
    );

    // Provenance lists the refused answer apart from the results kept.
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("letters.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(provenance["refused"]["answers"], 2);
    assert_eq!(provenance["refused"]["usage"]["input_tokens"], 2000);
    assert!(
        provenance["cost_usd"]
            .as_f64()
            .is_some_and(|c| (c - 3.0 * call).abs() < 1e-12)
    );
}

/// A structured answer cut off at the model's output limit is not asked for again, which would
/// stop at the same place and be billed again: it is sent once, fails with the reason, and what
/// it was billed counts in the cost and among the refused answers.
#[tokio::test]
async fn an_answer_cut_off_at_the_output_limit_is_sent_once_and_paid_for() {
    let f = fixture();
    let mut catalog = f.engine.catalog();
    let mut priced = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("a priced model"));
    priced.provider = ProviderId::new("fake");
    priced.id = "priced-model".into();
    priced.pricing.batch_discount = None;
    catalog.models.push(priced);
    f.engine.set_catalog(catalog);
    let usage = Usage {
        input_tokens: 1000,
        output_tokens: 100,
        ..Usage::default()
    };
    // Were it retried, the later calls would succeed.
    f.fake.fail(2, vec![ProviderError::CutOff { usage }]);
    let job = f.new_job(vec![f.pdf("letters.pdf", 2)], &["priced-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(f.fake.calls(), 2, "the cut-off answer is sent once");

    let pieces = f.store.pieces(job.id).unwrap_or_default();
    let failed = pieces
        .iter()
        .find(|p| p.status == PieceStatus::Failed)
        .unwrap_or_else(|| panic!("the cut-off page failed"));
    assert_eq!(failed.attempts, 1);
    assert!(
        failed
            .error
            .as_deref()
            .is_some_and(|e| e.contains("output limit")),
        "{:?}",
        failed.error
    );

    // Each call read 1,000 tokens and wrote 100: $0.001125, one kept and one refused.
    let call = 0.001_125;
    let charges = f.store.charges(job.id).unwrap_or_default();
    assert_eq!(charges.len(), 1, "{charges:?}");
    let spent = f.store.spent(job.id).unwrap_or_default();
    assert!((spent - 2.0 * call).abs() < 1e-12, "{spent}");
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("letters.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(provenance["refused"]["answers"], 1);
}

/// Provenance lists what was done to a source, not every fix its settings allowed: a spread
/// split where one was found, no sample of a document shorter than it.
#[tokio::test]
async fn provenance_lists_only_the_fixes_that_were_made() {
    let f = fixture();
    let write = |name: &str, pages: &[(f32, f32)]| {
        let path = f.dir.path().join(name);
        std::fs::write(&path, kalamu_media::sample_pdf(pages, false))
            .unwrap_or_else(|e| panic!("{e}"));
        let mut source = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
        source.split_spreads = true;
        source
    };
    let flat = write("flat.pdf", &[(595.0, 842.0); 2]);
    let open = write("open.pdf", &[(595.0, 842.0), (1190.0, 842.0)]);
    let job = f
        .engine
        .create_job(NewJob {
            name: "Spreads".into(),
            task: Task::TextExtraction,
            sources: vec![flat, open],
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: None,
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: kalamu_core::JobSettings {
                sample_pages: Some(5),
                ..Default::default()
            },
            output: OutputSettings {
                formats: vec![ExportFormat::Txt],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(f.dir.path().join(name)).unwrap_or_default())
            .unwrap_or_default()
    };
    assert!(
        read("flat.provenance.json").get("preprocessing").is_none(),
        "no spread to split, no page left out of the sample"
    );
    assert_eq!(
        read("open.provenance.json")["preprocessing"],
        serde_json::json!(["split_spreads"])
    );
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
    // The files are written later, and before they are listed.
    f.engine.outputs(job.id).unwrap_or_else(|e| panic!("{e}"));
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
async fn a_page_that_keeps_failing_does_not_keep_the_document_from_disk() {
    let f = fixture();
    let source = f.pdf("bulletin.pdf", 3);
    f.fake.fail(2, vec![ProviderError::blocked("refused")]);
    let job = f.new_job(vec![source], &["fake-model"]);
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let read = |name: &str| std::fs::read_to_string(f.dir.path().join(name)).unwrap_or_default();
    let txt = read("bulletin.txt");
    assert!(txt.contains("fake-model page 1") && txt.contains("fake-model page 3"));
    assert!(
        txt.contains(
            "--- Page 2 ---

[Not read: "
        ),
        "{txt}"
    );
    let provenance: serde_json::Value =
        serde_json::from_str(&read("bulletin.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["failed"], serde_json::json!([2]));

    // Read on a retry, the page takes its place and the files are written again.
    f.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let mut events = f.engine.subscribe();
    loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { status, .. })) => {
                assert_eq!(status, JobStatus::Completed);
                break;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    }
    let txt = read("bulletin.txt");
    assert!(
        txt.contains("fake-model page 2") && !txt.contains("Not read"),
        "{txt}"
    );
}

#[tokio::test]
async fn sources_of_the_same_name_keep_their_files_apart() {
    let f = fixture();
    let mut sources = Vec::new();
    for (folder, pages) in [("janvier", 1), ("février", 2)] {
        let dir = f.dir.path().join(folder);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        let path = dir.join("Scan.pdf");
        let bytes = kalamu_media::sample_pdf(&vec![(595.0, 842.0); pages], false);
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("{e}"));
        sources.push(kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}")));
    }
    let out = f.dir.path().join("résultats");
    let job = f
        .engine
        .create_job(NewJob {
            name: "Scans".into(),
            task: Task::TextExtraction,
            sources,
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
                formats: vec![ExportFormat::Txt],
                destination: OutputDestination::Folder(out.display().to_string()),
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let read = |name: &str| std::fs::read_to_string(out.join(name)).unwrap_or_default();
    assert!(!read("Scan.txt").contains("Page 2"), "the first, one page");
    assert!(
        read("Scan (2).txt").contains("--- Page 2 ---"),
        "the second, two pages"
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
    assert!(job.setups[0].prompt.text.contains("## Reading order"));
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
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
    let txt = std::fs::read_to_string(f.dir.path().join("entretien.txt")).unwrap_or_default();
    assert!(txt.contains("Imam Traoré: page 1"), "{txt}");
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("entretien.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(
        provenance["renamed_speakers"],
        serde_json::json!(["Speaker 1"]),
        "provenance says which labels were renamed"
    );
    assert!(
        !provenance.to_string().contains("Traoré"),
        "but not the names of the people recorded"
    );
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
    assert_eq!(f.engine.write_outdated().ok(), Some(0));
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
    let job = f.engine.create_job(NewJob {
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
    });
    // The fake provider takes PDFs, the model does not: the job is refused before anything is
    // sent, with the reason each piece would have failed with.
    match job {
        Err(e @ EngineError::CannotRead { .. }) => {
            assert!(e.to_string().contains("reads text only"), "{e}");
        }
        other => panic!("expected CannotRead, got {other:?}"),
    }
    assert_eq!(f.fake.calls(), 0);
}

#[test]
fn a_pipeline_record_carries_the_prompt_a_job_would_send() {
    let f = fixture();
    let pipeline = kalamu_prompts::Pipeline {
        id: String::new(),
        name: "Places".into(),
        task: Task::Analysis,
        setups: vec![kalamu_core::SetupRequest {
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
        let job = self.new_job_for(
            Task::AudioTranscription,
            vec![source.clone()],
            &["fake-model"],
        );
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

#[test]
fn a_job_is_refused_a_source_its_task_does_not_read() {
    let f = fixture();
    let source = f.pdf("issue.pdf", 1);
    let refused = f.engine.create_job(NewJob {
        name: "Test".into(),
        task: Task::AudioTranscription,
        sources: vec![source],
        setups: vec![SetupRequest {
            provider: ProviderId::new("fake"),
            model: "fake-model".into(),
            preset_id: None,
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        output: Default::default(),
    });
    assert!(matches!(refused, Err(EngineError::UnsupportedSource(name)) if name == "issue.pdf"));
}

#[test]
fn a_job_keeps_only_the_formats_its_task_writes() {
    let f = fixture();
    let source = f.pdf("issue.pdf", 1);
    let job = f
        .engine
        .create_job(NewJob {
            name: "Test".into(),
            task: Task::TextExtraction,
            sources: vec![source],
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
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(job.output.formats, vec![ExportFormat::Txt]);
}

#[tokio::test]
async fn a_set_up_s_reasoning_level_is_settled_and_sent_with_the_model_s_output_limit() {
    let f = fixture();
    // The fake model as the catalogue would list it: two levels, a documented output limit.
    let mut catalog = kalamu_catalog::Catalog::bundled();
    let mut fake = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("model"));
    fake.provider = ProviderId::new("fake");
    fake.id = "fake-model".into();
    fake.reasoning = Some(kalamu_catalog::Reasoning {
        levels: vec!["none".into(), "high".into()],
        default: None,
    });
    fake.limits.max_output_tokens = Some(4_096);
    // Its abilities reach the provider with the level, whatever it is called.
    fake.refusal_fallback = true;
    fake.speakers = false;
    catalog.models.push(fake);
    f.engine.set_catalog(catalog);

    let job = f.new_job(vec![f.pdf("lettre.pdf", 1)], &["fake-model"]);
    assert_eq!(
        job.setups[0].options.reasoning.as_deref(),
        Some("none"),
        "reading a page: the lowest"
    );
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
    let sent = f
        .fake
        .tunings
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    assert_eq!(
        sent.last(),
        Some(&kalamu_core::Tuning {
            reasoning: Some("none".into()),
            max_output_tokens: Some(4_096),
            refusal_fallback: true,
            speakers: false,
            no_prompt: false,
        })
    );

    // A level the model lacks becomes the nearest it has, and is part of the answer's key.
    let chosen = f
        .engine
        .create_job(NewJob {
            name: "Higher".into(),
            task: Task::TextExtraction,
            sources: job.sources.clone(),
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: None,
                label: None,
                options: kalamu_core::SetupOptions {
                    reasoning: Some("xhigh".into()),
                    ..Default::default()
                },
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(chosen.setups[0].options.reasoning.as_deref(), Some("high"));
    let calls = f.fake.calls();
    assert_eq!(f.run(chosen.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), calls + 1, "another level is another answer");

    // Names to spell and translation are kept only for a model that uses them (Whisper), so
    // they never change another model's answer or its key.
    let ignored = f
        .engine
        .create_job(NewJob {
            name: "Names".into(),
            task: Task::TextExtraction,
            sources: job.sources.clone(),
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: None,
                label: None,
                options: kalamu_core::SetupOptions {
                    vocabulary: Some("Samory Touré".into()),
                    translate: true,
                    ..Default::default()
                },
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ignored.setups[0].options.vocabulary, None);
    assert!(!ignored.setups[0].options.translate);
}

/// Gemini 3.5 Transcribe, as it answered live: priced by the minute in the catalogue, but its
/// usage counted in tokens, not seconds.
struct TranscribeFake;

#[async_trait]
impl Provider for TranscribeFake {
    fn id(&self) -> ProviderId {
        ProviderId::new("gemini")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["audio/wav"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        _request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        Ok(ProcessOutput {
            text: "Speaker 1: Bonjour.".into(),
            usage: kalamu_core::Usage {
                input_tokens: 1_307,
                output_tokens: 177,
                ..Default::default()
            },
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

/// A transcription billed in tokens is costed from them through a whole job, its cost in the
/// job's total and in provenance, not left at nothing because no seconds were reported.
#[tokio::test]
async fn gemini_transcribe_is_costed_from_its_tokens_through_a_whole_job() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![Arc::new(TranscribeFake)],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let path = dir.path().join("interview.wav");
    std::fs::write(&path, tone_wav()).unwrap_or_else(|_| panic!("write recording"));
    let source = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = engine
        .create_job(NewJob {
            name: "Interview".into(),
            task: Task::AudioTranscription,
            sources: vec![source],
            setups: vec![SetupRequest {
                provider: ProviderId::new("gemini"),
                model: "gemini-3.5-transcribe".into(),
                preset_id: None,
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
    let mut events = engine.subscribe();
    engine.start(job.id).unwrap_or_else(|e| panic!("{e}"));
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == job.id => {
                break status;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    };
    assert_eq!(status, JobStatus::Completed);
    // Google's token prices: $2 a million read, $12 a million written.
    let expected = (1_307.0 * 2.0 + 177.0 * 12.0) / 1_000_000.0;
    let spent = store.spent(job.id).unwrap_or_default();
    assert!(
        (spent - expected).abs() < 1e-9,
        "{spent} against {expected}"
    );
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.path().join("interview.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    let cost = provenance["cost_usd"]
        .as_f64()
        .or_else(|| provenance["usage"]["cost_usd"].as_f64())
        .unwrap_or_default();
    assert!((cost - expected).abs() < 1e-9, "{provenance}");
}

/// A recording keeps the cuts planned the first time only when they were recorded whole: a plan
/// recorded in part (by a version that wrote it a row at a time, closed part-way) is planned
/// again rather than taken for the whole recording; and a source's plan is now written at once.
#[tokio::test]
async fn a_recording_planned_in_part_is_planned_again_on_resume() {
    let f = fixture();
    let path = f.dir.path().join("interview.wav");
    std::fs::write(&path, tone_wav()).unwrap_or_else(|_| panic!("write recording"));
    let source = kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"));
    let job = f.new_job_for(Task::AudioTranscription, vec![source], &["fake-model"]);
    let mut source = job.sources[0].clone();
    source.duration_ms = Some(1_000);
    let opened =
        Arc::new(kalamu_media::OpenSource::open(&source, None).unwrap_or_else(|e| panic!("{e}")));
    let segment = |index: u32, start_ms: u64, end_ms: u64| kalamu_core::PieceRef::Segment {
        index,
        start_ms,
        end_ms,
    };
    let recorded = |cuts: &[(u64, u64)]| -> crate::plan::Existing {
        cuts.iter()
            .enumerate()
            .map(|(i, (start, end))| {
                let index = i as u32;
                let state = kalamu_core::PieceState {
                    job_id: job.id,
                    setup_id: job.setups[0].id,
                    source_id: source.id,
                    index,
                    piece: segment(index, *start, *end),
                    status: PieceStatus::Pending,
                    cache_key: None,
                    error: None,
                    attempts: 0,
                    updated_at: Timestamp(0),
                };
                ((state.setup_id, state.source_id, index), state)
            })
            .collect()
    };
    let plan = |existing: crate::plan::Existing| {
        let (engine, job, source, opened) = (&f.engine, &job, &source, &opened);
        async move {
            engine
                .plan(job, source, opened, &existing)
                .await
                .unwrap_or_else(|e| panic!("{e}"))
                .unwrap_or_else(|e| panic!("{e}"))
        }
    };

    // Only the first of two pieces was recorded: the recording is planned again, whole.
    assert_eq!(
        plan(recorded(&[(0, 400)])).await,
        vec![segment(0, 0, 1_000)]
    );
    // Recorded to its end: the cuts are kept, not moved under results already stored.
    assert_eq!(
        plan(recorded(&[(0, 400), (400, 1_000)])).await,
        vec![segment(0, 0, 400), segment(1, 400, 1_000)]
    );

    // Every piece of the source is recorded, in one go.
    let pieces = vec![segment(0, 0, 400), segment(1, 400, 1_000)];
    let work = f
        .engine
        .record_pieces(&job, &source, &pieces, &crate::plan::Existing::new())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(work.len(), 2);
    let stored = f.store.pieces(job.id).unwrap_or_default();
    assert_eq!(
        stored.iter().map(|p| p.piece.clone()).collect::<Vec<_>>(),
        pieces
    );
}

/// A chosen format that cannot be written reaches the researcher, the other files written all
/// the same; one the set-up's model cannot have (ALTO from a model that reports no positions,
/// in a comparison where another does) is skipped without a word.
#[tokio::test]
async fn a_chosen_format_that_cannot_be_written_is_reported() {
    let f = fixture();
    let mut catalog = f.engine.catalog();
    let listed = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("a listed model"));
    for (id, layout) in [("positional-model", true), ("plain-model", false)] {
        let mut model = listed.clone();
        model.provider = ProviderId::new("fake");
        model.id = id.into();
        model.layout = layout;
        catalog.models.push(model);
    }
    f.engine.set_catalog(catalog);
    let job = f
        .engine
        .create_job(NewJob {
            name: "Alto".into(),
            task: Task::TextExtraction,
            sources: vec![f.pdf("bulletin.pdf", 1)],
            setups: ["positional-model", "plain-model"]
                .map(|model| SetupRequest {
                    provider: ProviderId::new("fake"),
                    model: model.into(),
                    preset_id: None,
                    label: None,
                    options: Default::default(),
                    merge: None,
                })
                .into(),
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Alto],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("create job: {e}"));
    assert!(job.output.formats.contains(&ExportFormat::Alto));

    // The fake model reports no positions though listed as one that does: ALTO fails, once,
    // for that set-up alone, and says why.
    let mut events = f.engine.subscribe();
    assert_eq!(f.run(job.id).await, JobStatus::CompletedWithErrors);
    let (mut reported, mut written) = (Vec::new(), Vec::new());
    while let Ok(event) = events.try_recv() {
        match event {
            ProgressEvent::SourceFailed { error, .. } => reported.push(error),
            ProgressEvent::SourceExported { files, .. } => written.extend(files),
            _ => {}
        }
    }
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert!(reported[0].contains("text positions"), "{reported:?}");
    assert_eq!(
        written.iter().filter(|f| f.ends_with(".txt")).count(),
        2,
        "the text of both set-ups is written: {written:?}"
    );
    assert!(!written.iter().any(|f| f.ends_with(".xml")), "{written:?}");

    // Written again on request, the failure is the call's.
    let again = f.engine.export(job.id, job.setups[0].id, job.sources[0].id);
    assert!(
        matches!(&again, Err(EngineError::Output(e)) if e.contains("text positions")),
        "{again:?}"
    );
    assert!(
        f.engine
            .export(job.id, job.setups[1].id, job.sources[0].id)
            .is_ok()
    );
}

/// A pipeline saved with an id the catalogue has since corrected runs the same model, and the
/// job records and sends the id as it is now.
#[tokio::test]
async fn an_old_model_id_is_sent_as_the_catalogue_writes_it_now() {
    let f = fixture();
    let mut catalog = f.engine.catalog();
    let mut model = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("a model to copy"));
    model.provider = ProviderId::new("fake");
    model.id = "fake-model-1".into();
    catalog.models.push(model);
    catalog.aliases.push(kalamu_catalog::ModelAlias {
        provider: ProviderId::new("fake"),
        id: "fake-model-one".into(),
        model: "fake-model-1".into(),
    });
    f.engine.set_catalog(catalog);
    let job = f.new_job(vec![f.pdf("letters.pdf", 1)], &["fake-model-one"]);
    assert_eq!(job.setups[0].model, "fake-model-1");
    assert_eq!(f.run(job.id).await, JobStatus::Completed);
}
