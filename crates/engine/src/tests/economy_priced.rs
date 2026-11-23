//! Economy mode priced as it runs (#242): the estimate's economy figure counts the batch
//! discount only on the sources whose work the run puts in a batch, by the same rule. A
//! screening (which quotes the text) and a timed translation are sent at once, at the usual
//! price; a translation of a timed transcript and of plain texts batches the texts only.

use std::collections::BTreeSet;

use kalamu_core::{BatchContext, BatchProgress, JobSettings, LanguageChoice, LanguageCode, Pace};

use super::*;

/// A provider named as a catalogue provider whose model has a batch discount, which batches
/// every task: it notes the file name of every piece it is sent at once, and of every piece put
/// in a batch, and answers each with a line of text.
#[derive(Default)]
struct Halves {
    at_once: Mutex<BTreeSet<String>>,
    batched: Mutex<BTreeSet<String>>,
}

fn answered() -> ProcessOutput {
    ProcessOutput {
        text: "answered".into(),
        usage: Usage {
            input_tokens: 500,
            output_tokens: 50,
            ..Usage::default()
        },
        ..ProcessOutput::default()
    }
}

#[async_trait]
impl Provider for Halves {
    fn id(&self) -> ProviderId {
        ProviderId::new("gemini")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf", "text/plain"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.at_once
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(request.input.file_name.clone());
        Ok(answered())
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }

    fn batches(&self, _task: Task, _model: &str) -> bool {
        true
    }

    async fn submit_batch(
        &self,
        _key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let mut batched = self.batched.lock().unwrap_or_else(|p| p.into_inner());
        batched.extend(requests.iter().map(|(_, r)| r.input.file_name.clone()));
        Ok(format!("remote-{}", batched.len()))
    }

    async fn poll_batch(
        &self,
        _key: &ApiKey,
        _batch_id: &str,
        _context: BatchContext<'_>,
        ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        Ok(BatchProgress::Ended(
            ids.iter().map(|id| (id.clone(), Ok(answered()))).collect(),
        ))
    }
}

struct Priced {
    engine: Arc<Engine>,
    halves: Arc<Halves>,
    dir: tempfile::TempDir,
}

fn priced() -> Priced {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let halves = Arc::new(Halves::default());
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store,
        vec![Fake::new(), halves.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            batch_poll: Duration::from_millis(10),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Priced {
        engine,
        halves,
        dir,
    }
}

fn hydrogen(preset: &str, question: Option<&str>) -> SetupRequest {
    SetupRequest {
        provider: ProviderId::new("gemini"),
        model: "gemini-4-hydrogen".into(),
        preset_id: Some(preset.into()),
        label: None,
        options: kalamu_core::SetupOptions {
            question: question.map(String::from),
            ..Default::default()
        },
        merge: None,
    }
}

impl Priced {
    fn folder(&self) -> OutputDestination {
        OutputDestination::Folder(self.dir.path().display().to_string())
    }

    fn text(&self, name: &str, words: usize) -> SourceFile {
        let path = self.dir.path().join(name);
        let text = format!("{name} dit bonjour à la mairie. ").repeat(words / 6);
        std::fs::write(&path, text).unwrap_or_else(|e| panic!("{e}"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"))
    }

    /// A video transcribed by the fake model with times, frozen with its lines as *Use this
    /// text* freezes it.
    async fn timed_transcript(&self) -> SourceFile {
        let job = self
            .engine
            .create_job(NewJob {
                name: "Video".into(),
                task: Task::AudioTranscription,
                sources: self
                    .engine
                    .inspect_links(&["https://youtu.be/dQw4w9WgXcQ"])
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
                    formats: vec![ExportFormat::Txt],
                    destination: self.folder(),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(self.run(job.id).await, JobStatus::Completed);
        let mut frozen = self
            .engine
            .snapshot(job.id, job.setups[0].id)
            .unwrap_or_else(|e| panic!("{e}"));
        let source = frozen.remove(0);
        assert!(source.derived_from.as_ref().is_some_and(|d| d.timed));
        source
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

    fn estimate(&self, task: Task, sources: &[SourceFile], setup: &SetupRequest) -> Estimate {
        self.engine.estimate(&EstimateRequest {
            task,
            sources: sources.to_vec(),
            setups: vec![setup.clone()],
            sample_pages: None,
        })
    }

    /// Runs the job in economy mode and returns, of its sources, those whose pieces went in a
    /// batch and those sent at once, as the provider received them.
    async fn run_economy(
        &self,
        task: Task,
        sources: &[SourceFile],
        setup: &SetupRequest,
    ) -> (Vec<SourceFile>, Vec<SourceFile>) {
        let job = self
            .engine
            .create_job(NewJob {
                name: "Economy".into(),
                task,
                sources: sources.to_vec(),
                setups: vec![setup.clone()],
                settings: JobSettings {
                    pace: Pace::Economy,
                    languages: LanguageChoice::Explicit(vec![LanguageCode::new("en")]),
                    ..JobSettings::default()
                },
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    destination: self.folder(),
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("create job: {e}"));
        self.run(job.id).await;
        let batched = self
            .halves
            .batched
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        let at_once = self
            .halves
            .at_once
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        let of = |names: &BTreeSet<String>| -> Vec<SourceFile> {
            sources
                .iter()
                .filter(|s| {
                    let stem = crate::source_stem(&s.file_name, "page");
                    names.iter().any(|n| n.starts_with(&stem))
                })
                .cloned()
                .collect()
        };
        let (in_batches, sent_at_once) = (of(&batched), of(&at_once));
        // Every piece belongs to one source, and every source went one way only.
        assert_eq!(
            in_batches.len() + sent_at_once.len(),
            sources.len(),
            "batched {batched:?}, at once {at_once:?}"
        );
        (in_batches, sent_at_once)
    }

    /// What the run's pieces cost at catalogue prices: each source batched at its economy
    /// price, each sent at once at its usual price, priced alone.
    fn as_run(
        &self,
        task: Task,
        setup: &SetupRequest,
        in_batches: &[SourceFile],
        sent_at_once: &[SourceFile],
    ) -> f64 {
        let batched: f64 = in_batches
            .iter()
            .map(|s| {
                self.estimate(task, std::slice::from_ref(s), setup).rows[0]
                    .estimate
                    .economy_usd
                    .unwrap_or_else(|| panic!("{} has a batch price", s.file_name))
            })
            .sum();
        let at_once: f64 = sent_at_once
            .iter()
            .map(|s| {
                self.estimate(task, std::slice::from_ref(s), setup).rows[0]
                    .estimate
                    .usd
                    .unwrap_or_else(|| panic!("{} has a price", s.file_name))
            })
            .sum();
        batched + at_once
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9 * b.max(1e-9)
}

#[tokio::test]
async fn a_screening_in_economy_mode_is_priced_at_the_usual_price_it_runs_at() {
    let p = priced();
    let sources = vec![p.text("issue-1.txt", 900), p.text("issue-2.txt", 600)];
    let setup = hydrogen(
        "analysis.screening",
        Some("Les commerçantes et la municipalité"),
    );
    let estimate = p.estimate(Task::Analysis, &sources, &setup);
    let (in_batches, sent_at_once) = p.run_economy(Task::Analysis, &sources, &setup).await;
    assert!(in_batches.is_empty(), "a screening quotes the text");
    let row = &estimate.rows[0];
    assert_eq!(row.estimate.economy_usd, None, "no discount");
    assert_eq!(row.at_once, Some(crate::AtOnce::Quotes));
    let usd = estimate.total_usd.unwrap_or_default();
    let economy = estimate.total_economy_usd.unwrap_or_default();
    assert!(usd > 0.0);
    assert!(
        close(
            economy,
            p.as_run(Task::Analysis, &setup, &in_batches, &sent_at_once)
        ),
        "{economy} against {usd}"
    );
}

#[tokio::test]
async fn a_timed_translation_in_economy_mode_is_priced_at_the_usual_price_it_runs_at() {
    let p = priced();
    let sources = vec![p.timed_transcript().await];
    let setup = hydrogen("translation.research", None);
    let estimate = p.estimate(Task::Translation, &sources, &setup);
    let (in_batches, sent_at_once) = p.run_economy(Task::Translation, &sources, &setup).await;
    assert!(in_batches.is_empty(), "timed lines are sent at once");
    assert_eq!(estimate.rows[0].estimate.economy_usd, None);
    assert_eq!(estimate.rows[0].at_once, Some(crate::AtOnce::TimedLines));
    let economy = estimate.total_economy_usd.unwrap_or_default();
    assert!(economy > 0.0);
    assert!(close(
        economy,
        p.as_run(Task::Translation, &setup, &in_batches, &sent_at_once)
    ));
}

#[tokio::test]
async fn a_mixed_translation_is_priced_source_by_source_as_it_runs() {
    let p = priced();
    let sources = vec![
        p.timed_transcript().await,
        p.text("lettre-a.txt", 1_200),
        p.text("lettre-b.txt", 3_000),
    ];
    let setup = hydrogen("translation.research", None);
    let estimate = p.estimate(Task::Translation, &sources, &setup);
    let (in_batches, sent_at_once) = p.run_economy(Task::Translation, &sources, &setup).await;
    assert_eq!(in_batches.len(), 2, "the plain texts are batched");
    assert_eq!(sent_at_once.len(), 1, "the transcript is sent at once");
    assert!(sent_at_once[0].derived_from.is_some());
    let row = &estimate.rows[0];
    assert_eq!(row.at_once, None, "some of its work is batched");
    let (usd, economy) = (
        estimate.total_usd.unwrap_or_default(),
        estimate.total_economy_usd.unwrap_or_default(),
    );
    // Cheaper than at once, dearer than if everything were halved.
    assert!(
        economy < usd && economy > usd / 2.0,
        "{economy} against {usd}"
    );
    let as_run = p.as_run(Task::Translation, &setup, &in_batches, &sent_at_once);
    assert!(close(economy, as_run), "{economy} against {as_run}");
}
