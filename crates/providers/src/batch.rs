//! What the four batch services share: a body written once, item by item; answers that come
//! back as JSON Lines or lists under the ids they were sent with, counts given as numbers or
//! strings, and a reason for each failed item that the researcher can read.

use kalamu_core::{ApiKey, ProcessOutput, ProcessRequest, ProviderError};
use serde_json::Value;

use crate::http::{send_json_if_found, send_text};

/// One answer of a batch, under the item id it was sent with.
pub(crate) type Answer = (String, Result<ProcessOutput, ProviderError>);

/// A batch's body, written straight into the one buffer that is sent: each item is built on
/// its own, written, and dropped, so a batch is held once as its pieces and once as the bytes
/// on their way, never also as a tree of JSON values or a second serialised copy.
pub(crate) struct Body {
    bytes: Vec<u8>,
    first: bool,
}

impl Body {
    /// A body that starts with `head` (the JSON before the list of requests, or nothing for
    /// JSON Lines), with room for `requests` so the buffer is not copied as it grows.
    pub(crate) fn new(head: &str, requests: &[(String, ProcessRequest<'_>)]) -> Self {
        let mut bytes = Vec::with_capacity(head.len() + room(requests));
        bytes.extend_from_slice(head.as_bytes());
        Self { bytes, first: true }
    }

    /// One item of a JSON list: an object of `fields`, in their order.
    pub(crate) fn item(&mut self, fields: &[(&str, &Value)]) -> Result<(), ProviderError> {
        if !self.first {
            self.bytes.push(b',');
        }
        self.first = false;
        self.object(fields)
    }

    /// One line of a JSON Lines file: an object of `fields`, in their order.
    pub(crate) fn line(&mut self, fields: &[(&str, &Value)]) -> Result<(), ProviderError> {
        self.object(fields)?;
        self.bytes.push(b'\n');
        Ok(())
    }

    /// The body, closed with `tail`.
    pub(crate) fn finish(mut self, tail: &str) -> Vec<u8> {
        self.bytes.extend_from_slice(tail.as_bytes());
        self.bytes
    }

    fn object(&mut self, fields: &[(&str, &Value)]) -> Result<(), ProviderError> {
        self.bytes.push(b'{');
        for (i, (name, value)) in fields.iter().enumerate() {
            if i > 0 {
                self.bytes.push(b',');
            }
            serde_json::to_writer(&mut self.bytes, name).map_err(unwritable)?;
            self.bytes.push(b':');
            serde_json::to_writer(&mut self.bytes, value).map_err(unwritable)?;
        }
        self.bytes.push(b'}');
        Ok(())
    }
}

/// What a batch's requests should take once written: each input as base64, its prompts, and
/// room for the settings around them. Only a size to reserve, so an estimate is enough.
fn room(requests: &[(String, ProcessRequest<'_>)]) -> usize {
    requests
        .iter()
        .map(|(id, r)| {
            r.input.bytes.len().div_ceil(3) * 4
                + r.prompt.len()
                + r.user_prompt.len()
                + id.len()
                + 2048
        })
        .sum()
}

/// How many bytes `value` takes once written as JSON, counted without keeping them: to decide
/// how a batch travels before writing it.
pub(crate) fn written_len(value: &Value) -> Result<usize, ProviderError> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len();
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).map_err(unwritable)?;
    Ok(counter.0)
}

fn unwritable(err: serde_json::Error) -> ProviderError {
    ProviderError::Other(format!("the batch could not be written: {err}"))
}

/// A count the provider may write as `12` or `"12"`.
pub(crate) fn count(value: Option<&Value>) -> u32 {
    match value {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// The first of several places a field may be, since some services document two shapes.
pub(crate) fn first<'a>(value: &'a Value, pointers: &[&str]) -> Option<&'a Value> {
    pointers
        .iter()
        .find_map(|p| value.pointer(p).filter(|v| !v.is_null()))
}

/// The lines of a JSON Lines file, skipping blank and unreadable ones.
pub(crate) fn lines(body: &str) -> Vec<Value> {
    body.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// The lines of the file a batch's status names under `field` (`/files/{id}/content`, as OpenAI
/// and Mistral serve them); none when the status names no file there.
pub(crate) async fn file_lines(
    client: &reqwest::Client,
    base_url: &str,
    key: &ApiKey,
    status: &Value,
    field: &str,
) -> Result<Vec<Value>, ProviderError> {
    let Some(file) = status.get(field).and_then(Value::as_str) else {
        return Ok(Vec::new());
    };
    let body = send_text(
        client
            .get(format!("{base_url}/files/{file}/content"))
            .bearer_auth(key.expose()),
    )
    .await?;
    Ok(lines(&body))
}

/// The files a batch's status names under `fields`, each holding one id or a list of ids;
/// missing and empty ones are skipped.
pub(crate) fn named_files(status: &Value, fields: &[&str]) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for field in fields {
        let ids = match status.get(*field) {
            Some(Value::String(id)) => vec![id.as_str()],
            Some(Value::Array(ids)) => ids.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        for id in ids {
            if !id.is_empty() && !files.iter().any(|f| f == id) {
                files.push(id.to_string());
            }
        }
    }
    files
}

/// The status of a batch at `url`, or `None` when the provider no longer knows it.
pub(crate) async fn status_if_known(
    client: &reqwest::Client,
    url: String,
    key: &ApiKey,
) -> Result<Option<Value>, ProviderError> {
    send_json_if_found(client.get(url).bearer_auth(key.expose())).await
}

/// Deletes a file a batch left (`DELETE /files/{id}`, as OpenAI and Mistral serve it); one
/// already gone (404) counts as deleted.
pub(crate) async fn delete_file(
    client: &reqwest::Client,
    base_url: &str,
    key: &ApiKey,
    file: &str,
) -> Result<(), ProviderError> {
    send_json_if_found(
        client
            .delete(format!("{base_url}/files/{file}"))
            .bearer_auth(key.expose()),
    )
    .await
    .map(|_| ())
}

/// A failed item, from the error object or message the service gives for it. An item is never
/// retried inside its batch, so the kind only decides what the researcher reads.
pub(crate) fn item_error(error: Option<&Value>) -> ProviderError {
    let message = error
        .and_then(|e| {
            e.as_str().map(str::to_string).or_else(|| {
                first(e, &["/message", "/error/message"])?
                    .as_str()
                    .map(str::to_string)
            })
        })
        .unwrap_or_else(|| "the provider gave no reason".into());
    ProviderError::InvalidRequest(message)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn counts_and_errors_are_read_in_either_shape() {
        assert_eq!(count(Some(&json!(12))), 12);
        assert_eq!(count(Some(&json!("7"))), 7);
        assert_eq!(count(None), 0);
        let v = json!({ "a": null, "b": { "c": 1 } });
        assert_eq!(first(&v, &["/a", "/b/c"]), Some(&json!(1)));
        assert_eq!(lines("{\"a\":1}\n\nnot json\n{\"a\":2}\n").len(), 2);
        // `{"a":[1,"é"]}`: é is two bytes.
        assert_eq!(written_len(&json!({ "a": [1, "é"] })).ok(), Some(14));
        assert_eq!(
            item_error(Some(&json!({ "message": "too big" }))),
            ProviderError::InvalidRequest("too big".into())
        );
        assert_eq!(
            item_error(Some(&json!({ "error": { "message": "bad" } }))),
            ProviderError::InvalidRequest("bad".into())
        );
    }

    #[test]
    fn a_batch_s_files_are_read_from_single_ids_and_lists() {
        let status = json!({
            "input_files": ["in-1", "in-2"],
            "output_file": "out",
            "error_file": null,
            "input_file_id": "in-1",
        });
        assert_eq!(
            named_files(
                &status,
                &["input_files", "input_file_id", "output_file", "error_file"]
            ),
            vec!["in-1", "in-2", "out"]
        );
        assert!(named_files(&json!({ "output_file": "" }), &["output_file"]).is_empty());
    }
}
