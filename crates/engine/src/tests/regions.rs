//! The regions the review draws over a page: read from the store as the engine stored them, each
//! placed in the text shown, the correction included, with no provider called.

use kalamu_core::{Correction, TextLine, TextMapping, TextRegion, TextWord};

use super::*;

/// Places every word of page 1, as Tesseract does; reports no layout for page 2.
struct Placing;

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
    assert!(!page.words[0].estimated);

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
