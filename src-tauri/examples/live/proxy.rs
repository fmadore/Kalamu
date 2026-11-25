//! A recording relay between the providers and their services, so that the real error answers
//! met on the way are kept as fixtures.
//!
//! With `LIVE_FIXTURES=<folder>`, each provider is given the relay's local address in place of
//! its service's (the same `with_base_url` the crates' tests use). The relay sends every request
//! on unchanged to the service, over the same HTTPS client the app uses, and hands back its
//! answer unchanged; an answer with a status of 400 or more is also written to the folder, its
//! body as `<provider>_<status>[_<n>].<json|txt|html>` (the shape of
//! `crates/providers/tests/fixtures/`) and its status, path and kept headers in `index.json`,
//! with the keys and identifiers taken out. Only HTTP/1.1 from the providers' own client is
//! spoken; each connection carries one request.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::keys::Redactor;

/// The headers a fixture keeps: those that say how long to wait, how much is left, and what the
/// body is. Request identifiers and cookies are dropped.
const KEPT_HEADERS: [&str; 5] = [
    "retry-after",
    "retry-after-ms",
    "content-type",
    "x-ratelimit-",
    "anthropic-ratelimit-",
];

/// Headers that belong to one connection and are never passed on.
const HOP_HEADERS: [&str; 9] = [
    "host",
    "connection",
    "keep-alive",
    "proxy-connection",
    "transfer-encoding",
    "te",
    "upgrade",
    "content-length",
    // Asked again by the relay's own client, which decompresses what it receives.
    "accept-encoding",
];

/// One error answer kept, as `index.json` lists it.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Recorded {
    pub file: String,
    pub provider: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub headers: BTreeMap<String, String>,
}

/// The folder error answers are written to.
#[derive(Clone)]
pub struct Fixtures {
    dir: PathBuf,
    redactor: Redactor,
    recorded: Arc<Mutex<Vec<Recorded>>>,
}

impl Fixtures {
    pub fn new(dir: PathBuf, redactor: Redactor) -> std::io::Result<Self> {
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            redactor,
            recorded: Arc::default(),
        })
    }

    /// What was kept so far.
    pub fn recorded(&self) -> Vec<Recorded> {
        self.recorded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Writes one error answer, redacted, and the index beside it.
    pub fn keep(
        &self,
        provider: &str,
        method: &str,
        path: &str,
        status: u16,
        headers: &[(String, String)],
        body: &[u8],
    ) -> std::io::Result<()> {
        let text = String::from_utf8_lossy(body);
        let content_type = headers
            .iter()
            .find(|(n, _)| n == "content-type")
            .map(|(_, v)| v.as_str())
            .unwrap_or_default();
        let parsed: Option<serde_json::Value> = serde_json::from_str(&text).ok();
        let (extension, body) = match &parsed {
            Some(json) => (
                "json",
                serde_json::to_string_pretty(&self.redact_json(json)).unwrap_or_default() + "\n",
            ),
            None if content_type.contains("html") || text.trim_start().starts_with('<') => {
                ("html", self.redactor.text(&text))
            }
            None => ("txt", self.redactor.text(&text)),
        };
        let mut recorded = self.recorded.lock().unwrap_or_else(PoisonError::into_inner);
        let stem = format!("{provider}_{status}");
        let same = recorded
            .iter()
            .filter(|r| {
                r.file.starts_with(&format!("{stem}.")) || r.file.starts_with(&format!("{stem}_"))
            })
            .count();
        let file = if same == 0 {
            format!("{stem}.{extension}")
        } else {
            format!("{stem}_{}.{extension}", same + 1)
        };
        std::fs::write(self.dir.join(&file), body)?;
        recorded.push(Recorded {
            file,
            provider: provider.to_string(),
            method: method.to_string(),
            path: self.redactor.text(path),
            status,
            headers: headers
                .iter()
                .filter(|(n, _)| KEPT_HEADERS.iter().any(|k| n.starts_with(k)))
                .map(|(n, v)| (n.clone(), self.redactor.text(v)))
                .collect(),
        });
        let index = serde_json::to_string_pretty(&*recorded).unwrap_or_default() + "\n";
        std::fs::write(self.dir.join("index.json"), index)
    }

    /// Every string in a JSON body redacted, its shape kept.
    fn redact_json(&self, value: &serde_json::Value) -> serde_json::Value {
        use serde_json::Value;
        match value {
            Value::String(s) => Value::String(self.redactor.text(s)),
            Value::Array(items) => {
                Value::Array(items.iter().map(|v| self.redact_json(v)).collect())
            }
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.redact_json(v)))
                    .collect(),
            ),
            other => other.clone(),
        }
    }
}

/// The relay, listening on a local port.
pub struct Relay {
    pub address: String,
}

impl Relay {
    /// Starts relaying `/<name>/<rest>` to `<origin>/<rest>` for each `(name, origin)` route.
    pub async fn start(
        routes: Vec<(String, String)>,
        fixtures: Fixtures,
        client: reqwest::Client,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = format!("http://{}", listener.local_addr()?);
        let routes = Arc::new(routes);
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let (routes, fixtures, client) = (routes.clone(), fixtures.clone(), client.clone());
                tokio::spawn(async move {
                    if let Err(e) = serve(stream, &routes, &fixtures, &client).await {
                        eprintln!("  relay: {e}");
                    }
                });
            }
        });
        Ok(Self { address })
    }
}

/// A request as read from the connection.
struct Request {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

async fn serve(
    mut stream: TcpStream,
    routes: &[(String, String)],
    fixtures: &Fixtures,
    client: &reqwest::Client,
) -> std::io::Result<()> {
    let request = read_request(&mut stream).await?;
    let path = request.target.trim_start_matches('/');
    let (name, rest) = path.split_once('/').unwrap_or((path, ""));
    let Some((_, origin)) = routes.iter().find(|(n, _)| n == name) else {
        return respond(&mut stream, 404, &[], b"no such route").await;
    };
    let url = format!("{origin}/{rest}");
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let mut upstream = client.request(method, &url).body(request.body);
    for (n, v) in &request.headers {
        if !HOP_HEADERS.contains(&n.as_str()) {
            upstream = upstream.header(n, v);
        }
    }
    let answer = match upstream.send().await {
        Ok(answer) => answer,
        Err(e) => {
            let message = format!("the relay could not reach the service: {e}");
            return respond(&mut stream, 502, &[], message.as_bytes()).await;
        }
    };
    let status = answer.status().as_u16();
    let headers: Vec<(String, String)> = answer
        .headers()
        .iter()
        .filter(|(n, _)| !HOP_HEADERS.contains(&n.as_str()) && n.as_str() != "content-encoding")
        .filter_map(|(n, v)| Some((n.as_str().to_string(), v.to_str().ok()?.to_string())))
        .collect();
    let body = match answer.bytes().await {
        Ok(body) => body.to_vec(),
        Err(e) => {
            let message = format!("the relay lost the service's answer: {e}");
            return respond(&mut stream, 502, &[], message.as_bytes()).await;
        }
    };
    if status >= 400
        && let Err(e) = fixtures.keep(
            name,
            &request.method,
            &format!("/{rest}"),
            status,
            &headers,
            &body,
        )
    {
        eprintln!("  relay: a fixture could not be written: {e}");
    }
    respond(&mut stream, status, &headers, &body).await
}

/// Reads one HTTP/1.1 request: its line, its headers (names in lower case) and its body, by
/// `content-length` or in chunks.
async fn read_request<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<Request> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 8192];
    let head_end = loop {
        if let Some(at) = find(&buffer, b"\r\n\r\n") {
            break at;
        }
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return Err(std::io::Error::other(
                "connection closed before the request",
            ));
        }
        buffer.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
    let mut rest = buffer[head_end + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let method = first.next().unwrap_or_default().to_string();
    let target = first.next().unwrap_or_default().to_string();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let header = |name: &str| {
        headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    };
    let body = if header("transfer-encoding").is_some_and(|v| v.contains("chunked")) {
        read_chunked(stream, &mut rest).await?
    } else {
        let length: usize = header("content-length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        while rest.len() < length {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Err(std::io::Error::other("connection closed inside the body"));
            }
            rest.extend_from_slice(&chunk[..n]);
        }
        rest.truncate(length);
        rest
    };
    Ok(Request {
        method,
        target,
        headers,
        body,
    })
}

/// A body sent in chunks, `buffer` holding what was already read past the headers.
async fn read_chunked<R: AsyncRead + Unpin>(
    stream: &mut R,
    buffer: &mut Vec<u8>,
) -> std::io::Result<Vec<u8>> {
    let mut body = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        while find(buffer, b"\r\n").is_none() {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Err(std::io::Error::other("connection closed inside a chunk"));
            }
            buffer.extend_from_slice(&chunk[..n]);
        }
        let line_end = find(buffer, b"\r\n").unwrap_or(0);
        let size_text = String::from_utf8_lossy(&buffer[..line_end]).to_string();
        let size = usize::from_str_radix(size_text.split(';').next().unwrap_or("").trim(), 16)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        buffer.drain(..line_end + 2);
        if size == 0 {
            return Ok(body);
        }
        while buffer.len() < size + 2 {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                return Err(std::io::Error::other("connection closed inside a chunk"));
            }
            buffer.extend_from_slice(&chunk[..n]);
        }
        body.extend_from_slice(&buffer[..size]);
        buffer.drain(..size + 2);
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

async fn respond(
    stream: &mut TcpStream,
    status: u16,
    headers: &[(String, String)],
    body: &[u8],
) -> std::io::Result<()> {
    let reason = reqwest::StatusCode::from_u16(status)
        .ok()
        .and_then(|s| s.canonical_reason())
        .unwrap_or("");
    let mut head = format!("HTTP/1.1 {status} {reason}\r\n");
    for (n, v) in headers {
        head.push_str(&format!("{n}: {v}\r\n"));
    }
    head.push_str(&format!(
        "content-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    ));
    stream.write_all(head.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    stream.shutdown().await
}

#[cfg(test)]
mod tests {
    use kalamu_core::{ApiKey, EmbedRequest, EmbedRole, Embedder, ProviderError, StopSignal};
    use kalamu_providers::Gemini;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    const KEY: &str = "AIzaTestKeyForTheRelay0123456789";

    async fn relay_to(server: &MockServer, dir: &std::path::Path) -> (Relay, Fixtures) {
        let fixtures = Fixtures::new(dir.to_path_buf(), Redactor::new(vec![KEY.into()]))
            .unwrap_or_else(|e| panic!("{e}"));
        let relay = Relay::start(
            vec![("gemini".into(), server.uri())],
            fixtures.clone(),
            kalamu_providers::client(),
        )
        .await
        .unwrap_or_else(|e| panic!("{e}"));
        (relay, fixtures)
    }

    async fn embed(relay: &Relay) -> Result<kalamu_core::Embedded, ProviderError> {
        let gemini = Gemini::with_base_url(
            kalamu_providers::client(),
            &format!("{}/gemini/v1beta", relay.address),
        );
        let texts = vec!["Une réunion à Bamako.".to_string()];
        gemini
            .embed(
                Some(&ApiKey::new(KEY)),
                EmbedRequest {
                    model: "gemini-embedding-2",
                    role: EmbedRole::Query,
                    texts: &texts,
                    dimensions: None,
                    stop: &StopSignal::default(),
                },
            )
            .await
    }

    #[tokio::test]
    async fn a_success_passes_through_unchanged_and_is_not_kept() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-embedding-2:batchEmbedContents"))
            .and(header("x-goog-api-key", KEY))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [{ "values": [0.0, 0.6, 0.8] }]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let (relay, fixtures) = relay_to(&server, dir.path()).await;
        let embedded = embed(&relay).await.unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(embedded.vectors.len(), 1);
        assert!(fixtures.recorded().is_empty());
        assert!(!dir.path().join("index.json").exists());
    }

    #[tokio::test]
    async fn an_error_answer_reaches_the_provider_and_is_kept_without_the_key() {
        let server = MockServer::start().await;
        let body = serde_json::json!({
            "error": {
                "code": 429,
                "message": format!("You exceeded your current quota with key {KEY}. Please retry in 33.5s."),
                "status": "RESOURCE_EXHAUSTED",
                "details": [
                    { "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                      "metadata": { "consumer": "projects/123456789012" } },
                    { "@type": "type.googleapis.com/google.rpc.RetryInfo", "retryDelay": "33s" }
                ]
            }
        });
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("retry-after", "33")
                    .insert_header("x-request-id", "req_abcdef0123456789")
                    .set_body_json(body),
            )
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        let (relay, fixtures) = relay_to(&server, dir.path()).await;
        let error = embed(&relay).await.err();
        assert!(
            matches!(error, Some(ProviderError::RateLimited { .. })),
            "the relay must hand the 429 back as it came: {error:?}"
        );
        let recorded = fixtures.recorded();
        assert!(!recorded.is_empty());
        let first = &recorded[0];
        assert_eq!(first.file, "gemini_429.json");
        assert_eq!(first.status, 429);
        assert_eq!(
            first.headers.get("retry-after").map(String::as_str),
            Some("33")
        );
        assert!(!first.headers.contains_key("x-request-id"));
        assert_eq!(
            first.path,
            "/v1beta/models/gemini-embedding-2:batchEmbedContents"
        );
        let written = std::fs::read_to_string(dir.path().join("gemini_429.json"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(!written.contains(KEY), "{written}");
        assert!(!written.contains("123456789012"), "{written}");
        assert!(written.contains("Please retry in 33.5s"), "{written}");
        assert!(written.contains("RetryInfo"), "{written}");
        let index = std::fs::read_to_string(dir.path().join("index.json"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(index.contains("gemini_429.json"));
        // A second answer of the same status gets a file of its own.
        let _ = embed(&relay).await;
        assert!(
            fixtures
                .recorded()
                .iter()
                .any(|r| r.file.starts_with("gemini_429_"))
        );
    }

    #[tokio::test]
    async fn a_body_sent_in_chunks_or_by_length_is_read_whole() {
        let chunked = b"POST /gemini/upload HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n6\r\nhello \r\n5;ext=1\r\nworld\r\n0\r\n\r\n";
        let request = read_request(&mut &chunked[..])
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(request.method, "POST");
        assert_eq!(request.target, "/gemini/upload");
        assert_eq!(request.body, b"hello world");
        let sized =
            b"PUT /openai/v1/files HTTP/1.1\r\nContent-Length: 4\r\nX-Api-Key: k\r\n\r\nbodyEXTRA";
        let request = read_request(&mut &sized[..])
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(request.body, b"body");
        assert!(
            request
                .headers
                .iter()
                .any(|(n, v)| n == "x-api-key" && v == "k")
        );
    }
}
