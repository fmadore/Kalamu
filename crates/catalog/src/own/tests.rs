use super::*;
use crate::{CostEstimate, Workload};

fn model(id: &str, reads_pages: bool, fills_forms: bool) -> OwnModel {
    OwnModel {
        id: id.into(),
        reads_pages,
        fills_forms,
        context: 32_768,
        max_output: 8_192,
        input_per_mtok: None,
        output_per_mtok: None,
        tried_at: Some("2026-11-26".into()),
        digest: Some("sha256:abc".into()),
    }
}

fn server(id: &str, name: &str, address: &str, models: Vec<OwnModel>) -> OwnServer {
    OwnServer {
        id: ProviderId::new(id),
        name: name.into(),
        address: address.into(),
        dialect: Dialect::Ollama,
        key: false,
        at_once: 1,
        models,
    }
}

/// Two servers: one on this computer with a model of each kind, one on the network with a
/// model that reads and fills and has prices.
fn servers() -> OwnServers {
    let mut priced = model("qwen3-vl:8b", true, true);
    priced.input_per_mtok = Some(0.5);
    priced.output_per_mtok = Some(1.5);
    OwnServers {
        servers: vec![
            server(
                "own-0000abcd",
                "My Mac",
                "http://localhost:11434",
                vec![
                    model("pages-only", true, false),
                    model("forms-only", false, true),
                    model("neither", false, false),
                ],
            ),
            OwnServer {
                dialect: Dialect::OpenAi,
                key: true,
                at_once: 4,
                ..server(
                    "own-1234beef",
                    "University",
                    "https://LLM.Uni.example/v1",
                    vec![priced],
                )
            },
        ],
    }
}

fn find<'a>(catalog: &'a Catalog, provider: &str, id: &str) -> &'a ModelInfo {
    catalog
        .model(&ProviderId::new(provider), id)
        .unwrap_or_else(|| panic!("no model {provider}/{id}"))
}

#[test]
fn the_file_is_read_with_its_defaults_and_written_back_the_same() {
    let json = r#"{"servers": [{
        "id": "own-0000abcd", "name": "My Mac", "address": "http://localhost:11434",
        "dialect": "ollama",
        "models": [{"id": "gemma3:12b", "reads_pages": true, "context": 32768, "max_output": 8192}]
    }]}"#;
    let read = OwnServers::from_json(json).unwrap_or_else(|e| panic!("{e}"));
    let first = &read.servers[0];
    assert_eq!(first.at_once, 1);
    assert!(!first.key);
    assert!(first.models[0].reads_pages && !first.models[0].fills_forms);
    assert_eq!(first.models[0].input_per_mtok, None);

    let again = OwnServers::from_json(&read.to_json().unwrap_or_default());
    assert_eq!(again.ok(), Some(read));
    let both = servers();
    let json = both.to_json().unwrap_or_default();
    assert!(json.contains("\"dialect\": \"open_ai\""), "{json}");
    assert_eq!(OwnServers::from_json(&json).ok(), Some(both));
    assert_eq!(
        OwnServers::from_json("{}").ok(),
        Some(OwnServers::default())
    );
}

#[test]
fn a_file_that_cannot_be_read_is_invalid() {
    assert!(matches!(
        OwnServers::from_json("{\"servers\": [{\"id\": 3}]}"),
        Err(OwnServersError::Invalid(_))
    ));
    assert!(matches!(
        OwnServers::from_json(&" ".repeat(crate::MAX_BYTES + 1)),
        Err(OwnServersError::Refused(_))
    ));
}

/// The list with one change is refused whole, for a reason that says `why`.
fn refused(why: &str, change: impl FnOnce(&mut OwnServers)) {
    let mut list = servers();
    change(&mut list);
    let json = list.to_json().unwrap_or_default();
    match OwnServers::from_json(&json) {
        Err(OwnServersError::Refused(problems)) => {
            assert!(problems.contains(why), "{problems:?} does not say {why:?}");
        }
        other => panic!("not refused for {why:?}: {other:?}"),
    }
}

#[test]
fn every_problem_refuses_the_file() {
    assert!(
        servers().problems().is_empty(),
        "{:?}",
        servers().problems()
    );
    for bad in [
        "gemini",
        "own-",
        "own-1234",
        "own-1234ABCD",
        "own-1234abcg",
        "own-1234abcd0",
    ] {
        refused("not own- and 8 hex digits", |l| {
            l.servers[0].id = ProviderId::new(bad);
        });
    }
    refused("listed twice under the id", |l| {
        l.servers[1].id = l.servers[0].id.clone();
    });
    refused("has no name", |l| l.servers[0].name = "  ".into());
    refused("two servers are called “my mac”", |l| {
        l.servers[1].name = "my mac".into();
    });
    for (address, why) in [
        ("localhost:11434", "must start with http:// or https://"),
        ("ftp://localhost", "must start with http:// or https://"),
        ("http://", "not an address Kalamu can read"),
        ("http://:11434", "not an address Kalamu can read"),
        (
            "http://me:secret@server.lan",
            "Leave the user name and password out",
        ),
        ("http://server lan", "not an address Kalamu can read"),
        ("http://server.lan\\x", "not an address Kalamu can read"),
        (" http://server.lan", "not an address Kalamu can read"),
        ("http://server.lan/?x=1", "Leave out what follows ? or #"),
    ] {
        refused(why, |l| l.servers[0].address = address.into());
    }
    refused("from 1 to 8", |l| l.servers[0].at_once = 0);
    refused("from 1 to 8", |l| l.servers[0].at_once = 9);
    refused("model “pages-only” is listed twice", |l| {
        l.servers[0].models[1].id = "pages-only".into();
    });
    refused("a model without a name", |l| {
        l.servers[0].models[0].id = String::new();
    });
    refused("the longest answer is 0 tokens", |l| {
        l.servers[0].models[0].max_output = 0;
    });
    refused("must read more tokens than its longest answer", |l| {
        l.servers[0].models[0].context = 8_192;
    });
    refused("a price is negative", |l| {
        l.servers[1].models[0].input_per_mtok = Some(-1.0);
    });
    refused("not YYYY-MM-DD", |l| {
        l.servers[0].models[0].tried_at = Some("26/11/2026".into());
    });
    // HTTPS and HTTP in capitals are addresses too.
    let mut list = servers();
    list.servers[0].address = "HTTP://192.168.1.20:11434".into();
    assert!(list.problems().is_empty(), "{:?}", list.problems());
}

#[test]
fn a_new_id_has_the_form_and_is_not_taken() {
    let list = servers();
    for _ in 0..100 {
        let id = list.new_id();
        assert!(is_own_id(id.as_str()), "{id}");
        assert!(list.server(&id).is_none());
    }
}

#[test]
fn each_server_becomes_a_provider_that_says_where_it_is() {
    let bundled = Catalog::bundled();
    let merged = bundled.with_servers(&servers());
    assert_eq!(merged.providers.len(), bundled.providers.len() + 2);

    let mac = merged.provider(&ProviderId::new("own-0000abcd"));
    let mac = mac.unwrap_or_else(|| panic!("no provider"));
    assert_eq!(mac.name, "My Mac");
    assert_eq!(mac.key_url, "");
    assert!(!mac.local);
    assert_eq!(
        mac.own,
        Some(OwnInfo {
            address: "http://localhost:11434".into(),
            host: "localhost".into(),
            this_computer: true,
            key: false,
            dialect: Dialect::Ollama,
        })
    );
    let uni = merged.provider(&ProviderId::new("own-1234beef"));
    let uni = uni.and_then(|p| p.own.clone());
    let uni = uni.unwrap_or_else(|| panic!("no provider"));
    assert_eq!(uni.host, "llm.uni.example");
    assert!(!uni.this_computer);
    assert!(uni.key);
    assert_eq!(uni.dialect, Dialect::OpenAi);
    // The bundled providers say nothing of servers.
    assert!(bundled.providers.iter().all(|p| p.own.is_none()));
}

#[test]
fn a_model_is_offered_only_for_what_it_did_in_its_trial() {
    let merged = Catalog::bundled().with_servers(&servers());
    let pages = find(&merged, "own-0000abcd", "pages-only");
    assert_eq!(pages.tasks, [Task::TextExtraction]);
    assert_eq!(pages.inputs, ["image/png", "image/jpeg"]);
    assert!(pages.does(Task::TextExtraction));
    for task in [
        Task::Analysis,
        Task::Contents,
        Task::TextCorrection,
        Task::Translation,
        Task::AudioTranscription,
    ] {
        assert!(!pages.does(task), "{task:?}");
    }

    let forms = find(&merged, "own-0000abcd", "forms-only");
    assert_eq!(forms.tasks, [Task::Analysis]);
    assert_eq!(forms.inputs, ["text/plain"]);
    for task in [Task::Analysis, Task::TextCorrection, Task::Translation] {
        assert!(forms.does(task), "{task:?}");
    }
    // A table of contents reads pages; a model that only fills forms does not.
    for task in [
        Task::TextExtraction,
        Task::Contents,
        Task::AudioTranscription,
    ] {
        assert!(!forms.does(task), "{task:?}");
    }

    let both = find(&merged, "own-1234beef", "qwen3-vl:8b");
    assert_eq!(both.tasks, [Task::TextExtraction, Task::Analysis]);
    assert_eq!(both.inputs, ["text/plain", "image/png", "image/jpeg"]);
    for task in Task::ALL {
        assert_eq!(
            both.does(task),
            task != Task::AudioTranscription,
            "{task:?}"
        );
    }

    // A model that did neither is not listed.
    assert!(
        merged
            .model(&ProviderId::new("own-0000abcd"), "neither")
            .is_none()
    );
    let listed = |task| {
        merged
            .models_for(task)
            .into_iter()
            .filter(|m| m.provider.as_str().starts_with(ID_PREFIX))
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(listed(Task::TextExtraction), ["pages-only", "qwen3-vl:8b"]);
    assert_eq!(listed(Task::Analysis), ["forms-only", "qwen3-vl:8b"]);
    assert_eq!(listed(Task::Contents), ["qwen3-vl:8b"]);
    assert!(listed(Task::AudioTranscription).is_empty());
    // The pages' readings of a table of contents are combined by a model that fills forms.
    let merges: Vec<&str> = merged
        .merge_models()
        .into_iter()
        .filter(|m| m.provider.as_str().starts_with(ID_PREFIX))
        .map(|m| m.id.as_str())
        .collect();
    assert_eq!(merges, ["forms-only", "qwen3-vl:8b"]);
}

#[test]
fn a_servers_model_claims_nothing_the_module_does_not_do() {
    let merged = Catalog::bundled().with_servers(&servers());
    let m = find(&merged, "own-1234beef", "qwen3-vl:8b");
    assert_eq!(m.name, "qwen3-vl:8b");
    assert_eq!(m.description, "");
    assert_eq!(m.summary, None);
    assert_eq!(m.languages, LanguageSupport::Any("any".into()));
    assert_eq!((m.status, m.tier), (ModelStatus::Stable, Tier::Balanced));
    assert!(m.recommended_for.is_empty() && m.provider_default_for.is_empty());
    assert!(m.accepts_prompt && m.takes_language && m.detects_language);
    assert!(!m.speakers && !m.layout && !m.vocabulary && !m.translates);
    assert!(!m.costly && !m.refusal_fallback && m.is_offered());
    assert_eq!(m.reasoning, None);
    assert_eq!(m.pricing.batch_discount, None);
    assert_eq!(m.limits.max_pages, Some(1));
    assert_eq!(m.limits.max_output_tokens, Some(8_192));
    assert_eq!(m.limits.max_input_tokens, Some(32_768 - 8_192));
    assert_eq!(m.limits.max_request_mb, None);

    // What a request is sent: the longest answer, no reasoning, an ordinary prompt.
    let tuning = m.tuning(Task::Analysis, Some("high"));
    assert_eq!(tuning.reasoning, None);
    assert_eq!(tuning.max_output_tokens, Some(8_192));
    assert!(!tuning.no_prompt && !tuning.speakers && !tuning.refusal_fallback);

    // What a text may hold is measured against what it reads.
    assert_eq!(m.most_chars(), Some((32_768 - 8_192) * 4));
    assert!(m.too_long(&"x".repeat(200_000)).is_some());
    assert!(m.too_long("a short text").is_none());
}

#[test]
fn a_model_without_a_price_is_not_priced_and_one_with_is() {
    let merged = Catalog::bundled().with_servers(&servers());
    let work = Workload {
        pages: 10,
        documents: 1,
        prompt_chars: 400,
        text_chars: 4_000,
        ..Workload::default()
    };
    let free = find(&merged, "own-0000abcd", "pages-only");
    let not_priced = CostEstimate {
        usd: None,
        economy_usd: None,
    };
    assert_eq!(
        merged.estimate(free, Task::TextExtraction, &work),
        not_priced
    );
    let forms = find(&merged, "own-0000abcd", "forms-only");
    assert_eq!(merged.estimate(forms, Task::Analysis, &work), not_priced);
    let usage = kalamu_core::Usage {
        input_tokens: 1_000,
        output_tokens: 100,
        ..Default::default()
    };
    assert_eq!(merged.actual_cost(free, &usage), None);

    let priced = find(&merged, "own-1234beef", "qwen3-vl:8b");
    for task in [Task::TextExtraction, Task::Analysis, Task::Contents] {
        let estimate = merged.estimate(priced, task, &work);
        assert!(estimate.usd.is_some_and(|usd| usd > 0.0), "{task:?}");
        assert_eq!(estimate.economy_usd, None, "no batches: {task:?}");
    }
    let cost = merged.actual_cost(priced, &usage).unwrap_or_default();
    assert!((cost - (1_000.0 * 0.5 + 100.0 * 1.5) / 1_000_000.0).abs() < 1e-12);
}

#[test]
fn merging_again_or_into_a_refreshed_list_gives_the_same_catalogue() {
    let list = servers();
    let once = Catalog::bundled().with_servers(&list);
    assert_eq!(once.with_servers(&list), once);
    // A refresh is a fresh catalogue, merged again.
    assert_eq!(Catalog::bundled().with_servers(&list), once);
    // A server removed leaves with its models.
    let fewer = OwnServers {
        servers: list.servers[1..].to_vec(),
    };
    let after = once.with_servers(&fewer);
    assert!(after.provider(&ProviderId::new("own-0000abcd")).is_none());
    assert!(
        after
            .models
            .iter()
            .all(|m| m.provider.as_str() != "own-0000abcd")
    );
    assert_eq!(
        Catalog::bundled().with_servers(&OwnServers::default()),
        Catalog::bundled()
    );
    // The model's own lookups still find every bundled model.
    let bundled = Catalog::bundled();
    for m in &bundled.models {
        assert_eq!(once.model(&m.provider, &m.id), Some(m));
    }
}

#[test]
fn a_server_that_could_pass_for_another_provider_is_left_out() {
    let mut list = servers();
    list.servers[0].id = ProviderId::new("gemini");
    list.servers.push(server(
        "own-1234beef",
        "Second under the same id",
        "http://10.0.0.2",
        vec![model("other", true, true)],
    ));
    list.servers[1]
        .models
        .push(model("qwen3-vl:8b", false, true));
    let merged = Catalog::bundled().with_servers(&list);
    let gemini = merged.provider(&ProviderId::new("gemini"));
    assert!(gemini.is_some_and(|p| p.own.is_none()));
    let named: Vec<&str> = merged
        .providers
        .iter()
        .filter(|p| p.own.is_some())
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(named, ["University"]);
    let models: Vec<(&str, &[Task])> = merged
        .models
        .iter()
        .filter(|m| m.provider.as_str() == "own-1234beef")
        .map(|m| (m.id.as_str(), m.tasks.as_slice()))
        .collect();
    assert_eq!(
        models,
        [(
            "qwen3-vl:8b",
            [Task::TextExtraction, Task::Analysis].as_slice()
        )]
    );
}
