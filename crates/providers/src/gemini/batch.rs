//! Gemini batch mode: `generateContent` requests sent inline to `batchGenerateContent`, at half
//! price within 24 hours (a batch still waiting after 48 hours expires). The job comes back as a
//! long-running operation; Google's guide and reference place its state and answers at
//! different paths, so both are read.

use scriptorium_core::{ApiKey, BatchContext, BatchProgress, ProcessRequest, ProviderError};
use serde_json::{Value, json};

use super::Gemini;
use crate::batch::{self, Answer};
use crate::http::send_json;

impl Gemini {
    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let Some((_, first)) = requests.first() else {
            return Err(ProviderError::InvalidRequest("an empty batch".into()));
        };
        let inline: Vec<Value> = requests
            .iter()
            .map(|(id, request)| json!({ "request": Self::body(request), "metadata": { "key": id } }))
            .collect();
        let url = format!(
            "{}/models/{}:batchGenerateContent",
            self.base_url, first.model
        );
        let created = send_json(
            self.client
                .post(url)
                .header("x-goog-api-key", key.expose())
                .json(&json!({
                    "batch": {
                        "display_name": "scriptorium",
                        "input_config": { "requests": { "requests": inline } },
                    }
                })),
        )
        .await?;
        created
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Other("Gemini gave the batch no name".into()))
    }

    pub(super) async fn poll(
        &self,
        key: &ApiKey,
        batch_id: &str,
        context: BatchContext<'_>,
        ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        let status = send_json(
            self.client
                .get(format!("{}/{batch_id}", self.base_url))
                .header("x-goog-api-key", key.expose()),
        )
        .await?;
        Ok(progress(&status, context, ids))
    }
}

/// Where a batch operation stands. States are `JOB_STATE_…` in the guide and `BATCH_STATE_…` in
/// the reference, so only their endings are compared.
fn progress(status: &Value, context: BatchContext<'_>, ids: &[String]) -> BatchProgress {
    let state = batch::first(status, &["/metadata/state", "/state"])
        .and_then(Value::as_str)
        .unwrap_or("");
    let stats = |name: &str| {
        batch::count(batch::first(
            status,
            &[
                &format!("/metadata/batchStats/{name}"),
                &format!("/batchStats/{name}"),
            ],
        ))
    };
    if state.ends_with("SUCCEEDED") {
        let answers = batch::first(
            status,
            &[
                "/response/inlinedResponses/inlinedResponses",
                "/response/inlinedResponses",
                "/metadata/output/inlinedResponses/inlinedResponses",
                "/output/inlinedResponses/inlinedResponses",
                "/dest/inlinedResponses",
            ],
        )
        .and_then(Value::as_array);
        return BatchProgress::Ended(
            answers
                .into_iter()
                .flatten()
                .enumerate()
                .filter_map(|(position, item)| answer(context, ids, position, item))
                .collect(),
        );
    }
    if state.ends_with("FAILED") || state.ends_with("CANCELLED") || state.ends_with("EXPIRED") {
        let reason = status
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or(state);
        return BatchProgress::Abandoned(reason.to_string());
    }
    let finished = stats("successfulRequestCount") + stats("failedRequestCount");
    BatchProgress::Working {
        finished,
        total: stats("requestCount").max(finished),
    }
}

/// One inline answer, under the key it was sent with, or in the place it was sent when the key
/// is not echoed back.
fn answer(
    context: BatchContext<'_>,
    ids: &[String],
    position: usize,
    item: &Value,
) -> Option<Answer> {
    let id = batch::first(item, &["/metadata/key", "/key"])
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| ids.get(position).cloned())?;
    let answer = match item.get("response") {
        Some(response) => Gemini::read(context, response),
        None => Err(batch::item_error(item.get("error"))),
    };
    Some((id, answer))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use scriptorium_core::Task;

    use super::*;

    fn context() -> BatchContext<'static> {
        BatchContext {
            task: Task::TextExtraction,
            model: "gemini-3.8-flash",
            analysis: None,
        }
    }

    #[test]
    fn reads_either_documented_shape() {
        let ids = ["p0".to_string(), "p1".to_string()];
        let running = json!({ "metadata": { "state": "BATCH_STATE_RUNNING",
            "batchStats": { "requestCount": "2", "successfulRequestCount": "1" } } });
        assert!(matches!(
            progress(&running, context(), &ids),
            BatchProgress::Working {
                finished: 1,
                total: 2
            }
        ));

        let page = json!({ "candidates": [{ "content": { "parts": [{ "text": "Page" }] } }] });
        let done = json!({ "state": "JOB_STATE_SUCCEEDED", "dest": { "inlinedResponses": [
            { "response": page },
            { "metadata": { "key": "p0" }, "error": { "message": "too large" } },
        ] } });
        let BatchProgress::Ended(answers) = progress(&done, context(), &ids) else {
            panic!("not ended");
        };
        assert_eq!(answers[0].0, "p0");
        assert_eq!(answers[0].1.as_ref().unwrap().text, "Page");
        assert_eq!(answers[1].0, "p0");
        assert!(answers[1].1.is_err());

        let expired = json!({ "metadata": { "state": "JOB_STATE_EXPIRED" } });
        assert!(matches!(
            progress(&expired, context(), &ids),
            BatchProgress::Abandoned(_)
        ));
    }
}
