//! OpenAI's Batch API: the Responses requests written to a JSON Lines file, uploaded, and run as
//! one batch at half price within 24 hours. Answers come back in an output file and failures in
//! an error file, both keyed by `custom_id` and in any order.
//!
//! The three files stay in the account's storage (30 days by default, for `purpose=batch`)
//! until deleted: once a batch has settled, [`OpenAi::files_of`] names them from the batch and
//! each is deleted with `DELETE /v1/files/{file_id}`.

use kalamu_core::{ApiKey, BatchContext, BatchProgress, ProcessRequest, ProviderError};
use reqwest::multipart::{Form, Part};
use serde_json::{Value, json};

use super::OpenAi;
use crate::batch::{self, Answer, Body};
use crate::http::{send_json, upload_timeout};

impl OpenAi {
    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let mut jsonl = Body::new("", requests);
        let (method, url) = (Value::from("POST"), Value::from("/v1/responses"));
        for (id, request) in requests {
            jsonl.line(&[
                ("body", &Self::extraction_body(request)),
                ("custom_id", &Value::from(id.as_str())),
                ("method", &method),
                ("url", &url),
            ])?;
        }
        let jsonl = jsonl.finish("");
        let timeout = upload_timeout(jsonl.len());
        let file = Part::bytes(jsonl)
            .file_name("batch.jsonl")
            .mime_str("application/jsonl")
            .map_err(|e| ProviderError::InvalidRequest(e.to_string()))?;
        let uploaded = send_json(
            self.client
                .post(format!("{}/files", self.base_url))
                .bearer_auth(key.expose())
                .timeout(timeout)
                .multipart(Form::new().text("purpose", "batch").part("file", file)),
        )
        .await?;
        let file_id = uploaded
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| ProviderError::Other("OpenAI gave the batch file no id".into()))?;
        let created = self.create(key, file_id).await;
        if created.is_err() {
            // No batch will name the file: it is deleted now, or it would stay in the account's
            // storage with the pages it holds. Should the batch have been made all the same
            // (a request timed out), it then fails to start instead of being billed unread.
            let _ = batch::delete_file(&self.client, &self.base_url, key, file_id).await;
        }
        created
    }

    async fn create(&self, key: &ApiKey, file_id: &str) -> Result<String, ProviderError> {
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

    /// The files a settled batch left: the requests uploaded, the answers and the errors.
    /// `None` while it may still write some (validating, running, finalizing, cancelling).
    pub(super) async fn files_of(
        &self,
        key: &ApiKey,
        batch_id: &str,
    ) -> Result<Option<Vec<String>>, ProviderError> {
        let url = format!("{}/batches/{batch_id}", self.base_url);
        let Some(status) = batch::status_if_known(&self.client, url, key).await? else {
            return Ok(Some(Vec::new()));
        };
        match status.get("status").and_then(Value::as_str).unwrap_or("") {
            "completed" | "failed" | "expired" | "cancelled" => Ok(Some(batch::named_files(
                &status,
                &["input_file_id", "output_file_id", "error_file_id"],
            ))),
            _ => Ok(None),
        }
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
                let mut lines =
                    batch::file_lines(&self.client, &self.base_url, key, &status, "output_file_id")
                        .await?;
                lines.extend(
                    batch::file_lines(&self.client, &self.base_url, key, &status, "error_file_id")
                        .await?,
                );
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
