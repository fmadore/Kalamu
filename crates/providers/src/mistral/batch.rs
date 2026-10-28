//! Mistral batch jobs: OCR or chat requests sent inline to `/v1/batch/jobs`, at half price, with
//! a 24-hour limit. Results are read inline (`?inline=true`), or from the output file when the
//! job has one. The reference types each result only as a map, so the common shapes are read.

use kalamu_core::{
    ApiKey, BatchContext, BatchProgress, ProcessOutput, ProcessRequest, ProviderError, Task,
};
use serde_json::{Value, json};

use super::{Mistral, chat};
use crate::analysis;
use crate::batch::{self, Answer};
use crate::http::{send_json, send_text};

impl Mistral {
    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let Some((_, first)) = requests.first() else {
            return Err(ProviderError::InvalidRequest("an empty batch".into()));
        };
        let (endpoint, bodies) = match first.task {
            Task::TextExtraction => (
                "/v1/ocr",
                requests
                    .iter()
                    .map(|(id, r)| json!({ "custom_id": id, "body": Self::ocr_body(r) }))
                    .collect::<Vec<_>>(),
            ),
            Task::Analysis => {
                let mut bodies = Vec::new();
                for (id, request) in requests {
                    let fields = analysis::fields(request)?;
                    bodies.push(json!({ "custom_id": id, "body": chat::body(request, fields) }));
                }
                ("/v1/chat/completions", bodies)
            }
            Task::AudioTranscription => {
                return Err(ProviderError::Unsupported("batched transcription".into()));
            }
        };
        let created = send_json(
            self.client
                .post(format!("{}/batch/jobs", self.base_url))
                .bearer_auth(key.expose())
                .json(&json!({
                    "endpoint": endpoint,
                    "model": first.model,
                    "requests": bodies,
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
                results.extend(self.file_lines(key, &status, file).await?);
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
        (Task::Analysis, Some(fields)) => analysis::finish(chat::parse(body)?, fields),
        (Task::Analysis, None) => Err(ProviderError::InvalidRequest(
            "an analysis needs fields".into(),
        )),
        _ => Mistral::parse_ocr(body),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
