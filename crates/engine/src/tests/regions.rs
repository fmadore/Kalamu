//! The regions the review draws over a page: read from the store as the engine stored them, each
//! placed in the text shown, the correction included, with no provider called.

use kalamu_core::{Correction, TextLine, TextMapping, TextRegion, TextWord};

use super::*;

/// Places every word of page 1, as Tesseract does; reports no layout for page 2.
pub(super) struct Placing;

fn line(y: f32, words: &[&str]) -> TextLine {
    TextLine {
        x: 0.1,
        y,
        width: 0.5,
        height: 0.02,
        words: words
            .iter()
            .enumerate()
            .map(|(i, w)| TextWord {
                x: 0.1 + i as f32 * 0.1,
                y,
                width: 0.08,
                height: 0.02,
                text: (*w).to_owned(),
                confidence: Some(0.9),
            })
            .collect(),
    }
}

#[async_trait]
impl Provider for Placing {
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
        let kalamu_core::PieceRef::Page { number, .. } = request.input.piece else {
            return Err(ProviderError::Other("not a page".into()));
        };
        if number != 1 {
            return Ok(ProcessOutput {
                text: "Une page lue sans positions.".into(),
                ..ProcessOutput::default()
            });
        }
        let lines = vec![
            line(0.1, &["Le", "cercle", "de", "Kita"]),
            line(0.13, &["Le", "cercle", "de", "Kita"]),
            line(0.16, &["Dakar,", "le", "3", "rnai"]),
        ];
        let text = lines
            .iter()
            .map(TextLine::text)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ProcessOutput {
            text: text.clone(),
            regions: vec![TextRegion {
                x: 0.1,
                y: 0.1,
                width: 0.5,
                height: 0.08,
                text,
                lines,
            }],
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["fake-model".into()])
    }
}

#[tokio::test]
async fn regions_come_from_the_store_and_follow_a_correction() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![Arc::new(Placing)],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 1,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let fixture = Fixture {
        engine: engine.clone(),
        fake: Fake::new(),
        store: store.clone(),
        dir,
    };
    let source = fixture.pdf("rapport.pdf", 2);
    let job = fixture.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(fixture.run(job.id).await, JobStatus::Completed);
    let setup = job.setups[0].id;
    let regions = |index| {
        engine
            .page_regions(job.id, source.id, setup, index)
            .unwrap_or_else(|e| panic!("{e}"))
    };

    let page = regions(0);
    assert_eq!(page.mapping, TextMapping::Exact);
    assert!(!page.corrected);
    assert_eq!(
        (page.blocks.len(), page.lines.len(), page.words.len()),
        (1, 3, 12)
    );
    // The two lines that read alike each have their own place in the model's text.
    let spans: Vec<_> = page.lines.iter().map(|l| l.span).collect();
    assert!(spans.iter().all(Option::is_some));
    assert_ne!(spans[0], spans[1]);
    assert_eq!(page.words[0].origin, kalamu_core::RegionOrigin::Provider);

    // A page read without positions has none: the review falls back to the page.
    let plain = regions(1);
    assert!(plain.blocks.is_empty());
    // Nor does a piece the job does not have.
    assert!(regions(9).blocks.is_empty());

    // The researcher corrects a misread word: places stay only where the words still stand
    // once, and the boxes stay as the engine stored them.
    engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: setup,
            source_id: source.id,
            index: 0,
            text: Some("Le cercle de Kita\nLe cercle de Kita\nDakar, le 3 mai".into()),
            fields: None,
            reviewed: false,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let corrected = regions(0);
    assert_eq!(corrected.mapping, TextMapping::Unique);
    assert!(corrected.corrected);
    assert_eq!(corrected.lines[0].span, None, "a repeated line: no guess");
    assert_eq!(corrected.lines[2].span, None, "a corrected line: no guess");
    let rnai = corrected.words.iter().find(|w| w.text == "rnai");
    assert_eq!(rnai.map(|w| w.span), Some(None));
    let dakar = corrected.words.iter().find(|w| w.text == "Dakar,");
    assert!(dakar.is_some_and(|w| w.span.is_some()));
    assert_eq!(corrected.words[0].x, page.words[0].x);
}

/// Places each page's text as one block, its paragraph on one line, as Mistral OCR does.
struct Blocks;

/// A paragraph of 80 words, written on one line.
fn paragraph() -> String {
    ["Le cercle de Kita a reçu les notables"; 10].join(" ")
}

#[async_trait]
impl Provider for Blocks {
    fn id(&self) -> ProviderId {
        ProviderId::new("fake")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["application/pdf"]
    }

    async fn process(
        &self,
        _key: &ApiKey,
        _request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        let text = paragraph();
        Ok(ProcessOutput {
            text: text.clone(),
            regions: vec![TextRegion::block(0.1, 0.2, 0.3, 0.4, text)],
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["fake-model".into()])
    }
}

#[tokio::test]
async fn a_block_placed_whole_is_wrapped_by_the_page_s_proportions() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![Arc::new(Blocks)],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 1,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let fixture = Fixture {
        engine: engine.clone(),
        fake: Fake::new(),
        store: store.clone(),
        dir,
    };
    // An A4 page: 595 by 842 points.
    let source = fixture.pdf("journal.pdf", 1);
    let job = fixture.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(fixture.run(job.id).await, JobStatus::Completed);
    let setup = job.setups[0].id;
    let regions = || {
        engine
            .page_regions(job.id, source.id, setup, 0)
            .unwrap_or_else(|e| panic!("{e}"))
    };

    let page = regions();
    let expected =
        TextRegion::block(0.1, 0.2, 0.3, 0.4, paragraph()).estimated_lines(842.0 / 595.0);
    // The one line of text becomes the lines its column holds, each word on one of them.
    assert!(page.lines.len() > 10, "{}", page.lines.len());
    assert_eq!(page.lines.len(), expected.len());
    assert!(
        page.lines
            .iter()
            .all(|l| l.origin == kalamu_core::RegionOrigin::Estimated)
    );
    assert_eq!(page.words.len(), 80);
    assert!(page.lines[0].height < 0.04);

    // The page's size is kept for the source's pages: the file is not read again.
    std::fs::remove_file(&source.path).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(regions().lines.len(), expected.len());
    // Unread, and the file gone, the page's proportions are unknown: the block alone, with no
    // lines guessed.
    *crate::control::lock(&engine.page_sizes) = crate::regions::PageSizes::default();
    let alone = regions();
    assert_eq!(alone.blocks.len(), 1);
    assert!(alone.lines.is_empty() && alone.words.is_empty());
}

/// PAGE XML is written as one file per page beside the source, listed with the job's other
/// files in the order of the pages, and written again from the correction, dated by it.
#[tokio::test]
async fn page_xml_is_written_and_listed_one_file_per_page() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let mut catalog = kalamu_catalog::Catalog::bundled();
    let mut model = catalog
        .models
        .iter()
        .find(|m| m.id == "gemini-3.8-flash")
        .cloned()
        .unwrap_or_else(|| panic!("a listed model"));
    model.provider = ProviderId::new("fake");
    model.id = "fake-model".into();
    model.layout = true;
    catalog.models.push(model);
    let engine = Engine::new(
        store.clone(),
        vec![Arc::new(Placing)],
        catalog,
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 1,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    let fixture = Fixture {
        engine: engine.clone(),
        fake: Fake::new(),
        store: store.clone(),
        dir,
    };
    let source = fixture.pdf("rapport.pdf", 2);
    let job = engine
        .create_job(NewJob {
            name: "Page".into(),
            task: Task::TextExtraction,
            sources: vec![source.clone()],
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
                formats: vec![ExportFormat::Alto, ExportFormat::PageXml],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("create job: {e}"));
    assert!(job.output.formats.contains(&ExportFormat::PageXml));
    assert_eq!(fixture.run(job.id).await, JobStatus::Completed);

    let names = |files: Vec<String>| -> Vec<String> {
        files
            .iter()
            .filter_map(|f| {
                std::path::Path::new(f)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            })
            .collect()
    };
    let listed = names(engine.outputs(job.id).unwrap_or_else(|e| panic!("{e}")));
    assert_eq!(
        listed,
        [
            "rapport.alto.xml",
            "rapport.p0001.page.xml",
            "rapport.p0002.page.xml",
            "rapport.provenance.json"
        ]
    );
    let page_one = fixture.dir.path().join("rapport.p0001.page.xml");
    let read = std::fs::read_to_string(&page_one).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read.matches("<Word ").count(), 12, "{read}");
    assert!(read.contains("<Unicode>rnai</Unicode>"), "{read}");
    // Page 2 was read without positions: a page with no regions.
    let page_two = std::fs::read_to_string(fixture.dir.path().join("rapport.p0002.page.xml"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!page_two.contains("<TextRegion"), "{page_two}");

    // A correction writes the page again: the corrected word in its place, dated by it.
    engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: source.id,
            index: 0,
            text: Some("Le cercle de Kita\nLe cercle de Kita\nDakar, le 3 mai".into()),
            fields: None,
            reviewed: false,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    engine
        .export(job.id, job.setups[0].id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let corrected = std::fs::read_to_string(&page_one).unwrap_or_else(|e| panic!("{e}"));
    let saved = store
        .source_corrections(job.id, job.setups[0].id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let when = saved
        .first()
        .map(|c| c.updated_at)
        .unwrap_or_else(|| panic!("saved"));
    assert!(corrected.contains("<Unicode>mai</Unicode>"), "{corrected}");
    assert!(!corrected.contains("rnai"), "{corrected}");
    assert!(
        corrected.contains(&format!(
            "<LastChange>{}</LastChange>",
            when.utc_date_time()
        )),
        "{corrected}"
    );
}
