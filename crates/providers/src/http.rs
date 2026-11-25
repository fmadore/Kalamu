//! What every provider shares: the HTTP client and turning HTTP failures into
//! [`ProviderError`]s the engine knows how to handle.

use std::time::Duration;

use base64::Engine as _;
use kalamu_core::{ProviderError, Segment};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde_json::Value;

/// One client for all providers: it keeps connections alive and follows the system proxy, which
/// many institutional networks require.
pub fn client() -> Client {
    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        // Long handwritten pages and recordings can take minutes on the slower models, and
        // Google advises at least 15 minutes for hard prompts on Gemini 4 Argon.
        .timeout(Duration::from_secs(900))
        .user_agent(concat!("Kalamu/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
}

pub(crate) fn base64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub(crate) fn data_url(mime: &str, bytes: &[u8]) -> String {
    format!("data:{mime};base64,{}", base64(bytes))
}

/// The slowest upload a batch is given time for: 1 Mbit/s, a weak connection's upstream.
const SLOWEST_UPLOAD_BYTES_PER_SEC: u64 = 128 * 1024;

/// What a request that uploads `bytes` is given before it is called timed out: five minutes
/// for the provider to answer, plus the body at the slowest upload. A batch of 64 MB has about
/// thirteen minutes, one of 200 MB half an hour; a fixed fifteen minutes cut the large ones off
/// while they were still being sent.
pub(crate) fn upload_timeout(bytes: usize) -> Duration {
    Duration::from_secs(300 + bytes as u64 / SLOWEST_UPLOAD_BYTES_PER_SEC)
}

/// A request carrying a JSON body already written (a batch's), given the time to upload it.
/// The bytes become the request's body as they are, without another copy.
pub(crate) fn with_json_body(request: RequestBuilder, body: Vec<u8>) -> RequestBuilder {
    request
        .timeout(upload_timeout(body.len()))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
}

/// Sends a request and returns the JSON body of a successful response.
pub(crate) async fn send_json(request: RequestBuilder) -> Result<Value, ProviderError> {
    let body = send_text(request).await?;
    serde_json::from_str(&body).map_err(|e| ProviderError::Other(format!("invalid JSON: {e}")))
}

/// Sends a request and returns the body of a successful response as text, for the JSON Lines
/// files batch services answer with.
pub(crate) async fn send_text(request: RequestBuilder) -> Result<String, ProviderError> {
    let response = request.send().await.map_err(network)?;
    let status = response.status();
    let retry_after = header_retry_after(&response);
    let who = answered_by(response.url());
    let body = response.text().await.map_err(network)?;
    if status.is_success() {
        return Ok(body);
    }
    Err(status_error(status, retry_after, &body, who))
}

/// Sends a request and returns the header `name` of a successful response, for an upload
/// session that answers with where to send the bytes.
pub(crate) async fn send_for_header(
    request: RequestBuilder,
    name: &str,
) -> Result<Option<String>, ProviderError> {
    let response = request.send().await.map_err(network)?;
    let status = response.status();
    let retry_after = header_retry_after(&response);
    let who = answered_by(response.url());
    let value = response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = response.text().await.map_err(network)?;
    if status.is_success() {
        return Ok(value);
    }
    Err(status_error(status, retry_after, &body, who))
}

/// Sends a request and returns the JSON body of a successful response, or `None` when the
/// provider answers that what was asked for does not exist (404): a batch it no longer knows,
/// a file already deleted.
pub(crate) async fn send_json_if_found(
    request: RequestBuilder,
) -> Result<Option<Value>, ProviderError> {
    let response = request.send().await.map_err(network)?;
    let status = response.status();
    let retry_after = header_retry_after(&response);
    let who = answered_by(response.url());
    let body = response.text().await.map_err(network)?;
    if status == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(status_error(status, retry_after, &body, who));
    }
    if body.trim().is_empty() {
        return Ok(Some(Value::Null));
    }
    serde_json::from_str(&body)
        .map(Some)
        .map_err(|e| ProviderError::Other(format!("invalid JSON: {e}")))
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

/// A `Retry-After` header's seconds, as asked: [`status_error`] bounds them.
fn header_retry_after(response: &Response) -> Option<f64> {
    retry_after_secs(
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)?
            .to_str()
            .ok()?,
    )
}

/// The longest wait a provider may ask for before a retry. A longer one would hold a slot of
/// the job for nothing: the call is tried again after this, and asked to wait again if need be.
const MAX_RETRY_WAIT_SECS: f64 = 300.0;

/// A `Retry-After` header in seconds. An HTTP date, a negative or a non-finite number is
/// ignored, and the engine falls back to its own delay.
fn retry_after_secs(value: &str) -> Option<f64> {
    asked(value.trim().parse::<f64>().ok()?)
}

/// A wait in seconds as asked, not yet bounded; `None` for one that is negative or not a number.
fn asked(secs: f64) -> Option<f64> {
    (secs.is_finite() && secs >= 0.0).then_some(secs)
}

/// A wait in seconds as asked, capped at [`MAX_RETRY_WAIT_SECS`]; `None` for one that is
/// negative or not a number.
fn wait(secs: f64) -> Option<Duration> {
    asked(secs).map(|secs| Duration::from_secs_f64(secs.min(MAX_RETRY_WAIT_SECS)))
}

/// The longest error message passed on as the provider wrote it, in characters.
const MAX_MESSAGE_CHARS: usize = 300;

/// The providers whose services Kalamu calls, by the domain of their addresses, as a researcher
/// knows them.
const PROVIDER_DOMAINS: [(&str, &str); 4] = [
    ("mistral.ai", "Mistral"),
    ("openai.com", "OpenAI"),
    ("anthropic.com", "Anthropic"),
    ("googleapis.com", "Google"),
];

/// Who answered a request with an error, as [`error_message`] names it: the provider whose
/// service the address belongs to, or "The provider" for any other (a proxy, a test server).
fn answered_by(url: &reqwest::Url) -> &'static str {
    let host = url.host_str().unwrap_or_default();
    PROVIDER_DOMAINS
        .iter()
        .find(|(domain, _)| {
            host == *domain
                || host
                    .strip_suffix(domain)
                    .is_some_and(|sub| sub.ends_with('.'))
        })
        .map_or("The provider", |(_, name)| name)
}

/// What a researcher reads of an HTTP error's body: the provider's own message, from the shapes
/// the four providers use (`detail` as FastAPI lists it included), cut to a readable length; a
/// short plain text as it is; and for an error page in HTML, an empty body or a long one that is
/// no message, one sentence naming who answered and the status, "Mistral refused the request
/// (400 Bad Request)." rather than the page.
pub(crate) fn error_message(body: &str, status: StatusCode, who: &str) -> String {
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    let message = parsed.as_ref().and_then(json_message);
    let body = body.trim();
    let plain = !body.is_empty()
        && parsed.is_none()
        && body.chars().count() <= MAX_MESSAGE_CHARS
        && !looks_like_markup(body);
    match message.or(plain.then_some(body)) {
        Some(message) => shortened(message),
        None if status.is_server_error() => {
            format!(
                "{who} could not answer the request ({}).",
                status_words(status)
            )
        }
        None => format!("{who} refused the request ({}).", status_words(status)),
    }
}

/// The provider's own message in a JSON error body, whole, from the shapes the four providers
/// use.
fn json_message(body: &Value) -> Option<&str> {
    [
        "/error/message",
        "/message",
        "/detail",
        "/detail/0/msg",
        "/error",
    ]
    .iter()
    .filter_map(|at| body.pointer(at)?.as_str().map(str::trim))
    .find(|m| !m.is_empty())
}

/// Whether a body is a page or a document of markup rather than a message: it opens with a tag.
fn looks_like_markup(body: &str) -> bool {
    body.starts_with('<')
}

/// "400 Bad Request": the status's number and its standard words, or the number alone.
fn status_words(status: StatusCode) -> String {
    match status.canonical_reason() {
        Some(reason) => format!("{} {reason}", status.as_u16()),
        None => status.as_u16().to_string(),
    }
}

/// A message cut to [`MAX_MESSAGE_CHARS`], with an ellipsis where it was cut.
fn shortened(message: &str) -> String {
    if message.chars().count() <= MAX_MESSAGE_CHARS {
        return message.to_string();
    }
    let cut: String = message.chars().take(MAX_MESSAGE_CHARS - 1).collect();
    format!("{}…", cut.trim_end())
}

/// The wait a message states, in seconds as asked: "Please retry in 12.5s", as Gemini words
/// its per-minute limits, or "Please try again in 1.2s", "in 20ms", "in 6m0s", as OpenAI does.
fn stated_secs(message: &str) -> Option<f64> {
    let lower = message.to_lowercase();
    let at = ["retry in ", "try again in "]
        .iter()
        .filter_map(|words| lower.find(words).map(|i| i + words.len()))
        .min()?;
    duration_secs(&lower[at..])
}

/// The duration a text opens with, in seconds: "33s", "1.2 s", "20ms", "6m0s", "1h"; a number
/// with no unit is seconds.
fn duration_secs(text: &str) -> Option<f64> {
    let mut rest = text;
    let mut total = None;
    while rest.starts_with(|c: char| c.is_ascii_digit()) {
        let len = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        let number: f64 = rest[..len].parse().ok()?;
        rest = rest[len..].trim_start();
        let (scale, unit) = if rest.starts_with("ms") {
            (0.001, 2)
        } else if rest.starts_with('h') {
            (3600.0, 1)
        } else if rest.starts_with('m') {
            (60.0, 1)
        } else if rest.starts_with('s') {
            (1.0, 1)
        } else {
            (1.0, 0)
        };
        total = Some(total.unwrap_or(0.0) + number * scale);
        rest = &rest[unit..];
        if unit == 0 {
            break;
        }
    }
    asked(total?)
}

/// The wait Gemini states in a `google.rpc.RetryInfo` detail of its error: `"retryDelay":
/// "33s"`.
fn retry_info_secs(body: &Value) -> Option<f64> {
    body.pointer("/error/details")?
        .as_array()?
        .iter()
        .filter_map(|detail| detail.get("retryDelay")?.as_str())
        .find_map(duration_secs)
}

/// An HTTP failure as the engine handles it. A 429 that states a wait (a `Retry-After` header,
/// Gemini's `RetryInfo`, or words in the message) is a limit to wait out, whatever else the
/// message says: Google words its per-minute limits as an exceeded quota (#270). An exhausted
/// account still stops the job: a 402, OpenAI's `insufficient_quota`, a low credit balance, a
/// quota per day, a wait longer than [`MAX_RETRY_WAIT_SECS`], or quota words with no wait.
pub(crate) fn status_error(
    status: StatusCode,
    retry_after: Option<f64>,
    body: &str,
    who: &str,
) -> ProviderError {
    let message = error_message(body, status, who);
    let parsed: Option<Value> = serde_json::from_str(body).ok();
    // The whole message: the wait comes last, after the cut a researcher reads.
    let whole = parsed.as_ref().and_then(json_message).unwrap_or(body);
    let lower = whole.to_lowercase();
    let raw = body.to_lowercase();
    let quota = lower.contains("credit balance")
        || raw.contains("insufficient_quota")
        || lower.contains("exceeded your current quota")
        || lower.contains("billing")
        || (lower.contains("quota") && lower.contains("per day"));
    // What no wait cures, wherever the body says it: Gemini names a daily quota in its
    // `QuotaFailure` detail ("GenerateRequestsPerDayPerProjectPerModel").
    let exhausted = raw.contains("insufficient_quota")
        || lower.contains("credit balance")
        || raw.contains("per day")
        || raw.contains("perday")
        || raw.contains("daily");
    let stated = retry_after
        .or_else(|| parsed.as_ref().and_then(retry_info_secs))
        .or_else(|| stated_secs(whole));
    let waits = stated.is_some_and(|secs| secs <= MAX_RETRY_WAIT_SECS) && !exhausted;
    let retry_after = stated.and_then(wait);
    match status.as_u16() {
        401 | 403 => ProviderError::Auth(message),
        402 => ProviderError::QuotaExhausted(message),
        400 if lower.contains("api key not valid") || lower.contains("api_key_invalid") => {
            ProviderError::Auth(message)
        }
        429 if waits => ProviderError::RateLimited {
            retry_after,
            message,
        },
        _ if quota => ProviderError::QuotaExhausted(message),
        429 => ProviderError::RateLimited {
            retry_after,
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

/// `Speaker 1` from what a diarizing model calls a speaker: `A` (OpenAI), `speaker_1` (Voxtral),
/// `spk_1` (Gemini) or a number.
pub(crate) fn speaker_label(id: &str) -> String {
    let id = id.trim();
    let bare = ["speaker_", "speaker ", "spk_"]
        .iter()
        .find_map(|prefix| {
            id.get(..prefix.len())
                .filter(|p| p.eq_ignore_ascii_case(prefix))
                .map(|_| id[prefix.len()..].trim())
        })
        .unwrap_or(id);
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
        assert_eq!(speaker_label("spk_2"), "Speaker 2");
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
            "Google",
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
                r#"{"error":{"message":"API key not valid."}}"#,
                "Google"
            ),
            ProviderError::Auth(_)
        ));
        assert!(matches!(
            status_error(
                StatusCode::BAD_REQUEST,
                None,
                r#"{"type":"error","error":{"message":"Your credit balance is too low"}}"#,
                "Anthropic"
            ),
            ProviderError::QuotaExhausted(_)
        ));
        assert!(
            status_error(StatusCode::SERVICE_UNAVAILABLE, None, "busy", "Mistral").is_retryable()
        );
        assert!(!status_error(StatusCode::BAD_REQUEST, None, "bad", "Mistral").is_retryable());
    }

    /// The page Mistral answered an inline batch of two photographed pages with (1.10.0-rc2),
    /// shortened.
    const DJANGO_400: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n  <title>Bad Request \
        (400)</title>\n</head>\n<body>\n  <h1>Bad Request (400)</h1><p></p>\n</body>\n</html>\n";

    #[test]
    fn an_error_page_becomes_one_sentence_naming_the_provider_and_the_status() {
        let said = status_error(StatusCode::BAD_REQUEST, None, DJANGO_400, "Mistral");
        assert_eq!(
            said,
            ProviderError::InvalidRequest("Mistral refused the request (400 Bad Request).".into())
        );
        assert_eq!(
            said.to_string(),
            "The provider rejected the request: Mistral refused the request (400 Bad Request)."
        );
        // A gateway's page for a service that is down is still worth trying again.
        let down = status_error(
            StatusCode::BAD_GATEWAY,
            None,
            "<html><body>502 Bad Gateway</body></html>",
            "OpenAI",
        );
        assert_eq!(
            down,
            ProviderError::Transient(
                "OpenAI could not answer the request (502 Bad Gateway).".into()
            )
        );
        // An empty body, or one too long to be a message, says the same.
        assert_eq!(
            error_message("  ", StatusCode::BAD_REQUEST, "The provider"),
            "The provider refused the request (400 Bad Request)."
        );
        assert_eq!(
            error_message(&"x".repeat(301), StatusCode::FORBIDDEN, "Anthropic"),
            "Anthropic refused the request (403 Forbidden)."
        );
        assert_eq!(
            error_message("", StatusCode::from_u16(499).unwrap_or_default(), "Google"),
            "Google refused the request (499)."
        );
    }

    #[test]
    fn a_message_in_json_or_short_plain_text_is_kept() {
        let bad = StatusCode::BAD_REQUEST;
        // Mistral's own shape, and the list FastAPI gives for a request that does not validate.
        assert_eq!(
            error_message(
                r#"{"object":"error","message":"Invalid model: mistral-ocr-9","type":"invalid_request_error"}"#,
                bad,
                "Mistral"
            ),
            "Invalid model: mistral-ocr-9"
        );
        assert_eq!(
            error_message(
                r#"{"detail":[{"loc":["body","endpoint"],"msg":"Input should be '/v1/ocr'","type":"enum"}]}"#,
                StatusCode::UNPROCESSABLE_ENTITY,
                "Mistral"
            ),
            "Input should be '/v1/ocr'"
        );
        assert_eq!(
            error_message(r#"{"error":"file not found"}"#, bad, "OpenAI"),
            "file not found"
        );
        assert_eq!(error_message(" busy \n", bad, "Mistral"), "busy");
        // JSON with no message is no message, whatever its length.
        assert_eq!(
            error_message(r#"{"code":17}"#, bad, "Mistral"),
            "Mistral refused the request (400 Bad Request)."
        );
        // A long message is cut, and says so.
        let long = format!(r#"{{"message":"{}"}}"#, "é".repeat(400));
        let said = error_message(&long, bad, "Mistral");
        assert_eq!(said.chars().count(), 300);
        assert!(said.ends_with('…'));
    }

    #[test]
    fn who_answered_is_named_by_the_provider_s_address() {
        let named =
            |url: &str| answered_by(&reqwest::Url::parse(url).unwrap_or_else(|e| panic!("{e}")));
        assert_eq!(named("https://api.mistral.ai/v1/batch/jobs"), "Mistral");
        assert_eq!(named("https://api.openai.com/v1/files"), "OpenAI");
        assert_eq!(named("https://api.anthropic.com/v1/messages"), "Anthropic");
        assert_eq!(
            named("https://generativelanguage.googleapis.com/v1beta/models"),
            "Google"
        );
        assert_eq!(named("http://127.0.0.1:4000/batch/jobs"), "The provider");
        // A look-alike is not the provider.
        assert_eq!(named("https://notmistral.ai/v1"), "The provider");
    }

    #[test]
    fn an_upload_has_time_in_proportion_to_its_size() {
        assert_eq!(upload_timeout(0), Duration::from_secs(300));
        let mb = |n: usize| upload_timeout(n * 1_000_000).as_secs();
        assert_eq!(mb(64), 300 + 488);
        assert_eq!(mb(200), 300 + 1525);
        assert!(
            mb(200) > 900,
            "longer than the fixed fifteen minutes it replaces"
        );
    }

    #[test]
    fn a_wait_asked_for_is_bounded_and_nonsense_is_ignored() {
        assert_eq!(retry_after_value(" 7 "), Some(Duration::from_secs(7)));
        assert_eq!(retry_after_value("0.5"), Some(Duration::from_millis(500)));
        assert_eq!(retry_after_value("86400"), Some(Duration::from_secs(300)));
        for nonsense in [
            "-3",
            "NaN",
            "inf",
            "-inf",
            "1e400",
            "Wed, 21 Oct 2026 07:28:00 GMT",
        ] {
            assert_eq!(retry_after_value(nonsense), None, "{nonsense}");
        }
        assert_eq!(
            stated_retry("Please retry in 86400s."),
            Some(Duration::from_secs(300))
        );
        assert_eq!(
            stated_retry(&format!("Please retry in {}s", "9".repeat(400))),
            None
        );
    }

    /// A `Retry-After` header's wait, bounded as [`status_error`] bounds it.
    fn retry_after_value(value: &str) -> Option<Duration> {
        retry_after_secs(value).and_then(wait)
    }

    /// A message's stated wait, bounded as [`status_error`] bounds it.
    fn stated_retry(message: &str) -> Option<Duration> {
        stated_secs(message).and_then(wait)
    }

    #[test]
    fn a_wait_is_read_in_the_words_each_provider_uses() {
        assert_eq!(
            stated_retry("Please retry in 33s."),
            Some(Duration::from_secs(33))
        );
        assert_eq!(
            stated_retry("Please try again in 1.2s. Visit the rate limits page."),
            Some(Duration::from_secs_f64(1.2))
        );
        assert_eq!(
            stated_retry("Please try again in 20ms."),
            Some(Duration::from_millis(20))
        );
        assert_eq!(
            stated_retry("Please try again in 6m0s."),
            Some(Duration::from_secs(360).min(Duration::from_secs(300)))
        );
        assert_eq!(stated_secs("Please try again in 1m30s."), Some(90.0));
        assert_eq!(stated_secs("Retry in 2 seconds"), Some(2.0));
        assert_eq!(stated_secs("Please try again later."), None);
        assert_eq!(
            retry_info_secs(&serde_json::json!({"error": {"details": [
                {"@type": "type.googleapis.com/google.rpc.Help"},
                {"@type": "type.googleapis.com/google.rpc.RetryInfo", "retryDelay": "33s"}
            ]}})),
            Some(33.0)
        );
    }

    // The bodies below are as documented/remembered, to be replaced by a captured answer (#270).

    /// Google's per-minute limit on Gemini, worded as an exceeded quota, its wait at the end
    /// of a message longer than a researcher is shown.
    const GEMINI_PER_MINUTE: &str = r#"{"error":{"code":429,"message":"You exceeded your current quota, please check your plan and billing details. For more information on this error, head to: https://ai.google.dev/gemini-api/docs/rate-limits. To monitor your current usage, head to: https://ai.dev/usage?tab=rate-limit. \n* Quota exceeded for metric: generativelanguage.googleapis.com/generate_content_paid_tier_input_token_count, limit: 1000000, model: gemini-3.8-flash\nPlease retry in 33.5s.","status":"RESOURCE_EXHAUSTED","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaMetric":"generativelanguage.googleapis.com/generate_content_paid_tier_input_token_count","quotaId":"GenerateContentPaidTierInputTokensPerModelPerMinute"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"33s"}]}}"#;

    /// The same answer for a quota per day, which no wait of minutes cures.
    const GEMINI_PER_DAY: &str = r#"{"error":{"code":429,"message":"You exceeded your current quota, please check your plan and billing details. For more information on this error, head to: https://ai.google.dev/gemini-api/docs/rate-limits.\n* Quota exceeded for metric: generativelanguage.googleapis.com/generate_content_free_tier_requests, limit: 250, model: gemini-3.8-flash\nPlease retry in 33.5s.","status":"RESOURCE_EXHAUSTED","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests","quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"33s"}]}}"#;

    /// OpenAI's per-minute limit.
    const OPENAI_PER_MINUTE: &str = r#"{"error":{"message":"Rate limit reached for gpt-5.4 in organization org-abc on tokens per min (TPM): Limit 30000, Used 29500, Requested 1200. Please try again in 1.4s. Visit https://platform.openai.com/account/rate-limits to learn more.","type":"tokens","param":null,"code":"rate_limit_exceeded"}}"#;

    /// OpenAI's account out of credit.
    const OPENAI_NO_CREDIT: &str = r#"{"error":{"message":"You exceeded your current quota, please check your plan and billing details. For more information on this error, read the docs: https://platform.openai.com/docs/guides/error-codes/api-errors.","type":"insufficient_quota","param":null,"code":"insufficient_quota"}}"#;

    #[test]
    fn a_limit_that_states_a_wait_is_waited_out_whatever_it_is_called() {
        let too_many = StatusCode::TOO_MANY_REQUESTS;
        let gemini = status_error(too_many, None, GEMINI_PER_MINUTE, "Google");
        assert!(
            matches!(
                gemini,
                ProviderError::RateLimited { retry_after: Some(d), .. } if d == Duration::from_secs(33)
            ),
            "{gemini:?}"
        );
        // Without the structured detail, the words at the end of the whole message.
        let words_only = serde_json::json!({"error": {
            "code": 429,
            "message": serde_json::from_str::<Value>(GEMINI_PER_MINUTE)
                .ok()
                .and_then(|v| v.pointer("/error/message")?.as_str().map(str::to_string)),
        }})
        .to_string();
        assert!(matches!(
            status_error(too_many, None, &words_only, "Google"),
            ProviderError::RateLimited { retry_after: Some(d), .. } if d == Duration::from_secs_f64(33.5)
        ));
        assert!(matches!(
            status_error(too_many, None, OPENAI_PER_MINUTE, "OpenAI"),
            ProviderError::RateLimited { retry_after: Some(d), .. } if d == Duration::from_secs_f64(1.4)
        ));
        // A Retry-After header is a stated wait too.
        assert!(matches!(
            status_error(
                too_many,
                Some(20.0),
                r#"{"error":{"message":"You exceeded your current quota."}}"#,
                "Google"
            ),
            ProviderError::RateLimited { retry_after: Some(d), .. } if d == Duration::from_secs(20)
        ));
    }

    #[test]
    fn an_exhausted_account_still_stops_the_job() {
        let too_many = StatusCode::TOO_MANY_REQUESTS;
        for (body, who) in [
            (GEMINI_PER_DAY, "Google"),
            (OPENAI_NO_CREDIT, "OpenAI"),
            // Quota words and no wait.
            (
                r#"{"error":{"message":"You exceeded your current quota, please check your plan and billing details."}}"#,
                "Google",
            ),
            // A wait longer than any retry waits for.
            (
                r#"{"error":{"message":"You exceeded your current quota. Please retry in 7200s."}}"#,
                "Google",
            ),
        ] {
            let said = status_error(too_many, None, body, who);
            assert!(
                matches!(said, ProviderError::QuotaExhausted(_)),
                "{who}: {said:?}"
            );
        }
        // Even with a Retry-After header.
        assert!(matches!(
            status_error(too_many, Some(5.0), OPENAI_NO_CREDIT, "OpenAI"),
            ProviderError::QuotaExhausted(_)
        ));
        assert!(matches!(
            status_error(StatusCode::PAYMENT_REQUIRED, Some(5.0), "pay", "Mistral"),
            ProviderError::QuotaExhausted(_)
        ));
    }
}
