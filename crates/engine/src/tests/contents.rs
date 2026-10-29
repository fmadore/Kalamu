//! A table of contents read by one provider and combined by another, and corrections of text
//! that already exists, against fake providers that answer the schemas they are sent.

use kalamu_core::{Field, MergeModel, PieceRef};

use super::*;

/// Answers whatever schema it is sent: a page's entries (two articles over three pages), the
/// combined entries (grouped by title), or a corrected text.
struct Structured {
    id: &'static str,
    calls: AtomicUsize,
    /// What each call read, as text.
    inputs: Mutex<Vec<String>>,
    /// Pages that fail, until the list is cleared.
    failing: Mutex<Vec<u32>>,
}

impl Structured {
    fn new(id: &'static str) -> Arc<Self> {
        Arc::new(Self {
            id,
            calls: AtomicUsize::new(0),
            inputs: Mutex::new(Vec::new()),
            failing: Mutex::new(Vec::new()),
        })
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn inputs(&self) -> Vec<String> {
        self.inputs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
}

/// An answer for each of the researcher's fields, by kind.
fn fill(fields: &[Field], text: &str) -> Vec<FieldAnswer> {
    fields
        .iter()
        .map(|f| FieldAnswer {
            key: f.key.clone(),
            value: match &f.kind {
                FieldKind::List { .. } => FieldValue::List(vec!["La Rédaction".into()]),
                FieldKind::Choice { values } => FieldValue::Text(values[0].clone()),
                FieldKind::Choices { .. } => FieldValue::List(Vec::new()),
                _ => FieldValue::Text(text.into()),
            },
            reason: None,
        })
        .collect()
}

fn text_answer(key: &str, value: &str) -> FieldAnswer {
    FieldAnswer {
        key: key.into(),
        value: FieldValue::Text(value.into()),
        reason: None,
    }
}

#[async_trait]
impl Provider for Structured {
    fn id(&self) -> ProviderId {
        ProviderId::new(self.id)
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf", "text/plain"]
    }

    /// A large model reads at a high effort, the small one combines at the lowest.
    fn request_settings(&self, _task: Task, model: &str) -> Vec<kalamu_core::RequestSetting> {
        let effort = if model == "tiny" { "minimal" } else { "high" };
        vec![kalamu_core::RequestSetting::new("reasoning_effort", effort)]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let input = request.input;
        let read = String::from_utf8_lossy(&input.bytes).into_owned();
        self.inputs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(read.clone());
        let schema = request
            .analysis
            .unwrap_or_else(|| panic!("both tasks answer in fields"));
        let entry_fields = kalamu_core::contents::entry_fields(schema);
        let answers = match schema.fields[0].key.as_str() {
            // A correction.
            "text" => {
                assert_eq!(input.mime, "text/plain", "a correction sends text");
                vec![text_answer("text", &format!("Corrigé: {}", read.trim()))]
            }
            // A page of a table of contents.
            "entries" if schema.fields.len() == 2 => {
                assert!(input.bytes.starts_with(b"%PDF"), "a page arrives as a PDF");
                let PieceRef::Page { number, .. } = input.piece else {
                    panic!("not a page");
                };
                if self
                    .failing
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .contains(&number)
                {
                    return Err(ProviderError::InvalidRequest("unreadable page".into()));
                }
                let title = if number <= 2 { "Le Hajj" } else { "Fatwa" };
                let mut entry = vec![
                    text_answer("title", title),
                    text_answer("continues", if number == 1 { "suite page 2" } else { "" }),
                ];
                entry.extend(fill(&entry_fields, &format!("partie page {number}")));
                vec![
                    FieldAnswer {
                        key: "entries".into(),
                        value: FieldValue::Entries(vec![entry]),
                        reason: None,
                    },
                    text_answer("page_note", ""),
                ]
            }
            // The pages combined: one entry per title, with its pages.
            "entries" => {
                assert_eq!(input.piece, PieceRef::Whole);
                let pages: serde_json::Value =
                    serde_json::from_str(&read).unwrap_or_else(|e| panic!("{e}"));
                let mut titles: Vec<(String, Vec<String>)> = Vec::new();
                for page in pages["pages"].as_array().into_iter().flatten() {
                    let label = page["page"].as_str().unwrap_or_default().to_string();
                    for entry in page["entries"].as_array().into_iter().flatten() {
                        let title = entry["title"].as_str().unwrap_or_default().to_string();
                        match titles.iter_mut().find(|(t, _)| *t == title) {
                            Some((_, labels)) => labels.push(label.clone()),
                            None => titles.push((title, vec![label.clone()])),
                        }
                    }
                }
                let entries = titles
                    .into_iter()
                    .map(|(title, labels)| {
                        let pages = match labels.as_slice() {
                            [one] => one.clone(),
                            [first, .., last] => format!("{first}-{last}"),
                            [] => String::new(),
                        };
                        let mut entry =
                            vec![text_answer("title", &title), text_answer("pages", &pages)];
                        entry.extend(fill(&entry_fields, &format!("{} résumé", request.model)));
                        entry
                    })
                    .collect();
                kalamu_core::contents::answers(entries)
            }
            other => panic!("unexpected schema {other}"),
        };
        Ok(ProcessOutput {
            text: schema.render(&answers),
            fields: answers,
            usage: Usage {
                input_tokens: 500,
                output_tokens: 50,
                ..Usage::default()
            },
            model_version: Some(format!("{}-001", request.model)),
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

struct Two {
    engine: Arc<Engine>,
    reader: Arc<Structured>,
    small: Arc<Structured>,
    dir: tempfile::TempDir,
}

fn two() -> Two {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let (reader, small) = (Structured::new("reader"), Structured::new("small"));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store,
        vec![reader.clone(), small.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            ..EngineConfig::default()
        },
    );
    Two {
        engine,
        reader,
        small,
        dir,
    }
}

impl Two {
    fn file(&self, name: &str, bytes: &[u8]) -> SourceFile {
        let path = self.dir.path().join(name);
        std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("write"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("inspect: {e}"))
    }

    fn job(&self, task: Task, sources: Vec<SourceFile>, preset: &str, merge: Option<&str>) -> Job {
        self.engine
            .create_job(NewJob {
                name: "Issue".into(),
                task,
                sources,
                setups: vec![SetupRequest {
                    provider: ProviderId::new("reader"),
                    model: "big".into(),
                    preset_id: Some(preset.into()),
                    label: None,
                    options: Default::default(),
                    merge: merge.map(|model| MergeModel {
                        provider: ProviderId::new("small"),
                        model: model.into(),
                    }),
                }],
                settings: Default::default(),
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt, ExportFormat::Markdown, ExportFormat::Csv],
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

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(name)).unwrap_or_default()
    }
}

fn issue(two: &Two) -> SourceFile {
    two.file(
        "issue.pdf",
        &kalamu_media::sample_pdf(&[(595.0, 842.0); 3], false),
    )
}

#[tokio::test]
async fn a_large_model_reads_the_pages_and_another_providers_model_combines_them() {
    let t = two();
    let job = t.job(
        Task::Contents,
        vec![issue(&t)],
        "contents.periodical",
        Some("tiny"),
    );
    assert_eq!(
        job.setups[0].merge.as_ref().map(|m| m.model.as_str()),
        Some("tiny")
    );
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(t.reader.calls(), 3, "one call per page");
    assert_eq!(t.small.calls(), 1, "one call to combine them");
    let combined = &t.small.inputs()[0];
    assert!(combined.contains("\"page\": \"1\"") && combined.contains("suite page 2"));

    let md = t.read("issue.contents.md");
    assert!(md.contains("## Le Hajj\n\n- **Pages:** 1-2"), "{md}");
    assert!(md.contains("## Fatwa\n\n- **Pages:** 3"), "{md}");
    assert!(md.contains("- **Summary:** tiny résumé"), "{md}");
    let txt = t.read("issue.contents.txt");
    assert!(
        txt.starts_with("1. Le Hajj\n   Pages: 1-2\n   Authors: La Rédaction"),
        "{txt}"
    );
    let csv = t.read("issue.contents.csv");
    assert_eq!(
        csv.lines().count(),
        3,
        "a header and one row per article: {csv}"
    );
    assert!(
        csv.lines()
            .next()
            .is_some_and(|h| h.contains("Title,Pages,Authors,Summary"))
    );

    let provenance: serde_json::Value =
        serde_json::from_str(&t.read("issue.contents.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["model"], "big");
    assert_eq!(provenance["merge"]["provider"], "small");
    assert_eq!(provenance["merge"]["model"], "tiny");
    assert_eq!(provenance["merge"]["model_versions"][0], "tiny-001");
    assert_eq!(provenance["engine"], "cloud");
    // Each step's settings, as recorded with its results.
    assert_eq!(
        provenance["request"],
        serde_json::json!([{ "name": "reasoning_effort", "value": "high" }])
    );
    assert_eq!(
        provenance["merge"]["request"],
        serde_json::json!([{ "name": "reasoning_effort", "value": "minimal" }])
    );
    assert_eq!(provenance["pieces"], 4);

    // The same issue again: nothing is read twice. Another merge model combines the same pages.
    let again = t.job(
        Task::Contents,
        vec![issue(&t)],
        "contents.periodical",
        Some("tiny"),
    );
    assert_eq!(t.run(again.id).await, JobStatus::Completed);
    assert_eq!((t.reader.calls(), t.small.calls()), (3, 1));
    let other = t.job(
        Task::Contents,
        vec![issue(&t)],
        "contents.periodical",
        Some("tiny-2"),
    );
    assert_eq!(t.run(other.id).await, JobStatus::Completed);
    assert_eq!((t.reader.calls(), t.small.calls()), (3, 2));
}

#[tokio::test]
async fn without_a_merge_model_the_pages_model_combines_them() {
    let t = two();
    let job = t.job(
        Task::Contents,
        vec![issue(&t)],
        "contents.archival_file",
        None,
    );
    let merge = job.setups[0]
        .merge
        .clone()
        .unwrap_or_else(|| panic!("no merge"));
    assert_eq!(
        (merge.provider.as_str(), merge.model.as_str()),
        ("reader", "big")
    );
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(t.reader.calls(), 4);
    assert_eq!(t.small.calls(), 0);
}

#[tokio::test]
async fn a_page_that_fails_holds_the_combining_until_it_is_retried() {
    let t = two();
    t.reader
        .failing
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push(2);
    let job = t.job(
        Task::Contents,
        vec![issue(&t)],
        "contents.periodical",
        Some("tiny"),
    );
    assert_eq!(t.run(job.id).await, JobStatus::CompletedWithErrors);
    assert_eq!(t.small.calls(), 0);
    let pieces = t.engine.pieces(job.id).unwrap_or_default();
    let merge = pieces
        .iter()
        .find(|p| p.piece == PieceRef::Whole)
        .unwrap_or_else(|| panic!("no merge piece"));
    assert_eq!(merge.status, PieceStatus::Failed);
    assert!(!t.dir.path().join("issue.contents.md").exists());

    t.reader
        .failing
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clear();
    let mut events = t.engine.subscribe();
    t.engine
        .retry_failed(job.id)
        .unwrap_or_else(|e| panic!("{e}"));
    loop {
        if let Ok(Ok(ProgressEvent::JobFinished { status, .. })) =
            tokio::time::timeout(Duration::from_secs(10), events.recv()).await
        {
            assert_eq!(status, JobStatus::Completed);
            break;
        }
    }
    assert_eq!(t.small.calls(), 1);
    assert!(t.read("issue.contents.md").contains("## Le Hajj"));
}

#[tokio::test]
async fn existing_text_is_corrected_page_by_page_and_compared_with_what_it_was() {
    let t = two();
    let pdf = t.file(
        "bulletin.pdf",
        &kalamu_media::sample_pdf(&[(595.0, 842.0); 2], true),
    );
    let txt = t.file(
        "gallica.txt",
        "--- Page 1 ---\n\nLe Hajj de 1957\n\n--- Page 2 ---\n\nSuite du rccit\n".as_bytes(),
    );
    let job = t.job(
        Task::TextCorrection,
        vec![pdf.clone(), txt.clone()],
        "text_correction.general",
        None,
    );
    assert!(job.setups[0].merge.is_none());
    assert_eq!(t.run(job.id).await, JobStatus::Completed);
    assert_eq!(t.reader.calls(), 4);

    let corrected = t.read("gallica.corrected.txt");
    assert!(
        corrected.contains("--- Page 2 ---\n\nCorrigé: Suite du rccit"),
        "{corrected}"
    );
    assert!(
        t.read("gallica.txt").contains("rccit"),
        "the source is untouched"
    );
    assert!(
        t.read("bulletin.corrected.txt")
            .contains("Corrigé: Page 1 of the Bulletin")
    );

    let views = t
        .engine
        .source_pieces(job.id, txt.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(views[1].ai_text.as_deref(), Some("Corrigé: Suite du rccit"));
    assert_eq!(views[1].source_text.as_deref(), Some("Suite du rccit"));
    assert!(views[1].ai_fields.is_empty(), "stored as text, not fields");
}
