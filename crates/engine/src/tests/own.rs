//! Servers of the researcher's own: a job run on one end to end, what provenance says of it,
//! the providers replaced while a job runs, and the rules that offer a server's model only for
//! what it did in its trial.

use kalamu_catalog::{Catalog, Dialect, OwnModel, OwnServer, OwnServers};
use kalamu_core::{Reach, ServerRecord};
use tokio::sync::{Notify, Semaphore};

use super::*;

const SERVER: &str = "own-0000abcd";

/// A server of one's own: it reads every page as its model's name and the page's number, from
/// a picture, and can be made to wait inside each call until let go.
struct Own {
    reach: Reach,
    address: String,
    calls: AtomicUsize,
    /// The types of file it was sent, in order.
    sent: Mutex<Vec<String>>,
    /// When set, each call says it started and waits for a permit.
    gated: bool,
    entered: Notify,
    release: Semaphore,
}

impl Own {
    fn new(reach: Reach, address: &str, gated: bool) -> Arc<Self> {
        Arc::new(Self {
            reach,
            address: address.into(),
            calls: AtomicUsize::new(0),
            sent: Mutex::new(Vec::new()),
            gated,
            entered: Notify::new(),
            release: Semaphore::new(0),
        })
    }
}

#[async_trait]
impl Provider for Own {
    fn id(&self) -> ProviderId {
        ProviderId::new(SERVER)
    }

    fn needs_key(&self) -> bool {
        false
    }

    fn reach(&self) -> Reach {
        self.reach
    }

    fn server(&self) -> Option<ServerRecord> {
        Some(ServerRecord {
            name: "Office Mac".into(),
            address: self.address.clone(),
        })
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["image/png", "image/jpeg"],
            _ => &["text/plain", "image/png", "image/jpeg"],
        }
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.sent
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(request.input.mime.clone());
        if self.gated {
            self.entered.notify_one();
            if let Ok(permit) = self.release.acquire().await {
                permit.forget();
            }
        }
        let number = match &request.input.piece {
            kalamu_core::PieceRef::Page { number, .. } => *number,
            _ => 0,
        };
        Ok(ProcessOutput {
            text: format!("{} page {number}", request.model),
            usage: Usage {
                input_tokens: 1000,
                output_tokens: 100,
                ..Usage::default()
            },
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["pages-only".into(), "forms-only".into()])
    }
}

fn own_model(id: &str, reads_pages: bool, fills_forms: bool) -> OwnModel {
    OwnModel {
        id: id.into(),
        reads_pages,
        fills_forms,
        context: 16_384,
        max_output: 4_096,
        input_per_mtok: None,
        output_per_mtok: None,
        tried_at: None,
        digest: None,
    }
}

fn servers(address: &str) -> OwnServers {
    OwnServers {
        servers: vec![OwnServer {
            id: ProviderId::new(SERVER),
            name: "Office Mac".into(),
            address: address.into(),
            dialect: Dialect::Ollama,
            key: false,
            at_once: 1,
            models: vec![
                own_model("pages-only", true, false),
                own_model("forms-only", false, true),
            ],
        }],
    }
}

struct Bench {
    engine: Arc<Engine>,
    own: Arc<Own>,
    dir: tempfile::TempDir,
}

fn engine_with(own: Arc<Own>, concurrency: usize) -> Bench {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store,
        vec![Fake::new(), own.clone()],
        Catalog::bundled().with_servers(&servers(&own.address)),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency,
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Bench { engine, own, dir }
}

impl Bench {
    fn pdf(&self, pages: usize) -> SourceFile {
        let path = self.dir.path().join("letters.pdf");
        let bytes = kalamu_media::sample_pdf(&vec![(595.0, 842.0); pages], false);
        std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("write pdf"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"))
    }

    fn request(&self, sources: Vec<SourceFile>) -> NewJob {
        NewJob {
            name: "Letters".into(),
            task: Task::TextExtraction,
            sources,
            setups: vec![SetupRequest {
                provider: ProviderId::new(SERVER),
                model: "pages-only".into(),
                preset_id: None,
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Json],
                ..Default::default()
            },
        }
    }

    async fn finished(&self, id: JobId) -> JobStatus {
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

    fn provenance(&self) -> serde_json::Value {
        let path = self.dir.path().join("letters.provenance.json");
        serde_json::from_slice(&std::fs::read(path).unwrap_or_default()).unwrap_or_default()
    }
}

/// One small job on a server, from estimate to provenance: not priced, each page sent as a
/// picture, one a call, and the files saying where it was read.
async fn a_job_on(reach: Reach, address: &str) {
    let s = engine_with(Own::new(reach, address, false), 2);
    let source = s.pdf(2);
    let request = s.request(vec![source]);

    let estimate = s.engine.estimate(&EstimateRequest {
        task: request.task,
        sources: request.sources.clone(),
        setups: request.setups.clone(),
        sample_pages: None,
    });
    assert_eq!(estimate.total_usd, None, "a server without a price");
    assert!(estimate.rows.iter().all(|r| r.estimate.usd.is_none()));

    let job = s
        .engine
        .create_job(request)
        .unwrap_or_else(|e| panic!("create job: {e}"));
    assert_eq!(s.finished(job.id).await, JobStatus::Completed);
    assert_eq!(s.own.calls.load(Ordering::SeqCst), 2, "one page a call");
    let sent = s.own.sent.lock().unwrap_or_else(|p| p.into_inner()).clone();
    assert!(sent.iter().all(|m| m == "image/png"), "{sent:?}");

    let provenance = s.provenance();
    assert_eq!(provenance["engine"], "own");
    assert_eq!(provenance["provider"], SERVER);
    assert_eq!(provenance["server"]["name"], "Office Mac");
    assert_eq!(provenance["server"]["address"], address);
    assert_eq!(provenance["cost_usd"], serde_json::Value::Null);
    let txt = std::fs::read_to_string(s.dir.path().join("letters.txt")).unwrap_or_default();
    assert!(txt.contains("pages-only page 2"), "{txt}");
}

#[tokio::test]
async fn a_job_on_a_server_on_the_network_records_the_server() {
    a_job_on(Reach::Network, "http://192.168.1.20:11434").await;
}

#[tokio::test]
async fn a_job_on_a_server_on_this_computer_is_still_own() {
    a_job_on(Reach::ThisComputer, "http://localhost:11434").await;
}

/// The engine's own providers keep their words: `offline` for an engine inside Kalamu,
/// `cloud` for a company's, and no server for either.
#[test]
fn provenance_names_cloud_offline_and_own_apart() {
    let s = engine_with(Own::new(Reach::Network, "http://10.0.0.2", false), 1);
    assert_eq!(
        s.engine.where_read(&ProviderId::new("fake")),
        ("cloud", None)
    );
    let (engine, server) = s.engine.where_read(&ProviderId::new(SERVER));
    assert_eq!(engine, "own");
    assert_eq!(server.map(|s| s.address), Some("http://10.0.0.2".into()));
    // A provider no longer registered reads as before servers existed.
    assert_eq!(
        s.engine.where_read(&ProviderId::new("gone")),
        ("cloud", None)
    );
    // Replaced providers are asked from then on.
    s.engine.set_providers(vec![Fake::new()]);
    assert_eq!(
        s.engine.where_read(&ProviderId::new(SERVER)),
        ("cloud", None)
    );
    assert!(matches!(
        s.engine.provider(&ProviderId::new(SERVER)),
        Err(EngineError::UnknownProvider(_))
    ));
}

/// A server removed while its job runs: the call already sent finishes and is kept, and the
/// job stops at its next page with the unknown provider's error, without a panic and without
/// leaving a page marked as running.
#[tokio::test]
async fn a_server_removed_while_its_job_runs_stops_the_job_at_its_next_page() {
    let s = engine_with(Own::new(Reach::Network, "http://10.0.0.2", true), 1);
    let source = s.pdf(3);
    let job = s
        .engine
        .create_job(s.request(vec![source]))
        .unwrap_or_else(|e| panic!("create job: {e}"));

    let mut events = s.engine.subscribe();
    s.engine
        .start(job.id)
        .unwrap_or_else(|e| panic!("start: {e}"));
    tokio::time::timeout(Duration::from_secs(10), s.own.entered.notified())
        .await
        .unwrap_or_else(|_| panic!("the first page was never sent"));
    s.engine.set_providers(vec![Fake::new()]);
    s.own.release.add_permits(10);
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(10), events.recv()).await {
            Ok(Ok(ProgressEvent::JobFinished { job_id, status })) if job_id == job.id => {
                break status;
            }
            Ok(Ok(_)) => {}
            other => panic!("no JobFinished event: {other:?}"),
        }
    };
    // Stopped by an engine error, as for a key removed while a job runs: resumable.
    assert_eq!(status, JobStatus::Interrupted);
    assert_eq!(s.own.calls.load(Ordering::SeqCst), 1);
    let pieces = s.engine.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 1, "{pieces:?}");
    assert_eq!(count(&pieces, PieceStatus::Running), 0, "{pieces:?}");
    assert_eq!(count(&pieces, PieceStatus::Pending), 2, "{pieces:?}");

    // Put back, the server finishes the job without reading the first page again.
    s.engine
        .set_providers(vec![Fake::new(), s.own.clone() as Arc<dyn Provider>]);
    assert_eq!(s.finished(job.id).await, JobStatus::Completed);
    assert_eq!(s.own.calls.load(Ordering::SeqCst), 3);
}

/// The safeguard the researcher asked for: a model is offered only for what it did in its
/// trial, in the rules New job reads and in what the engine sends it.
#[test]
fn a_servers_model_is_offered_only_for_what_it_did_in_its_trial() {
    let catalog = Catalog::bundled().with_servers(&servers("http://localhost:11434"));
    let rules = task_rules(&catalog);
    let offered = |task: Task| -> Vec<String> {
        rules
            .iter()
            .find(|r| r.task == task)
            .map(|r| {
                r.models
                    .iter()
                    .filter(|m| m.provider.as_str() == SERVER)
                    .map(|m| m.model.clone())
                    .collect()
            })
            .unwrap_or_default()
    };
    assert_eq!(offered(Task::TextExtraction), ["pages-only"]);
    for task in [Task::Analysis, Task::TextCorrection, Task::Translation] {
        assert_eq!(offered(task), ["forms-only"], "{task:?}");
    }
    // A table of contents reads pictures and fills a form: neither model did both.
    assert!(offered(Task::Contents).is_empty());
    assert!(offered(Task::AudioTranscription).is_empty());
    // Never anyone's default.
    assert!(
        rules
            .iter()
            .flat_map(|r| &r.models)
            .all(|m| { m.provider.as_str() != SERVER || (!m.recommended && !m.provider_default) })
    );

    // What the engine sends each: a text-only model text alone, a pages-only model pictures.
    let own = Own::new(Reach::ThisComputer, "http://localhost:11434", false);
    let forms = catalog.model(&ProviderId::new(SERVER), "forms-only");
    let pages = catalog.model(&ProviderId::new(SERVER), "pages-only");
    assert_eq!(
        crate::piece::accepted_inputs(own.as_ref(), forms, Task::Analysis),
        ["text/plain"]
    );
    assert_eq!(
        crate::piece::accepted_inputs(own.as_ref(), pages, Task::TextExtraction),
        ["image/png", "image/jpeg"]
    );
    assert!(crate::piece::accepted_inputs(own.as_ref(), forms, Task::TextExtraction).is_empty());
}

/// A job asking a server's model for what its trial did not show is refused before anything
/// is sent.
#[test]
fn a_job_asking_a_model_for_what_it_did_not_do_is_refused() {
    let s = engine_with(
        Own::new(Reach::ThisComputer, "http://localhost:11434", false),
        1,
    );
    let mut request = s.request(vec![s.pdf(1)]);
    request.setups[0].model = "forms-only".into();
    assert!(matches!(
        s.engine.create_job(request),
        Err(EngineError::UnsupportedModel(_))
    ));
    assert_eq!(s.own.calls.load(Ordering::SeqCst), 0);
}
