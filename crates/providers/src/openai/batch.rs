//! OpenAI's Batch API: the Responses requests written to a JSON Lines file, uploaded, and run as
//! one batch at half price within 24 hours. Answers come back in an output file and failures in
//! an error file, both keyed by `custom_id` and in any order.

use reqwest::multipart::{Form, Part};
use scriptorium_core::{ApiKey, BatchContext, BatchProgress, ProcessRequest, ProviderError};
use serde_json::{Value, json};

use super::OpenAi;
use crate::batch::{self, Answer};
use crate::http::{send_json, send_text};

impl OpenAi {
    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let mut jsonl = String::new();
        for (id, request) in requests {
            let line = json!({
                "custom_id": id,
                "method": "POST",
                "url": "/v1/responses",
                "body": Self::extraction_body(request),
            });
            jsonl.push_str(&line.to_string());
            jsonl.push('\n');
        }
        let file = Part::bytes(jsonl.into_bytes())
            .file_name("batch.jsonl")
            .mime_str("application/jsonl")
            .map_err(|e| ProviderError::InvalidRequest(e.to_string()))?;
        let uploaded = send_json(
            self.client
                .post(format!("{}/files", self.base_url))
                .bearer_auth(key.expose())
                .multipart(Form::new().text("purpose", "batch").part("file", file)),
        )
        .await?;
        let file_id = uploaded
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| ProviderError::Other("OpenAI gave the batch file no id".into()))?;
        let created = send_json(
            self.client
                .post(format!("{}/batches", self.base_url))
                .bearer_auth(key.expose())
                .json(&json!({
                    "input_file_id": file_id,
                    "endpoint": "/v1/responses",
                    "completion_window": "24h",
                })),
        )
        .await?;
        created
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Other("OpenAI gave the batch no id".into()))
    }

    pub(super) async fn poll(
        &self,
        key: &ApiKey,
        batch_id: &str,
        context: BatchContext<'_>,
    ) -> Result<BatchProgress, ProviderError> {
        let status = send_json(
            self.client
                .get(format!("{}/batches/{batch_id}", self.base_url))
                .bearer_auth(key.expose()),
        )
        .await?;
        let counts = |name: &str| batch::count(status.pointer(&format!("/request_counts/{name}")));
        match status.get("status").and_then(Value::as_str).unwrap_or("") {
            // An expired or cancelled batch keeps what it finished; the rest are in the error
            // file.
            "completed" | "expired" | "cancelled" => {
                let mut lines = self.file_lines(key, &status, "output_file_id").await?;
                lines.extend(self.file_lines(key, &status, "error_file_id").await?);
                Ok(BatchProgress::Ended(
                    lines
                        .iter()
                        .filter_map(|line| answer(context, line))
                        .collect(),
                ))
            }
            "failed" => Ok(BatchProgress::Abandoned(
                batch::first(&status, &["/errors/data/0/message", "/errors/0/message"])
                    .and_then(Value::as_str)
                    .unwrap_or("OpenAI refused the batch")
                    .to_string(),
            )),
            _ => Ok(BatchProgress::Working {
                finished: counts("completed") + counts("failed"),
                total: counts("total"),
            }),
        }
    }

    async fn file_lines(
        &self,
        key: &ApiKey,
        status: &Value,
        field: &str,
    ) -> Result<Vec<Value>, ProviderError> {
        let Some(file) = status.get(field).and_then(Value::as_str) else {
            return Ok(Vec::new());
        };
        let body = send_text(
            self.client
                .get(format!("{}/files/{file}/content", self.base_url))
                .bearer_auth(key.expose()),
        )
        .await?;
        Ok(batch::lines(&body))
    }
}

/// One line of the output or error file.
fn answer(context: BatchContext<'_>, line: &Value) -> Option<Answer> {
    let id = line.get("custom_id")?.as_str()?.to_string();
    let status = line
        .pointer("/response/status_code")
        .and_then(Value::as_u64);
    let answer = match (status, line.pointer("/response/body")) {
        (Some(200), Some(body)) => OpenAi::read(context, body),
        _ => {
            let error = batch::first(line, &["/error", "/response/body/error"]);
            if error.and_then(|e| e.get("code")).and_then(Value::as_str) == Some("batch_expired") {
                Err(ProviderError::Transient(
                    "the batch ended before OpenAI reached this piece".into(),
                ))
            } else {
                Err(batch::item_error(error))
            }
        }
    };
    Some((id, answer))
}
