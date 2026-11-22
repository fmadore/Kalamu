//! Mistral batch jobs: OCR or chat requests written to a JSON Lines file, uploaded to
//! `/v1/files` (`purpose=batch`), and run as one job of `/v1/batch/jobs` naming it
//! (`input_files`), at half price, with a 24-hour limit. Results are read inline
//! (`?inline=true`), or from the output file when the job has one. The reference types each
//! result only as a map, so the common shapes are read.
//!
//! The requests are not sent inline (`requests`), though Mistral takes them so: an inline job's
//! body is refused past about 2.5 MB with an HTML page, "Bad Request (400)", before the batch
//! service reads it, which two photographed pages already pass (1.10.0-rc2's test pass: a
//! 1.8 MB and a 0.13 MB JPEG, 2.6 MB once in base64). An uploaded file has no such limit.
//!
//! A job keeps its files in the account's storage, with no stated end ("the results do not
//! expire at this time"): the input file, the output file and the error file. Once a job has
//! settled, [`Mistral::files_of`] names them from the job and each is deleted with
//! `DELETE /v1/files/{file_id}`; an uploaded file no job names, because making the job failed,
//! is deleted at once.

use kalamu_core::{
    ApiKey, BatchContext, BatchProgress, ProcessOutput, ProcessRequest, ProviderError, Task,
};
use reqwest::multipart::{Form, Part};
use serde_json::{Value, json};

use super::{Mistral, chat};
use crate::analysis;
use crate::batch::{self, Answer, Body};
use crate::http::{send_json, upload_timeout};

impl Mistral {
    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let Some((_, first)) = requests.first() else {
            return Err(ProviderError::InvalidRequest("an empty batch".into()));
        };
        let endpoint = match first.task {
            Task::TextExtraction => "/v1/ocr",
            Task::Analysis | Task::Contents | Task::TextCorrection | Task::Translation => {
                "/v1/chat/completions"
            }
            Task::AudioTranscription => {
                return Err(ProviderError::Unsupported("batched transcription".into()));
            }
        };
        let mut jsonl = Body::new("", requests);
        for (id, request) in requests {
            let item = match first.task {
                Task::TextExtraction => Self::ocr_body(request),
                _ => chat::body(request, analysis::fields(request)?),
            };
            jsonl.line(&[("custom_id", &Value::from(id.as_str())), ("body", &item)])?;
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
            .ok_or_else(|| ProviderError::Other("Mistral gave the batch file no id".into()))?;
        let created = self.create(key, file_id, endpoint, first.model).await;
        if created.is_err() {
            // No job will name the file: it is deleted now, or it would stay in the account's
            // storage with the pages it holds.
            let _ = batch::delete_file(&self.client, &self.base_url, key, file_id).await;
        }
        created
    }

    /// A job of the requests in the file uploaded, on one endpoint and model.
    async fn create(
        &self,
        key: &ApiKey,
        file_id: &str,
        endpoint: &str,
        model: &str,
    ) -> Result<String, ProviderError> {
        let created = send_json(
            self.client
                .post(format!("{}/batch/jobs", self.base_url))
                .bearer_auth(key.expose())
                .json(&json!({
                    "input_files": [file_id],
                    "endpoint": endpoint,
                    "model": model,
                    "timeout_hours": 24,
                })),
        )
        .await?;
        created
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Other("Mistral gave the batch no id".into()))
    }

    /// The files a settled job left: its input files, output file and error file. `None` while
    /// it may still write some (queued, running, cancellation requested).
    pub(super) async fn files_of(
        &self,
        key: &ApiKey,
        batch_id: &str,
    ) -> Result<Option<Vec<String>>, ProviderError> {
        let url = format!("{}/batch/jobs/{batch_id}", self.base_url);
        let Some(status) = batch::status_if_known(&self.client, url, key).await? else {
            return Ok(Some(Vec::new()));
        };
        match status.get("status").and_then(Value::as_str).unwrap_or("") {
            "SUCCESS" | "FAILED" | "TIMEOUT_EXCEEDED" | "CANCELLED" => Ok(Some(
                batch::named_files(&status, &["input_files", "output_file", "error_file"]),
            )),
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
                .get(format!(
                    "{}/batch/jobs/{batch_id}?inline=true",
                    self.base_url
                ))
                .bearer_auth(key.expose()),
        )
        .await?;
        let state = status.get("status").and_then(Value::as_str).unwrap_or("");
        if matches!(state, "QUEUED" | "RUNNING" | "CANCELLATION_REQUESTED" | "") {
            let finished = batch::count(status.get("completed_requests")).max(
                batch::count(status.get("succeeded_requests"))
                    + batch::count(status.get("failed_requests")),
            );
            return Ok(BatchProgress::Working {
                finished,
                total: batch::count(status.get("total_requests")),
            });
        }
        let mut results: Vec<Value> = status
            .get("outputs")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for file in ["output_file", "error_file"] {
            if results.is_empty() || file == "error_file" {
                results.extend(
                    batch::file_lines(&self.client, &self.base_url, key, &status, file).await?,
                );
            }
        }
        let answers: Vec<Answer> = results
            .iter()
            .filter_map(|line| answer(context, line))
            .collect();
        // A job that timed out keeps what it finished; one that failed or was cancelled with
        // nothing to show is abandoned as a whole.
        if answers.is_empty() && state != "SUCCESS" {
            let reason = batch::first(&status, &["/errors/0/message"])
                .and_then(Value::as_str)
                .unwrap_or(state);
            return Ok(BatchProgress::Abandoned(reason.to_string()));
        }
        Ok(BatchProgress::Ended(answers))
    }
}

/// One result: `{custom_id, response: {status_code, body}}`, a bare body, or an error.
fn answer(context: BatchContext<'_>, line: &Value) -> Option<Answer> {
    let id = line.get("custom_id")?.as_str()?.to_string();
    let status = line
        .pointer("/response/status_code")
        .and_then(Value::as_u64);
    let body = batch::first(line, &["/response/body", "/body", "/response"]);
    let failed = status.is_some_and(|s| s != 200)
        || line.get("error").is_some_and(|e| !e.is_null())
        || body.is_none();
    let answer = if failed {
        Err(batch::item_error(batch::first(
            line,
            &["/error", "/response/body/error", "/response/body"],
        )))
    } else {
        body.map_or_else(
            || Err(ProviderError::Other("Mistral returned no answer".into())),
            |body| read(context, body),
        )
    };
    Some((id, answer))
}

fn read(context: BatchContext<'_>, body: &Value) -> Result<ProcessOutput, ProviderError> {
    match (context.task, context.analysis) {
        (
            Task::Analysis | Task::Contents | Task::TextCorrection | Task::Translation,
            Some(fields),
        ) => analysis::finish(chat::parse(body)?, fields),
        (Task::Analysis | Task::Contents | Task::TextCorrection | Task::Translation, None) => Err(
            ProviderError::InvalidRequest("an analysis needs fields".into()),
        ),
        _ => Mistral::parse_ocr(body),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn reads_results_wrapped_or_bare() {
        let context = BatchContext {
            task: Task::TextExtraction,
            model: "mistral-ocr-4-1",
            analysis: None,
        };
        let page = json!({ "pages": [{ "markdown": "Page one" }], "usage_info": { "pages_processed": 1 } });
        let wrapped =
            json!({ "custom_id": "p0", "response": { "status_code": 200, "body": page } });
        let (id, text) = answer(context, &wrapped).unwrap();
        assert_eq!(
            (id.as_str(), text.unwrap().text.as_str()),
            ("p0", "Page one")
        );

        let bare = json!({ "custom_id": "p1", "body": page });
        assert!(answer(context, &bare).unwrap().1.is_ok());

        let failed = json!({ "custom_id": "p2", "response": { "status_code": 422,
            "body": { "message": "unreadable document" } } });
        assert_eq!(
            answer(context, &failed).unwrap().1,
            Err(ProviderError::InvalidRequest("unreadable document".into()))
        );
    }
}
