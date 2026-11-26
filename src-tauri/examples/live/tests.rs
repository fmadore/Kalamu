//! The tool against local stand-ins: never the network, never the system keychain.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use kalamu_catalog::ModelStatus;
use kalamu_core::{Pace, ProviderId, Task};
use kalamu_engine::EstimateRequest;
use kalamu_providers::Gemini;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::report::Outcome;

const KEY: &str = "AIzaStandInKeyForTheLiveTests0123";

fn samples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../samples")
}

fn gemini_keys() -> Arc<Keys> {
    let env = |name: &str| (name == "KALAMU_GEMINI_API_KEY").then(|| KEY.to_string());
    Arc::new(Keys::gather(&[ProviderId::new("gemini")], env, None))
}

/// A Gemini answer to a page, billed `output_tokens` of output.
fn page_answer(output_tokens: u64) -> serde_json::Value {
    serde_json::json!({
        "candidates": [{
            "content": { "role": "model", "parts": [{ "text": "LE BULLETIN\nUne page lue." }] },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": 1290,
            "candidatesTokenCount": output_tokens,
            "totalTokenCount": 1290 + output_tokens
        },
        "modelVersion": "stand-in",
        "responseId": "stand-in"
    })
}

/// A runner whose only provider is Gemini at the stand-in's address.
fn runner(
    server: &MockServer,
    out: &std::path::Path,
    ceiling: Option<f64>,
) -> (Arc<Runner>, Arc<Engine>) {
    let gemini = Arc::new(Gemini::with_base_url(
        kalamu_providers::client(),
        &server.uri(),
    ));
    let keys = gemini_keys();
    let (engine, store) =
        engine(out, vec![gemini.clone()], keys.clone(), None).unwrap_or_else(|e| panic!("{e}"));
    let runner = Runner::new(
        engine.clone(),
        store,
        vec![gemini],
        keys,
        samples(),
        out.to_path_buf(),
        Duration::from_secs(60),
        ceiling,
    );
    (runner, engine)
}

fn page_step(engine: &Engine, name: &str) -> Step {
    steps::cheapest(
        engine,
        &samples(),
        &ProviderId::new("gemini"),
        name,
        Task::TextExtraction,
        steps::PAGE,
        None,
        Pace::Fast,
    )
}

#[test]
fn the_relay_knows_every_provider_the_registry_lists() {
    let registry: BTreeSet<String> = kalamu_providers::registry(kalamu_providers::client())
        .iter()
        .map(|p| p.id().as_str().to_string())
        .collect();
    let services: BTreeSet<String> = SERVICES.iter().map(|(id, _, _)| id.to_string()).collect();
    assert_eq!(
        registry, services,
        "a provider was added: give SERVICES its address"
    );
    let (providers, embedders) = cloud(Some(&Relay {
        address: "http://127.0.0.1:9".into(),
    }));
    let relayed: BTreeSet<String> = providers.iter().map(|p| p.id().as_str().into()).collect();
    assert_eq!(relayed, services);
    let embedding: BTreeSet<String> = kalamu_providers::embedders(kalamu_providers::client())
        .iter()
        .map(|e| e.id().as_str().into())
        .collect();
    let relayed: BTreeSet<String> = embedders.iter().map(|e| e.id().as_str().into()).collect();
    assert_eq!(relayed, embedding);
}

#[tokio::test]
async fn the_weekly_set_takes_each_providers_cheapest_offered_model() {
    let out = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let keys = Arc::new(Keys::default());
    let (providers, _) = cloud(None);
    // No FFmpeg, as on GitHub's runner.
    let (engine, _) = engine(out.path(), providers, keys, None).unwrap_or_else(|e| panic!("{e}"));
    let ids: Vec<ProviderId> = SERVICES
        .iter()
        .map(|(id, _, _)| ProviderId::new(*id))
        .collect();
    let steps = steps::weekly(&engine, &samples(), &ids);
    assert_eq!(steps.len(), ids.len() * 5, "five steps a provider");
    assert!(
        steps.iter().take(ids.len()).all(Step::is_batch),
        "the batches come first"
    );
    let catalog = engine.catalog();
    let mut jobs = 0;
    for step in &steps {
        let Work::Job(plan) = &step.work else {
            continue;
        };
        jobs += 1;
        let chosen = &plan.setups[0];
        let economy = plan.settings.pace == Pace::Economy;
        let source = engine
            .inspect(&[samples().join(&plan.files[0])])
            .into_iter()
            .next()
            .and_then(Result::ok)
            .map(|(s, _)| s)
            .unwrap_or_else(|| panic!("{}", plan.files[0]));
        let price = |model: &str| {
            let e = engine.estimate(&EstimateRequest {
                task: plan.task,
                sources: vec![source.clone()],
                setups: vec![steps::setup(
                    chosen.provider.as_str(),
                    model,
                    chosen.preset_id.as_deref(),
                    Default::default(),
                )],
                sample_pages: None,
            });
            if economy {
                e.total_economy_usd
            } else {
                e.total_usd
            }
        };
        let model = catalog
            .models
            .iter()
            .find(|m| m.id == chosen.model)
            .unwrap_or_else(|| panic!("{}", chosen.model));
        assert!(
            model.is_offered() && model.status != ModelStatus::Deprecated,
            "{}",
            step.name
        );
        let best = price(&chosen.model).unwrap_or_else(|| panic!("{} has a price", step.name));
        for other in catalog.models.iter().filter(|m| {
            m.provider == chosen.provider
                && m.is_offered()
                && m.status != ModelStatus::Deprecated
                && m.does(plan.task)
                && m.inputs.iter().any(|i| i == steps::mime(&plan.files[0]))
        }) {
            if economy
                && !engine
                    .provider(&chosen.provider)
                    .is_ok_and(|p| p.batches(plan.task, &other.id))
            {
                continue;
            }
            if let Some(p) = price(&other.id) {
                assert!(
                    best <= p,
                    "{}: {} is cheaper than {}",
                    step.name,
                    other.id,
                    chosen.model
                );
            }
        }
    }
    assert!(jobs >= 12, "most steps have a model: {jobs}");
    // Anthropic offers neither recordings nor embeddings.
    for name in ["recording-anthropic", "embedding-anthropic"] {
        let step = steps.iter().find(|s| s.name == name);
        assert!(
            matches!(step.map(|s| &s.work), Some(Work::Absent { .. })),
            "{name}"
        );
    }
}

/// The first weekly run on GitHub (26 November 2026), with no FFmpeg on the runner, estimated
/// every recording at $0 and billed up to $0.0057 for each: the sample's length was unknown and
/// counted as none, so the cheapest model was whichever came first and the ceiling held nothing.
/// The WAV file's header now gives its length, and each provider's recording step is priced.
#[tokio::test]
async fn a_recording_is_priced_without_ffmpeg() {
    let out = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let (providers, _) = cloud(None);
    let (engine, _) = engine(out.path(), providers, Arc::new(Keys::default()), None)
        .unwrap_or_else(|e| panic!("{e}"));
    let source = engine
        .inspect(&[samples().join(steps::RECORDING)])
        .into_iter()
        .next()
        .and_then(Result::ok)
        .map(|(s, _)| s)
        .unwrap_or_else(|| panic!("{}", steps::RECORDING));
    let ms = source.duration_ms.unwrap_or_default();
    assert!((21_000..22_000).contains(&ms), "{ms}");
    for id in ["gemini", "openai", "mistral"] {
        let step = steps::cheapest(
            &engine,
            &samples(),
            &ProviderId::new(id),
            "recording",
            Task::AudioTranscription,
            steps::RECORDING,
            None,
            Pace::Fast,
        );
        let Work::Job(plan) = &step.work else {
            panic!("{id}: no recording step");
        };
        let usd = engine
            .estimate(&EstimateRequest {
                task: plan.task,
                sources: vec![source.clone()],
                setups: plan.setups.clone(),
                sample_pages: None,
            })
            .total_usd
            .unwrap_or_default();
        // Twenty seconds: a few hundredths of a cent, not nothing.
        assert!((0.000_01..0.01).contains(&usd), "{id}: ${usd}");
    }
}

#[tokio::test]
async fn a_step_whose_estimate_passes_the_ceiling_sends_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_answer(40)))
        .expect(0)
        .mount(&server)
        .await;
    let out = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let (runner, engine) = runner(&server, out.path(), Some(0.000_000_1));
    let rows = run_all(&runner, vec![page_step(&engine, "page-gemini")]).await;
    assert_eq!(rows.len(), 1);
    assert!(
        matches!(&rows[0].outcome, Outcome::Ceiling(why) if why.contains("estimate")),
        "{:?}",
        rows[0].outcome
    );
    assert_eq!(runner.spent(), 0.0);
}

#[tokio::test]
async fn once_a_real_cost_passes_the_ceiling_nothing_more_starts() {
    let server = MockServer::start().await;
    // Ten million tokens written: far past any estimate and any ceiling.
    Mock::given(method("POST"))
        .and(path_regex(":generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page_answer(10_000_000)))
        .expect(1)
        .mount(&server)
        .await;
    let out = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let (runner, engine) = runner(&server, out.path(), Some(0.05));
    let steps = vec![
        page_step(&engine, "page-gemini"),
        page_step(&engine, "page-gemini-again"),
    ];
    let rows = run_all(&runner, steps).await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].outcome, Outcome::Passed, "the call itself answered");
    assert!(rows[0].billed.unwrap_or(0.0) > 0.05, "{:?}", rows[0].billed);
    assert!(
        matches!(&rows[1].outcome, Outcome::Ceiling(why) if why.contains("passed earlier")),
        "{:?}",
        rows[1].outcome
    );
    assert!(runner.ledger_passed());
}

#[tokio::test]
async fn a_key_the_provider_echoes_never_reaches_the_summary() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "code": 400,
                "message": format!("API key not valid: {KEY}. Please pass a valid API key."),
                "status": "INVALID_ARGUMENT"
            }
        })))
        .mount(&server)
        .await;
    let out = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let (runner, engine) = runner(&server, out.path(), Some(1.0));
    let rows = run_all(&runner, vec![page_step(&engine, "page-gemini")]).await;
    assert!(
        matches!(&rows[0].outcome, Outcome::Failed(_)),
        "{:?}",
        rows[0].outcome
    );
    let table = report::markdown(
        &rows,
        &Totals {
            set: "weekly".into(),
            ceiling: Some(1.0),
            spent: runner.spent(),
            passed: false,
            catalogue: String::new(),
            date: today(),
        },
    );
    assert!(!table.contains(KEY), "{table}");
    assert!(table.contains("[key]"), "{table}");
}

#[test]
fn today_is_a_date() {
    let d = today();
    assert_eq!(d.len(), 10);
    assert!(d.starts_with("20"));
    assert_eq!(&d[4..5], "-");
}
