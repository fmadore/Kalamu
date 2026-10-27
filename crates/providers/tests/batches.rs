//! Economy mode against a local mock server: a batch is sent in the provider's format, polled,
//! and its answers read back under the ids they were sent with, failures included. Response
//! bodies follow each batch API's documented shape.

use scriptorium_core::{
    ApiKey, BatchContext, BatchProgress, LanguageChoice, PieceRef, PreparedInput, ProcessRequest,
    Provider, ProviderError, SetupOptions, StopSignal, Task,
};
use scriptorium_providers::{Anthropic, OpenAi, client};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_default()
}

fn page(number: u32) -> PreparedInput {
    PreparedInput {
        piece: PieceRef::page(number),
        mime: "application/pdf".into(),
        bytes: b"%PDF-1.5 fake".to_vec(),
        file_name: format!("lettre-p{number}.pdf"),
    }
}

static STOP: std::sync::LazyLock<StopSignal> = std::sync::LazyLock::new(StopSignal::default);

fn requests<'a>(
    model: &'a str,
    pages: &'a [PreparedInput],
    languages: &'a LanguageChoice,
    options: &'a SetupOptions,
) -> Vec<(String, ProcessRequest<'a>)> {
    pages
        .iter()
        .enumerate()
        .map(|(i, input)| {
            (
                format!("p{i}"),
                ProcessRequest {
                    task: Task::TextExtraction,
                    model,
                    prompt: "Transcribe diplomatically.",
                    user_prompt: "Transcribe this page.",
                    input,
                    languages,
                    options,
                    analysis: None,
                    stop: &STOP,
                },
            )
        })
        .collect()
}

fn context(model: &str) -> BatchContext<'_> {
    BatchContext {
        task: Task::TextExtraction,
        model,
        analysis: None,
    }
}

fn jsonl(lines: &[serde_json::Value]) -> String {
    lines.iter().map(|l| format!("{l}\n")).collect()
}

#[tokio::test]
async fn claude_sends_a_message_batch_without_fallbacks_and_reads_its_results() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/messages/batches"))
        .and(body_partial_json(serde_json::json!({ "requests": [
            { "custom_id": "p0", "params": {
                "model": "claude-opus-5-5",
                "system": [{ "cache_control": { "type": "ephemeral" } }],
            } },
            { "custom_id": "p1" },
        ] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msgbatch_1", "type": "message_batch", "processing_status": "in_progress",
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/messages/batches/msgbatch_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msgbatch_1",
            "processing_status": "in_progress",
            "request_counts": { "processing": 1, "succeeded": 1, "errored": 0,
                "canceled": 0, "expired": 0 },
        })))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/messages/batches/msgbatch_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msgbatch_1",
            "processing_status": "ended",
            "results_url": format!("{}/messages/batches/msgbatch_1/results", server.uri()),
        })))
        .with_priority(2)
        .mount(&server)
        .await;
    let results = jsonl(&[
        serde_json::json!({ "custom_id": "p1", "result": { "type": "errored", "error": {
            "type": "error", "error": { "type": "invalid_request_error",
                "message": "The PDF could not be read" } } } }),
        serde_json::json!({ "custom_id": "p0", "result": {
            "type": "succeeded", "message": fixture("anthropic_ok.json") } }),
    ]);
    Mock::given(method("GET"))
        .and(path("/messages/batches/msgbatch_1/results"))
        .respond_with(ResponseTemplate::new(200).set_body_string(results))
        .mount(&server)
        .await;

    let claude = Anthropic::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    let (pages, langs, opts) = (
        [page(1), page(2)],
        LanguageChoice::Auto,
        SetupOptions::default(),
    );
    assert!(claude.batches(Task::TextExtraction, "claude-opus-5-5"));
    let id = claude
        .submit_batch(&key, &requests("claude-opus-5-5", &pages, &langs, &opts))
        .await;
    assert_eq!(id.as_deref(), Ok("msgbatch_1"));
    let sent = server.received_requests().await.unwrap_or_default();
    let body: serde_json::Value = serde_json::from_slice(&sent[0].body).unwrap_or_default();
    assert!(
        body.pointer("/requests/0/params/fallbacks").is_none(),
        "a batch refuses fallbacks"
    );

    let ids = ["p0".to_string(), "p1".to_string()];
    let ctx = context("claude-opus-5-5");
    let working = claude.poll_batch(&key, "msgbatch_1", ctx, &ids).await;
    assert!(matches!(
        working,
        Ok(BatchProgress::Working {
            finished: 1,
            total: 2
        })
    ));
    let Ok(BatchProgress::Ended(mut answers)) =
        claude.poll_batch(&key, "msgbatch_1", ctx, &ids).await
    else {
        panic!("the batch did not end");
    };
    answers.sort_by(|a, b| a.0.cmp(&b.0));
    let text = answers[0].1.as_ref().map(|o| o.text.as_str());
    assert_eq!(text, Ok("بسم الله الرحمن الرحيم"));
    assert_eq!(
        answers[1].1,
        Err(ProviderError::InvalidRequest(
            "The PDF could not be read".into()
        ))
    );
}

#[tokio::test]
async fn openai_uploads_a_jsonl_file_and_reads_the_output_and_error_files() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/files"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": "file-in" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/batches"))
        .and(body_partial_json(serde_json::json!({
            "input_file_id": "file-in",
            "endpoint": "/v1/responses",
            "completion_window": "24h",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "batch_1", "status": "validating",
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/batches/batch_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "batch_1",
            "status": "expired",
            "output_file_id": "file-out",
            "error_file_id": "file-err",
            "request_counts": { "total": 2, "completed": 1, "failed": 1 },
        })))
        .mount(&server)
        .await;
    let output = jsonl(&[serde_json::json!({
        "id": "batch_req_1", "custom_id": "p0", "error": null,
        "response": { "status_code": 200, "request_id": "r1", "body": fixture("openai_ok.json") },
    })]);
    let errors = jsonl(&[serde_json::json!({
        "id": "batch_req_2", "custom_id": "p1", "response": null,
        "error": { "code": "batch_expired", "message": "This request could not be executed" },
    })]);
    for (file, body) in [("file-out", output), ("file-err", errors)] {
        Mock::given(method("GET"))
            .and(path(format!("/files/{file}/content")))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
    }

    let openai = OpenAi::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    let (pages, langs, opts) = (
        [page(1), page(2)],
        LanguageChoice::Auto,
        SetupOptions::default(),
    );
    assert!(!openai.batches(Task::AudioTranscription, "gpt-transcribe"));
    let id = openai
        .submit_batch(&key, &requests("gpt-6-luna", &pages, &langs, &opts))
        .await;
    assert_eq!(id.as_deref(), Ok("batch_1"));
    let sent = server.received_requests().await.unwrap_or_default();
    let upload = String::from_utf8_lossy(&sent[0].body);
    assert!(upload.contains(r#""custom_id":"p1""#), "{upload}");
    assert!(upload.contains(r#""url":"/v1/responses""#));
    assert!(upload.contains("name=\"purpose\"") && upload.contains("batch"));

    let ids = ["p0".to_string(), "p1".to_string()];
    let Ok(BatchProgress::Ended(mut answers)) = openai
        .poll_batch(&key, "batch_1", context("gpt-6-luna"), &ids)
        .await
    else {
        panic!("the batch did not end");
    };
    answers.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(answers[0].1.is_ok());
    assert!(matches!(answers[1].1, Err(ProviderError::Transient(_))));
}
