//! Finding what a server is and what it has, from its address alone: `GET /api/version`
//! answered means Ollama, whose models come from `/api/tags` and what each can do from
//! `/api/show`; else `GET /models`, then `/v1/models`, means an OpenAI-style server. Nothing to
//! choose by hand.

use std::time::Duration;

use kalamu_core::ApiKey;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use super::{AddressError, Dialect, checked_address, host_of};
use crate::http::{Exchange, exchange, shortened};

/// How long each request of a discovery waits: a server that has not answered by then is not
/// one to wait for, and the researcher is waiting.
const TIMEOUT: Duration = Duration::from_secs(10);

/// What a server is and the models it lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Found {
    pub dialect: Dialect,
    /// The address to keep, requests being written after it: the one typed, checked, or with
    /// `/v1` where an OpenAI-style server answered only there.
    pub address: String,
    /// Ollama's version; `None` for an OpenAI-style server.
    pub version: Option<String>,
    pub models: Vec<FoundModel>,
}

/// A model as the server describes it. `None` wherever the server does not say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundModel {
    pub id: String,
    /// Reads pictures, as the server says (Ollama's `vision`, an OpenAI-style list's input
    /// modalities); the trial decides in the end.
    pub vision: Option<bool>,
    pub audio: Option<bool>,
    /// The context the model was trained for, or the one the server gives it, in tokens.
    pub context: Option<u32>,
    /// Where it runs when not on this server: Ollama's `remote_host`, a model run at
    /// ollama.com. Such a model is shown but cannot be ticked.
    pub elsewhere: Option<String>,
    pub size_bytes: Option<u64>,
    pub digest: Option<String>,
}

/// Why no server was found at an address, in words a researcher can act on.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiscoverError {
    #[error(transparent)]
    Address(#[from] AddressError),
    #[error(
        "Nothing answers at {host}: check the address, that the server is running, and that \
         it accepts connections from this computer"
    )]
    NothingAnswers { host: String },
    #[error(
        "Something answers at {host}, but it is not a server Kalamu knows: Ollama, or a server \
         that speaks like OpenAI's"
    )]
    NotAServer { host: String },
    #[error("The server at {host} asks for a key: tick “This server asks for a key” and give it")]
    KeyNeeded { host: String },
    #[error("The server at {host} refused the key")]
    KeyRefused { host: String },
    #[error("The server at {host} could not be read: {message}")]
    Other { host: String, message: String },
}

impl DiscoverError {
    /// The id the interface looks the sentence up by: the address's own where the address was
    /// refused, else one per way of failing.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Address(e) => e.code(),
            Self::NothingAnswers { .. } => "server.nothing_answers",
            Self::NotAServer { .. } => "server.not_a_server",
            Self::KeyNeeded { .. } => "server.key_needed",
            Self::KeyRefused { .. } => "server.key_refused",
            Self::Other { .. } => "server.unreadable",
        }
    }

    /// The host the failure names, with its port; none where the address was refused.
    pub fn host(&self) -> Option<&str> {
        match self {
            Self::Address(_) => None,
            Self::NothingAnswers { host }
            | Self::NotAServer { host }
            | Self::KeyNeeded { host }
            | Self::KeyRefused { host }
            | Self::Other { host, .. } => Some(host),
        }
    }
}

/// Finds the server at `address` and lists its models. `key` is sent where the researcher gave
/// one.
pub async fn discover(
    client: &Client,
    address: &str,
    key: Option<&ApiKey>,
) -> Result<Found, DiscoverError> {
    let address = checked_address(address)?;
    let probe = Probe {
        client,
        host: host_of(&address),
        key,
    };
    if let Some(answer) = probe
        .json(probe.get(format!("{address}/api/version")))
        .await?
        && let Some(version) = answer.get("version").and_then(Value::as_str)
    {
        let models = ollama_models(&probe, &address).await?;
        return Ok(Found {
            dialect: Dialect::Ollama,
            address,
            version: Some(version.to_string()),
            models,
        });
    }
    for base in [address.clone(), format!("{address}/v1")] {
        if let Some(list) = probe.json(probe.get(format!("{base}/models"))).await?
            && let Some(data) = list.get("data").and_then(Value::as_array)
        {
            return Ok(Found {
                dialect: Dialect::OpenAi,
                address: base,
                version: None,
                models: data.iter().filter_map(openai_model).collect(),
            });
        }
    }
    Err(DiscoverError::NotAServer { host: probe.host })
}

struct Probe<'a> {
    client: &'a Client,
    host: String,
    key: Option<&'a ApiKey>,
}

impl Probe<'_> {
    fn get(&self, url: String) -> RequestBuilder {
        self.authorised(self.client.get(url))
    }

    fn post(&self, url: String, body: &Value) -> RequestBuilder {
        self.authorised(self.client.post(url).json(body))
    }

    fn authorised(&self, request: RequestBuilder) -> RequestBuilder {
        let request = request.timeout(TIMEOUT);
        match self.key {
            Some(key) if !key.expose().is_empty() => request.bearer_auth(key.expose()),
            _ => request,
        }
    }

    /// The JSON of a successful answer; `None` for any other answer, which says the address is
    /// not this kind of server. A key refused or asked for, and an address where nothing
    /// answers, end the discovery.
    async fn json(&self, request: RequestBuilder) -> Result<Option<Value>, DiscoverError> {
        let answer = self.send(request).await?;
        if !answer.status.is_success() {
            return Ok(None);
        }
        Ok(serde_json::from_str(&answer.body).ok())
    }

    async fn send(&self, request: RequestBuilder) -> Result<Exchange, DiscoverError> {
        let host = self.host.clone();
        let answer = exchange(request).await.map_err(|e| {
            if e.is_timeout() || e.is_connect() {
                DiscoverError::NothingAnswers { host: host.clone() }
            } else {
                DiscoverError::Other {
                    host: host.clone(),
                    message: shortened(&e.to_string()),
                }
            }
        })?;
        match answer.status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(match self.key {
                Some(key) if !key.expose().is_empty() => DiscoverError::KeyRefused { host },
                _ => DiscoverError::KeyNeeded { host },
            }),
            _ => Ok(answer),
        }
    }
}

/// Ollama's models (`/api/tags`), each with what `/api/show` says it can do.
async fn ollama_models(probe: &Probe<'_>, address: &str) -> Result<Vec<FoundModel>, DiscoverError> {
    let Some(tags) = probe.json(probe.get(format!("{address}/api/tags"))).await? else {
        return Err(DiscoverError::Other {
            host: probe.host.clone(),
            message: "Ollama did not list its models".into(),
        });
    };
    let mut models = Vec::new();
    for tag in tags
        .get("models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(id) = tag
            .get("name")
            .or_else(|| tag.get("model"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        // A model whose details cannot be read is still listed, its abilities unknown.
        let shown = probe
            .json(probe.post(format!("{address}/api/show"), &json!({ "model": id })))
            .await
            .ok()
            .flatten();
        models.push(ollama_model(id, tag, shown.as_ref()));
    }
    Ok(models)
}

fn ollama_model(id: &str, tag: &Value, shown: Option<&Value>) -> FoundModel {
    let capabilities = shown
        .and_then(|s| s.get("capabilities"))
        .or_else(|| tag.get("capabilities"))
        .and_then(Value::as_array);
    let can = |what: &str| capabilities.map(|c| c.iter().any(|v| v.as_str() == Some(what)));
    let trained = shown
        .and_then(|s| s.get("model_info"))
        .and_then(Value::as_object)
        .and_then(|info| {
            info.iter()
                .find(|(key, _)| key.ends_with(".context_length"))
                .and_then(|(_, v)| v.as_u64())
        })
        .or_else(|| {
            tag.pointer("/details/context_length")
                .and_then(Value::as_u64)
        });
    FoundModel {
        id: id.to_string(),
        vision: can("vision"),
        audio: can("audio"),
        context: trained.map(tokens),
        elsewhere: tag
            .get("remote_host")
            .and_then(Value::as_str)
            .filter(|h| !h.is_empty())
            .map(str::to_string),
        size_bytes: tag.get("size").and_then(Value::as_u64),
        digest: tag
            .get("digest")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

/// A model of an OpenAI-style list, with a context where the server gives one, in any of the
/// fields servers use: vLLM's `max_model_len`, OpenRouter's `context_length`, llama.cpp's
/// `meta.n_ctx_train`.
fn openai_model(model: &Value) -> Option<FoundModel> {
    let id = model.get("id").and_then(Value::as_str)?;
    let context = ["/max_model_len", "/context_length", "/meta/n_ctx_train"]
        .iter()
        .find_map(|at| model.pointer(at).and_then(Value::as_u64));
    let modalities = model
        .pointer("/architecture/input_modalities")
        .and_then(Value::as_array);
    let takes = |what: &str| modalities.map(|m| m.iter().any(|v| v.as_str() == Some(what)));
    Some(FoundModel {
        id: id.to_string(),
        vision: takes("image"),
        audio: takes("audio"),
        context: context.map(tokens),
        elsewhere: None,
        size_bytes: None,
        digest: None,
    })
}

fn tokens(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
