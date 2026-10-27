//! What the four batch services share: answers that come back as JSON Lines or lists under the
//! ids they were sent with, counts given as numbers or strings, and a reason for each failed
//! item that the researcher can read.

use scriptorium_core::{ProcessOutput, ProviderError};
use serde_json::Value;

/// One answer of a batch, under the item id it was sent with.
pub(crate) type Answer = (String, Result<ProcessOutput, ProviderError>);

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
