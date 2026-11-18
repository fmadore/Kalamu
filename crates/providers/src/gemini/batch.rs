//! Gemini batch mode: `generateContent` requests sent to `batchGenerateContent`, at half price
//! within 24 hours (a batch still waiting after 48 hours expires). The job comes back as a
//! long-running operation; Google's guide and reference place its state and answers at
//! different paths, so both are read.
//!
//! A batch whose requests total under 20 MB is sent inline and answered inside the batch. A
//! larger one is written as JSON Lines, one `{"key", "request"}` a line, uploaded through the
//! Files API (2 GB at most) with a resumable upload, and named by the batch
//! (`input_config.file_name`); its answers come back in a file Google writes (`responsesFile`),
//! one line a request under its key, read with `files/{id}:download?alt=media`.
//!
//! Google deletes an uploaded file after 48 hours. Once the batch has ended,
//! [`Gemini::files_of`] names the file it was made from and the file of its answers, from the
//! batch, and each is deleted with `DELETE /v1beta/files/{id}`. Google states neither how long it
//! keeps the file it writes nor that this deletes it: a refusal is retried, then left, as any
//! failed deletion is.

use kalamu_core::{ApiKey, BatchContext, BatchProgress, ProcessRequest, ProviderError};
use serde_json::{Value, json};

use super::Gemini;
use crate::batch::{self, Answer, Body};
use crate::http::{
    send_for_header, send_json, send_json_if_found, send_text, upload_timeout, with_json_body,
};

/// "Inline requests: suitable for smaller batches that keep the total request size under 20MB",
/// read as decimal megabytes, the smaller.
const INLINE_BATCH_BYTES: usize = 20_000_000;

const INLINE_HEAD: &str =
    r#"{"batch":{"display_name":"kalamu","input_config":{"requests":{"requests":["#;
const INLINE_TAIL: &str = "]}}}}";

/// Where a batch's state and its two files may be, in the reference's shape and the guide's.
const STATE: &[&str] = &["/metadata/state", "/state"];
const INPUT_FILE: &[&str] = &[
    "/metadata/inputConfig/fileName",
    "/inputConfig/fileName",
    "/metadata/input_config/file_name",
    "/input_config/file_name",
];
const RESPONSES_FILE: &[&str] = &[
    "/response/responsesFile",
    "/metadata/output/responsesFile",
    "/output/responsesFile",
    "/dest/fileName",
    "/dest/file_name",
];

impl Gemini {
    /// Uploads the requests as a JSON Lines file when they are too large to send inline, and
    /// returns the file's name (`files/…`); `None` for a batch that goes inline.
    pub(super) async fn upload(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<Option<String>, ProviderError> {
        if requests.is_empty() || fits_inline(requests)? {
            return Ok(None);
        }
        let mut jsonl = Body::new("", requests);
        for (id, request) in requests {
            jsonl.line(&[
                ("key", &Value::from(id.as_str())),
                ("request", &Self::body(request)),
            ])?;
        }
        self.upload_file(key, jsonl.finish("")).await.map(Some)
    }

    /// The Files API's resumable upload: a session opened with the file's size and type, then
    /// the bytes sent to the address it gives, in one request given time for their size. The key
    /// goes only to the API's own address, not to the session's, which needs none.
    async fn upload_file(&self, key: &ApiKey, jsonl: Vec<u8>) -> Result<String, ProviderError> {
        let session = send_for_header(
            self.client
                .post(format!("{}/files", self.upload_url))
                .header("x-goog-api-key", key.expose())
                .header("X-Goog-Upload-Protocol", "resumable")
                .header("X-Goog-Upload-Command", "start")
                .header("X-Goog-Upload-Header-Content-Length", jsonl.len())
                .header("X-Goog-Upload-Header-Content-Type", "application/jsonl")
                .json(&json!({ "file": { "display_name": "kalamu-batch" } })),
            "x-goog-upload-url",
        )
        .await?
        .ok_or_else(|| ProviderError::Other("Google gave no address to upload the batch".into()))?;
        let uploaded = send_json(
            self.client
                .post(session)
                .timeout(upload_timeout(jsonl.len()))
                .header("X-Goog-Upload-Offset", "0")
                .header("X-Goog-Upload-Command", "upload, finalize")
                .body(jsonl),
        )
        .await?;
        batch::first(&uploaded, &["/file/name", "/name"])
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Other("Google gave the batch file no name".into()))
    }

    pub(super) async fn submit(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let Some((_, first)) = requests.first() else {
            return Err(ProviderError::InvalidRequest("an empty batch".into()));
        };
        let mut body = Body::new(INLINE_HEAD, requests);
        for (id, request) in requests {
            let metadata = json!({ "key": id });
            body.item(&[("metadata", &metadata), ("request", &Self::body(request))])?;
        }
        let created = send_json(with_json_body(
            self.client
                .post(self.create_url(first.model))
                .header("x-goog-api-key", key.expose()),
            body.finish(INLINE_TAIL),
        ))
        .await?;
        batch_name(&created)
    }

    /// Makes the batch from the file [`Gemini::upload`] uploaded.
    pub(super) async fn submit_file(
        &self,
        key: &ApiKey,
        file: &str,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let Some((_, first)) = requests.first() else {
            return Err(ProviderError::InvalidRequest("an empty batch".into()));
        };
        let created = send_json(
            self.client
                .post(self.create_url(first.model))
                .header("x-goog-api-key", key.expose())
                .json(&json!({ "batch": {
                    "display_name": "kalamu",
                    "input_config": { "file_name": file },
                } })),
        )
        .await?;
        batch_name(&created)
    }

    fn create_url(&self, model: &str) -> String {
        format!("{}/models/{model}:batchGenerateContent", self.base_url)
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
        if state(&status).ends_with("SUCCEEDED")
            && let Some(file) = named_file(&status, RESPONSES_FILE)
        {
            let body = send_text(
                self.client
                    .get(format!("{}/{file}:download?alt=media", self.download_url))
                    .header("x-goog-api-key", key.expose()),
            )
            .await?;
            return Ok(BatchProgress::Ended(
                batch::lines(&body)
                    .iter()
                    .enumerate()
                    .filter_map(|(position, line)| answer(context, ids, position, line))
                    .collect(),
            ));
        }
        Ok(progress(&status, context, ids))
    }

    /// The files an ended batch left: the one it was made from and the one of its answers.
    /// An inline batch names none. `None` while it may still change (pending, running).
    pub(super) async fn files_of(
        &self,
        key: &ApiKey,
        batch_id: &str,
    ) -> Result<Option<Vec<String>>, ProviderError> {
        let Some(status) = send_json_if_found(
            self.client
                .get(format!("{}/{batch_id}", self.base_url))
                .header("x-goog-api-key", key.expose()),
        )
        .await?
        else {
            return Ok(Some(Vec::new()));
        };
        if !ended(state(&status)) {
            return Ok(None);
        }
        let mut files: Vec<String> = Vec::new();
        for file in [
            named_file(&status, INPUT_FILE),
            named_file(&status, RESPONSES_FILE),
        ]
        .into_iter()
        .flatten()
        {
            if !files.iter().any(|f| f == file) {
                files.push(file.to_string());
            }
        }
        Ok(Some(files))
    }

    /// Deletes one of a batch's files (`files/…`); one already gone (404) counts as deleted.
    /// Nothing but a file is ever deleted here.
    pub(super) async fn delete_file(&self, key: &ApiKey, file: &str) -> Result<(), ProviderError> {
        if !is_file(file) {
            return Err(ProviderError::InvalidRequest(format!(
                "{file} is not a file of Gemini's"
            )));
        }
        send_json_if_found(
            self.client
                .delete(format!("{}/{file}", self.base_url))
                .header("x-goog-api-key", key.expose()),
        )
        .await
        .map(|_| ())
    }
}

/// Whether the requests, written as the inline batch, stay under Google's limit for it. Each is
/// counted, not kept; the count stops as soon as it passes.
fn fits_inline(requests: &[(String, ProcessRequest<'_>)]) -> Result<bool, ProviderError> {
    let mut len = INLINE_HEAD.len() + INLINE_TAIL.len() + requests.len().saturating_sub(1);
    for (id, request) in requests {
        len += batch::written_len(&json!({
            "metadata": { "key": id },
            "request": Gemini::body(request),
        }))?;
        if len >= INLINE_BATCH_BYTES {
            return Ok(false);
        }
    }
    Ok(true)
}

fn batch_name(created: &Value) -> Result<String, ProviderError> {
    created
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| ProviderError::Other("Gemini gave the batch no name".into()))
}

fn state(status: &Value) -> &str {
    batch::first(status, STATE)
        .and_then(Value::as_str)
        .unwrap_or("")
}

/// A batch that will change no more. States are `JOB_STATE_…` in the guide and `BATCH_STATE_…`
/// in the reference, so only their endings are compared.
fn ended(state: &str) -> bool {
    ["SUCCEEDED", "FAILED", "CANCELLED", "EXPIRED"]
        .iter()
        .any(|end| state.ends_with(end))
}

fn is_file(name: &str) -> bool {
    name.strip_prefix("files/")
        .is_some_and(|id| !id.is_empty() && !id.contains('/'))
}

/// A file the batch names at one of `pointers`, if it is one.
fn named_file<'a>(status: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    batch::first(status, pointers)
        .and_then(Value::as_str)
        .filter(|name| is_file(name))
}

/// Where a batch operation stands, its answers read from inside it.
fn progress(status: &Value, context: BatchContext<'_>, ids: &[String]) -> BatchProgress {
    let state = state(status);
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
    if ended(state) {
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

/// One answer, inline or a line of the answers' file, under the key it was sent with, or in
/// the place it was sent when the key is not echoed back.
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
    use kalamu_core::Task;

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

    #[test]
    fn only_files_are_named_or_deleted() {
        assert!(is_file("files/batch-abc123"));
        assert!(!is_file("batches/123"));
        assert!(!is_file("files/"));
        assert!(!is_file("files/a/../../batches/1"));
        let status = json!({ "metadata": { "state": "BATCH_STATE_SUCCEEDED",
            "inputConfig": { "fileName": "files/in" } },
            "response": { "responsesFile": "batches/1" } });
        assert_eq!(named_file(&status, INPUT_FILE), Some("files/in"));
        assert_eq!(named_file(&status, RESPONSES_FILE), None);
        assert!(ended("BATCH_STATE_CANCELLED") && !ended("JOB_STATE_PENDING"));
    }
}
