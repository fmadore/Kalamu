//! Records (#171) through every provider with structured output: a page sent with the
//! repeating group as each provider's schema, its rows required column by column, and the
//! answer read back in reading order, every cell as the model wrote it. The answers in
//! `fixtures/records/` stand for three pages: a directory printed in two columns, a register
//! with ditto marks, and a list in a right-to-left script.

use kalamu_core::{
    AnalysisSchema, ApiKey, Field, FieldKind, LanguageChoice, PieceRef, PreparedInput,
    ProcessRequest, Provider, SetupOptions, StopSignal, Task, Tuning, records,
};
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/records/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap_or_default();
    // One line, as a model writes its structured answer.
    serde_json::from_str::<serde_json::Value>(&text)
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn column(key: &str, kind: FieldKind) -> Field {
    Field {
        key: key.into(),
        label: key.replace('_', " "),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn rows(columns: Vec<Field>) -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![Field {
            key: "entries".into(),
            label: "Entries".into(),
            instructions: "Every entry on this page, in reading order.".into(),
            kind: FieldKind::Entries { fields: columns },
            reason: false,
        }],
    }
}

fn directory() -> AnalysisSchema {
    rows(vec![
        column("name", FieldKind::Text),
        column("occupation", FieldKind::Text),
        column("address", FieldKind::Text),
    ])
}

fn register() -> AnalysisSchema {
    rows(vec![
        column("number", FieldKind::Text),
        column("date", FieldKind::Text),
        column(
            "kind_of_entry",
            FieldKind::Choice {
                values: vec![
                    "Birth or baptism".into(),
                    "Marriage".into(),
                    "Death or burial".into(),
                    "Other".into(),
                ],
            },
        ),
        column("name", FieldKind::Text),
        column("people_named", FieldKind::List { max: None }),
    ])
}

fn page() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::page(4),
        mime: "image/png".into(),
        bytes: vec![0x89, b'P', b'N', b'G', 0, 0, 0, 0],
        file_name: "annuaire-p4.png".into(),
    }
}

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);
static TUNING: std::sync::LazyLock<Tuning> = std::sync::LazyLock::new(Tuning::default);

/// Mounts each provider's endpoint, answering `answer` when the request holds the group as an
/// array of objects whose every column is required, in the columns' order.
async fn serve(server: &MockServer, schema: &AnalysisSchema, answer: &str) {
    let required: Vec<&str> = records::columns(schema)
        .iter()
        .map(|c| c.key.as_str())
        .collect();
    let group = serde_json::json!({
        "type": "object",
        "required": ["entries"],
        "properties": { "entries": { "type": "array", "items": {
            "type": "object", "required": required, "additionalProperties": false } } },
    });
    Mock::given(method("POST"))
        .and(path("/models/gemini-3.8-flash:generateContent"))
        .and(body_partial_json(serde_json::json!({
            "generationConfig": { "responseMimeType": "application/json",
                "responseJsonSchema": group } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{ "finishReason": "STOP",
                "content": { "parts": [{ "text": answer }] } }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_partial_json(serde_json::json!({
            "text": { "format": { "type": "json_schema", "strict": true, "schema": group } },
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
        .and(body_partial_json(serde_json::json!({
            "output_config": { "format": { "type": "json_schema", "schema": group } },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model": "claude-haiku-5-5", "stop_reason": "end_turn",
            "content": [{ "type": "text", "text": answer }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(serde_json::json!({
            "response_format": { "type": "json_schema",
                "json_schema": { "strict": true, "schema": group } },
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

/// Each provider's rows for one page, as the engine would store them.
async fn read_by_every_provider(schema: &AnalysisSchema, answer: &str) -> Vec<Vec<Vec<String>>> {
    let server = MockServer::start().await;
    serve(&server, schema, answer).await;
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
    let (input, languages, options) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let mut out = Vec::new();
    for (provider, model) in providers {
        assert!(
            provider
                .accepted_inputs(Task::Analysis)
                .contains(&"image/png"),
            "{model} reads a page's image"
        );
        let request = ProcessRequest {
            task: Task::Analysis,
            model,
            prompt: "List every entry on the page.",
            user_prompt: "List every entry on this page, in reading order.",
            input: &input,
            languages: &languages,
            options: &options,
            tuning: &TUNING,
            analysis: Some(schema),
            stop: &STOP,
        };
        let output = provider
            .process(&ApiKey::new("k"), request)
            .await
            .unwrap_or_else(|e| panic!("{model}: {e}"));
        out.push(
            records::rows(schema, &output.fields)
                .iter()
                .map(|row| row.iter().map(|a| a.value.joined()).collect())
                .collect(),
        );
    }
    out
}

#[tokio::test]
async fn a_two_column_directory_page_comes_back_in_reading_order_from_every_provider() {
    let read = read_by_every_provider(&directory(), &fixture("directory-two-columns.json")).await;
    for rows in &read {
        assert_eq!(rows.len(), 6);
        // Down the left column, then the right.
        assert_eq!(rows[2][0], "Allard (P.-M.)");
        assert_eq!(rows[3][0], "Bonnet (L.)");
        // Ditto marks for the address above, kept as printed.
        assert_eq!(rows[3][2], "\"");
        assert_eq!(rows[4][2], "do.");
        assert_eq!(rows[5][1], "", "an empty cell stays empty");
    }
    assert!(
        read.windows(2).all(|w| w[0] == w[1]),
        "every provider alike"
    );
}

#[tokio::test]
async fn a_register_with_ditto_marks_keeps_them_as_written_from_every_provider() {
    let read = read_by_every_provider(&register(), &fixture("register-ditto.json")).await;
    for rows in &read {
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1][1], "〃", "never the date it repeats");
        assert_eq!(rows[2][1], "do.");
        assert_eq!(rows[2][2], "Birth or baptism", "a choice spelled as listed");
        assert_eq!(rows[0][4], "Jean Coulibaly; Awa Keïta");
    }
}

#[tokio::test]
async fn a_right_to_left_list_comes_back_as_written_from_every_provider() {
    let read = read_by_every_provider(&directory(), &fixture("list-rtl.json")).await;
    for rows in &read {
        assert_eq!(rows[0][0], "عبد الله بن محمد");
        assert_eq!(rows[1][2], "جني");
        assert_eq!(rows[2][2], "〃");
    }
    // A row outside the schema is refused, never stored empty.
    let server = MockServer::start().await;
    serve(&server, &register(), r#"{"entries":[{"number":"1"}]}"#).await;
    let (input, languages, options) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let schema = register();
    let refused = Gemini::with_base_url(client(), &server.uri())
        .process(
            &ApiKey::new("k"),
            ProcessRequest {
                task: Task::Analysis,
                model: "gemini-3.8-flash",
                prompt: "List every entry on the page.",
                user_prompt: "List every entry on this page.",
                input: &input,
                languages: &languages,
                options: &options,
                tuning: &TUNING,
                analysis: Some(&schema),
                stop: &STOP,
            },
        )
        .await;
    assert!(refused.is_err());
}
