//! Economy mode against a local mock server: a batch is sent in the provider's format, polled,
//! and its answers read back under the ids they were sent with, failures included. Response
//! bodies follow each batch API's documented shape.

use kalamu_core::{
    ApiKey, BatchContext, BatchProgress, LanguageChoice, PieceRef, PreparedInput, ProcessRequest,
    Provider, ProviderError, SetupOptions, StopSignal, Task, Tuning,
};
use kalamu_providers::{Anthropic, Gemini, Mistral, OpenAi, client};
use wiremock::matchers::{body_partial_json, header, method, path};
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

/// A model the catalogue lacks: each provider's own settings.
static TUNING: std::sync::LazyLock<Tuning> = std::sync::LazyLock::new(Tuning::default);

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
                    tuning: &TUNING,
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
    // Each line caches its instructions explicitly, as a call made at once does.
    assert!(
        upload.contains(r#""prompt_cache_options":{"mode":"explicit"}"#),
        "{upload}"
    );
    assert!(upload.contains(r#""role":"developer""#), "{upload}");
    assert!(upload.contains(r#""prompt_cache_breakpoint":{"mode":"explicit"}"#));

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

/// Gemini's and Mistral's batches are written item by item into the body that is sent: still
/// one JSON document of the documented shape, each request under its id, sent as JSON.
#[tokio::test]
async fn gemini_and_mistral_send_each_request_of_a_batch_under_its_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "name": "batches/1", "id": "job_1" })),
        )
        .mount(&server)
        .await;
    let key = ApiKey::new("k");
    let (pages, langs, opts) = (
        [page(1), page(2), page(3)],
        LanguageChoice::Auto,
        SetupOptions::default(),
    );
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let sent = gemini
        .submit_batch(&key, &requests("gemini-3.8-flash", &pages, &langs, &opts))
        .await;
    assert_eq!(sent.as_deref(), Ok("batches/1"));
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let sent = mistral
        .submit_batch(&key, &requests("mistral-ocr-latest", &pages, &langs, &opts))
        .await;
    assert_eq!(sent.as_deref(), Ok("job_1"));

    let received = server.received_requests().await.unwrap_or_default();
    assert_eq!(received.len(), 2);
    for request in &received {
        assert_eq!(
            request
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
    }
    let gemini: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap_or_default();
    assert_eq!(gemini["batch"]["display_name"], "kalamu");
    let items = gemini
        .pointer("/batch/input_config/requests/requests")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(items.len(), 3, "{gemini}");
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item["metadata"]["key"], format!("p{i}"));
        assert!(item["request"]["contents"].is_array(), "{item}");
    }

    let mistral: serde_json::Value = serde_json::from_slice(&received[1].body).unwrap_or_default();
    assert_eq!(mistral["endpoint"], "/v1/ocr");
    assert_eq!(mistral["model"], "mistral-ocr-latest");
    assert_eq!(mistral["timeout_hours"], 24);
    let items = mistral["requests"].as_array().cloned().unwrap_or_default();
    assert_eq!(items.len(), 3, "{mistral}");
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item["custom_id"], format!("p{i}"));
        assert_eq!(
            item["body"]["document"]["document_url"],
            "data:application/pdf;base64,JVBERi0xLjUgZmFrZQ=="
        );
    }
}

/// What a settled OpenAI batch left is named from the batch, by its id, and deleted file by
/// file; a batch still running names nothing yet, and a file or batch already gone is no error.
#[tokio::test]
async fn openai_names_a_settled_batch_s_files_and_deletes_them() {
    let server = MockServer::start().await;
    for (id, status) in [("batch_done", "completed"), ("batch_busy", "cancelling")] {
        Mock::given(method("GET"))
            .and(path(format!("/batches/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": id,
                "object": "batch",
                "status": status,
                "input_file_id": "file-in",
                "output_file_id": "file-out",
                "error_file_id": null,
            })))
            .mount(&server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/batches/batch_gone"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "message": "No batch found with id 'batch_gone'.",
                "type": "invalid_request_error" } })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/file-in"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "file-in", "object": "file", "deleted": true })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/file-out"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "message": "No such File object: file-out" } })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/file-busy"))
        .respond_with(ResponseTemplate::new(503).set_body_string("overloaded"))
        .mount(&server)
        .await;

    let openai = OpenAi::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    assert_eq!(
        openai.batch_files(&key, "batch_done").await,
        Ok(Some(vec!["file-in".into(), "file-out".into()]))
    );
    assert_eq!(openai.batch_files(&key, "batch_busy").await, Ok(None));
    assert_eq!(
        openai.batch_files(&key, "batch_gone").await,
        Ok(Some(Vec::new()))
    );
    assert_eq!(openai.delete_batch_file(&key, "file-in").await, Ok(()));
    assert_eq!(
        openai.delete_batch_file(&key, "file-out").await,
        Ok(()),
        "already gone"
    );
    assert!(matches!(
        openai.delete_batch_file(&key, "file-busy").await,
        Err(ProviderError::Transient(_))
    ));
    // Only the batches asked about and the files named were touched: nothing was listed.
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received.iter().all(|r| r.url.path() != "/files"),
        "{received:?}"
    );
}

/// An uploaded file no batch will name, because creating the batch failed, is deleted at once.
#[tokio::test]
async fn openai_deletes_the_uploaded_file_when_the_batch_is_not_made() {
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
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": { "message": "Invalid endpoint" } })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/file-in"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "file-in", "object": "file", "deleted": true })))
        .mount(&server)
        .await;
    let openai = OpenAi::with_base_url(client(), &server.uri());
    let (pages, langs, opts) = ([page(1)], LanguageChoice::Auto, SetupOptions::default());
    let sent = openai
        .submit_batch(
            &ApiKey::new("k"),
            &requests("gpt-6-luna", &pages, &langs, &opts),
        )
        .await;
    assert_eq!(
        sent,
        Err(ProviderError::InvalidRequest("Invalid endpoint".into()))
    );
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received
            .iter()
            .any(|r| r.method.as_str() == "DELETE" && r.url.path() == "/files/file-in"),
        "{received:?}"
    );
}

/// A settled Mistral job names its input, output and error files; one still running names
/// nothing yet. Each is deleted by its id.
#[tokio::test]
async fn mistral_names_a_settled_job_s_files_and_deletes_them() {
    let server = MockServer::start().await;
    for (id, status) in [
        ("job_done", "SUCCESS"),
        ("job_busy", "CANCELLATION_REQUESTED"),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/batch/jobs/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": id,
                "object": "batch",
                "status": status,
                "input_files": ["7ad0c39f-inline"],
                "output_file": "c1d2e3f4-out",
                "error_file": "a9b8c7d6-err",
                "endpoint": "/v1/ocr",
                "total_requests": 2,
            })))
            .mount(&server)
            .await;
    }
    for file in ["7ad0c39f-inline", "c1d2e3f4-out"] {
        Mock::given(method("DELETE"))
            .and(path(format!("/files/{file}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": file, "object": "file", "deleted": true })))
            .mount(&server)
            .await;
    }
    let mistral = Mistral::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    assert_eq!(
        mistral.batch_files(&key, "job_done").await,
        Ok(Some(vec![
            "7ad0c39f-inline".into(),
            "c1d2e3f4-out".into(),
            "a9b8c7d6-err".into()
        ]))
    );
    assert_eq!(mistral.batch_files(&key, "job_busy").await, Ok(None));
    assert_eq!(
        mistral.delete_batch_file(&key, "7ad0c39f-inline").await,
        Ok(())
    );
    // Not mounted: the mock server answers 404, which counts as deleted.
    assert_eq!(
        mistral.delete_batch_file(&key, "a9b8c7d6-err").await,
        Ok(())
    );
    // The job's status is read without its answers inline.
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received.iter().all(|r| r.url.query().is_none()),
        "{received:?}"
    );
}

/// Claude keeps an ended batch, its requests and answers, until it is deleted: the batch itself
/// is named, by its id, and deleted with the key in its header; one still running or being
/// cancelled names nothing yet, and a batch already gone is no error.
#[tokio::test]
async fn claude_names_an_ended_batch_and_deletes_it() {
    let server = MockServer::start().await;
    for (id, status) in [
        ("msgbatch_done", "ended"),
        ("msgbatch_busy", "in_progress"),
        ("msgbatch_stopping", "canceling"),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/messages/batches/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": id,
                "type": "message_batch",
                "processing_status": status,
                "request_counts": { "processing": 0, "succeeded": 2, "errored": 0,
                    "canceled": 0, "expired": 0 },
                "ended_at": null,
                "created_at": "2026-11-16T09:00:00Z",
                "expires_at": "2026-11-17T09:00:00Z",
                "archived_at": null,
                "cancel_initiated_at": null,
                "results_url": null,
            })))
            .mount(&server)
            .await;
    }
    let not_found = || {
        ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "type": "error",
            "error": { "type": "not_found_error", "message": "Not found" },
        }))
    };
    Mock::given(method("GET"))
        .and(path("/messages/batches/msgbatch_gone"))
        .respond_with(not_found())
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/messages/batches/msgbatch_done"))
        .and(header("x-api-key", "k"))
        .and(header("anthropic-version", "2026-10-21"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msgbatch_done", "type": "message_batch_deleted" })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/messages/batches/msgbatch_gone"))
        .respond_with(not_found())
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/messages/batches/msgbatch_overloaded"))
        .respond_with(ResponseTemplate::new(529).set_body_json(serde_json::json!({
            "type": "error",
            "error": { "type": "overloaded_error", "message": "Overloaded" },
        })))
        .mount(&server)
        .await;

    let claude = Anthropic::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    assert_eq!(
        claude.batch_files(&key, "msgbatch_done").await,
        Ok(Some(vec!["msgbatch_done".into()]))
    );
    assert_eq!(claude.batch_files(&key, "msgbatch_busy").await, Ok(None));
    assert_eq!(
        claude.batch_files(&key, "msgbatch_stopping").await,
        Ok(None)
    );
    assert_eq!(
        claude.batch_files(&key, "msgbatch_gone").await,
        Ok(Some(Vec::new()))
    );
    assert_eq!(
        claude.delete_batch_file(&key, "msgbatch_done").await,
        Ok(())
    );
    assert_eq!(
        claude.delete_batch_file(&key, "msgbatch_gone").await,
        Ok(()),
        "already gone"
    );
    assert!(
        claude
            .delete_batch_file(&key, "msgbatch_overloaded")
            .await
            .is_err()
    );
    // Only the batches asked about were touched: nothing was listed, nothing cancelled, and
    // the key went only in its header.
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received.iter().all(|r| r.url.path() != "/messages/batches"
            && !r.url.path().ends_with("/cancel")
            && r.url.query().is_none()),
        "{received:?}"
    );
}

/// An inline Gemini batch leaves no file: once it has ended it names nothing, nor does one
/// Google no longer knows. A batch small enough to go inline uploads nothing.
#[tokio::test]
async fn an_inline_gemini_batch_uploads_and_names_nothing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/batches/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "batches/1",
            "metadata": { "state": "BATCH_STATE_SUCCEEDED",
                "output": { "inlinedResponses": { "inlinedResponses": [
                    { "response": fixture("gemini_ok.json"), "metadata": { "key": "p0" } },
                ] } } },
            "done": true,
        })))
        .mount(&server)
        .await;
    let key = ApiKey::new("k");
    let gemini = Gemini::with_base_url(client(), &server.uri());
    let (pages, langs, opts) = (
        [page(1), page(2)],
        LanguageChoice::Auto,
        SetupOptions::default(),
    );
    assert_eq!(
        gemini
            .upload_batch(&key, &requests("gemini-3.8-flash", &pages, &langs, &opts))
            .await,
        Ok(None)
    );
    assert_eq!(
        gemini.batch_files(&key, "batches/1").await,
        Ok(Some(Vec::new()))
    );
    // Not mounted: Google no longer knows the batch.
    assert_eq!(
        gemini.batch_files(&key, "batches/gone").await,
        Ok(Some(Vec::new()))
    );
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received.iter().all(|r| r.method.as_str() == "GET"),
        "{received:?}"
    );
}

/// A page large enough that two of them pass the 20 MB a batch may carry inline.
fn large_page(number: u32) -> PreparedInput {
    PreparedInput {
        bytes: vec![b'%'; 8 * 1024 * 1024],
        ..page(number)
    }
}

/// A Gemini batch above the inline limit is written as JSON Lines, uploaded through the Files
/// API's resumable upload, and made from the file; its answers are read from the file Google
/// writes, by key, a failed line included. Once it has ended it names both files, which are
/// deleted, one already gone counting as deleted. Shapes from Google's batch, Files and
/// reference pages; no live run has confirmed them.
#[tokio::test]
async fn gemini_uploads_a_large_batch_as_a_file_and_reads_the_answers_file() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/upload/files"))
        .and(header("x-goog-api-key", "k"))
        .and(header("X-Goog-Upload-Protocol", "resumable"))
        .and(header("X-Goog-Upload-Command", "start"))
        .and(header(
            "X-Goog-Upload-Header-Content-Type",
            "application/jsonl",
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "x-goog-upload-url",
                    format!("{}/upload-session/1?upload_id=xyz", server.uri()),
                )
                .set_body_json(serde_json::json!({})),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/upload-session/1"))
        .and(header("X-Goog-Upload-Offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "file": {
                "name": "files/kalamu-in",
                "displayName": "kalamu-batch",
                "mimeType": "application/jsonl",
                "sizeBytes": "22369700",
                "uri": "https://generativelanguage.googleapis.com/v1beta/files/kalamu-in",
                "state": "ACTIVE",
                "source": "UPLOADED",
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/models/gemini-3.8-flash:batchGenerateContent"))
        .and(body_partial_json(serde_json::json!({ "batch": {
            "display_name": "kalamu",
            "input_config": { "file_name": "files/kalamu-in" },
        } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "batches/big",
            "metadata": { "state": "BATCH_STATE_PENDING" },
        })))
        .mount(&server)
        .await;
    let ended = serde_json::json!({
        "name": "batches/big",
        "metadata": {
            "name": "batches/big",
            "model": "models/gemini-3.8-flash",
            "state": "BATCH_STATE_SUCCEEDED",
            "inputConfig": { "fileName": "files/kalamu-in" },
            "output": { "responsesFile": "files/batch-out" },
            "batchStats": { "requestCount": "2", "successfulRequestCount": "1",
                "failedRequestCount": "1" },
        },
        "done": true,
        "response": { "responsesFile": "files/batch-out" },
    });
    Mock::given(method("GET"))
        .and(path("/batches/big"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ended))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/batches/busy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "batches/busy",
            "metadata": { "state": "BATCH_STATE_RUNNING",
                "inputConfig": { "fileName": "files/other-in" } },
        })))
        .mount(&server)
        .await;
    let answers = jsonl(&[
        serde_json::json!({ "key": "p1", "error": { "code": 400,
            "message": "The document has no pages.", "status": "INVALID_ARGUMENT" } }),
        serde_json::json!({ "key": "p0", "response": fixture("gemini_ok.json") }),
    ]);
    Mock::given(method("GET"))
        .and(path("/download/files/batch-out:download"))
        .and(header("x-goog-api-key", "k"))
        .respond_with(ResponseTemplate::new(200).set_body_string(answers))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/kalamu-in"))
        .and(header("x-goog-api-key", "k"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/files/batch-out"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "code": 404, "message": "File not found.", "status": "NOT_FOUND" } })))
        .mount(&server)
        .await;

    let gemini = Gemini::with_base_url(client(), &server.uri());
    let key = ApiKey::new("k");
    let (pages, langs, opts) = (
        [large_page(1), large_page(2)],
        LanguageChoice::Auto,
        SetupOptions::default(),
    );
    let batch = requests("gemini-3.8-flash", &pages, &langs, &opts);
    let file = gemini.upload_batch(&key, &batch).await;
    assert_eq!(file, Ok(Some("files/kalamu-in".to_string())));
    let id = gemini
        .submit_uploaded_batch(&key, "files/kalamu-in", &batch)
        .await;
    assert_eq!(id.as_deref(), Ok("batches/big"));

    let sent = server.received_requests().await.unwrap_or_default();
    let upload = sent
        .iter()
        .find(|r| r.url.path() == "/upload-session/1")
        .map(|r| r.body.clone())
        .unwrap_or_default();
    let start = sent.iter().find(|r| r.url.path() == "/upload/files");
    assert_eq!(
        start
            .and_then(|r| r.headers.get("X-Goog-Upload-Header-Content-Length"))
            .and_then(|v| v.to_str().ok()),
        Some(upload.len().to_string().as_str())
    );
    let lines: Vec<serde_json::Value> = String::from_utf8_lossy(&upload)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert_eq!(lines.len(), 2);
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(line["key"], format!("p{i}"));
        assert!(line["request"]["contents"].is_array());
        assert_eq!(
            line["request"]["generationConfig"]["mediaResolution"],
            "MEDIA_RESOLUTION_HIGH"
        );
    }
    // The bytes are sent and the upload finished in one request (wiremock's matcher would split
    // the value at its comma), and the key goes to the API's own address, never to the upload
    // session's.
    let session: Vec<_> = sent
        .iter()
        .filter(|r| r.url.path() == "/upload-session/1")
        .collect();
    assert_eq!(session.len(), 1);
    assert_eq!(
        session[0]
            .headers
            .get("X-Goog-Upload-Command")
            .and_then(|v| v.to_str().ok()),
        Some("upload, finalize")
    );
    assert!(!session[0].headers.contains_key("x-goog-api-key"));

    let ids = ["p0".to_string(), "p1".to_string()];
    let Ok(BatchProgress::Ended(mut answers)) = gemini
        .poll_batch(&key, "batches/big", context("gemini-3.8-flash"), &ids)
        .await
    else {
        panic!("the batch did not end");
    };
    answers.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        answers[0].1.as_ref().map(|o| o.text.as_str()),
        Ok("Bamako, le 3 mai 1957\nMonsieur le Gouverneur,")
    );
    assert_eq!(
        answers[1].1,
        Err(ProviderError::InvalidRequest(
            "The document has no pages.".into()
        ))
    );

    assert_eq!(
        gemini.batch_files(&key, "batches/big").await,
        Ok(Some(vec![
            "files/kalamu-in".into(),
            "files/batch-out".into()
        ]))
    );
    assert_eq!(gemini.batch_files(&key, "batches/busy").await, Ok(None));
    assert_eq!(
        gemini.delete_batch_file(&key, "files/kalamu-in").await,
        Ok(())
    );
    assert_eq!(
        gemini.delete_batch_file(&key, "files/batch-out").await,
        Ok(()),
        "already gone"
    );
    // Nothing but a file is deleted, and nothing was listed.
    assert!(gemini.delete_batch_file(&key, "batches/big").await.is_err());
    let received = server.received_requests().await.unwrap_or_default();
    assert!(
        received
            .iter()
            .all(|r| r.url.path() != "/files" && r.url.path() != "/batches"),
        "{received:?}"
    );
}
