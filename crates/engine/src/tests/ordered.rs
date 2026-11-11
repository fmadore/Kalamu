//! A document of image files read as its pages (#52): read in its order, written as one
//! document, and never read again because a page moved.

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage};
use kalamu_core::PieceRef;
use kalamu_core::fingerprint::sha256_hex;

use super::*;

/// Reads page images only, and answers with which image it was sent: the first characters of
/// its bytes' hash, so a test can tell which file each page of the document was.
struct Eye {
    calls: AtomicUsize,
    /// Pages stopped part-way the first time they are sent, as a pause would.
    stop_once: Mutex<Vec<u32>>,
}

#[async_trait]
impl Provider for Eye {
    fn id(&self) -> ProviderId {
        ProviderId::new("eye")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["image/png"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let input = request.input;
        assert_eq!(input.mime, "image/png", "a page arrives as its image");
        let PieceRef::Page { number, .. } = input.piece else {
            return Err(ProviderError::Other("not a page".into()));
        };
        {
            let mut stop = self.stop_once.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(at) = stop.iter().position(|n| *n == number) {
                stop.remove(at);
                return Err(ProviderError::Interrupted);
            }
        }
        Ok(ProcessOutput {
            text: format!("read {}", &sha256_hex(&input.bytes)[..8]),
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(Vec::new())
    }
}

struct Pages {
    engine: Arc<Engine>,
    eye: Arc<Eye>,
    store: Arc<SqliteStore>,
    dir: tempfile::TempDir,
}

fn pages() -> Pages {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let eye = Arc::new(Eye {
        calls: AtomicUsize::new(0),
        stop_once: Mutex::new(Vec::new()),
    });
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![eye.clone()],
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
    Pages {
        engine,
        eye,
        store,
        dir,
    }
}

fn png(width: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let _ = DynamicImage::ImageRgb8(RgbImage::new(width, 1000))
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png);
    out
}

impl Pages {
    /// Image files in the folder, inspected as New job does, in the order given.
    fn images(&self, names: &[&str]) -> Vec<SourceFile> {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let path = self.dir.path().join(name);
                std::fs::write(&path, png(700 + i as u32)).unwrap_or_else(|e| panic!("{e}"));
                kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("inspect: {e}"))
            })
            .collect()
    }

    fn job(&self, document: SourceFile) -> Job {
        self.engine
            .create_job(NewJob {
                name: "Letters".into(),
                task: Task::TextExtraction,
                sources: vec![document],
                setups: vec![SetupRequest {
                    provider: ProviderId::new("eye"),
                    model: "eye-1".into(),
                    preset_id: None,
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings: Default::default(),
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt, ExportFormat::Markdown],
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

/// What the model answers for a file.
fn answer(file: &SourceFile) -> String {
    format!("read {}", &file.sha256[..8])
}

#[tokio::test]
async fn a_document_of_images_is_read_in_order_and_a_page_moved_is_not_read_again() {
    let p = pages();
    let files = p.images(&["page 10.png", "page 2.png", "page 1.png"]);
    let (ten, two, one) = (&files[0], &files[1], &files[2]);
    let made = p
        .engine
        .image_document("Letters 1923", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    let order: Vec<&str> = made
        .source
        .pages
        .iter()
        .map(|p| p.file_name.as_str())
        .collect();
    assert_eq!(order, ["page 1.png", "page 2.png", "page 10.png"]);
    // 3 to 9 are missing before page 10.
    assert_eq!(made.notes[2].missing_from, Some(3));

    let job = p.job(made.source.clone());
    assert_eq!(p.run(job.id).await, JobStatus::Completed);
    assert_eq!(p.eye.calls.load(Ordering::SeqCst), 3, "one call per page");
    let txt = p.read("Letters 1923.txt");
    let at = |file: &SourceFile| txt.find(&answer(file)).unwrap_or(usize::MAX);
    assert!(at(one) < at(two) && at(two) < at(ten), "{txt}");
    let md = p.read("Letters 1923.md");
    assert!(md.contains(&answer(ten)), "{md}");

    // Page 10 moved to the front: a new order, the same pages.
    let mut moved = made.source.clone();
    let last = moved.pages.remove(2);
    moved.pages.insert(0, last);
    moved.id = kalamu_core::SourceId::new();
    let again = p.job(moved);
    assert_ne!(
        again.sources[0].sha256, job.sources[0].sha256,
        "another order"
    );
    assert_eq!(p.run(again.id).await, JobStatus::Completed);
    assert_eq!(
        p.eye.calls.load(Ordering::SeqCst),
        3,
        "no page is read again because it moved"
    );
    let pieces = p.store.pieces(again.id).unwrap_or_default();
    assert!(pieces.iter().all(|s| s.status == PieceStatus::Cached));
    let txt = p.read("Letters 1923.txt");
    let at = |file: &SourceFile| txt.find(&answer(file)).unwrap_or(usize::MAX);
    assert!(at(ten) < at(one) && at(one) < at(two), "{txt}");

    // provenance.json records the pages in the order read, each with its file and hash.
    let provenance: serde_json::Value =
        serde_json::from_str(&p.read("Letters 1923.provenance.json"))
            .unwrap_or_else(|e| panic!("{e}"));
    let recorded = &provenance["source"]["pages"];
    assert_eq!(recorded[0]["page"], 1);
    assert_eq!(recorded[0]["file_name"], "page 10.png");
    assert_eq!(recorded[0]["sha256"], ten.sha256.as_str());
    assert_eq!(recorded[2]["file_name"], "page 2.png");
    assert_eq!(recorded[2]["path"], two.path.as_str());
    assert_eq!(provenance["source"]["file_name"], "Letters 1923");

    // The review links each page to its original file.
    let views = p
        .engine
        .source_pieces(again.id, again.sources[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(views[0].original_file.as_deref(), Some(ten.path.as_str()));
}

#[tokio::test]
async fn a_page_read_alone_before_is_not_read_again_in_a_document() {
    let p = pages();
    let files = p.images(&["f1.png", "f2.png"]);
    // The first file read on its own, as a separate document.
    let alone = p.job(files[0].clone());
    assert_eq!(p.run(alone.id).await, JobStatus::Completed);
    let made = p
        .engine
        .image_document("Folder", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    let job = p.job(made.source);
    assert_eq!(p.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        p.eye.calls.load(Ordering::SeqCst),
        2,
        "only the second file is new"
    );
    // Unnamed, a document takes its folder's name.
    let unnamed = p
        .engine
        .image_document("  ", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    let folder = p
        .dir
        .path()
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    assert_eq!(Some(unnamed.source.file_name), folder);
}

#[tokio::test]
async fn a_document_stopped_part_way_resumes_with_the_pages_not_yet_read() {
    let p = pages();
    p.eye
        .stop_once
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(2);
    let files = p.images(&["a1.png", "a2.png", "a3.png"]);
    let made = p
        .engine
        .image_document("Diary", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    let job = p.job(made.source);
    assert_eq!(p.run(job.id).await, JobStatus::Paused);
    let pieces = p.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Pending), 1);
    // The manifest is the job's: read back from the store as it was made.
    let stored = p.store.job(job.id).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(stored.sources, job.sources);
    let before = p.eye.calls.load(Ordering::SeqCst);
    assert_eq!(p.run(job.id).await, JobStatus::Completed);
    assert_eq!(
        p.eye.calls.load(Ordering::SeqCst),
        before + 1,
        "only the page stopped is sent again"
    );
    let txt = p.read("Diary.txt");
    assert!(files.iter().all(|f| txt.contains(&answer(f))), "{txt}");
}

#[tokio::test]
async fn a_missing_or_replaced_page_fails_alone_and_is_named() {
    let p = pages();
    let files = p.images(&["s1.png", "s2.png", "s3.png"]);
    let made = p
        .engine
        .image_document("Box", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(p.dir.path().join("s2.png"), png(1200)).unwrap_or_else(|e| panic!("{e}"));
    std::fs::remove_file(p.dir.path().join("s3.png")).unwrap_or_else(|e| panic!("{e}"));
    let job = p.job(made.source);
    assert_eq!(p.run(job.id).await, JobStatus::CompletedWithErrors);
    let pieces = p.store.pieces(job.id).unwrap_or_default();
    assert_eq!(count(&pieces, PieceStatus::Done), 1);
    let error = |i: usize| pieces[i].error.clone().unwrap_or_default();
    assert!(
        error(1).contains("s2.png") && error(1).contains("changed"),
        "{}",
        error(1)
    );
    assert!(error(2).contains("s3.png"), "{}", error(2));
    assert_eq!(
        p.eye.calls.load(Ordering::SeqCst),
        1,
        "nothing sent for them"
    );
    // The document is still written, the pages not read marked in it.
    let txt = p.read("Box.txt");
    assert!(
        txt.contains(&answer(&files[0])) && txt.contains("[Not read"),
        "{txt}"
    );
    // The originals are left as they are.
    assert!(p.dir.path().join("s1.png").is_file());
}

#[test]
fn an_analysis_of_a_document_of_images_is_refused() {
    let p = pages();
    let files = p.images(&["x1.png", "x2.png"]);
    let made = p
        .engine
        .image_document("X", &files)
        .unwrap_or_else(|e| panic!("{e}"));
    let refused = p.engine.create_job(NewJob {
        name: "X".into(),
        task: Task::Analysis,
        sources: vec![made.source],
        setups: vec![SetupRequest {
            provider: ProviderId::new("eye"),
            model: "eye-1".into(),
            preset_id: None,
            label: None,
            options: Default::default(),
            merge: None,
        }],
        settings: Default::default(),
        output: Default::default(),
    });
    assert!(matches!(refused, Err(EngineError::UnsupportedSource(_))));
}
