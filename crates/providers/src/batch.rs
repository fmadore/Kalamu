//! What the four batch services share: a body written once, item by item; answers that come
//! back as JSON Lines or lists under the ids they were sent with, counts given as numbers or
//! strings, and a reason for each failed item that the researcher can read.

use kalamu_core::{ApiKey, ProcessOutput, ProcessRequest, ProviderError};
use serde_json::Value;

use crate::http::send_text;

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

fn unwritable(err: serde_json::Error) -> ProviderError {
    ProviderError::Other(format!("the batch could not be written: {err}"))
}

/// A JSON string, quoted and escaped, for a body's head.
pub(crate) fn quoted(text: &str) -> String {
    Value::from(text).to_string()
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
        assert_eq!(
            item_error(Some(&json!({ "message": "too big" }))),
            ProviderError::InvalidRequest("too big".into())
        );
        assert_eq!(
            item_error(Some(&json!({ "error": { "message": "bad" } }))),
            ProviderError::InvalidRequest("bad".into())
        );
    }
}
