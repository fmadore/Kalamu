//! A server of the researcher's own: Ollama spoken at its own address ([`ollama`]), or any
//! OpenAI-style server through Chat Completions ([`chat`]), found and listed by [`discover`] and
//! each model tried before it is offered ([`trial`]). One provider per server added, under the
//! id the shell made for it once (`own-<8 hex digits>`), so a rename or a changed address keeps
//! its results, cache, consent and key.
//!
//! Never a reading in part: Ollama is sent the model's context, `truncate: false` and
//! `shift: false`, so a text too long is refused rather than cut; an OpenAI-style server's
//! count of the tokens it read is checked against the text sent. Thinking is kept out of the
//! answer. Failures name the server by the researcher's name for it, and one that does not
//! answer is not blamed on the internet connection.
//!
//! See `docs/decisions/2026-11-26-a-server-of-the-researchers-own.md`.

mod address;
mod chat;
mod discover;
mod ollama;
mod trial;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId, Reach,
    RequestSetting, ServerRecord, StopSignal, Task, Tuning,
};
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Semaphore, SemaphorePermit};

use crate::analysis;
use crate::http::{Exchange, error_message, exchange, status_error};
use address::host_is;

pub use address::{AddressError, checked_address, host_of, on_this_computer};
pub use discover::{DiscoverError, Found, FoundModel, discover};
pub use trial::{TrialFailure, TrialResult, try_model};

/// How a server is spoken to, one type with the catalogue's stored entry.
pub use kalamu_core::Dialect;

/// A server as the shell hands it in, from the researcher's stored entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// `own-<8 hex digits>`, made once and kept through renames and a changed address.
    pub id: ProviderId,
    /// The researcher's name for it, which every failure names.
    pub name: String,
    /// The address requests are written after: [`Found::address`], so for an OpenAI-style
    /// server with `/v1` where discovery found it there. Already [`checked_address`].
    pub address: String,
    pub dialect: Dialect,
    /// A key is saved for it, sent as `Authorization: Bearer` (a reverse proxy, a gateway).
    pub key: bool,
    /// Requests at the same time; at least 1.
    pub at_once: usize,
    pub models: Vec<ModelConfig>,
}

/// A model of a server, as the researcher kept it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    /// *Reads up to*, in tokens: Ollama's `num_ctx`.
    pub context: u32,
}

/// *Reads up to* where nothing better is known: an OpenAI-style server that does not say, or a
/// model the configuration lacks.
pub const DEFAULT_CONTEXT: u32 = 8_192;
/// The largest *Reads up to* suggested from a model's trained context: a larger context takes
/// memory the machine may not have.
pub const MAX_SUGGESTED_CONTEXT: u32 = 32_768;
/// The longest answer suggested, when the context allows it.
pub const DEFAULT_ANSWER: u32 = 8_192;

/// *Reads up to* and *Longest answer* as Kalamu fills them for a model whose trained context is
/// `trained` (from discovery): the trained context capped at [`MAX_SUGGESTED_CONTEXT`], else
/// [`DEFAULT_CONTEXT`]; and [`longest_answer`] of that.
pub fn suggested_limits(trained: Option<u32>) -> (u32, u32) {
    let context = trained
        .filter(|c| *c > 0)
        .map_or(DEFAULT_CONTEXT, |c| c.min(MAX_SUGGESTED_CONTEXT));
    (context, longest_answer(context))
}

/// The longest answer for a context: [`DEFAULT_ANSWER`], or a quarter of the context if smaller.
pub fn longest_answer(context: u32) -> u32 {
    (context / 4).clamp(1, DEFAULT_ANSWER)
}

/// What a request is allowed: the context the server is asked to read in, and the longest
/// answer. Decided once per request, and read both by the body and by the settings recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Limits {
    pub context: u32,
    pub answer: u32,
}

/// The providers for the servers the researcher added, sharing one HTTP client: the
/// registration line.
pub fn providers(client: Client, servers: &[ServerConfig]) -> Vec<Arc<dyn Provider>> {
    servers
        .iter()
        .map(|server| Arc::new(OwnServer::new(client.clone(), server.clone())) as Arc<dyn Provider>)
        .collect()
}

/// One server of the researcher's own, as a provider.
pub struct OwnServer {
    client: Client,
    config: ServerConfig,
    /// Its own limit on requests at the same time, so a request does not run out its time
    /// waiting in the server's queue.
    permits: Semaphore,
}

/// How often a request waiting for its turn looks whether the job was stopped.
const STOP_CHECK: Duration = Duration::from_millis(200);

impl OwnServer {
    pub fn new(client: Client, config: ServerConfig) -> Self {
        let permits = Semaphore::new(config.at_once.max(1));
        Self {
            client,
            config,
            permits,
        }
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    fn base(&self) -> &str {
        self.config.address.trim_end_matches('/')
    }

    fn host(&self) -> String {
        host_of(&self.config.address)
    }

    /// The limits for `model`: its *Reads up to*, and the longest answer the engine asks for.
    pub(crate) fn limits(&self, model: &str, tuning: &Tuning) -> Limits {
        let context = self
            .config
            .models
            .iter()
            .find(|m| m.id == model)
            .map(|m| m.context)
            .filter(|c| *c > 0)
            .unwrap_or(DEFAULT_CONTEXT);
        let answer = tuning
            .max_output_tokens
            .filter(|a| *a > 0)
            .unwrap_or_else(|| longest_answer(context));
        Limits { context, answer }
    }

    fn settings(&self, task: Task, limits: &Limits) -> Vec<RequestSetting> {
        if task == Task::AudioTranscription {
            return Vec::new();
        }
        match self.config.dialect {
            Dialect::Ollama => ollama::settings(limits),
            Dialect::OpenAi => chat::settings(limits),
        }
    }

    /// Waits for this server's turn, looking now and then whether the job was stopped.
    async fn turn(&self, stop: &StopSignal) -> Result<SemaphorePermit<'_>, ProviderError> {
        if let Ok(permit) = self.permits.try_acquire() {
            return Ok(permit);
        }
        loop {
            match tokio::time::timeout(STOP_CHECK, self.permits.acquire()).await {
                Ok(Ok(permit)) => return Ok(permit),
                Ok(Err(_)) => return Err(ProviderError::Interrupted),
                Err(_) if stop.is_stopped() => return Err(ProviderError::Interrupted),
                Err(_) => {}
            }
        }
    }

    fn authorised(&self, request: RequestBuilder, key: &ApiKey) -> RequestBuilder {
        if self.config.key && !key.expose().is_empty() {
            request.bearer_auth(key.expose())
        } else {
            request
        }
    }

    /// Sends a request and returns the JSON of a successful answer, every failure worded with
    /// the server's name.
    async fn send(
        &self,
        request: RequestBuilder,
        key: &ApiKey,
        model: Option<&str>,
    ) -> Result<Value, ProviderError> {
        let answer = exchange(self.authorised(request, key))
            .await
            .map_err(|e| self.unreachable(&e))?;
        if !answer.status.is_success() {
            return Err(self.refused(&answer, model));
        }
        serde_json::from_str(&answer.body).map_err(|e| {
            ProviderError::Other(format!(
                "{} answered with something other than JSON: {e}",
                self.config.name
            ))
        })
    }

    fn unreachable(&self, err: &reqwest::Error) -> ProviderError {
        let (name, host) = (&self.config.name, self.host());
        ProviderError::Transient(if err.is_timeout() {
            format!("{name} did not answer at {host} in time")
        } else if err.is_connect() {
            format!(
                "{name} did not answer at {host}: check that the server is running and that \
                 this computer can reach it"
            )
        } else {
            format!("{name} at {host}: {err}")
        })
    }

    /// A failed answer as the engine handles it: a text longer than the model reads, a model
    /// gone from the server and a lack of memory said in plain words and never retried; the
    /// rest as every provider's.
    fn refused(&self, answer: &Exchange, model: Option<&str>) -> ProviderError {
        let name = &self.config.name;
        let message = error_message(&answer.body, answer.status, name);
        let lower = message.to_lowercase();
        let model = model.unwrap_or("this model");
        if matches!(
            answer.status,
            StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE
        ) && overflows(&lower)
        {
            return ProviderError::InvalidRequest(format!(
                "The page or text is longer than {model} reads on {name}. If the machine has \
                 the memory for it, raise Reads up to for this model in Settings › Other \
                 servers. ({message})"
            ));
        }
        if answer.status == StatusCode::NOT_FOUND && lower.contains("model") {
            return ProviderError::InvalidRequest(format!(
                "{model} is no longer on {name}: it may have been removed or renamed there. \
                 ({message})"
            ));
        }
        if lower.contains("more system memory") || lower.contains("out of memory") {
            return ProviderError::InvalidRequest(format!(
                "{name} has not the memory to run {model}: {message}"
            ));
        }
        status_error(answer.status, answer.retry_after, &answer.body, name)
    }

    async fn ask(
        &self,
        key: &ApiKey,
        request: &ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        let fields = match request.task {
            Task::AudioTranscription => {
                return Err(ProviderError::Unsupported(format!(
                    "{} is not used for recordings in this version of Kalamu",
                    self.config.name
                )));
            }
            Task::TextExtraction => None,
            Task::Analysis | Task::Contents | Task::TextCorrection | Task::Translation => {
                Some(analysis::fields(request)?)
            }
        };
        let limits = self.limits(request.model, request.tuning);
        let name = self.config.name.as_str();
        let output = match self.config.dialect {
            Dialect::Ollama => {
                let body = ollama::body(request, fields, &limits, name)?;
                let _turn = self.turn(request.stop).await?;
                let url = format!("{}/api/chat", self.base());
                let response = self
                    .send(self.client.post(url).json(&body), key, Some(request.model))
                    .await?;
                ollama::parse(&response, name)?
            }
            Dialect::OpenAi => {
                let openrouter = host_is(&self.config.address, "openrouter.ai");
                let sent = chat::body(request, fields, &limits, openrouter, name)?;
                let _turn = self.turn(request.stop).await?;
                let url = format!("{}/chat/completions", self.base());
                let response = self
                    .send(
                        self.client.post(url).json(&sent.body),
                        key,
                        Some(request.model),
                    )
                    .await?;
                chat::parse(&response, name, sent.text_chars)?
            }
        };
        match fields {
            Some(fields) => analysis::finish(output, fields),
            None => Ok(output),
        }
    }
}

/// Whether an error's words say the request was longer than the model's context, as Ollama
/// ("the prompt is longer than the context length"), llama.cpp ("exceeds the available context
/// size") and vLLM ("This model's maximum context length is …") word it.
fn overflows(lower: &str) -> bool {
    lower.contains("context")
        && (lower.contains("longer than")
            || lower.contains("exceed")
            || lower.contains("maximum context length"))
}

/// The answer without a leading `<think>…</think>` block, which some models write even when
/// told not to think. An unclosed block is all thinking, and leaves nothing.
pub(crate) fn without_thinking(text: &str) -> &str {
    let start = text.trim_start();
    match start.strip_prefix("<think>") {
        Some(rest) => match rest.find("</think>") {
            Some(end) => rest[end + "</think>".len()..].trim_start(),
            None => "",
        },
        None => text,
    }
}

/// What kind of input a request carries, for the two ways of speaking.
pub(crate) enum Input {
    Text,
    Image,
}

pub(crate) fn input_kind(request: &ProcessRequest<'_>, name: &str) -> Result<Input, ProviderError> {
    match request.input.mime.as_str() {
        "text/plain" => Ok(Input::Text),
        mime if mime.starts_with("image/") => Ok(Input::Image),
        mime => Err(ProviderError::Unsupported(format!(
            "{name} is not sent files of type {mime}"
        ))),
    }
}

#[async_trait]
impl Provider for OwnServer {
    fn id(&self) -> ProviderId {
        self.config.id.clone()
    }

    fn needs_key(&self) -> bool {
        self.config.key
    }

    fn reach(&self) -> Reach {
        if on_this_computer(&self.config.address) {
            Reach::ThisComputer
        } else {
            Reach::Network
        }
    }

    fn server(&self) -> Option<ServerRecord> {
        Some(ServerRecord {
            name: self.config.name.clone(),
            address: self.config.address.clone(),
        })
    }

    /// Every type such a server may read; each model's catalogue entry, written from its trial,
    /// narrows it.
    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::AudioTranscription => &[],
            Task::TextExtraction
            | Task::Analysis
            | Task::Contents
            | Task::TextCorrection
            | Task::Translation => &["text/plain", "image/png", "image/jpeg"],
        }
    }

    fn request_settings(&self, task: Task, model: &str, tuning: &Tuning) -> Vec<RequestSetting> {
        self.settings(task, &self.limits(model, tuning))
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        self.ask(key, &request).await
    }

    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        let (path, list) = match self.config.dialect {
            Dialect::Ollama => ("api/tags", "/models"),
            Dialect::OpenAi => ("models", "/data"),
        };
        let url = format!("{}/{path}", self.base());
        let response = self.send(self.client.get(url), key, None).await?;
        let key = match self.config.dialect {
            Dialect::Ollama => "name",
            Dialect::OpenAi => "id",
        };
        Ok(response
            .pointer(list)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|m| m.get(key).and_then(Value::as_str))
            .map(str::to_string)
            .collect())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn thinking_is_kept_out_of_the_answer() {
        assert_eq!(without_thinking("<think>Hmm.</think>\n\n4729"), "4729");
        assert_eq!(without_thinking("  <think>a</think>b"), "b");
        assert_eq!(without_thinking("<think>never closed"), "");
        assert_eq!(
            without_thinking("4729 <think>x</think>"),
            "4729 <think>x</think>"
        );
    }

    #[test]
    fn overflow_is_read_in_each_server_s_words() {
        for words in [
            "the prompt is longer than the context length",
            "the request exceeds the available context size, try increasing it",
            "This model's maximum context length is 8192 tokens. However, you requested 9000",
        ] {
            assert!(overflows(&words.to_lowercase()), "{words}");
        }
        assert!(!overflows("model 'x' not found"));
    }

    #[test]
    fn the_suggested_limits_follow_the_decision() {
        assert_eq!(suggested_limits(Some(131_072)), (32_768, 8_192));
        assert_eq!(suggested_limits(Some(4_096)), (4_096, 1_024));
        assert_eq!(suggested_limits(None), (8_192, 2_048));
        assert_eq!(longest_answer(0), 1);
    }

    #[test]
    fn openrouter_is_known_by_its_host_alone() {
        assert!(host_is("https://openrouter.ai/api/v1", "openrouter.ai"));
        assert!(host_is("https://eu.openrouter.ai/api/v1", "openrouter.ai"));
        assert!(!host_is("https://notopenrouter.ai/v1", "openrouter.ai"));
        assert!(!host_is("http://localhost:8000/v1", "openrouter.ai"));
    }
}
