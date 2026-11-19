//! An analysis whose fields cite the text, through every provider with structured output: the
//! transcript sent with its turns numbered, each cited field asked for as entries with their
//! own fields and their quotations (a turn's number and the words), required property by
//! property, and the answer read back with each quotation as the model gave it, for the engine
//! to look for in the text. The answer in `fixtures/cited/` stands for an interview in French
//! with two speakers.

use kalamu_core::analysis::cited::{Claim, numbered};
use kalamu_core::snapshot::{PartText, TimedPart, build_with_turns};
use kalamu_core::{
    AnalysisSchema, ApiKey, Field, FieldKind, LanguageChoice, PieceRef, PreparedInput,
    ProcessRequest, Provider, Segment, SetupOptions, StopSignal, Task, Tuning,
};
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::{body_partial_json, body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/cited/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<serde_json::Value>(&text)
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn field(key: &str, kind: FieldKind) -> Field {
    Field {
        key: key.into(),
        label: key.replace('_', " "),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn interview() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field("summary", FieldKind::Text),
            field(
                "themes",
                FieldKind::Cited {
                    fields: vec![
                        field("name", FieldKind::Text),
                        field("description", FieldKind::Text),
                    ],
                    max: Some(8),
                },
            ),
            field("keywords", FieldKind::List { max: Some(12) }),
            field(
                "quotations",
                FieldKind::Cited {
                    fields: vec![field("why_it_matters", FieldKind::Text)],
                    max: None,
                },
            ),
        ],
    }
}

/// The transcript as an analysis of it sends it: one numbered turn a line.
fn transcript() -> PreparedInput {
    let piece = PieceRef::Whole;
    let segments = [
        ("Interviewer", 1_000, "Comment avez-vous commencé au port ?"),
        (
            "Awa",
            9_000,
            "J’ai commencé en 1998. Au début, on ne savait pas.",
        ),
        ("Interviewer", 40_000, "Et ensuite ?"),
        (
            "Awa",
            52_000,
            "Ensuite, «\u{a0}on ne savait pas\u{a0}» est devenu notre devise.",
        ),
    ]
    .map(|(speaker, start_ms, text)| Segment {
        start_ms,
        end_ms: start_ms + 5_000,
        speaker: Some(speaker.into()),
        text: text.into(),
    });
    let text = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let built = build_with_turns(
        Task::AudioTranscription,
        &[TimedPart {
            text: PartText {
                piece: &piece,
                text: &text,
                reviewed: true,
                corrected: false,
                failed: false,
            },
            segments: &segments,
        }],
    );
    PreparedInput {
        piece,
        mime: "text/plain".into(),
        bytes: numbered(&built.text, &built.turns).into_bytes(),
        file_name: "entretien-turns.txt".into(),
    }
}

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);
static TUNING: std::sync::LazyLock<Tuning> = std::sync::LazyLock::new(Tuning::default);

/// Mounts each provider's endpoint, answering `answer` when the request holds the numbered
/// turns and asks for the themes as entries whose fields and quotations are all required.
async fn serve(server: &MockServer, answer: &str) {
    let quotes = serde_json::json!({ "type": "array", "items": {
        "type": "object", "required": ["turn", "words"], "additionalProperties": false,
        "properties": { "turn": { "type": "integer" }, "words": { "type": "string" } } } });
    let cited = serde_json::json!({
        "type": "object",
        "required": ["summary", "themes", "keywords", "quotations"],
        "properties": { "themes": { "type": "array", "items": {
            "type": "object", "required": ["name", "description", "quotes"],
            "additionalProperties": false, "properties": { "quotes": quotes } } } },
    });
    let turn = "[4] [00:00:52] Awa: Ensuite,";
    Mock::given(method("POST"))
        .and(path("/models/gemini-3.8-flash:generateContent"))
        .and(body_string_contains(turn))
        .and(body_partial_json(serde_json::json!({
            "generationConfig": { "responseMimeType": "application/json",
                "responseJsonSchema": cited } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{ "finishReason": "STOP",
                "content": { "parts": [{ "text": answer }] } }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_string_contains(turn))
        .and(body_partial_json(serde_json::json!({
            "text": { "format": { "type": "json_schema", "strict": true, "schema": cited } },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "gpt-6-luna",
            "output": [{ "type": "message",
                "content": [{ "type": "output_text", "text": answer }] }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_string_contains(turn))
        .and(body_partial_json(serde_json::json!({
            "output_config": { "format": { "type": "json_schema", "schema": cited } },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "claude-haiku-5-5", "stop_reason": "end_turn",
            "content": [{ "type": "text", "text": answer }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_string_contains(turn))
        .and(body_partial_json(serde_json::json!({
            "response_format": { "type": "json_schema",
                "json_schema": { "strict": true, "schema": cited } },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "mistral-small-2603",
            "choices": [{ "index": 0, "finish_reason": "stop",
                "message": { "role": "assistant", "content": answer } }],
            "usage": { "prompt_tokens": 1200, "completion_tokens": 300, "total_tokens": 1500 },
        })))
        .mount(server)
        .await;
}

fn request<'a>(
    model: &'a str,
    input: &'a PreparedInput,
    schema: &'a AnalysisSchema,
    languages: &'a LanguageChoice,
    options: &'a SetupOptions,
) -> ProcessRequest<'a> {
    ProcessRequest {
        task: Task::Analysis,
        model,
        prompt: "Read the interview.",
        user_prompt: "Fill in each field.",
        input,
        languages,
        options,
        tuning: &TUNING,
        analysis: Some(schema),
        stop: &STOP,
    }
}

/// Each provider's themes and quotations, as the model gave them.
async fn read_by_every_provider(answer: &str) -> Vec<Vec<(String, Vec<Claim>)>> {
    let server = MockServer::start().await;
    serve(&server, answer).await;
    let providers: [(Box<dyn Provider>, &str); 4] = [
        (
            Box::new(Gemini::with_base_url(client(), &server.uri())),
            "gemini-3.8-flash",
        ),
        (
            Box::new(OpenAi::with_base_url(client(), &server.uri())),
            "gpt-6-luna",
        ),
        (
            Box::new(Anthropic::with_base_url(client(), &server.uri())),
            "claude-haiku-5-5",
        ),
        (
            Box::new(Mistral::with_base_url(client(), &server.uri())),
            "mistral-small-2603",
        ),
    ];
    let schema = interview();
    let (input, languages, options) = (transcript(), LanguageChoice::Auto, SetupOptions::default());
    let mut out = Vec::new();
    for (provider, model) in providers {
        assert!(
            provider
                .accepted_inputs(Task::Analysis)
                .contains(&"text/plain"),
            "{model} reads a text"
        );
        let output = provider
            .process(
                &ApiKey::new("k"),
                request(model, &input, &schema, &languages, &options),
            )
            .await
            .unwrap_or_else(|e| panic!("{model}: {e}"));
        let themes = output.fields[1].value.cited().cloned().unwrap_or_default();
        out.push(
            themes
                .entries
                .into_iter()
                .map(|e| (e.answers[0].value.joined(), e.claims))
                .collect(),
        );
    }
    out
}

#[tokio::test]
async fn every_provider_asks_for_quotations_by_turn_and_gives_them_back_as_written() {
    let read = read_by_every_provider(&fixture("interview.json")).await;
    for themes in &read {
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].0, "Les débuts");
        assert_eq!(
            themes[0].1,
            [
                Claim {
                    turn: Some(2),
                    words: "J’ai commencé en 1998.".into()
                },
                Claim {
                    turn: Some(2),
                    words: "Au début, on ne savait pas.".into()
                },
            ]
        );
        assert_eq!(themes[1].1[0].turn, Some(4));
    }
    assert!(
        read.windows(2).all(|w| w[0] == w[1]),
        "every provider alike"
    );
}

#[tokio::test]
async fn an_entry_without_its_quotations_is_refused_as_a_broken_answer() {
    let server = MockServer::start().await;
    let broken = r#"{"summary":"s","themes":[{"name":"a","description":"b"}],"keywords":[],"quotations":[]}"#;
    serve(&server, broken).await;
    let schema = interview();
    let (input, languages, options) = (transcript(), LanguageChoice::Auto, SetupOptions::default());
    let refused = Gemini::with_base_url(client(), &server.uri())
        .process(
            &ApiKey::new("k"),
            request("gemini-3.8-flash", &input, &schema, &languages, &options),
        )
        .await;
    match refused {
        Err(e) => assert!(e.is_retryable(), "{e}"),
        Ok(out) => panic!("kept: {:?}", out.fields),
    }
}
