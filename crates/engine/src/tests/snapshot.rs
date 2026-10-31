//! Text frozen from a result and read by a later analysis: corrections included, nothing read
//! again, a later correction flagging the analysis without changing what it read.

use kalamu_core::Correction;

use super::*;

fn analysis_of(f: &Fixture, sources: Vec<SourceFile>) -> Job {
    f.engine
        .create_job(NewJob {
            name: "People and places".into(),
            task: Task::Analysis,
            sources,
            setups: vec![SetupRequest {
                provider: ProviderId::new("fake"),
                model: "fake-model".into(),
                preset_id: Some("analysis.entities".into()),
                label: None,
                options: Default::default(),
                merge: None,
            }],
            settings: Default::default(),
            output: OutputSettings {
                formats: vec![ExportFormat::Json],
                ..Default::default()
            },
        })
        .unwrap_or_else(|e| panic!("create analysis: {e}"))
}

fn correct(f: &Fixture, job: &Job, index: u32, text: &str) {
    f.engine
        .save_correction(Correction {
            job_id: job.id,
            setup_id: job.setups[0].id,
            source_id: job.sources[0].id,
            index,
            text: Some(text.into()),
            fields: None,
            reviewed: true,
            updated_at: Timestamp::now(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

#[tokio::test]
async fn an_analysis_reads_the_corrected_text_without_reading_the_scan_again() {
    let f = fixture();
    let scan = f.pdf("registre.pdf", 3);
    let extraction = f.new_job(vec![scan.clone()], &["fake-model"]);
    assert_eq!(f.run(extraction.id).await, JobStatus::Completed);
    correct(&f, &extraction, 1, "Le Père Éric écrit à l'imam de Bamako.");
    let calls = f.fake.calls();

    let sources = f
        .engine
        .snapshot(extraction.id, extraction.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(sources.len(), 1);
    let frozen = &sources[0];
    assert_eq!(frozen.kind, InputKind::Text);
    assert_eq!(frozen.file_name, "registre.txt");
    let from = frozen
        .derived_from
        .clone()
        .unwrap_or_else(|| panic!("lineage"));
    assert_eq!((from.job_id, from.source_id), (extraction.id, scan.id));

    let analysis = analysis_of(&f, sources.clone());
    assert_eq!(f.run(analysis.id).await, JobStatus::Completed);
    assert_eq!(
        f.fake.calls(),
        calls + 1,
        "one call: the analysis, not the pages again"
    );
    let read = f
        .fake
        .read
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone();
    assert_eq!(read.len(), 1);
    assert!(read[0].contains("--- Page 2 ---\n\nLe Père Éric écrit à l'imam de Bamako."));
    assert!(read[0].contains("fake-model page 3"), "{}", read[0]);

    // Written beside the original, not in the app's data folder, with its lineage.
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("registre.analysis.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    let derived = &provenance["source"]["derived_from"];
    assert_eq!(derived["job_id"], extraction.id.to_string());
    assert_eq!(derived["task"], "text_extraction");
    assert_eq!(
        (derived["pieces"].as_u64(), derived["corrected"].as_u64()),
        (Some(3), Some(1))
    );
    assert_eq!(provenance["source"]["sha256"], frozen.sha256);
    assert_eq!(
        f.engine.derived_states(analysis.id).unwrap_or_default()[0].freshness,
        Freshness::Current
    );

    // A later correction: the analysis is out of date, what it read is kept, and freezing again
    // gives a new text, analysed again; the same text again is not.
    let first_file = std::fs::read_to_string(&frozen.path).unwrap_or_default();
    correct(&f, &extraction, 2, "Fin du registre.");
    assert_eq!(
        f.engine.derived_states(analysis.id).unwrap_or_default()[0].freshness,
        Freshness::Changed
    );
    assert_eq!(
        std::fs::read_to_string(&frozen.path).unwrap_or_default(),
        first_file
    );
    let again = f
        .engine
        .snapshot(extraction.id, extraction.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(again[0].sha256, frozen.sha256);
    let rerun = analysis_of(&f, again.clone());
    assert_eq!(f.run(rerun.id).await, JobStatus::Completed);
    assert_eq!(f.fake.calls(), calls + 2);
    let same = analysis_of(&f, again);
    assert_eq!(f.run(same.id).await, JobStatus::Completed);
    assert_eq!(
        f.fake.calls(),
        calls + 2,
        "the same text is not analysed twice"
    );

    // The earlier job deleted: the frozen copy still stands.
    f.engine
        .delete_job(extraction.id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        f.engine.derived_states(analysis.id).unwrap_or_default()[0].freshness,
        Freshness::Gone
    );
    assert!(std::path::Path::new(&frozen.path).is_file());
}

#[tokio::test]
async fn only_text_results_can_be_frozen_and_failed_pages_are_listed() {
    let f = fixture();
    let analysis = analysis_of(&f, vec![f.pdf("a.pdf", 1)]);
    assert_eq!(f.run(analysis.id).await, JobStatus::Completed);
    assert!(matches!(
        f.engine.snapshot(analysis.id, analysis.setups[0].id),
        Err(EngineError::NotText)
    ));

    f.fake
        .fail(2, vec![ProviderError::InvalidRequest("no text".into())]);
    let extraction = f.new_job(vec![f.pdf("b.pdf", 3)], &["fake-model"]);
    assert_eq!(f.run(extraction.id).await, JobStatus::CompletedWithErrors);
    let sources = f
        .engine
        .snapshot(extraction.id, extraction.setups[0].id)
        .unwrap_or_else(|e| panic!("{e}"));
    let text = std::fs::read_to_string(&sources[0].path).unwrap_or_default();
    assert!(!text.contains("Page 2"), "{text}");
    let analysis = analysis_of(&f, sources);
    assert_eq!(f.run(analysis.id).await, JobStatus::Completed);
    let provenance: serde_json::Value = serde_json::from_slice(
        &std::fs::read(f.dir.path().join("b.analysis.provenance.json")).unwrap_or_default(),
    )
    .unwrap_or_default();
    assert_eq!(
        provenance["source"]["derived_from"]["missing"],
        serde_json::json!([2])
    );
}
