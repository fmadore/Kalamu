//! Records (#171): an analysis whose fields are one repeating group reads each page on its own,
//! one row per entry; its pages are cached and resumed as any piece, cut-off and failed pages
//! are marked, and rows are corrected, added and removed in the correction layer.

use kalamu_core::{DocumentId, PieceRef, ReviewScope, ReviewTarget, RowEdit, Signal, records};

use super::*;

/// Answers each page with two rows from its columns, the page and the row in each cell; can
/// be told to stop a page part-way once, or to fail pages.
struct Rows {
    calls: AtomicUsize,
    /// Pages stopped part-way the first time they are sent, as a pause would.
    stop_once: Mutex<Vec<u32>>,
    /// Pages and the error they fail with, every time.
    failing: Mutex<Vec<(u32, ProviderError)>>,
}

#[async_trait]
impl Provider for Rows {
    fn id(&self) -> ProviderId {
        ProviderId::new("rows")
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
        let schema = request
            .analysis
            .unwrap_or_else(|| panic!("an analysis answers in fields"));
        let group = records::group(schema).unwrap_or_else(|| panic!("a records analysis"));
        let input = request.input;
        let number = match input.piece {
            PieceRef::Page { number, .. } => {
                assert!(input.bytes.starts_with(b"%PDF"), "a page arrives as a PDF");
                number
            }
            // A text file has no pages: read whole.
            PieceRef::Whole => 0,
            _ => panic!("records read pages"),
        };
        {
            let mut stop = self.stop_once.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(at) = stop.iter().position(|n| *n == number) {
                stop.remove(at);
                return Err(ProviderError::Interrupted);
            }
        }
        let failing = self.failing.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((_, err)) = failing.iter().find(|(n, _)| *n == number) {
            return Err(match err {
                ProviderError::CutOff { usage } => ProviderError::CutOff {
                    usage: usage.clone(),
                },
                other => ProviderError::InvalidRequest(other.to_string()),
            });
        }
        let row = |n: u32| -> Vec<FieldAnswer> {
            records::columns(schema)
                .iter()
                .map(|c| FieldAnswer {
                    key: c.key.clone(),
                    value: match c.kind {
                        FieldKind::List { .. } => FieldValue::List(vec![format!("p{number}")]),
                        _ => FieldValue::Text(format!("{} p{number} r{n}", c.label)),
                    },
                    reason: None,
                })
                .collect()
        };
        // The second model misses the first entry of page 2.
        let found = if request.model == "rows-2" && number == 2 {
            vec![row(2)]
        } else {
            vec![row(1), row(2)]
        };
        let fields = vec![FieldAnswer {
            key: group.key.clone(),
            value: FieldValue::Entries(found),
            reason: None,
        }];
        Ok(ProcessOutput {
            text: schema.render(&fields),
            fields,
            usage: Usage {
                input_tokens: 800,
                output_tokens: 120,
                ..Usage::default()
            },
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

struct Records {
    engine: Arc<Engine>,
    rows: Arc<Rows>,
    store: Arc<SqliteStore>,
    dir: tempfile::TempDir,
}

fn records_fixture() -> Records {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let rows = Arc::new(Rows {
        calls: AtomicUsize::new(0),
        stop_once: Mutex::new(Vec::new()),
        failing: Mutex::new(Vec::new()),
    });
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![rows.clone()],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    Records {
        engine,
        rows,
        store,
        dir,
    }
}

impl Records {
    fn pdf(&self, name: &str, pages: usize) -> SourceFile {
        let path = self.dir.path().join(name);
        let bytes = kalamu_media::sample_pdf(&vec![(595.0, 842.0); pages], false);
        std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("write pdf"));
        kalamu_media::inspect(&path, None).unwrap_or_else(|_| panic!("inspect"))
    }

    fn new_job(&self, sources: Vec<SourceFile>, presets: &[&str]) -> Result<Job, EngineError> {
        self.engine.create_job(NewJob {
            name: "Directory".into(),
            task: Task::Analysis,
            sources,
            setups: presets
                .iter()
                .enumerate()
                .map(|(i, preset)| SetupRequest {
                    provider: ProviderId::new("rows"),
                    model: format!("rows-{}", i + 1),
                    preset_id: Some((*preset).into()),
                    label: None,
                    options: Default::default(),
                    merge: None,
                })
                .collect(),
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Csv, ExportFormat::Json],
                ..Default::default()
            },
        })
    }

    fn job(&self, sources: Vec<SourceFile>) -> Job {
        self.new_job(sources, &["analysis.directory"])
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

    fn calls(&self) -> usize {
        self.rows.calls.load(Ordering::SeqCst)
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(name)).unwrap_or_default()
    }

    /// Each page's rows as the review shows them, by their names.
    fn names(&self, job: &Job) -> Vec<Vec<String>> {
        let schema = job.setups[0].prompt.schema.clone().unwrap_or_default();
        self.engine
            .source_pieces(job.id, job.sources[0].id)
            .unwrap_or_default()
            .iter()
            .map(|v| {
                let answers = v.corrected_fields.as_ref().unwrap_or(&v.ai_fields);
                records::rows(&schema, answers)
                    .iter()
                    .map(|row| row[0].value.joined())
                    .collect()
            })
            .collect()
    }
}

#[tokio::test]
async fn a_records_job_reads_each_page_once_and_resumes_without_repeating_or_losing_rows() {
    let r = records_fixture();
    r.rows
        .stop_once
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push(3);
    let source = r.pdf("annuaire.pdf", 4);
    let job = r.job(vec![source.clone()]);
    assert_eq!(r.run(job.id).await, JobStatus::Paused);
    let pieces = r.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces.len(), 4, "one piece per page");
    assert!(
        pieces
            .iter()
            .all(|p| matches!(p.piece, PieceRef::Page { .. }))
    );
    assert_eq!(count(&pieces, PieceStatus::Done), 3);
    assert_eq!(count(&pieces, PieceStatus::Pending), 1);
    assert_eq!(r.calls(), 4);

    // Resumed: only the page stopped is sent again, and every page has its rows.
    assert_eq!(r.run(job.id).await, JobStatus::Completed);
    assert_eq!(r.calls(), 5, "the pages read are not sent again");
    let names = r.names(&job);
    assert_eq!(names.len(), 4);
    assert_eq!(names[2], ["Name p3 r1", "Name p3 r2"], "in reading order");

    // The same pages under the same settings are not paid for twice.
    let again = r.job(vec![source]);
    assert_eq!(r.run(again.id).await, JobStatus::Completed);
    assert_eq!(r.calls(), 5);

    // One row per entry, the page and the place on it given; the provenance as elsewhere.
    let csv = r.read("annuaire.analysis.csv");
    let mut lines = csv.trim_start_matches('\u{feff}').lines();
    assert_eq!(
        lines.next(),
        Some("file,page,position,Name,Occupation,Address,Remarks,corrected,reviewed,not_read")
    );
    assert_eq!(lines.count(), 8, "{csv}");
    assert!(
        csv.contains("annuaire.pdf,3,2,Name p3 r2,Occupation p3 r2,"),
        "{csv}"
    );
    let provenance: serde_json::Value =
        serde_json::from_str(&r.read("annuaire.analysis.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["pieces"], 4);
}

#[tokio::test]
async fn a_page_cut_off_or_failed_is_marked_and_listed_never_complete() {
    let r = records_fixture();
    r.rows
        .failing
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .extend([
            (
                2,
                ProviderError::CutOff {
                    usage: Usage::default(),
                },
            ),
            (3, ProviderError::Other("refused".into())),
        ]);
    let source = r.pdf("registre.pdf", 3);
    let job = r.job(vec![source.clone()]);
    assert_eq!(r.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = r.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Failed), 2);

    // The table is written with the rows read, each page not read marked in its place.
    let csv = r.read("registre.analysis.csv");
    let rows: Vec<&str> = csv.lines().skip(1).collect();
    assert_eq!(rows.len(), 4, "{csv}");
    assert!(
        rows[2].starts_with("registre.pdf,2,,,,,,false,false,"),
        "{csv}"
    );
    assert!(!rows[2].ends_with(','), "the reason is given: {csv}");
    let json: serde_json::Value =
        serde_json::from_str(&r.read("registre.analysis.json")).unwrap_or_default();
    assert_eq!(json["rows"].as_array().map(Vec::len), Some(2));
    assert!(json["parts"][1]["failed"].is_string());
    let provenance: serde_json::Value =
        serde_json::from_str(&r.read("registre.analysis.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["failed"], serde_json::json!([2, 3]));

    // What to check lists both pages, kept apart from the answers to review.
    let document = DocumentId::new();
    let queue = r
        .engine
        .review_queue(&ReviewScope {
            documents: 1,
            targets: vec![ReviewTarget {
                document_id: document,
                document: source.file_name.clone(),
                job_id: job.id,
                source_id: source.id,
                setup_id: job.setups[0].id,
                preferred: false,
            }],
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let failed: Vec<u32> = queue
        .items
        .iter()
        .filter(|i| i.signals.iter().any(|s| matches!(s, Signal::Failed { .. })))
        .filter_map(|i| i.index)
        .collect();
    assert_eq!(failed, [1, 2]);
    assert_eq!(queue.counts.failed, 2);
}

#[tokio::test]
async fn set_ups_that_read_a_page_differently_are_listed_in_what_to_check() {
    let r = records_fixture();
    let source = r.pdf("liste.pdf", 2);
    // The same columns from two models, which read page 2 differently.
    let job = r
        .new_job(
            vec![source.clone()],
            &["analysis.directory", "analysis.directory"],
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(r.run(job.id).await, JobStatus::Completed);
    let queue = r
        .engine
        .review_queue(&ReviewScope {
            documents: 1,
            targets: vec![ReviewTarget {
                document_id: DocumentId::new(),
                document: source.file_name.clone(),
                job_id: job.id,
                source_id: source.id,
                setup_id: job.setups[0].id,
                preferred: false,
            }],
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let disagrees: Vec<(u32, Vec<String>)> = queue
        .items
        .iter()
        .flat_map(|i| {
            i.signals.iter().filter_map(move |s| match s {
                Signal::Disagrees { with, fields, .. } => {
                    assert_eq!(with, &["rows-2".to_string()]);
                    Some((i.index.unwrap_or_default(), fields.clone()))
                }
                _ => None,
            })
        })
        .collect();
    assert_eq!(disagrees, [(1, vec!["Entries".to_string()])]);
    // Every page is to review until marked reviewed.
    assert_eq!(queue.counts.processed, 2);
    assert_eq!(queue.items.len(), 2);
}

#[tokio::test]
async fn rows_are_corrected_added_and_removed_in_the_correction_layer() {
    let r = records_fixture();
    let source = r.pdf("membres.pdf", 2);
    let job = r.job(vec![source.clone()]);
    assert_eq!(r.run(job.id).await, JobStatus::Completed);
    let (setup, at) = (job.setups[0].id, source.id);
    let edit = |change: RowEdit| {
        r.engine
            .edit_rows(job.id, setup, at, 1, &change)
            .unwrap_or_else(|e| panic!("{e}"));
    };
    edit(RowEdit::Cell {
        row: 0,
        key: "name".into(),
        value: " =Diallo ".into(),
    });
    edit(RowEdit::Add { at: 2 });
    edit(RowEdit::Cell {
        row: 2,
        key: "name".into(),
        value: "Traoré".into(),
    });
    edit(RowEdit::Remove { row: 1 });
    assert_eq!(r.names(&job)[1], ["=Diallo", "Traoré"]);

    let views = r
        .engine
        .source_pieces(job.id, source.id)
        .unwrap_or_default();
    let schema = job.setups[0].prompt.schema.clone().unwrap_or_default();
    assert_eq!(
        records::rows(&schema, &views[1].ai_fields).len(),
        2,
        "the model's rows are kept under the correction"
    );
    assert!(
        !views[1].reviewed,
        "an edit leaves the page as reviewed as it was"
    );
    assert!(views[0].corrected_fields.is_none());

    // A row that is not there is refused.
    assert!(matches!(
        r.engine
            .edit_rows(job.id, setup, at, 1, &RowEdit::Remove { row: 9 }),
        Err(EngineError::Rows(kalamu_core::RowError::NoRow(9)))
    ));

    // The files are written again from the corrected rows; a cell a spreadsheet would compute
    // is written as text in the CSV, exactly in the JSON.
    assert_eq!(r.engine.write_outdated().ok(), Some(0));
    let csv = r.read("membres.analysis.csv");
    assert!(csv.contains("membres.pdf,2,1,'=Diallo,"), "{csv}");
    assert!(
        csv.contains("membres.pdf,2,2,Traoré,,,,true,false,"),
        "{csv}"
    );
    assert!(!csv.contains("Name p2 r2"), "{csv}");
    assert_eq!(csv.lines().count(), 5, "{csv}");
    let json: serde_json::Value =
        serde_json::from_str(&r.read("membres.analysis.json")).unwrap_or_default();
    assert_eq!(json["rows"][2]["fields"]["name"], "=Diallo");
    let provenance: serde_json::Value =
        serde_json::from_str(&r.read("membres.analysis.provenance.json")).unwrap_or_default();
    assert_eq!(provenance["corrected"], serde_json::json!([2]));
}

#[test]
fn records_and_whole_document_analyses_are_not_compared() {
    let r = records_fixture();
    let source = r.pdf("mixte.pdf", 1);
    let mixed = r.new_job(
        vec![source],
        &["analysis.directory", "analysis.summary_research"],
    );
    assert!(matches!(mixed, Err(EngineError::MixedReading)));
}

#[tokio::test]
async fn a_text_file_has_no_pages_and_is_read_whole() {
    let r = records_fixture();
    let path = r.dir.path().join("liste.txt");
    std::fs::write(&path, "Diallo, tailleur\nTraoré, commerçant\n")
        .unwrap_or_else(|e| panic!("{e}"));
    let text = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let job = r.job(vec![text]);
    assert_eq!(r.run(job.id).await, JobStatus::Completed);
    let pieces = r.store.pieces(job.id).unwrap_or_default();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].piece, PieceRef::Whole);
}
