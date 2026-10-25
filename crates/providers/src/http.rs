//! What every provider shares: the HTTP client and turning HTTP failures into
//! [`ProviderError`]s the engine knows how to handle.

use std::time::Duration;

use base64::Engine as _;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use scriptorium_core::{ProviderError, Segment};
use serde_json::Value;

/// One client for all providers: it keeps connections alive and follows the system proxy, which
/// many institutional networks require.
pub fn client() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        // Long handwritten pages and recordings can take minutes on the slower models.
        .timeout(Duration::from_secs(600))
        .user_agent(concat!("Scriptorium/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
}

pub(crate) fn base64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(crate) fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64(bytes))
}

/// Sends a request and returns the JSON body of a successful response.
pub(crate) async fn send_json(request: RequestBuilder) -> Result<Value, ProviderError> {
    let response = request.send().await.map_err(network)?;
    let status = response.status();
    let retry_after = header_retry_after(&response);
    let body = response.text().await.map_err(network)?;
    if status.is_success() {
        return serde_json::from_str(&body)
            .map_err(|e| ProviderError::Other(format!("invalid JSON: {e}")));
    }
    Err(status_error(status, retry_after, &body))
}

fn network(err: reqwest::Error) -> ProviderError {
    if err.is_timeout() {
        ProviderError::Transient("the request timed out".into())
    } else if err.is_connect() {
        ProviderError::Transient("could not connect; check the internet connection".into())
    } else {
        ProviderError::Transient(err.to_string())
    }
}

fn header_retry_after(response: &Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
        .map(Duration::from_secs_f64)
}

/// The provider's own error message, from the shapes the four providers use.
pub(crate) fn error_message(body: &str) -> String {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let message = parsed.as_ref().and_then(|v| {
        v.pointer("/error/message")
            .or_else(|| v.pointer("/message"))
            .or_else(|| v.pointer("/detail"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    let message = message.unwrap_or_else(|| body.chars().take(300).collect());
    message.trim().to_string()
}

/// "Please retry in 12.5s", as Gemini words its per-minute limits.
fn stated_retry(message: &str) -> Option<Duration> {
    let lower = message.to_lowercase();
    let at = lower.find("retry in ")? + "retry in ".len();
    let digits: String = lower[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let secs: f64 = digits.parse().ok()?;
    let unit = lower[at + digits.len()..].trim_start();
    let secs = if unit.starts_with("ms") {
        secs / 1000.0
    } else {
        secs
    };
    Some(Duration::from_secs_f64(secs.min(300.0)))
}

pub(crate) fn status_error(
    status: StatusCode,
    retry_after: Option<Duration>,
    body: &str,
) -> ProviderError {
    let message = error_message(body);
    let lower = message.to_lowercase();
    let quota = lower.contains("credit balance")
        || lower.contains("insufficient_quota")
        || lower.contains("exceeded your current quota")
        || lower.contains("billing")
        || (lower.contains("quota") && lower.contains("per day"));
    match status.as_u16() {
        401 | 403 => ProviderError::Auth(message),
        402 => ProviderError::QuotaExhausted(message),
        400 if lower.contains("api key not valid") || lower.contains("api_key_invalid") => {
            ProviderError::Auth(message)
        }
        _ if quota => ProviderError::QuotaExhausted(message),
        429 => ProviderError::RateLimited {
            retry_after: retry_after.or_else(|| stated_retry(&message)),
            message,
        },
        408 | 409 | 425 | 500 | 502 | 503 | 504 | 529 => ProviderError::Transient(message),
        413 => ProviderError::InvalidRequest(format!("the page or file is too large: {message}")),
        _ => ProviderError::InvalidRequest(message),
    }
}

/// `data[].id`, the list shape Anthropic, OpenAI and Mistral share.
pub(crate) fn ids(response: &Value) -> Vec<String> {
    response
        .get("data")
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(|m| m.get("id").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Seconds as a JSON number, in milliseconds.
pub(crate) fn secs_to_ms(value: Option<&Value>) -> u64 {
    value
        .and_then(Value::as_f64)
        .map(|s| (s * 1000.0).round().max(0.0) as u64)
        .unwrap_or(0)
}

/// `Speaker 1` from what a diarizing model calls a speaker: `A` (OpenAI), `speaker_1` (Voxtral)
/// or a number.
pub(crate) fn speaker_label(id: &str) -> String {
    let id = id.trim();
    let bare = id
        .get(..8)
        .filter(|p| p.eq_ignore_ascii_case("speaker_") || p.eq_ignore_ascii_case("speaker "))
        .map_or(id, |_| id[8..].trim());
    format!("Speaker {bare}")
}

/// Longest turn made by [`join_turns`], so subtitle cues cut from it keep close timings.
const MAX_TURN_MS: u64 = 30_000;
/// Longest pause inside one turn.
const MAX_PAUSE_MS: u64 = 2_000;

/// Consecutive segments from the same speaker joined into one turn. Diarizing models return a
/// segment per clause, which reads as a new line at every comma.
pub(crate) fn join_turns(segments: Vec<Segment>) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::with_capacity(segments.len());
    for seg in segments {
        if let Some(last) = out.last_mut()
            && seg.speaker.is_some()
            && last.speaker == seg.speaker
            && seg.start_ms.saturating_sub(last.end_ms) <= MAX_PAUSE_MS
            && seg.end_ms.saturating_sub(last.start_ms) <= MAX_TURN_MS
        {
            last.text = format!("{} {}", last.text, seg.text).trim().to_string();
            last.end_ms = last.end_ms.max(seg.end_ms);
            continue;
        }
        out.push(seg);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: u64, end: u64, speaker: &str, text: &str) -> Segment {
        Segment {
            start_ms: start,
            end_ms: end,
            speaker: Some(speaker.into()),
            text: text.into(),
        }
    }

    #[test]
    fn speakers_are_named_alike_whatever_the_model_calls_them() {
        assert_eq!(speaker_label("A"), "Speaker A");
        assert_eq!(speaker_label("speaker_1"), "Speaker 1");
        assert_eq!(speaker_label("0"), "Speaker 0");
        assert_eq!(speaker_label("Speakerine"), "Speaker Speakerine");
    }

    #[test]
    fn clauses_from_one_speaker_become_one_turn() {
        let turns = join_turns(vec![
            seg(0, 900, "Speaker A", "Bonjour,"),
            seg(1_000, 2_000, "Speaker A", "merci de me recevoir."),
            seg(2_200, 4_000, "Speaker B", "Je vous en prie."),
            // A long pause starts a new turn.
            seg(7_000, 8_000, "Speaker B", "Alors,"),
        ]);
        let texts: Vec<&str> = turns.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Bonjour, merci de me recevoir.",
                "Je vous en prie.",
                "Alors,"
            ]
        );
        assert_eq!((turns[0].start_ms, turns[0].end_ms), (0, 2_000));
        // Turns stay short enough for subtitles.
        let long: Vec<Segment> = (0..40)
            .map(|i| seg(i * 1_000, i * 1_000 + 900, "Speaker A", "mot"))
            .collect();
        assert_eq!(join_turns(long).len(), 2);
    }

    #[test]
    fn errors_are_sorted_by_what_to_do_next() {
        let rate = status_error(
            StatusCode::TOO_MANY_REQUESTS,
            None,
            r#"{"error":{"message":"Resource exhausted. Please retry in 12.5s."}}"#,
        );
        assert_eq!(
            rate,
            ProviderError::RateLimited {
                retry_after: Some(Duration::from_secs_f64(12.5)),
                message: "Resource exhausted. Please retry in 12.5s.".into()
            }
        );
        assert!(matches!(
            status_error(
                StatusCode::BAD_REQUEST,
                None,
                r#"{"error":{"message":"API key not valid."}}"#
            ),
            ProviderError::Auth(_)
        ));
        assert!(matches!(
            status_error(
                StatusCode::BAD_REQUEST,
                None,
                r#"{"type":"error","error":{"message":"Your credit balance is too low"}}"#
            ),
            ProviderError::QuotaExhausted(_)
        ));
        assert!(status_error(StatusCode::SERVICE_UNAVAILABLE, None, "busy").is_retryable());
        assert!(!status_error(StatusCode::BAD_REQUEST, None, "bad").is_retryable());
    }
}
