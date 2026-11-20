//! Preparation runs and questions through the engine, with the fake embedder over a real job
//! database and vector index: consent, keys and readiness checked first, a run stopped part-way
//! going on without embedding anything twice, a rate limit retried, a refused key recorded, and
//! the cost of runs and questions kept.

#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

use kalamu_core::test_support::FakeEmbedder;
use kalamu_core::{
    ApiKey, Document, DocumentId, DocumentLink, EmbedLimits, EmbeddingLog, EmbeddingModel,
    EmbeddingRunState, InputKind, Job, JobId, JobSettings, JobStatus, JobStore, OutputSettings,
    PieceRef, PieceState, PieceStatus, ProcessOutput, Project, ProjectId, ProjectStore,
    PromptSnapshot, ProviderError, ProviderId, Setup, SetupId, SetupOptions, SourceFile, SourceId,
    StoredResult, Task, Timestamp,
};
use kalamu_prompts::PromptLibrary;
use kalamu_search::{MeaningRequest, ResultPolicy, ReviewFilter, Search};
use kalamu_store::{SqliteIndex, SqliteStore, SqliteVectors};

use super::*;
use crate::{EngineConfig, KeySource};

/// A key for the cloud embedder only.
struct Keys;

impl KeySource for Keys {
    fn key(&self, provider: &ProviderId) -> Option<ApiKey> {
        (provider.as_str() == "cloud").then(|| ApiKey::new("test"))
    }
}

fn model(provider: &str, local: bool) -> EmbeddingModel {
    EmbeddingModel {
        id: format!("{provider}-embed"),
        provider: ProviderId::new(provider),
        name: format!("{provider} embed"),
        dimensions: vec![64, 32],
        max_chars: 2000,
        input_price: (!local).then_some(0.15),
        local,
        multilingual: true,
    }
}

const LIMITS: EmbedLimits = EmbedLimits {
    texts: 2,
    chars: 2000,
};

struct Fixture {
    meaning: Arc<Meaning>,
    cloud: Arc<FakeEmbedder>,
    local: Arc<FakeEmbedder>,
    store: Arc<SqliteStore>,
    project: ProjectId,
}

fn fixture(pages: usize) -> Fixture {
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let engine = Engine::new(
        store.clone(),
        Vec::new(),
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(5),
            ..EngineConfig::default()
        },
    );
    let vectors = Arc::new(SqliteVectors::in_memory().unwrap());
    let search = Arc::new(
        Search::new(
            store.clone(),
            store.clone(),
            Arc::new(SqliteIndex::in_memory().unwrap()),
        )
        .with_vectors(vectors.clone()),
    );
    let cloud = Arc::new(FakeEmbedder::cloud("cloud", LIMITS));
    let local = Arc::new(FakeEmbedder::local("offline", LIMITS));
    // A provider with no key saved.
    let nokey = Arc::new(FakeEmbedder::cloud("nokey", LIMITS));
    let meaning = Meaning::new(
        engine,
        search,
        vectors,
        store.clone(),
        vec![cloud.clone(), local.clone(), nokey],
        vec![
            model("cloud", false),
            model("offline", true),
            model("nokey", false),
            // Its embedder is missing: not offered.
            model("absent", false),
        ],
    );
    let project = project_with_pages(&store, pages);
    Fixture {
        meaning,
        cloud,
        local,
        store,
        project,
    }
}

/// A project of one document whose extraction read `pages` pages, each about a different
/// subject.
fn project_with_pages(store: &SqliteStore, pages: usize) -> ProjectId {
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
    let source = SourceFile {
        id: SourceId::new(),
        path: "archive/report.pdf".into(),
        file_name: "report.pdf".into(),
        sha256: "x".into(),
        size_bytes: 1,
        modified_ms: None,
        kind: InputKind::Pdf,
        page_count: Some(pages as u32),
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: None,
        pages: Vec::new(),
    };
    let document = Document {
        id: DocumentId::new(),
        project_id: project.id,
        title: "Report".into(),
        source: source.clone(),
        created_at: Timestamp(1),
        tags: Vec::new(),
        source_description: Default::default(),
    };
    store.put_document(&document).unwrap();
    let setup = Setup {
        id: SetupId::new(),
        label: "Model".into(),
        provider: ProviderId::new("fake"),
        model: "m".into(),
        prompt: PromptSnapshot {
            preset_id: None,
            preset_name: "P".into(),
            template: String::new(),
            text: String::new(),
            user_prompt: String::new(),
            schema: None,
        },
        options: SetupOptions::default(),
        merge: None,
        pages_cap: None,
        windows: None,
    };
    let job = Job {
        id: JobId::new(),
        name: "Extraction".into(),
        task: Task::TextExtraction,
        created_at: Timestamp(1),
        sources: vec![source.clone()],
        setups: vec![setup.clone()],
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        status: JobStatus::Completed,
    };
    store.create_job(&job).unwrap();
    for i in 0..pages {
        let key = format!("{}-{i}", job.id);
        store
            .put_result(&StoredResult {
                cache_key: key.clone(),
                provider: ProviderId::new("fake"),
                model: "m".into(),
                output: ProcessOutput {
                    text: format!(
                        "Page {i} of the report tells of subject number {i}, the mosque, the \
                         school and the market of the town in the year 19{i:02}."
                    ),
                    ..ProcessOutput::default()
                },
                request: Vec::new(),
                cost_usd: None,
                duration_ms: 1,
                created_at: Timestamp(1),
            })
            .unwrap();
        store
            .upsert_piece(&PieceState {
                job_id: job.id,
                setup_id: setup.id,
                source_id: source.id,
                index: i as u32,
                piece: PieceRef::page(i as u32 + 1),
                status: PieceStatus::Done,
                cache_key: Some(key),
                error: None,
                attempts: 1,
                updated_at: Timestamp(1),
            })
            .unwrap();
    }
    store
        .put_link(&DocumentLink {
            document_id: document.id,
            job_id: job.id,
            source_id: source.id,
            linked_at: Timestamp(1),
        })
        .unwrap();
    project.id
}

fn ask(query: &str) -> MeaningRequest {
    MeaningRequest {
        query: query.into(),
        collection: None,
        tag: None,
        task: None,
        review: ReviewFilter::Any,
        results: ResultPolicy::Preferred,
    }
}

impl Fixture {
    fn choose(&self, provider: &str) {
        self.meaning
            .choose(
                self.project,
                &ProviderId::new(provider),
                &format!("{provider}-embed"),
            )
            .unwrap();
    }

    async fn prepare(&self) -> Result<EmbeddingRun, MeaningError> {
        let ready = self.meaning.check(self.project)?;
        self.meaning.clone().run(ready).await
    }
}

#[tokio::test]
async fn nothing_is_sent_without_a_model_consent_a_key_or_the_model_downloaded() {
    let f = fixture(3);
    assert!(matches!(
        f.meaning.check(f.project),
        Err(MeaningError::NoModel)
    ));
    let offered: Vec<String> = f.meaning.models().into_iter().map(|m| m.id).collect();
    assert_eq!(offered, ["cloud-embed", "offline-embed", "nokey-embed"]);
    assert!(matches!(
        f.meaning
            .choose(f.project, &ProviderId::new("absent"), "absent-embed"),
        Err(MeaningError::UnknownModel(_))
    ));

    // A cloud model: refused until the researcher agrees, the question too.
    f.choose("cloud");
    let status = f.meaning.status(f.project).unwrap();
    assert_eq!(status.counts.as_ref().unwrap().chunks, 3);
    assert!(status.estimated_cost_usd.unwrap() > 0.0);
    assert_eq!(status.choice.unwrap().space.dimensions, 64);
    assert!(matches!(
        f.meaning.check(f.project),
        Err(MeaningError::NoConsent(_))
    ));
    assert!(matches!(
        f.meaning.search(f.project, ask("mosque")).await,
        Err(MeaningError::NoConsent(_))
    ));

    f.choose("nokey");
    assert!(matches!(
        f.meaning.check(f.project),
        Err(MeaningError::MissingKey(_))
    ));
    assert_eq!(
        f.meaning.status(f.project).unwrap().blocked.as_deref(),
        Some("provider.missing_key")
    );

    f.choose("offline");
    f.local.set_missing(true);
    let err = f.meaning.check(f.project).err().unwrap();
    assert_eq!(err.code(), "provider.not_installed");
    assert!(f.cloud.calls().is_empty() && f.local.calls().is_empty());

    // A local model needs no consent: nothing leaves the computer.
    f.local.set_missing(false);
    let run = f.prepare().await.unwrap();
    assert_eq!(run.state, EmbeddingRunState::Done);
    assert_eq!((run.chunks, run.tokens, run.cost_usd), (3, 0, None));
    assert!(f.local.calls().iter().all(|c| !c.keyed));
}

#[tokio::test]
async fn a_run_records_its_cost_and_a_question_adds_its_own() {
    let f = fixture(5);
    f.choose("cloud");
    f.meaning.consent(f.project).unwrap();
    let run = f.prepare().await.unwrap();
    assert_eq!(run.state, EmbeddingRunState::Done);
    assert_eq!(run.chunks, 5);
    assert!(run.tokens > 0);
    let expected = run.tokens as f64 * 0.15 / 1_000_000.0;
    assert!((run.cost_usd.unwrap() - expected).abs() < 1e-12);
    // Three calls of at most two texts.
    assert_eq!(f.cloud.calls().len(), 3);
    assert!(
        f.cloud
            .calls()
            .iter()
            .all(|c| c.keyed && c.texts.len() <= 2)
    );

    let view = f
        .meaning
        .search(f.project, ask("subject number 3"))
        .await
        .unwrap();
    assert_eq!(view.documents.len(), 1);
    assert_eq!(view.documents[0].passages[0].hit.index, 3);
    let status = f.meaning.status(f.project).unwrap();
    let choice = status.choice.unwrap();
    assert_eq!(choice.queries, 1);
    assert!(choice.query_tokens > 0);
    assert_eq!(status.counts.unwrap().chunks, 0);
    assert!(status.prepared_at.is_some());
    assert!(status.spent_usd.unwrap() > run.cost_usd.unwrap());
    assert_eq!(f.store.embedding_runs(f.project).unwrap().len(), 1);
    // The map needs five documents; one is not enough.
    let map = f.meaning.map(f.project).unwrap();
    assert!(map.points.is_empty());
    assert_eq!(map.placed, 1);
}

#[tokio::test]
async fn a_run_paused_part_way_goes_on_without_embedding_anything_twice() {
    let f = fixture(10);
    f.choose("cloud");
    f.meaning.consent(f.project).unwrap();
    let meaning = Arc::downgrade(&f.meaning);
    let project = f.project;
    f.cloud.after_call(move |n| {
        if n == 2
            && let Some(m) = meaning.upgrade()
        {
            m.pause(project);
        }
    });
    let mut progress = f.meaning.subscribe();
    let first = f.prepare().await.unwrap();
    assert_eq!(first.state, EmbeddingRunState::Paused);
    assert_eq!(first.chunks, 4);
    let mut last = None;
    while let Ok(p) = progress.try_recv() {
        last = Some(p);
    }
    let last = last.unwrap();
    assert_eq!((last.done, last.total), (4, 10));
    assert_eq!(
        f.meaning.status(f.project).unwrap().counts.unwrap().chunks,
        6
    );

    let second = f.prepare().await.unwrap();
    assert_eq!(second.state, EmbeddingRunState::Done);
    assert_eq!(second.chunks, 6);
    let mut sent = f.cloud.passages();
    assert_eq!(sent.len(), 10);
    sent.sort();
    sent.dedup();
    assert_eq!(sent.len(), 10, "no chunk sent twice");
    let runs = f.store.embedding_runs(f.project).unwrap();
    assert_eq!(runs.len(), 2);
}

#[tokio::test]
async fn a_rate_limit_is_waited_out_and_a_refused_key_ends_the_run() {
    let f = fixture(2);
    f.choose("cloud");
    f.meaning.consent(f.project).unwrap();
    f.cloud.fail_with([ProviderError::RateLimited {
        retry_after: Some(Duration::from_millis(1)),
        message: "slow down".into(),
    }]);
    let run = f.prepare().await.unwrap();
    assert_eq!((run.state, run.chunks), (EmbeddingRunState::Done, 2));

    // Another project, with a key the provider refuses.
    let f = fixture(2);
    f.choose("cloud");
    f.meaning.consent(f.project).unwrap();
    f.cloud.fail_with([ProviderError::Auth("bad key".into())]);
    let err = f.prepare().await.unwrap_err();
    assert_eq!(err.code(), "provider.auth");
    let runs = f.store.embedding_runs(f.project).unwrap();
    assert_eq!(runs[0].state, EmbeddingRunState::Failed);
    assert_eq!(runs[0].error.as_deref(), Some("provider.auth"));
    assert_eq!(runs[0].chunks, 0);
    // Nothing kept from it: the next run embeds all.
    assert_eq!(
        f.meaning.status(f.project).unwrap().counts.unwrap().chunks,
        2
    );
}

#[tokio::test]
async fn a_second_run_does_not_start_beside_the_first_and_runs_left_running_are_interrupted() {
    let f = fixture(2);
    f.choose("offline");
    let ready = f.meaning.check(f.project).unwrap();
    assert!(matches!(
        f.meaning.check(f.project),
        Err(MeaningError::AlreadyRunning)
    ));
    f.meaning.clone().run(ready).await.unwrap();
    assert!(f.meaning.check(f.project).is_ok());

    let mut run = f.store.embedding_runs(f.project).unwrap()[0].clone();
    run.state = EmbeddingRunState::Running;
    f.store.update_run(&run).unwrap();
    assert_eq!(f.meaning.recover().unwrap(), 1);
}
