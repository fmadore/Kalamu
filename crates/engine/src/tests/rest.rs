//! A sample first (#215): a few pages spread through a document, then the rest with the same
//! set-up, or another model on the same pages, the pages already read never paid twice.

use super::*;

/// Each call of the priced model reads 1,000 tokens and writes 100, at Gemini 3.8 Flash's
/// prices: $0.001125.
const CALL: f64 = 0.001_125;

impl Fixture {
    /// The fake provider's priced model, as Gemini 3.8 Flash is priced.
    fn priced(&self) {
        let mut catalog = self.engine.catalog();
        let mut priced = catalog
            .models
            .iter()
            .find(|m| m.id == "gemini-3.8-flash")
            .cloned()
            .unwrap_or_else(|| panic!("a priced model"));
        priced.provider = ProviderId::new("fake");
        priced.id = "priced-model".into();
        priced.pricing.batch_discount = None;
        catalog.models.push(priced);
        self.engine.set_catalog(catalog);
    }

    /// Three pages spread through `source`, read with `model`.
    fn sample(&self, source: SourceFile, model: &str) -> Job {
        let mut settings = kalamu_core::JobSettings {
            sample_pages: Some(3),
            ..Default::default()
        };
        settings.sample_spread = true;
        self.engine
            .create_job(NewJob {
                name: "Bulletin".into(),
                task: Task::TextExtraction,
                sources: vec![source],
                setups: vec![SetupRequest {
                    provider: ProviderId::new("fake"),
                    model: model.into(),
                    preset_id: Some("text_extraction.newspaper".into()),
                    label: None,
                    options: Default::default(),
                    merge: None,
                }],
                settings,
                output: OutputSettings {
                    formats: vec![ExportFormat::Txt],
                    ..Default::default()
                },
            })
            .unwrap_or_else(|e| panic!("{e}"))
    }

    /// The pages read for `job`, in order, and how many came from the cache.
    fn pages_read(&self, job: JobId) -> (Vec<u32>, usize) {
        let pieces = self.store.pieces(job).unwrap_or_default();
        let mut pages: Vec<u32> = pieces
            .iter()
            .filter(|p| p.status.is_finished())
            .filter_map(|p| match p.piece {
                kalamu_core::PieceRef::Page { number, .. } => Some(number),
                _ => None,
            })
            .collect();
        pages.sort_unstable();
        (pages, count(&pieces, PieceStatus::Cached))
    }
}

#[tokio::test]
async fn the_rest_of_a_sample_pays_only_for_the_pages_it_did_not_read() {
    let f = fixture();
    f.priced();
    let sample = f.sample(f.pdf("bulletin.pdf", 40), "priced-model");
    assert_eq!(f.run(sample.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3);
    // Typical pages: one from each stretch of the document, the cover left out.
    assert_eq!(f.pages_read(sample.id), (vec![8, 21, 34], 0));
    let provenance =
        std::fs::read_to_string(f.dir.path().join("bulletin.provenance.json")).unwrap_or_default();
    assert!(provenance.contains("sample_spread_3_pages"), "{provenance}");

    // The estimate prices the 37 pages left; the three read cost nothing.
    let setup = sample.setups[0].id;
    let rest = f
        .engine
        .rest_estimate(sample.id, setup)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(rest.cached_pages, 3);
    assert_eq!(rest.estimate.rows[0].pages, 37);
    let whole = f.engine.estimate(&EstimateRequest {
        task: Task::TextExtraction,
        sources: sample.sources.clone(),
        setups: vec![SetupRequest {
            provider: ProviderId::new("fake"),
            model: "priced-model".into(),
            preset_id: Some("text_extraction.newspaper".into()),
            label: None,
            options: Default::default(),
            merge: None,
        }],
        sample_pages: None,
    });
    assert_eq!(whole.rows[0].pages, 40);
    let (Some(rest_usd), Some(whole_usd)) = (rest.estimate.total_usd, whole.total_usd) else {
        panic!("both priced");
    };
    assert!(
        (rest_usd - whole_usd * 37.0 / 40.0).abs() < 1e-9,
        "{rest_usd} {whole_usd}"
    );

    // A prompt changed since the sample changes nothing: the set-up is copied as it ran.
    f.engine.with_prompts(|p| {
        let mut preset = p
            .get("text_extraction.newspaper")
            .unwrap_or_else(|e| panic!("{e}"));
        preset.instructions.push_str("\nRead slowly.");
        p.save(preset).unwrap_or_else(|e| panic!("{e}"));
    });
    let whole_job = f
        .engine
        .rest_of_sample(sample.id, setup, "Bulletin, every page".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(whole_job.settings.sample_pages, None);
    assert!(!whole_job.settings.sample_spread);
    assert_eq!(whole_job.setups[0].prompt, sample.setups[0].prompt);
    assert_ne!(whole_job.setups[0].id, setup);
    assert_eq!(f.run(whole_job.id).await, JobStatus::Completed);

    // Only the 37 other pages were sent and paid; the sample's three came from the cache.
    assert_eq!(f.fake.calls(), 40);
    let (pages, cached) = f.pages_read(whole_job.id);
    assert_eq!(pages, (1..=40).collect::<Vec<u32>>());
    assert_eq!(cached, 3);
    let spent = f.store.spent(whole_job.id).unwrap_or_default();
    assert!((spent - 37.0 * CALL).abs() < 1e-9, "{spent}");
}

#[tokio::test]
async fn another_model_on_a_sample_pays_only_for_that_model() {
    let f = fixture();
    f.priced();
    let sample = f.sample(f.pdf("bulletin.pdf", 40), "fake-model");
    assert_eq!(f.run(sample.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 3);

    let compared = f
        .engine
        .compare_sample(
            sample.id,
            ProviderId::new("fake"),
            "priced-model".into(),
            "Bulletin, two models".into(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    // The same pages, the sample's set-up as it ran beside the new one, asked the same thing.
    assert_eq!(compared.settings, sample.settings);
    assert_eq!(compared.setups.len(), 2);
    assert_eq!(compared.setups[0].prompt, sample.setups[0].prompt);
    assert_eq!(compared.setups[1].model, "priced-model");
    assert_eq!(
        compared.setups[1].prompt.preset_id.as_deref(),
        Some("text_extraction.newspaper")
    );
    assert_ne!(compared.setups[0].label, compared.setups[1].label);
    assert_eq!(f.run(compared.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 6, "only the new model read the three pages");
    assert_eq!(f.pages_read(compared.id), (vec![8, 8, 21, 21, 34, 34], 3));
    let spent = f.store.spent(compared.id).unwrap_or_default();
    assert!((spent - 3.0 * CALL).abs() < 1e-9, "{spent}");

    // Then the whole document with the model chosen: its three pages are not read again.
    let chosen = compared.setups[1].id;
    let rest = f
        .engine
        .rest_estimate(compared.id, chosen)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(rest.cached_pages, 3);
    let whole = f
        .engine
        .rest_of_sample(compared.id, chosen, "Bulletin".into())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(whole.setups.len(), 1);
    assert_eq!(whole.setups[0].model, "priced-model");
    assert_eq!(f.run(whole.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), 6 + 37);
}

#[tokio::test]
async fn a_set_up_the_sample_does_not_have_is_refused() {
    let f = fixture();
    let sample = f.sample(f.pdf("bulletin.pdf", 10), "fake-model");
    let other = kalamu_core::SetupId::new();
    assert!(matches!(
        f.engine.rest_estimate(sample.id, other),
        Err(EngineError::Store(StoreError::NotFound))
    ));
    assert!(matches!(
        f.engine.rest_of_sample(sample.id, other, "x".into()),
        Err(EngineError::Store(StoreError::NotFound))
    ));
}
