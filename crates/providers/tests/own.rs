//! A server of the researcher's own against a local mock server: Ollama at its own address and
//! an OpenAI-style server, their discovery, and the trial of a model.
//!
//! The answers in `fixtures/own/` are written from the servers' documentation (Ollama's
//! docs.ollama.com/api/chat, /api/tags and /api/show; OpenAI's Chat Completions and models list;
//! vLLM's and llama.cpp's additions to the list), not captured. Replace them with recorded
//! answers once a real server has been tried.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kalamu_core::{
    AnalysisSchema, ApiKey, CutReason, Field, FieldKind, FieldValue, LanguageChoice, PieceRef,
    PreparedInput, ProcessRequest, Provider, ProviderError, ProviderId, Reach, ServerRecord,
    SetupOptions, StopSignal, Task, Tuning,
};
use kalamu_providers::client;
use kalamu_providers::own::{
    AddressError, Dialect, DiscoverError, ModelConfig, OwnServer, ServerConfig, checked_address,
    discover, host_of, on_this_computer, providers, try_model,
};
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/own/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

const MODEL: &str = "gemma3:12b";

fn config(address: &str, dialect: Dialect) -> ServerConfig {
    ServerConfig {
        id: ProviderId::new("own-1a2b3c4d"),
        name: "Office Mac".into(),
        address: address.into(),
        dialect,
        key: false,
        at_once: 1,
        models: vec![ModelConfig {
            id: MODEL.into(),
            context: 16_384,
        }],
    }
}

fn server(address: &str, dialect: Dialect) -> OwnServer {
    OwnServer::new(client(), config(address, dialect))
}

fn page() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::page(1),
        mime: "image/png".into(),
        bytes: b"\x89PNG fake".to_vec(),
        file_name: "lettre-p1.png".into(),
    }
}

fn document() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::Whole,
        mime: "text/plain".into(),
        bytes: "Réunion à la mosquée de Bamako, 3 mai 1957."
            .as_bytes()
            .to_vec(),
        file_name: "rapport.txt".into(),
    }
}

static FIELDS: std::sync::LazyLock<AnalysisSchema> = std::sync::LazyLock::new(|| {
    let mut schema = AnalysisSchema {
        fields: vec![
            Field {
                key: String::new(),
                label: "Summary".into(),
                instructions: String::new(),
                kind: FieldKind::Text,
                reason: false,
            },
            Field {
                key: String::new(),
                label: "Keywords".into(),
                instructions: String::new(),
                kind: FieldKind::List { max: None },
                reason: false,
            },
        ],
    };
    schema.assign_keys();
    schema
});

static TUNING: std::sync::LazyLock<Tuning> = std::sync::LazyLock::new(Tuning::default);
static OPTIONS: std::sync::LazyLock<SetupOptions> = std::sync::LazyLock::new(SetupOptions::default);
static NEVER: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);

fn request<'a>(task: Task, input: &'a PreparedInput, stop: &'a StopSignal) -> ProcessRequest<'a> {
    ProcessRequest {
        task,
        model: MODEL,
        prompt: "Transcribe diplomatically.",
        user_prompt: "Transcribe this page.",
        input,
        languages: &LanguageChoice::Auto,
        options: &OPTIONS,
        tuning: &TUNING,
        analysis: (task == Task::Analysis).then(|| &*FIELDS),
        stop,
    }
}

fn no_key() -> ApiKey {
    ApiKey::new("")
}

async fn bodies(server: &MockServer) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| r.body_json::<Value>().unwrap_or_default())
        .collect()
}

fn ok(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

// Ollama ------------------------------------------------------------------------------------

#[tokio::test]
async fn ollama_reads_a_page_asking_for_no_cut_and_no_thinking() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({
            "model": MODEL,
            "stream": false,
            "think": false,
            "truncate": false,
            "shift": false,
            "options": { "num_ctx": 16384, "num_predict": 4096 },
        })))
        .respond_with(ok(fixture("ollama_page.json")))
        .expect(1)
        .mount(&mock)
        .await;
    let ollama = server(&mock.uri(), Dialect::Ollama);
    let input = page();
    let out = ollama
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(out.text, "Bamako, le 3 mai 1957\n\nMonsieur le Gouverneur,");
    assert!(!out.text.contains("transcribe it"), "thinking kept out");
    assert!(!out.truncated);
    assert_eq!((out.usage.input_tokens, out.usage.output_tokens), (812, 21));
    assert_eq!(out.model_version, None);

    let body = &bodies(&mock).await[0];
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(body["messages"][1]["content"], "Transcribe this page.");
    assert_eq!(body["messages"][1]["images"][0], "iVBORyBmYWtl");
    assert!(body.get("format").is_none(), "no schema for a page");
    // No key saved: no Authorization header.
    let received = mock.received_requests().await.unwrap_or_default();
    assert!(received[0].headers.get("authorization").is_none());

    // What the record says is what was sent.
    let settings = ollama.sent_settings(&request(Task::TextExtraction, &input, &NEVER), false);
    for setting in settings {
        let at = format!("/{}", setting.name.replace('.', "/"));
        let sent = body.pointer(&at).unwrap_or_else(|| panic!("{at} not sent"));
        assert_eq!(sent.to_string(), setting.value, "{at}");
    }
}

#[tokio::test]
async fn ollama_fills_an_analysis_through_its_schema() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({ "format": { "type": "object" } })))
        .respond_with(ok(fixture("ollama_analysis.json")))
        .expect(1)
        .mount(&mock)
        .await;
    let input = document();
    let out = server(&mock.uri(), Dialect::Ollama)
        .process(&no_key(), request(Task::Analysis, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(out.fields.len(), 2);
    assert_eq!(
        out.fields[0].value,
        FieldValue::Text("Réunion de 200 fidèles à Bamako.".into())
    );
    let body = &bodies(&mock).await[0];
    let user = body["messages"][1]["content"].as_str().unwrap_or_default();
    assert!(
        user.starts_with("<document name=\"rapport.txt\">"),
        "{user}"
    );
    assert!(user.ends_with("Transcribe this page."), "{user}");
    assert!(body["messages"][1].get("images").is_none());
}

#[tokio::test]
async fn ollama_marks_an_answer_cut_at_its_length() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ok(fixture("ollama_length.json")))
        .mount(&mock)
        .await;
    let input = page();
    let out = server(&mock.uri(), Dialect::Ollama)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(out.truncated);
    assert_eq!(out.cut_reason, CutReason::Limit);
}

#[tokio::test]
async fn ollama_refusing_a_text_too_long_is_said_plainly_and_not_retried() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(400).set_body_json(fixture("ollama_overflow.json")))
        .mount(&mock)
        .await;
    let input = document();
    let err = server(&mock.uri(), Dialect::Ollama)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .err()
        .unwrap_or_else(|| panic!("refused"));
    let ProviderError::InvalidRequest(message) = &err else {
        panic!("{err:?}");
    };
    assert!(
        message.contains("longer than gemma3:12b reads on Office Mac"),
        "{message}"
    );
    assert!(message.contains("Reads up to"), "{message}");
    assert!(!err.is_retryable() && !err.is_fatal_for_job());
}

#[tokio::test]
async fn a_model_gone_from_the_server_and_a_server_that_does_not_answer_are_named() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"error": "model 'gemma3:12b' not found"})),
        )
        .mount(&mock)
        .await;
    let input = page();
    let err = server(&mock.uri(), Dialect::Ollama)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .err();
    assert!(
        matches!(&err, Some(ProviderError::InvalidRequest(m)) if m.contains("no longer on Office Mac")),
        "{err:?}"
    );

    let closed = closed_address();
    let err = server(&closed, Dialect::Ollama)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .err();
    let Some(ProviderError::Transient(message)) = &err else {
        panic!("{err:?}");
    };
    assert!(
        message.starts_with("Office Mac did not answer at 127.0.0.1:"),
        "{message}"
    );
    assert!(!message.contains("internet"), "{message}");
}

#[tokio::test]
async fn a_saved_key_is_sent_to_a_server_behind_a_proxy() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(header("authorization", "Bearer proxy-key"))
        .respond_with(ok(fixture("ollama_page.json")))
        .expect(1)
        .mount(&mock)
        .await;
    let ollama = OwnServer::new(
        client(),
        ServerConfig {
            key: true,
            ..config(&mock.uri(), Dialect::Ollama)
        },
    );
    assert!(ollama.needs_key());
    let input = page();
    ollama
        .process(
            &ApiKey::new("proxy-key"),
            request(Task::TextExtraction, &input, &NEVER),
        )
        .await
        .unwrap_or_else(|e| panic!("{e}"));
}

// OpenAI-style ------------------------------------------------------------------------------

#[tokio::test]
async fn an_openai_style_server_reads_a_page_with_only_portable_fields() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(
            json!({ "model": MODEL, "max_tokens": 4096, "stream": false }),
        ))
        .respond_with(ok(fixture("chat_page.json")))
        .expect(1)
        .mount(&mock)
        .await;
    let chat = server(&format!("{}/v1", mock.uri()), Dialect::OpenAi);
    let input = page();
    let out = chat
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(out.text, "Bamako, le 3 mai 1957\n\nMonsieur le Gouverneur,");
    assert_eq!(
        (out.usage.input_tokens, out.usage.output_tokens),
        (1210, 21)
    );

    let body = &bodies(&mock).await[0];
    let image = &body["messages"][1]["content"][0];
    assert_eq!(image["type"], "image_url");
    assert_eq!(
        image["image_url"]["url"],
        "data:image/png;base64,iVBORyBmYWtl"
    );
    for absent in [
        "max_completion_tokens",
        "reasoning_effort",
        "response_format",
        "plugins",
    ] {
        assert!(body.get(absent).is_none(), "{absent} sent");
    }
    let settings = chat.sent_settings(&request(Task::TextExtraction, &input, &NEVER), false);
    assert_eq!(settings.len(), 1);
    assert_eq!(settings[0].name, "max_tokens");
    assert_eq!(body["max_tokens"].to_string(), settings[0].value);
}

#[tokio::test]
async fn an_openai_style_server_fills_an_analysis_through_a_strict_schema() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({
            "response_format": { "type": "json_schema", "json_schema": { "strict": true } },
        })))
        .respond_with(ok(fixture("chat_analysis.json")))
        .expect(1)
        .mount(&mock)
        .await;
    let input = document();
    let out = server(&mock.uri(), Dialect::OpenAi)
        .process(&no_key(), request(Task::Analysis, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        out.fields[1].value,
        FieldValue::List(vec!["Bamako".into(), "mosquée".into()])
    );
}

#[tokio::test]
async fn an_openai_style_answer_cut_at_its_length_is_marked() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ok(fixture("chat_length.json")))
        .mount(&mock)
        .await;
    let input = page();
    let out = server(&mock.uri(), Dialect::OpenAi)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(out.truncated);
    assert_eq!(out.cut_reason, CutReason::Limit);
}

#[tokio::test]
async fn an_openai_style_server_refusing_a_text_too_long_is_said_plainly() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400).set_body_json(fixture("vllm_overflow.json")))
        .mount(&mock)
        .await;
    let input = document();
    let err = server(&mock.uri(), Dialect::OpenAi)
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .err();
    assert!(
        matches!(&err, Some(ProviderError::InvalidRequest(m)) if m.contains("reads on Office Mac")),
        "{err:?}"
    );
}

#[tokio::test]
async fn a_text_read_in_part_is_refused_and_one_without_usage_is_taken_on_its_word() {
    let mock = MockServer::start().await;
    let input = PreparedInput {
        bytes: "Bamako. ".repeat(500).into_bytes(),
        ..document()
    };
    // 4,000 characters and more, read as 100 tokens.
    let mut part = fixture("chat_page.json");
    part["usage"]["prompt_tokens"] = json!(100);
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ok(part))
        .up_to_n_times(1)
        .mount(&mock)
        .await;
    let chat = server(&mock.uri(), Dialect::OpenAi);
    let err = chat
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .err();
    let Some(ProviderError::InvalidRequest(message)) = &err else {
        panic!("{err:?}");
    };
    assert!(
        message.contains("Office Mac read only part of the text"),
        "{message}"
    );

    let mut silent = fixture("chat_page.json");
    silent
        .as_object_mut()
        .map(|o| o.remove("usage"))
        .unwrap_or_default();
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ok(silent))
        .mount(&mock)
        .await;
    chat.process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
}

#[tokio::test]
async fn openrouter_alone_is_told_not_to_compress() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/chat/completions"))
        .respond_with(ok(fixture("chat_page.json")))
        .mount(&mock)
        .await;
    let addr: SocketAddr = mock.address().to_owned();
    // openrouter.ai, reached at the mock server.
    let resolving = reqwest::Client::builder()
        .no_proxy()
        .resolve("openrouter.ai", addr)
        .build()
        .unwrap_or_else(|e| panic!("{e}"));
    let input = page();
    let openrouter = OwnServer::new(
        resolving,
        config(
            &format!("http://openrouter.ai:{}/api/v1", addr.port()),
            Dialect::OpenAi,
        ),
    );
    openrouter
        .process(&no_key(), request(Task::TextExtraction, &input, &NEVER))
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    let body = &bodies(&mock).await[0];
    assert_eq!(
        body["plugins"],
        json!([{ "id": "context-compression", "enabled": false }])
    );
    assert_eq!(openrouter.reach(), Reach::Network);
}

// Both --------------------------------------------------------------------------------------

/// Answers after a delay, noting when each request reached the server: requests sent one at a
/// time reach it at least the delay apart, however slow the machine running the test.
struct Noted {
    answer: ResponseTemplate,
    received: Arc<Mutex<Vec<Instant>>>,
}

impl Respond for Noted {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        self.received
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Instant::now());
        self.answer.clone()
    }
}

#[tokio::test]
async fn requests_wait_for_the_server_s_turn() {
    let mock = MockServer::start().await;
    let delay = Duration::from_millis(400);
    let received = Arc::new(Mutex::new(Vec::new()));
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(Noted {
            answer: ok(fixture("ollama_page.json")).set_delay(delay),
            received: received.clone(),
        })
        .mount(&mock)
        .await;
    let ollama = server(&mock.uri(), Dialect::Ollama);
    let input = page();
    let key = no_key();
    let (a, b) = tokio::join!(
        ollama.process(&key, request(Task::TextExtraction, &input, &NEVER)),
        ollama.process(&key, request(Task::TextExtraction, &input, &NEVER)),
    );
    assert!(a.is_ok() && b.is_ok());
    let times = received.lock().unwrap_or_else(|p| p.into_inner()).clone();
    assert_eq!(times.len(), 2);
    // The second was sent only once the first was answered, never beside it.
    assert!(
        times[1].duration_since(times[0]) >= delay,
        "one at a time: {:?} apart",
        times[1].duration_since(times[0])
    );

    // A request waiting for its turn stops when the job does, and never reaches the server.
    let stop = StopSignal::default();
    stop.stop();
    let (first, waiting) = tokio::join!(
        ollama.process(&key, request(Task::TextExtraction, &input, &NEVER)),
        ollama.process(&key, request(Task::TextExtraction, &input, &stop)),
    );
    assert!(first.is_ok());
    assert_eq!(waiting.err(), Some(ProviderError::Interrupted));
    assert_eq!(received.lock().unwrap_or_else(|p| p.into_inner()).len(), 3);
}

#[tokio::test]
async fn a_server_says_where_it_is_and_what_it_is_not_used_for() {
    let local = server("http://localhost:11434", Dialect::Ollama);
    assert_eq!(local.id(), ProviderId::new("own-1a2b3c4d"));
    assert!(!local.needs_key());
    assert_eq!(local.reach(), Reach::ThisComputer);
    assert_eq!(
        local.server(),
        Some(ServerRecord {
            name: "Office Mac".into(),
            address: "http://localhost:11434".into(),
        })
    );
    assert_eq!(
        local.accepted_inputs(Task::TextExtraction),
        ["text/plain", "image/png", "image/jpeg"]
    );
    assert!(local.accepted_inputs(Task::AudioTranscription).is_empty());
    assert_eq!(
        server("http://192.168.1.20:11434", Dialect::Ollama).reach(),
        Reach::Network
    );
    let audio = PreparedInput {
        mime: "audio/mpeg".into(),
        ..page()
    };
    assert!(matches!(
        local
            .process(&no_key(), request(Task::AudioTranscription, &audio, &NEVER))
            .await,
        Err(ProviderError::Unsupported(_))
    ));
    // The longest answer the engine asks for wins over the one Kalamu would fill.
    let asked = Tuning {
        max_output_tokens: Some(1_000),
        ..Tuning::default()
    };
    let settings = local.request_settings(Task::Analysis, MODEL, &asked);
    assert!(
        settings
            .iter()
            .any(|s| s.name == "options.num_predict" && s.value == "1000")
    );
    // A model the configuration lacks reads 8,192 tokens.
    let settings = local.request_settings(Task::Analysis, "other", &Tuning::default());
    assert!(
        settings
            .iter()
            .any(|s| s.name == "options.num_ctx" && s.value == "8192")
    );
    assert_eq!(
        providers(client(), &[config("http://localhost:1", Dialect::OpenAi)]).len(),
        1
    );
}

#[tokio::test]
async fn each_server_lists_its_models() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ok(fixture("ollama_tags.json")))
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ok(fixture("vllm_models.json")))
        .mount(&mock)
        .await;
    let ollama = server(&mock.uri(), Dialect::Ollama);
    assert_eq!(
        ollama.list_models(&no_key()).await,
        Ok(vec!["gemma3:12b".into(), "gpt-oss:120b-cloud".into()])
    );
    let chat = server(&format!("{}/v1", mock.uri()), Dialect::OpenAi);
    assert_eq!(
        chat.list_models(&no_key()).await,
        Ok(vec!["Qwen/Qwen3-VL-8B-Instruct".into()])
    );
}

// Addresses ---------------------------------------------------------------------------------

#[test]
fn addresses_are_checked_in_one_place() {
    assert_eq!(
        checked_address("  http://localhost:11434/ "),
        Ok("http://localhost:11434".into())
    );
    assert_eq!(
        checked_address("https://gpu.example.edu/llm/v1/"),
        Ok("https://gpu.example.edu/llm/v1".into())
    );
    assert_eq!(checked_address(""), Err(AddressError::Empty));
    assert_eq!(checked_address("ftp://host"), Err(AddressError::Scheme));
    assert_eq!(
        checked_address("localhost:11434"),
        Err(AddressError::Scheme)
    );
    assert_eq!(
        checked_address("http://me:secret@host:11434"),
        Err(AddressError::Credentials)
    );
    assert_eq!(
        checked_address("http://me@host"),
        Err(AddressError::Credentials)
    );
    assert_eq!(
        checked_address("http://host/?key=1"),
        Err(AddressError::Query)
    );
    assert_eq!(checked_address("http://host/#x"), Err(AddressError::Query));
    assert_eq!(
        checked_address("not an address"),
        Err(AddressError::NotAnAddress)
    );
}

#[test]
fn this_computer_is_known_by_its_address_alone() {
    for here in [
        "http://localhost:11434",
        "http://LOCALHOST:11434",
        "http://localhost.:11434",
        "http://127.0.0.1:11434",
        "http://127.0.0.2:8080",
        "http://[::1]:11434",
    ] {
        assert!(on_this_computer(here), "{here}");
    }
    for away in [
        "http://192.168.1.20:11434",
        "https://gpu.example.edu/v1",
        "http://localhost.example.com",
        "http://128.0.0.1",
        "nonsense",
    ] {
        assert!(!on_this_computer(away), "{away}");
    }
    assert_eq!(
        host_of("http://192.168.1.20:11434/v1"),
        "192.168.1.20:11434"
    );
    assert_eq!(host_of("http://[::1]:11434"), "[::1]:11434");
    assert_eq!(host_of("https://gpu.example.edu/v1"), "gpu.example.edu");
}

// Discovery ---------------------------------------------------------------------------------

#[tokio::test]
async fn ollama_is_found_with_what_each_model_can_do() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ok(fixture("ollama_version.json")))
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ok(fixture("ollama_tags.json")))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_partial_json(json!({ "model": "gemma3:12b" })))
        .respond_with(ok(fixture("ollama_show.json")))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({"error": "remote"})))
        .mount(&mock)
        .await;
    let found = discover(&client(), &format!("{}/", mock.uri()), None)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found.dialect, Dialect::Ollama);
    assert_eq!(found.address, mock.uri());
    assert_eq!(found.version.as_deref(), Some("0.40.18"));
    let [gemma, cloud] = found.models.as_slice() else {
        panic!("{:?}", found.models);
    };
    assert_eq!(gemma.id, "gemma3:12b");
    assert_eq!(gemma.vision, Some(true));
    assert_eq!(gemma.audio, Some(false));
    assert_eq!(gemma.context, Some(131_072));
    assert_eq!(gemma.size_bytes, Some(8_149_190_253));
    assert!(
        gemma
            .digest
            .as_deref()
            .is_some_and(|d| d.starts_with("f4031aab"))
    );
    assert_eq!(gemma.elsewhere, None);
    assert_eq!(cloud.elsewhere.as_deref(), Some("https://ollama.com:443"));
    assert_eq!(cloud.vision, None, "the server did not say");
}

#[tokio::test]
async fn an_openai_style_server_is_found_at_its_address_or_under_v1() {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ok(fixture("vllm_models.json")))
        .mount(&mock)
        .await;
    let found = discover(&client(), &mock.uri(), None)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found.dialect, Dialect::OpenAi);
    assert_eq!(found.address, mock.uri());
    assert_eq!(found.version, None);
    assert_eq!(found.models[0].id, "Qwen/Qwen3-VL-8B-Instruct");
    assert_eq!(found.models[0].context, Some(32_768));
    assert_eq!(found.models[0].vision, None);

    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ok(fixture("other_models.json")))
        .mount(&mock)
        .await;
    let found = discover(&client(), &mock.uri(), None)
        .await
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(found.address, format!("{}/v1", mock.uri()));
    let contexts: Vec<Option<u32>> = found.models.iter().map(|m| m.context).collect();
    assert_eq!(contexts, [Some(131_072), Some(131_072), Some(163_840)]);
    let vision: Vec<Option<bool>> = found.models.iter().map(|m| m.vision).collect();
    assert_eq!(vision, [None, Some(true), Some(false)]);
}

#[tokio::test]
async fn each_failure_to_find_a_server_is_told_apart() {
    let closed = closed_address();
    assert!(matches!(
        discover(&client(), &closed, None).await,
        Err(DiscoverError::NothingAnswers { host }) if closed.ends_with(&host)
    ));
    let words = discover(&client(), &closed, None)
        .await
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(!words.contains("internet"), "{words}");

    let nothing = MockServer::start().await;
    assert!(matches!(
        discover(&client(), &nothing.uri(), None).await,
        Err(DiscoverError::NotAServer { .. })
    ));

    let locked = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({"error": "unauthorized"})))
        .mount(&locked)
        .await;
    assert!(matches!(
        discover(&client(), &locked.uri(), None).await,
        Err(DiscoverError::KeyNeeded { .. })
    ));
    assert!(matches!(
        discover(&client(), &locked.uri(), Some(&ApiKey::new("wrong"))).await,
        Err(DiscoverError::KeyRefused { .. })
    ));
    let refused = discover(&client(), "http://me:pw@localhost:11434", None).await;
    assert!(matches!(
        refused,
        Err(DiscoverError::Address(AddressError::Credentials))
    ));

    // Each failure has its own id for the interface, and names the host where there is one.
    let ids = [
        (
            DiscoverError::NothingAnswers { host: "h:1".into() },
            "server.nothing_answers",
        ),
        (
            DiscoverError::NotAServer { host: "h:1".into() },
            "server.not_a_server",
        ),
        (
            DiscoverError::KeyNeeded { host: "h:1".into() },
            "server.key_needed",
        ),
        (
            DiscoverError::KeyRefused { host: "h:1".into() },
            "server.key_refused",
        ),
        (
            DiscoverError::Other {
                host: "h:1".into(),
                message: "x".into(),
            },
            "server.unreadable",
        ),
    ];
    for (error, id) in ids {
        assert_eq!(error.code(), id);
        assert_eq!(error.host(), Some("h:1"));
    }
    let refused = refused.err().unwrap_or_else(|| panic!("refused"));
    assert_eq!(refused.code(), "server.address_credentials");
    assert_eq!(refused.host(), None);
}

/// An address where nothing listens: a port the system gave, then closed.
fn closed_address() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| panic!("{e}"));
    let port = listener.local_addr().map(|a| a.port()).unwrap_or_default();
    drop(listener);
    format!("http://127.0.0.1:{port}")
}

// The trial ---------------------------------------------------------------------------------

fn sends_picture(request: &Request) -> bool {
    String::from_utf8_lossy(&request.body).contains("\"images\"")
}

fn sends_form(request: &Request) -> bool {
    String::from_utf8_lossy(&request.body).contains("\"format\"")
}

fn says(content: &str) -> ResponseTemplate {
    ok(json!({
        "model": MODEL,
        "message": { "role": "assistant", "content": content },
        "done": true, "done_reason": "stop",
        "prompt_eval_count": 40, "eval_count": 5,
    }))
}

async fn trial_server(picture: &str, form: &str) -> MockServer {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(sends_picture)
        .respond_with(says(picture))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(sends_form)
        .respond_with(says(form))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(|r: &Request| !sends_picture(r) && !sends_form(r))
        .respond_with(says("The name is Kalamu."))
        .mount(&mock)
        .await;
    mock
}

fn trial_model() -> ModelConfig {
    ModelConfig {
        id: MODEL.into(),
        context: 8_192,
    }
}

#[tokio::test]
async fn a_model_that_reads_the_picture_and_fills_the_form_does_both() {
    let mock = trial_server("4729\nKALAMU", r#"{"city":"Bamako","year":"1957"}"#).await;
    let result = try_model(
        &client(),
        &config(&mock.uri(), Dialect::Ollama),
        &trial_model(),
        Some(true),
        None,
    )
    .await;
    assert_eq!(result.answers, Ok(()));
    assert!(result.reads_pages && result.fills_forms, "{result:?}");
    assert_eq!(result.note, None);
    let sent = bodies(&mock).await;
    assert_eq!(sent.len(), 3);
    // The bundled picture, as the server received it.
    let picture = sent
        .iter()
        .find_map(|b| b["messages"][1]["images"][0].as_str())
        .unwrap_or_default();
    assert!(picture.starts_with("iVBORw0KGgo"), "a PNG");
}

#[tokio::test]
async fn a_model_that_fails_both_is_said_to_and_a_blind_one_is_not_shown_the_picture() {
    let mock = trial_server("I see a number.", r#"{"city":"Dakar","year":""}"#).await;
    let result = try_model(
        &client(),
        &config(&mock.uri(), Dialect::Ollama),
        &trial_model(),
        Some(false),
        None,
    )
    .await;
    assert_eq!(result.answers, Ok(()));
    assert!(!result.reads_pages && !result.fills_forms);
    let note = result.note.unwrap_or_default();
    assert!(note.contains("reads no pictures"), "{note}");
    assert!(note.contains("did not fill the form"), "{note}");
    assert!(
        !bodies(&mock)
            .await
            .iter()
            .any(|b| b.to_string().contains("\"images\""))
    );

    let result = try_model(
        &client(),
        &config(&mock.uri(), Dialect::Ollama),
        &trial_model(),
        None,
        None,
    )
    .await;
    assert!(!result.reads_pages);
    assert!(
        result
            .note
            .unwrap_or_default()
            .contains("did not read the number"),
    );
}

#[tokio::test]
async fn a_model_that_does_not_answer_stops_the_trial_with_the_server_s_words() {
    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "model requires more system memory (12.4 GiB) than is available (7.1 GiB)"
        })))
        .expect(1)
        .mount(&mock)
        .await;
    let result = try_model(
        &client(),
        &config(&mock.uri(), Dialect::Ollama),
        &trial_model(),
        Some(true),
        None,
    )
    .await;
    let Err(why) = &result.answers else {
        panic!("{result:?}");
    };
    assert!(why.words.contains("more system memory"), "{why:?}");
    assert_eq!(why.code, "provider.invalid_request");
    assert!(!result.reads_pages && !result.fills_forms);
}

/// The shell runs a discovery and a trial each on a task of its own, which a trial's
/// cancellation aborts: both must be sendable to another thread.
#[test]
fn a_discovery_and_a_trial_can_run_on_a_task_of_their_own() {
    fn sendable<T: Send>(_: &T) {}
    let client = client();
    let server = config("http://localhost:11434", Dialect::Ollama);
    let model = trial_model();
    let key = no_key();
    sendable(&try_model(&client, &server, &model, None, Some(&key)));
    sendable(&discover(&client, "http://localhost:11434", Some(&key)));
}
