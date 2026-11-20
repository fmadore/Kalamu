//! A document read against a research question (the default screening prompt's fields, #51),
//! through every provider with structured output: a text read from two scanned pages sent in
//! numbered paragraphs, the passages asked for as entries of quotations alone, and the recorded
//! answer in `fixtures/cited/screening.json` read back the same by each. Of its two passages one
//! stands in the text and one does not: looked for in the text, the first is kept with its page
//! and the second left out, and an answer holding only the second is one the engine refuses.

use kalamu_core::analysis::cited::{CitedText, cite, numbered};
use kalamu_core::snapshot::{Built, PartText, TimedPart, build_with_turns};
use kalamu_core::{
    AnalysisSchema, ApiKey, Field, FieldAnswer, FieldKind, LanguageChoice, PieceRef, PreparedInput,
    ProcessRequest, Provider, SetupOptions, StopSignal, Task, Tuning,
};
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::{body_partial_json, body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn answer() -> String {
    let path = format!(
        "{}/tests/fixtures/cited/screening.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<serde_json::Value>(&text)
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn field(key: &str, kind: FieldKind) -> Field {
    Field {
        key: key.into(),
        label: key.into(),
        instructions: String::new(),
        kind,
        reason: false,
    }
}

fn screening() -> AnalysisSchema {
    AnalysisSchema {
        fields: vec![
            field(
                "relevance",
                FieldKind::Choice {
                    values: ["Relevant", "Partly relevant", "Not relevant", "Unsure"]
                        .map(String::from)
                        .to_vec(),
                },
            ),
            field("reason", FieldKind::Text),
            field(
                "passages",
                FieldKind::Cited {
                    fields: vec![],
                    max: Some(3),
                },
            ),
        ],
    }
}

/// Two pages read from a scan, the first in two paragraphs.
fn pages() -> Built {
    let (one, two) = (PieceRef::page(1), PieceRef::page(2));
    let part = |piece, text| TimedPart {
        text: PartText {
            piece,
            text,
            reviewed: true,
            corrected: true,
            failed: false,
        },
        segments: &[],
    };
    build_with_turns(
        Task::TextExtraction,
        &[
            part(
                &one,
                "Le conseil municipal s’est réuni le 3 mars.\n\nIl a voté le budget du marché.",
            ),
            part(
                &two,
                "Les commerçantes du marché demandent un abri contre la pluie.",
            ),
        ],
    )
}

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);
static TUNING: std::sync::LazyLock<Tuning> = std::sync::LazyLock::new(Tuning::default);

/// Mounts each provider's endpoint, answering when the request holds the numbered paragraphs
/// and asks for the passages as entries whose only property is their quotations.
async fn serve(server: &MockServer, answer: &str) {
    let schema = serde_json::json!({
        "type": "object",
        "required": ["relevance", "reason", "passages"],
        "properties": { "passages": { "type": "array", "items": {
            "type": "object", "required": ["quotes"], "additionalProperties": false } } },
    });
    let passage = "[3] Les commerçantes du marché";
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    Mock::given(method("POST"))
        .and(path("/models/gemini-3.8-flash:generateContent"))
        .and(body_string_contains(passage))
        .and(body_partial_json(serde_json::json!({
            "generationConfig": { "responseJsonSchema": schema } })))
        .respond_with(ok(serde_json::json!({
            "candidates": [{ "finishReason": "STOP",
                "content": { "parts": [{ "text": answer }] } }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(body_string_contains(passage))
        .and(body_partial_json(serde_json::json!({
            "text": { "format": { "schema": schema } } })))
        .respond_with(ok(serde_json::json!({
            "model": "gpt-6-luna",
            "output": [{ "type": "message",
                "content": [{ "type": "output_text", "text": answer }] }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_string_contains(passage))
        .and(body_partial_json(serde_json::json!({
            "output_config": { "format": { "schema": schema } } })))
        .respond_with(ok(serde_json::json!({
            "model": "claude-haiku-5-5", "stop_reason": "end_turn",
            "content": [{ "type": "text", "text": answer }],
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_string_contains(passage))
        .and(body_partial_json(serde_json::json!({
            "response_format": { "json_schema": { "schema": schema } } })))
        .respond_with(ok(serde_json::json!({
            "model": "mistral-small-2603",
            "choices": [{ "index": 0, "finish_reason": "stop",
                "message": { "role": "assistant", "content": answer } }],
            "usage": { "prompt_tokens": 900, "completion_tokens": 120, "total_tokens": 1020 },
        })))
        .mount(server)
        .await;
}

/// The answer each provider gives, read into fields.
async fn read_by_every_provider(answer: &str) -> Vec<Vec<FieldAnswer>> {
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
    let built = pages();
    let input = PreparedInput {
        piece: PieceRef::Whole,
        mime: "text/plain".into(),
        bytes: numbered(&built.text, &built.turns).into_bytes(),
        file_name: "conseil-1957-turns.txt".into(),
    };
    let (schema, languages, options) = (screening(), LanguageChoice::Auto, SetupOptions::default());
    let mut out = Vec::new();
    for (provider, model) in providers {
        let output = provider
            .process(
                &ApiKey::new("k"),
                ProcessRequest {
                    task: Task::Analysis,
                    model,
                    prompt: "Does the document speak of the question?",
                    user_prompt: "Fill in each field.",
                    input: &input,
                    languages: &languages,
                    options: &options,
                    tuning: &TUNING,
                    analysis: Some(&schema),
                    stop: &STOP,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("{model}: {e}"));
        out.push(output.fields);
    }
    out
}

#[tokio::test]
async fn every_provider_reads_the_answer_and_its_passages_are_looked_for_in_the_pages() {
    let read = read_by_every_provider(&answer()).await;
    assert!(
        read.windows(2).all(|w| w[0] == w[1]),
        "every provider alike"
    );
    let mut fields = read[0].clone();
    assert_eq!(fields[0].value.joined(), "Relevant");
    let passages = fields[2].value.cited().cloned().unwrap_or_default();
    assert_eq!(passages.entries.len(), 2);
    assert_eq!(passages.entries[0].claims[0].turn, Some(3));

    // Looked for in the text: the passage on page 2 kept, the one found nowhere left out.
    let built = pages();
    let doc = CitedText {
        text: &built.text,
        parts: &built.parts,
        turns: &built.turns,
    };
    let counted = cite(&screening().fields, &mut fields, doc);
    assert_eq!((counted.given, counted.found), (2, 1));
    let passages = fields[2].value.cited().cloned().unwrap_or_default();
    assert_eq!((passages.entries.len(), passages.left_out), (1, 1));
    assert_eq!(
        passages.entries[0].evidence[0].page,
        Some(PieceRef::page(2))
    );

    // An answer holding only the invented passage finds none: the engine refuses such an answer.
    let invented = r#"{"relevance":"Relevant","reason":"r","passages":[{"quotes":[{"turn":1,"words":"Les femmes du marché ont manifesté devant la mairie."}]}]}"#;
    let mut fields = read_by_every_provider(invented).await.remove(0);
    let counted = cite(&screening().fields, &mut fields, doc);
    assert_eq!((counted.given, counted.found), (1, 0));

    // A document that does not speak of the question: no passage, nothing to look for.
    let none = r#"{"relevance":"Not relevant","reason":"Rien sur le marché.","passages":[]}"#;
    let mut fields = read_by_every_provider(none).await.remove(0);
    assert_eq!(fields[0].value.joined(), "Not relevant");
    assert_eq!(
        cite(&screening().fields, &mut fields, doc).given,
        0,
        "no passage is not a refusal"
    );
}
