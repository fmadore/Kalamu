//! The pretend model and the sources an interview analysis is tested with.

use std::collections::VecDeque;

use kalamu_core::analysis::cited::CitedValue;
use kalamu_core::{Correction, OutputDestination};

use super::super::*;

/// Answers each call with the next scripted JSON answer (the last one again once they run
/// out), read as any provider reads an analysis's answer. Can be told to stop part-way through
/// the call for a text holding some words, once.
pub(super) struct Interviewer {
    calls: AtomicUsize,
    /// What each call read, and the request that came with it.
    sent: Mutex<Vec<(String, String)>>,
    answers: Mutex<VecDeque<String>>,
    last: Mutex<String>,
    /// Words of a text whose first call is stopped, as a pause stops it.
    pub(super) interrupt: Mutex<Option<String>>,
    /// Words of a text whose every call fails, as a provider refusing it would.
    pub(super) fail: Mutex<Option<String>>,
}

impl Interviewer {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            calls: AtomicUsize::new(0),
            sent: Mutex::new(Vec::new()),
            answers: Mutex::new(VecDeque::new()),
            last: Mutex::new(String::new()),
            interrupt: Mutex::new(None),
            fail: Mutex::new(None),
        })
    }

    pub(super) fn script(&self, answers: &[serde_json::Value]) {
        let mut queue = self.answers.lock().unwrap_or_else(|p| p.into_inner());
        queue.clear();
        queue.extend(answers.iter().map(|a| a.to_string()));
    }

    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub(super) fn sent(&self) -> Vec<(String, String)> {
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

#[async_trait]
impl Provider for Interviewer {
    fn id(&self) -> ProviderId {
        ProviderId::new("interviewer")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf", "text/plain"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let read = String::from_utf8_lossy(&request.input.bytes).into_owned();
        self.sent
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push((read.clone(), request.user_prompt.to_string()));
        {
            let mut interrupt = self.interrupt.lock().unwrap_or_else(|p| p.into_inner());
            if interrupt.as_deref().is_some_and(|w| read.contains(w)) {
                *interrupt = None;
                return Err(ProviderError::Interrupted);
            }
        }
        if self
            .fail
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_deref()
            .is_some_and(|w| read.contains(w))
        {
            return Err(ProviderError::InvalidRequest("this part is refused".into()));
        }
        let answer = {
            let mut queue = self.answers.lock().unwrap_or_else(|p| p.into_inner());
            let mut last = self.last.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(next) = queue.pop_front() {
                *last = next;
            }
            last.clone()
        };
        let schema = request
            .analysis
            .unwrap_or_else(|| panic!("an analysis answers in fields"));
        kalamu_core::structured::finish(
            ProcessOutput {
                text: answer,
                usage: Usage {
                    input_tokens: 2_000,
                    output_tokens: 300,
                    ..Usage::default()
                },
                ..ProcessOutput::default()
            },
            schema,
        )
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

pub(super) struct Interviews {
    pub(super) engine: Arc<Engine>,
    pub(super) model: Arc<Interviewer>,
    pub(super) dir: tempfile::TempDir,
}

pub(super) fn interviews() -> Interviews {
    interviews_with(kalamu_catalog::Catalog::bundled())
}

/// The pretend model listed in the catalogue as `Interviewer One`, reading at most `tokens`
/// tokens at once.
pub(super) fn interviews_reading(tokens: u32) -> Interviews {
    let mut catalog = kalamu_catalog::Catalog::bundled();
    let mut model = catalog
        .models_for(Task::Analysis)
        .into_iter()
        .find(|m| m.inputs.iter().any(|i| i == "text/plain"))
        .cloned()
        .unwrap_or_else(|| panic!("an analysis model that reads text"));
    model.provider = ProviderId::new("interviewer");
    model.id = "interviewer-1".into();
    model.name = "Interviewer One".into();
    model.successor = None;
    model.retired = false;
    model.limits.max_input_tokens = Some(tokens);
    catalog.models.push(model);
    interviews_with(catalog)
}

fn interviews_with(catalog: kalamu_catalog::Catalog) -> Interviews {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let model = Interviewer::new();
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store,
        vec![Fake::new(), model.clone()],
        catalog,
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 3,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Interviews { engine, model, dir }
}

/// An interview in French, two speakers, a phrase said twice, French typography.
pub(super) const TRANSCRIPT: &str = "[00:00:05] Interviewer: Comment avez-vous commencé au port ?\n\
[00:00:12] Awa: J’ai commencé en 1998. Au début, on ne savait pas.\n\
[00:01:10] Interviewer: Et ensuite ?\n\
[00:01:15] Awa: Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.";

/// The model's answer: the phrase said twice is named in turn 4, a quotation given under the
/// wrong turn, one not in the text, and a speaker and time of its own that must be ignored.
pub(super) fn answer() -> serde_json::Value {
    serde_json::json!({
        "summary": "Awa raconte ses débuts au port.",
        "themes": [
            {"name": "Les débuts", "description": "Elle commence en 1998.",
             "quotes": [{"turn": 1, "words": "J'ai commencé en 1998.",
                         "speaker": "Interviewer", "time": "00:00:01"},
                        {"turn": 2, "words": "Elle n'a jamais dit cela."}]},
            {"name": "Une devise", "description": "Une phrase reprise.",
             "quotes": [{"turn": 4, "words": "on ne savait pas"}]},
        ],
        "keywords": ["port", "1998"],
        "quotations": [
            {"why_it_matters": "La devise.", "quotes": [{"turn": 4, "words": "est devenu notre devise."}]},
        ],
    })
}

/// An answer none of whose quotations is in the text.
pub(super) fn invented() -> serde_json::Value {
    serde_json::json!({
        "summary": "s",
        "themes": [{"name": "n", "description": "d",
                    "quotes": [{"turn": 2, "words": "Nous étions des pionnières."}]}],
        "keywords": [],
        "quotations": [],
    })
}

impl Interviews {
    pub(super) fn folder(&self) -> OutputDestination {
        OutputDestination::Folder(self.dir.path().display().to_string())
    }

    pub(super) async fn run(&self, id: JobId) -> JobStatus {
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

    /// A video transcribed, corrected into the interview above, and frozen.
    pub(super) async fn frozen(&self) -> SourceFile {
        let job = self
            .engine
            .create_job(NewJob {
                name: "Entretien".into(),
                task: Task::AudioTranscription,
                sources: self
                    .engine
                    .inspect_links(&["https://youtu.be/dQw4w9WgXcQ"])
                    .into_iter()
                    .flatten()
                    .collect(),
                setups: vec![setup("fake", "fake-model", None, None)],
                settings: Default::default(),
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    destination: self.folder(),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(self.run(job.id).await, JobStatus::Completed);
        self.engine
            .save_correction(Correction {
                job_id: job.id,
                setup_id: job.setups[0].id,
                source_id: job.sources[0].id,
                index: 0,
                text: Some(TRANSCRIPT.into()),
                fields: None,
                reviewed: true,
                updated_at: Timestamp::now(),
            })
            .unwrap_or_else(|e| panic!("{e}"));
        let mut frozen = self
            .engine
            .snapshot(job.id, job.setups[0].id)
            .unwrap_or_else(|e| panic!("{e}"));
        frozen.remove(0)
    }

    /// A plain text file.
    pub(super) fn text_file(&self, name: &str, text: &str) -> SourceFile {
        let path = self.dir.path().join(name);
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("{e}"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"))
    }

    pub(super) fn analysis(
        &self,
        sources: Vec<SourceFile>,
        question: Option<&str>,
    ) -> Result<Job, EngineError> {
        self.engine.create_job(NewJob {
            name: "Analyse".into(),
            task: Task::Analysis,
            sources,
            setups: vec![setup(
                "interviewer",
                "interviewer-1",
                Some("analysis.interview"),
                question,
            )],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Txt, ExportFormat::Json],
                destination: self.folder(),
                ..Default::default()
            },
        })
    }

    pub(super) fn fields(&self, job: &Job, source: usize) -> Vec<FieldAnswer> {
        self.engine
            .source_pieces(job.id, job.sources[source].id)
            .unwrap_or_default()
            .first()
            .map(|p| p.corrected_fields.clone().unwrap_or(p.ai_fields.clone()))
            .unwrap_or_default()
    }

    pub(super) fn provenance(&self, job: &Job) -> serde_json::Value {
        let path = self
            .engine
            .outputs(job.id)
            .unwrap_or_default()
            .into_iter()
            .find(|p| p.ends_with("provenance.json"))
            .unwrap_or_default();
        serde_json::from_slice(&std::fs::read(path).unwrap_or_default()).unwrap_or_default()
    }
}

pub(super) fn setup(
    provider: &str,
    model: &str,
    preset: Option<&str>,
    question: Option<&str>,
) -> SetupRequest {
    SetupRequest {
        provider: ProviderId::new(provider),
        model: model.into(),
        preset_id: preset.map(String::from),
        label: None,
        options: kalamu_core::SetupOptions {
            question: question.map(String::from),
            ..Default::default()
        },
        merge: None,
    }
}

pub(super) fn cited(fields: &[FieldAnswer], key: &str) -> CitedValue {
    fields
        .iter()
        .find(|a| a.key == key)
        .and_then(|a| a.value.cited())
        .cloned()
        .unwrap_or_default()
}
