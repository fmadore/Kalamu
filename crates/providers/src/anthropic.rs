//! Anthropic Claude through the Messages API, as raw HTTP (there is no official Rust SDK).
//!
//! API version 2026-10-21, which refuses sampling parameters, so none are sent. Effort is set to
//! `low`, the setting iwac-ai-pipelines uses for page transcription. On models that support it,
//! a safety-classifier refusal is retried server-side on a fallback model (`fallbacks:
//! "default"`); the response's `model` field names the model that wrote the text, and that is
//! what provenance records.

use async_trait::async_trait;
use reqwest::Client;
use scriptorium_core::{
    ApiKey, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId, Task, Usage,
};
use serde_json::{Value, json};

use crate::http::{base64, send_json};

const BASE_URL: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Haiku has no server-side fallback.
const FALLBACK_MODELS: &[&str] = &["claude-opus-5-5", "claude-sonnet-5-5"];

pub struct Anthropic {
    client: Client,
    base_url: String,
}

impl Anthropic {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, BASE_URL)
    }

    pub fn with_base_url(client: Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn body(request: &ProcessRequest<'_>) -> Value {
        let data = base64(&request.input.bytes);
        let media = if request.input.mime == "application/pdf" {
            json!({
                "type": "document",
                "source": { "type": "base64", "media_type": "application/pdf", "data": data },
            })
        } else {
            json!({
                "type": "image",
                "source": { "type": "base64", "media_type": request.input.mime, "data": data },
            })
        };
        let mut body = json!({
            "model": request.model,
            "max_tokens": 16_000,
            "system": request.prompt,
            "messages": [{
                "role": "user",
                "content": [media, { "type": "text", "text": request.user_prompt }],
            }],
            "output_config": { "effort": "low" },
        });
        if FALLBACK_MODELS.contains(&request.model) {
            body["fallbacks"] = json!("default");
        }
        body
    }

    fn parse(response: &Value) -> Result<ProcessOutput, ProviderError> {
        let stop = response
            .get("stop_reason")
            .and_then(Value::as_str)
            .unwrap_or("end_turn");
        if stop == "refusal" {
            let category = response
                .pointer("/stop_details/category")
                .and_then(Value::as_str)
                .unwrap_or("unspecified");
            return Err(ProviderError::Blocked(format!(
                "Claude declined this page (category: {category})"
            )));
        }
        let text: String = response
            .get("content")
            .and_then(Value::as_array)
            .map(|blocks| {
                blocks
                    .iter()
                    .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();
        if text.is_empty() && stop != "max_tokens" {
            return Err(ProviderError::Other("Claude returned no text".into()));
        }
        let count = |key: &str| {
            response
                .pointer(&format!("/usage/{key}"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        };
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                input_tokens: count("input_tokens"),
                output_tokens: count("output_tokens"),
                ..Usage::default()
            },
            model_version: response
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

#[async_trait]
impl Provider for Anthropic {
    fn id(&self) -> ProviderId {
        ProviderId::new("anthropic")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            Task::AudioTranscription => &[],
        }
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        if request.task == Task::AudioTranscription {
            return Err(ProviderError::Unsupported(
                "Claude does not accept audio".into(),
            ));
        }
        let mut http = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", key.expose())
            .header("anthropic-version", API_VERSION)
            .json(&Self::body(&request));
        if FALLBACK_MODELS.contains(&request.model) {
            http = http.header("anthropic-beta", FALLBACK_BETA);
        }
        Self::parse(&send_json(http).await?)
    }

    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        let response = send_json(
            self.client
                .get(format!("{}/models?limit=1000", self.base_url))
                .header("x-api-key", key.expose())
                .header("anthropic-version", API_VERSION),
        )
        .await?;
        Ok(ids(&response))
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
