//! Each provider against a local mock server: the request must carry the settings ported from
//! the Python pipelines, and the provider's response must parse into text, usage and model
//! version. Response bodies in `fixtures/` follow each API's documented shape; replace them
//! with recorded responses when live keys are available for a recording run.

use scriptorium_core::{
    ApiKey, LanguageChoice, LanguageCode, PieceRef, PreparedInput, ProcessRequest, Provider,
    ProviderError, SetupOptions, StopSignal, Task,
};
use scriptorium_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

fn page() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::page(1),
        mime: "application/pdf".into(),
        bytes: b"%PDF-1.5 fake".to_vec(),
        file_name: "lettre-p1.pdf".into(),
    }
}

fn audio() -> PreparedInput {
    PreparedInput {
        piece: PieceRef::Segment {
            index: 0,
            start_ms: 0,
            end_ms: 60_000,
        },
        mime: "audio/mpeg".into(),
        bytes: vec![0u8; 64],
        file_name: "entretien.mp3".into(),
    }
}

fn request<'a>(
    task: Task,
    model: &'a str,
    input: &'a PreparedInput,
    languages: &'a LanguageChoice,
    options: &'a SetupOptions,
) -> ProcessRequest<'a> {
    ProcessRequest {
        task,
        model,
        prompt: "Transcribe diplomatically.",
        user_prompt: "Transcribe this page.",
        input,
        languages,
        options,
        stop: &STOP,
    }
}

/// Cloud providers ignore the stop signal; one shared, never-set signal will do.
static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);

#[tokio::test]
async fn gemini_sends_pages_at_high_resolution_and_reads_the_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/gemini-3.8-flash:generateContent"))
        .and(header("x-goog-api-key", "test-key"))
        .and(body_partial_json(serde_json::json!({
            "systemInstruction": { "parts": [{ "text": "Transcribe diplomatically." }] },
            "contents": [{ "parts": [
                { "inlineData": { "mimeType": "application/pdf" },
                  "mediaResolution": { "level": "MEDIA_RESOLUTION_HIGH" } },
                { "text": "Transcribe this page." }
            ]}],
            "generationConfig": { "thinkingConfig": { "thinkingLevel": "LOW" } },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("gemini_ok.json")))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let out = gemini
        .process(
            &ApiKey::new("test-key"),
            request(
                Task::TextExtraction,
                "gemini-3.8-flash",
                &input,
                &langs,
                &opts,
            ),
        )
        .await;
    let out = out.unwrap_or_default();
    assert_eq!(out.text, "Bamako, le 3 mai 1957\nMonsieur le Gouverneur,");
    assert_eq!(out.usage.input_tokens, 1290);
    assert_eq!(out.usage.output_tokens, 48 + 120);
    assert_eq!(out.model_version.as_deref(), Some("gemini-3.8-flash-001"));
}

#[tokio::test]
async fn gemini_rate_limits_and_recitation_are_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/models/limited:generateContent"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": { "code": 429, "message": "Quota exceeded. Please retry in 7s.", "status": "RESOURCE_EXHAUSTED" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/recites:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "candidates": [{ "finishReason": "RECITATION", "content": { "parts": [] } }]
        })))
        .mount(&server)
        .await;
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let key = ApiKey::new("k");
    let limited = gemini
        .process(
            &key,
            request(Task::TextExtraction, "limited", &input, &langs, &opts),
        )
        .await;
    assert!(matches!(
        limited,
        Err(ProviderError::RateLimited { retry_after: Some(d), .. }) if d.as_secs() == 7
    ));
    let recites = gemini
        .process(
            &key,
            request(Task::TextExtraction, "recites", &input, &langs, &opts),
        )
        .await;
    assert!(matches!(recites, Err(ProviderError::Blocked(_))));
}

#[tokio::test]
async fn claude_uses_the_current_api_version_and_skips_thinking_blocks() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2026-10-21"))
        .and(header("anthropic-beta", "server-side-fallback-2026-07-01"))
        .and(body_partial_json(serde_json::json!({
            "model": "claude-opus-5-5",
            "system": "Transcribe diplomatically.",
            "output_config": { "effort": "low" },
            "fallbacks": "default",
            "messages": [{ "role": "user", "content": [
                { "type": "document", "source": { "type": "base64", "media_type": "application/pdf" } },
                { "type": "text", "text": "Transcribe this page." }
            ]}],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("anthropic_ok.json")))
        .mount(&server)
        .await;
    let claude = Anthropic::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let out = claude
        .process(
            &ApiKey::new("test-key"),
            request(
                Task::TextExtraction,
                "claude-opus-5-5",
                &input,
                &langs,
                &opts,
            ),
        )
        .await
        .unwrap_or_default();
    assert_eq!(out.text, "بسم الله الرحمن الرحيم");
    assert_eq!(out.model_version.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(out.usage.output_tokens, 31);
}

#[tokio::test]
async fn claude_refusals_and_credit_errors_are_told_apart() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_partial_json(serde_json::json!({ "model": "refuses" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "message", "model": "refuses", "content": [],
            "stop_reason": "refusal", "stop_details": { "type": "refusal", "category": null }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(body_partial_json(serde_json::json!({ "model": "broke" })))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "type": "error",
            "error": { "type": "invalid_request_error", "message": "Your credit balance is too low to access the Anthropic API." }
        })))
        .mount(&server)
        .await;
    let claude = Anthropic::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let key = ApiKey::new("k");
    let refused = claude
        .process(
            &key,
            request(Task::TextExtraction, "refuses", &input, &langs, &opts),
        )
        .await;
    assert!(matches!(refused, Err(ProviderError::Blocked(_))));
    let broke = claude
        .process(
            &key,
            request(Task::TextExtraction, "broke", &input, &langs, &opts),
        )
        .await;
    assert!(matches!(&broke, Err(e) if e.is_fatal_for_job()));
}

#[tokio::test]
async fn openai_sends_pdf_pages_as_input_files() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/responses"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_partial_json(serde_json::json!({
            "model": "gpt-6-sol",
            "instructions": "Transcribe diplomatically.",
            "input": [{ "role": "user", "content": [
                { "type": "input_file", "filename": "lettre-p1.pdf" },
                { "type": "input_text", "text": "Transcribe this page." }
            ]}],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("openai_ok.json")))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let out = openai
        .process(
            &ApiKey::new("test-key"),
            request(Task::TextExtraction, "gpt-6-sol", &input, &langs, &opts),
        )
        .await
        .unwrap_or_default();
    assert_eq!(
        out.text,
        "RÉPUBLIQUE DU NIGER\nFraternité – Travail – Progrès"
    );
    assert_eq!(out.usage.input_tokens, 1500);
}

#[tokio::test]
async fn openai_diarized_transcripts_become_segments() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("openai_diarized.json")))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let langs = LanguageChoice::Explicit(vec![LanguageCode::new("fr")]);
    let (input, opts) = (audio(), SetupOptions::default());
    let out = openai
        .process(
            &ApiKey::new("k"),
            request(
                Task::AudioTranscription,
                "gpt-4o-transcribe-diarize",
                &input,
                &langs,
                &opts,
            ),
        )
        .await
        .unwrap_or_default();
    assert_eq!(out.segments.len(), 2);
    assert_eq!(out.segments[1].speaker.as_deref(), Some("Speaker B"));
    assert_eq!(out.segments[1].start_ms, 4_200);
}

#[tokio::test]
async fn mistral_ocr_returns_markdown_without_image_links() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/ocr"))
        .and(body_partial_json(serde_json::json!({
            "model": "mistral-ocr-4-1",
            "document": { "type": "document_url" },
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("mistral_ocr.json")))
        .mount(&server)
        .await;
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let (input, langs, opts) = (page(), LanguageChoice::Auto, SetupOptions::default());
    let out = mistral
        .process(
            &ApiKey::new("k"),
            request(
                Task::TextExtraction,
                "mistral-ocr-4-1",
                &input,
                &langs,
                &opts,
            ),
        )
        .await
        .unwrap_or_default();
    assert_eq!(out.text, "# L'Essor\n\nBamako, 12 juin 1958.");
    assert_eq!(out.usage.pages, 1);
}

#[tokio::test]
async fn voxtral_returns_segments_with_speakers_and_language() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("voxtral.json")))
        .mount(&server)
        .await;
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let opts = SetupOptions {
        diarize: true,
        timestamps: true,
        ..SetupOptions::default()
    };
    let (input, langs) = (audio(), LanguageChoice::Auto);
    let out = mistral
        .process(
            &ApiKey::new("k"),
            request(
                Task::AudioTranscription,
                "voxtral-small-transcribe-3-0",
                &input,
                &langs,
                &opts,
            ),
        )
        .await
        .unwrap_or_default();
    assert_eq!(out.segments.len(), 2);
    assert_eq!(out.segments[0].speaker.as_deref(), Some("Speaker 0"));
    assert_eq!(
        out.detected_language.as_ref().map(LanguageCode::as_str),
        Some("fr")
    );
    assert!((out.usage.audio_seconds - 61.5).abs() < 1e-9);
}

#[tokio::test]
async fn bad_keys_are_reported_as_auth_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": { "message": "Incorrect API key provided", "type": "invalid_request_error" }
        })))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let result = openai.list_models(&ApiKey::new("wrong")).await;
    assert!(matches!(result, Err(ProviderError::Auth(_))));
}
