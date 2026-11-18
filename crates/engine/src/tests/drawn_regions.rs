//! Regions the researcher draws over a scan, or whose box they adjust: kept apart from the stored
//! result, laid over the engine's regions in the review and on pages with no text piece, listed
//! in `provenance.json`, and never a reason to call a provider.

use kalamu_core::{
    AdjustedRegion, Correction, DrawnRegion, Granularity, PageHalf, PageRegions, RegionDraft,
    RegionOrigin,
};

use super::regions::Placing;
use super::*;

/// A provider that fails the test if it is ever called: drawing regions sends nothing.
struct Silent;

#[async_trait]
impl Provider for Silent {
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
        panic!("a region drawn or adjusted called the provider")
    }

    async fn list_models(&self, _key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        panic!("a region drawn or adjusted called the provider")
    }
}

fn engine_with(
    store: Arc<SqliteStore>,
    provider: Arc<dyn Provider>,
    dir: &std::path::Path,
) -> Arc<Engine> {
    Engine::new(
        store,
        vec![provider],
        kalamu_catalog::Catalog::bundled(),
        PromptLibrary::in_memory(),
        Arc::new(Keys),
        EngineConfig {
            concurrency: 1,
            max_attempts: 1,
            base_delay: Duration::from_millis(5),
            app_version: "test".into(),
            listening_cache: dir.join("listening"),
            snapshots: dir.join("snapshots"),
            ..EngineConfig::default()
        },
    )
}

/// A two-page report read by `Placing` (Tesseract's words on page 1, nothing placed on page 2),
/// and an engine on the same store whose provider must never be called.
async fn read_report() -> (Fixture, Job, SourceFile, Arc<Engine>) {
    let store = Arc::new(SqliteStore::in_memory().unwrap_or_else(|_| panic!("store")));
    let dir = tempfile::tempdir().unwrap_or_else(|_| panic!("tempdir"));
    let engine = engine_with(store.clone(), Arc::new(Placing), dir.path());
    let quiet = engine_with(store.clone(), Arc::new(Silent), dir.path());
    let fixture = Fixture {
        engine,
        fake: Fake::new(),
        store,
        dir,
    };
    let source = fixture.pdf("rapport.pdf", 2);
    let job = fixture.new_job(vec![source.clone()], &["fake-model"]);
    assert_eq!(fixture.run(job.id).await, JobStatus::Completed);
    (fixture, job, source, quiet)
}

fn draft(
    job: &Job,
    source: &SourceFile,
    page: u32,
    adjusts: Option<AdjustedRegion>,
) -> RegionDraft {
    RegionDraft {
        id: None,
        job_id: job.id,
        source_id: source.id,
        page,
        half: None,
        x: 0.6,
        y: 0.05,
        width: 0.3,
        height: 0.1,
        text: "Note en marge : reçu le 4 mai".into(),
        adjusts,
    }
}

fn ok<T>(r: Result<T, EngineError>) -> T {
    r.unwrap_or_else(|e| panic!("{e}"))
}

fn code<T: std::fmt::Debug>(r: Result<T, EngineError>) -> &'static str {
    match r {
        Ok(v) => panic!("refused, yet {v:?}"),
        Err(e) => e.code(),
    }
}

#[tokio::test]
async fn drawn_and_adjusted_regions_are_laid_over_the_engine_s() {
    let (_f, job, source, quiet) = read_report().await;
    let setup = job.setups[0].id;
    let page = |index| ok(quiet.page_regions(job.id, source.id, setup, index));
    let engine_only = page(0);
    let result = engine_only
        .result
        .clone()
        .unwrap_or_else(|| panic!("the page's result"));
    assert!(engine_only.drawn.is_empty());
    assert_eq!(engine_only.lines[1].id, "r1_l2");

    // A note in the margin, drawn on its own; the second line given a tighter box.
    let note = ok(quiet.save_region(draft(&job, &source, 1, None)));
    assert_eq!(note.source_hash, source.sha256);
    let adjusts = AdjustedRegion {
        result: result.clone(),
        region: "r1_l2".into(),
    };
    let tighter = ok(quiet.save_region(RegionDraft {
        x: 0.1,
        y: 0.128,
        width: 0.45,
        height: 0.024,
        text: String::new(),
        ..draft(&job, &source, 1, Some(adjusts.clone()))
    }));
    let shown = page(0);
    let line = &shown.lines[1];
    assert_eq!(line.id, "r1_l2");
    assert_eq!(line.origin, RegionOrigin::Manual);
    assert!((line.x - 0.1).abs() < 1e-6 && (line.width - 0.45).abs() < 1e-6);
    assert_eq!(
        line.text, engine_only.lines[1].text,
        "the engine's text kept"
    );
    assert_eq!(line.span, engine_only.lines[1].span);
    assert_eq!(shown.words, engine_only.words);
    assert_eq!(shown.drawn.len(), 1);
    assert_eq!(shown.drawn[0].id, note.id);
    assert_eq!(shown.drawn[0].origin, RegionOrigin::Manual);
    assert_eq!(shown.drawn[0].granularity, Granularity::Block);
    assert_eq!(shown.drawn[0].text, "Note en marge : reçu le 4 mai");

    // A correction changes the text shown, not the boxes.
    ok(quiet.save_correction(Correction {
        job_id: job.id,
        setup_id: setup,
        source_id: source.id,
        index: 0,
        text: Some("Le cercle de Kita\nLe cercle de Kita\nDakar, le 3 mai".into()),
        fields: None,
        reviewed: false,
        updated_at: Timestamp::now(),
    }));
    let corrected = page(0);
    assert!(corrected.corrected);
    assert_eq!(corrected.lines[1].origin, RegionOrigin::Manual);
    assert_eq!(corrected.drawn.len(), 1);
    assert_eq!(corrected.result.as_deref(), Some(result.as_str()));

    // Page 2, read without positions: no engine's regions, the researcher's all the same.
    let second = ok(quiet.save_region(draft(&job, &source, 2, None)));
    let plain = page(1);
    assert!(plain.blocks.is_empty());
    assert!(plain.result.is_some());
    assert_eq!(plain.drawn.len(), 1);
    assert_eq!(plain.drawn[0].id, second.id);

    // A review with no text piece shows what was drawn; the box adjusting the reading still
    // shown in the text review is that review's.
    let analysis = ok(quiet.drawn_regions(job.id, source.id, 1, None));
    assert!(analysis.blocks.is_empty() && analysis.result.is_none());
    let ids: Vec<&str> = analysis.drawn.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, [note.id.as_str()]);
    assert!(
        ok(quiet.drawn_regions(job.id, source.id, 1, Some(PageHalf::Left)))
            .drawn
            .is_empty()
    );

    // A second box for the same engine's region replaces the first; a change keeps its id and
    // the time it was made.
    let again = ok(quiet.save_region(RegionDraft {
        width: 0.4,
        ..draft(&job, &source, 1, Some(adjusts))
    }));
    assert_eq!(again.id, tighter.id);
    assert_eq!(again.created_at, tighter.created_at);
    let renamed = ok(quiet.save_region(RegionDraft {
        id: Some(note.id.clone()),
        text: "Tampon du cercle".into(),
        ..draft(&job, &source, 1, None)
    }));
    assert_eq!(
        (renamed.id.as_str(), renamed.created_at),
        (note.id.as_str(), note.created_at)
    );
    let regions = ok(quiet
        .store
        .source_regions(job.id, source.id)
        .map_err(Into::into));
    assert_eq!(regions.len(), 3);

    // Removed, then put back by Undo, with its id.
    let removed = ok(quiet.remove_region(&note.id));
    assert_eq!(removed, renamed);
    assert_eq!(page(0).drawn.len(), 0);
    assert_eq!(code(quiet.remove_region(&note.id)), "region.not_found");
    let back = ok(quiet.restore_region(removed.clone()));
    assert_eq!(back, removed);
    assert_eq!(page(0).drawn[0].id, note.id);
}

#[tokio::test]
async fn a_region_is_refused_off_the_page_or_adjusting_what_is_not_there() {
    let (_f, job, source, quiet) = read_report().await;
    let result = ok(quiet.page_regions(job.id, source.id, job.setups[0].id, 0))
        .result
        .unwrap_or_default();
    let adjusting = |result: &str, region: &str| {
        draft(
            &job,
            &source,
            1,
            Some(AdjustedRegion {
                result: result.into(),
                region: region.into(),
            }),
        )
    };
    assert_eq!(
        code(quiet.save_region(adjusting(&result, "r9"))),
        "region.adjusts"
    );
    assert_eq!(
        code(quiet.save_region(adjusting("another-reading", "r1"))),
        "region.adjusts"
    );
    // The region is on page 1's reading, not page 2's.
    assert_eq!(
        code(quiet.save_region(RegionDraft {
            page: 2,
            ..adjusting(&result, "r1")
        })),
        "region.adjusts"
    );
    assert_eq!(
        code(quiet.save_region(draft(&job, &source, 3, None))),
        "region.no_page"
    );
    assert_eq!(
        code(quiet.save_region(RegionDraft {
            half: Some(PageHalf::Right),
            ..draft(&job, &source, 1, None)
        })),
        "region.no_page"
    );
    assert_eq!(
        code(quiet.save_region(RegionDraft {
            width: 2.0,
            ..draft(&job, &source, 1, None)
        })),
        "region.outside"
    );
    assert_eq!(
        code(quiet.save_region(RegionDraft {
            job_id: JobId::new(),
            ..draft(&job, &source, 1, None)
        })),
        "job.not_found"
    );
    assert_eq!(
        code(quiet.save_region(RegionDraft {
            id: Some("never-saved".into()),
            ..draft(&job, &source, 1, None)
        })),
        "region.not_found"
    );
    assert!(
        ok(quiet
            .store
            .source_regions(job.id, source.id)
            .map_err(Into::into))
        .is_empty()
    );
}

/// A box made on a reading no set-up shows any more (the page read again since) adjusts
/// nothing: it is shown as one the researcher drew, in every review of the page.
#[tokio::test]
async fn an_adjustment_of_a_reading_gone_is_shown_as_drawn() {
    let (f, job, source, quiet) = read_report().await;
    let old = DrawnRegion {
        id: "d-old".into(),
        job_id: job.id,
        source_id: source.id,
        page: 1,
        half: None,
        x: 0.1,
        y: 0.1,
        width: 0.5,
        height: 0.02,
        text: String::new(),
        adjusts: Some(AdjustedRegion {
            result: "a-reading-replaced".into(),
            region: "r1_l1".into(),
        }),
        source_hash: source.sha256.clone(),
        created_at: Timestamp(1),
        updated_at: Timestamp(1),
    };
    f.store.put_region(&old).unwrap_or_else(|e| panic!("{e}"));
    let page: PageRegions = ok(quiet.page_regions(job.id, source.id, job.setups[0].id, 0));
    assert!(
        page.lines
            .iter()
            .all(|l| l.origin == RegionOrigin::Provider)
    );
    assert_eq!(page.drawn.len(), 1);
    assert_eq!(page.drawn[0].id, "d-old");
    let analysis = ok(quiet.drawn_regions(job.id, source.id, 1, None));
    assert_eq!(analysis.drawn.len(), 1);
}

#[tokio::test]
async fn provenance_lists_the_researcher_s_regions_without_their_text() {
    let (f, job, source, quiet) = read_report().await;
    let setup = job.setups[0].id;
    let provenance = || -> serde_json::Value {
        ok(quiet.export(job.id, setup, source.id));
        let text = std::fs::read_to_string(f.dir.path().join("rapport.provenance.json"))
            .unwrap_or_else(|e| panic!("{e}"));
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}"))
    };
    // Nothing drawn: no list at all, as before regions could be drawn.
    assert!(provenance().get("regions").is_none());

    let result = ok(quiet.page_regions(job.id, source.id, setup, 0))
        .result
        .unwrap_or_default();
    let note = ok(quiet.save_region(draft(&job, &source, 1, None)));
    let tighter = ok(quiet.save_region(RegionDraft {
        text: String::new(),
        ..draft(
            &job,
            &source,
            1,
            Some(AdjustedRegion {
                result: result.clone(),
                region: "r1".into(),
            }),
        )
    }));
    let written = provenance();
    let regions = written
        .get("regions")
        .and_then(|r| r.as_array())
        .unwrap_or_else(|| panic!("regions in {written}"));
    assert_eq!(regions.len(), 2);
    let by_id = |id: &str| {
        regions
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("{id} in {written}"))
    };
    let drawn = by_id(&note.id);
    assert_eq!(drawn["page"], 1);
    assert_eq!(drawn["origin"], "manual");
    assert_eq!(drawn["has_text"], true);
    assert!(drawn.get("adjusts").is_none());
    let adjusted = by_id(&tighter.id);
    assert_eq!(adjusted["has_text"], false);
    assert_eq!(adjusted["adjusts"]["result"], result.as_str());
    assert_eq!(adjusted["adjusts"]["region"], "r1");
    // The researcher's words are in the files, not in the methods note.
    assert!(!written.to_string().contains("Note en marge"));
}
