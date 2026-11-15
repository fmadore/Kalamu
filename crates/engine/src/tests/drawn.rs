//! PDF pages drawn as images for an engine that reads images only: the resolution each page was
//! drawn at is kept with its result, gathered once per value in `provenance.json`, and handed to
//! the exporters with each page.

use kalamu_core::provenance::PAGE_RESOLUTION;
use kalamu_core::{CacheKey, RequestSetting};

use super::*;

/// Reads page images only, as Tesseract does, and records the resolution the drawn page states.
struct Drawing;

/// The resolution `kalamu-media` writes into a page it draws, as Tesseract reads it back.
fn stated(png: &[u8]) -> Option<String> {
    let keyword = b"Kalamu drawn dpi\0";
    let at = png.windows(keyword.len()).position(|w| w == keyword)? + keyword.len();
    let value: Vec<u8> = png[at..]
        .iter()
        .take_while(|b| b.is_ascii_digit() || **b == b'.')
        .copied()
        .collect();
    String::from_utf8(value).ok()
}

#[async_trait]
impl Provider for Drawing {
    fn id(&self) -> ProviderId {
        ProviderId::new("fake")
    }

    fn accepted_inputs(&self, _task: Task) -> &'static [&'static str] {
        &["image/png"]
    }

    fn sent_settings(&self, request: &ProcessRequest<'_>, _batched: bool) -> Vec<RequestSetting> {
        let mut settings = vec![RequestSetting::new("page_segmentation", "auto")];
        if let Some(dpi) = stated(&request.input.bytes) {
            settings.push(RequestSetting::new(PAGE_RESOLUTION, format!("{dpi} dpi")));
        }
        settings
    }

    async fn process(
        &self,
        _key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        assert_eq!(request.input.mime, "image/png", "a page arrives drawn");
        Ok(ProcessOutput {
            text: request.input.piece.label(),
            ..ProcessOutput::default()
        })
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        Ok(vec!["fake-model".into()])
    }
}

#[tokio::test]
async fn each_page_keeps_the_resolution_it_was_drawn_at_and_provenance_lists_each_once() {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = Engine::new(
        store.clone(),
        vec![Arc::new(Drawing)],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 2,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.path().join("listening"),
            snapshots: dir.path().join("snapshots"),
            ..EngineConfig::default()
        },
    );
    // A4, a poster drawn 7,000 pixels wide (252 dpi), and A4 again.
    let path = dir.path().join("affiche.pdf");
    let pdf = kalamu_media::sample_pdf(&[(595.0, 842.0), (2000.0, 1000.0), (595.0, 842.0)], true);
    std::fs::write(&path, pdf).unwrap_or_else(|e| panic!("{e}"));
    let source = kalamu_media::inspect(&path, None).unwrap_or_else(|e| panic!("{e}"));
    let fixture = Fixture {
        engine: engine.clone(),
        fake: Fake::new(),
        store: store.clone(),
        dir,
    };
    let job = fixture.new_job(vec![source], &["fake-model"]);
    assert_eq!(fixture.run(job.id).await, JobStatus::Completed);
    let job = store.job(job.id).unwrap_or_else(|e| panic!("{e}"));
    let (setup, source) = (&job.setups[0], &job.sources[0]);

    // The cache keys are what they were: what was recorded plays no part in them.
    let pieces = store
        .source_pieces(job.id, setup.id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    for piece in &pieces {
        let key = CacheKey::compute(
            job.task,
            &source.sha256,
            &piece.piece,
            setup,
            &job.settings.languages,
            &[],
        );
        assert_eq!(piece.cache_key.as_deref(), Some(key.as_str()));
    }

    let doc = engine
        .export_document(&job, setup.id, source.id)
        .unwrap_or_else(|e| panic!("{e}"));
    let drawn: Vec<Option<f32>> = doc.parts.iter().map(|p| p.drawn_dpi).collect();
    assert_eq!(drawn, [Some(300.0), Some(252.0), Some(300.0)]);
    // Each value once, in the order first read: not once per page.
    let request =
        serde_json::to_string_pretty(&doc.provenance.request).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        request,
        r#"[
  {
    "name": "page_segmentation",
    "value": "auto"
  },
  {
    "name": "page_resolution",
    "value": "300 dpi"
  },
  {
    "name": "page_resolution",
    "value": "252 dpi"
  }
]"#
    );
}
