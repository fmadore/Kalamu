//! Anthropic Claude through the Messages API, as raw HTTP (there is no official Rust SDK).
//!
//! API version 2026-10-21, which refuses sampling parameters, so none are sent. Effort is set to
//! `low`, the setting iwac-ai-pipelines uses for page transcription. On models that support it,
//! a safety-classifier refusal is retried server-side on a fallback model (`fallbacks:
//! "default"`); the response's `model` field names the model that wrote the text, and that is
//! what provenance records.
//!
//! The system prompt carries a cache marker: it is the part every page of a job shares, and on
//! the current models a prefix of 512 tokens or more is then read back at a tenth of the price
//! or less. The marker is on the system block, not at the request's top level, where the
//! automatic breakpoint would fall after the page and write every page to the cache for nothing.
//!
//! Economy mode uses Message Batches: the same requests at half price, answered within a day.
//! A batch refuses `fallbacks`, so batched requests go without it.

use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use scriptorium_core::{
    ApiKey, BatchContext, BatchLimits, BatchProgress, ProcessOutput, ProcessRequest, Provider,
    ProviderError, ProviderId, RequestSetting, Task, Usage,
};
use serde_json::{Value, json};

use crate::analysis;
use crate::batch::{self, Answer};
use crate::http::{base64, ids, send_json, send_text};

const BASE_URL: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2026-10-21";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Haiku has no server-side fallback.
const FALLBACK_MODELS: &[&str] = &["claude-opus-5-5", "claude-sonnet-5-5"];
const MAX_TOKENS: u32 = 16_000;
const EFFORT: &str = "low";

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

    fn body(request: &ProcessRequest<'_>, batched: bool) -> Value {
        let input = request.input;
        let data = base64(&input.bytes);
        let media = if input.mime == "text/plain" {
            json!({
                "type": "document",
                "source": {
                    "type": "text",
                    "media_type": "text/plain",
                    "data": analysis::document_text(input),
                },
                "title": input.file_name,
            })
        } else if input.mime == "application/pdf" {
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
            "max_tokens": MAX_TOKENS,
            "system": [{
                "type": "text",
                "text": request.prompt,
                "cache_control": { "type": "ephemeral" },
            }],
            "messages": [{
                "role": "user",
                "content": [media, { "type": "text", "text": request.user_prompt }],
            }],
            "output_config": { "effort": EFFORT },
        });
        if !batched && FALLBACK_MODELS.contains(&request.model) {
            body["fallbacks"] = json!("default");
        }
        if let Some(fields) = request.analysis {
            body["output_config"]["format"] = json!({
                "type": "json_schema",
                "schema": analysis::schema(fields),
            });
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
        // `input_tokens` counts only what follows the last cache marker.
        let (read, written) = (
            count("cache_read_input_tokens"),
            count("cache_creation_input_tokens"),
        );
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                input_tokens: count("input_tokens") + read + written,
                output_tokens: count("output_tokens"),
                cached_input_tokens: read,
                cache_write_tokens: written,
                ..Usage::default()
            },
            model_version: response
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string),
            fields: Vec::new(),
            regions: Vec::new(),
        })
    }

    /// A message as the task reads it: an analysis's answer is checked against its fields.
    fn read(context: BatchContext<'_>, response: &Value) -> Result<ProcessOutput, ProviderError> {
        let output = Self::parse(response)?;
        match (context.task, context.analysis) {
            (Task::Analysis, Some(fields)) => analysis::finish(output, fields),
            (Task::Analysis, None) => Err(ProviderError::InvalidRequest(
                "an analysis needs fields".into(),
            )),
            _ => Ok(output),
        }
    }

    fn authorised(&self, request: RequestBuilder, key: &ApiKey) -> RequestBuilder {
        request
            .header("x-api-key", key.expose())
            .header("anthropic-version", API_VERSION)
    }

    /// One line of a batch's results.
    fn answer(context: BatchContext<'_>, line: &Value) -> Option<Answer> {
        let id = line.get("custom_id")?.as_str()?.to_string();
        let result = line.get("result");
        let answer = match result.and_then(|r| r.get("type")).and_then(Value::as_str) {
            Some("succeeded") => match result.and_then(|r| r.get("message")) {
                Some(message) => Self::read(context, message),
                None => Err(ProviderError::Other("Claude returned no message".into())),
            },
            Some("expired") => Err(ProviderError::Transient(
                "the batch ended before Claude reached this piece".into(),
            )),
            Some("canceled") => Err(ProviderError::Interrupted),
            _ => Err(batch::item_error(
                result.and_then(|r| batch::first(r, &["/error/error", "/error"])),
            )),
        };
        Some((id, answer))
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
            Task::Analysis => &[
                "text/plain",
                "application/pdf",
                "image/png",
                "image/jpeg",
                "image/webp",
            ],
        }
    }

    fn request_settings(&self, task: Task, model: &str) -> Vec<RequestSetting> {
        if task == Task::AudioTranscription {
            return Vec::new();
        }
        let mut settings = vec![
            RequestSetting::new("output_config.effort", EFFORT),
            RequestSetting::new("max_tokens", MAX_TOKENS),
            RequestSetting::new("system.*.cache_control.type", "ephemeral"),
        ];
        if FALLBACK_MODELS.contains(&model) {
            settings.push(RequestSetting::new("fallbacks", "default"));
        }
        settings
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
            .authorised(self.client.post(format!("{}/messages", self.base_url)), key)
            .json(&Self::body(&request, false));
        if FALLBACK_MODELS.contains(&request.model) {
            http = http.header("anthropic-beta", FALLBACK_BETA);
        }
        let context = BatchContext {
            task: request.task,
            model: request.model,
            analysis: request.analysis,
        };
        Self::read(context, &send_json(http).await?)
    }

    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        let response = send_json(
            self.authorised(
                self.client
                    .get(format!("{}/models?limit=1000", self.base_url)),
                key,
            ),
        )
        .await?;
        Ok(ids(&response))
    }

    fn batches(&self, task: Task, _model: &str) -> bool {
        task != Task::AudioTranscription
    }

    /// 256 MB of requests a batch, with pages sent as base64 (a third larger).
    fn batch_limits(&self) -> BatchLimits {
        BatchLimits {
            max_items: 100_000,
            max_bytes: 150 * 1024 * 1024,
        }
    }

    async fn submit_batch(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let requests: Vec<Value> = requests
            .iter()
            .map(|(id, request)| json!({ "custom_id": id, "params": Self::body(request, true) }))
            .collect();
        let response = send_json(
            self.authorised(
                self.client
                    .post(format!("{}/messages/batches", self.base_url)),
                key,
            )
            .json(&json!({ "requests": requests })),
        )
        .await?;
        response
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ProviderError::Other("Claude gave the batch no id".into()))
    }

    async fn poll_batch(
        &self,
        key: &ApiKey,
        batch_id: &str,
        context: BatchContext<'_>,
        _ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        let url = format!("{}/messages/batches/{batch_id}", self.base_url);
        let status = send_json(self.authorised(self.client.get(&url), key)).await?;
        let counts = |name: &str| batch::count(status.pointer(&format!("/request_counts/{name}")));
        if status.get("processing_status").and_then(Value::as_str) != Some("ended") {
            let finished = counts("succeeded") + counts("errored") + counts("canceled");
            return Ok(BatchProgress::Working {
                finished,
                total: finished + counts("processing") + counts("expired"),
            });
        }
        let results = status
            .get("results_url")
            .and_then(Value::as_str)
            .map_or_else(|| format!("{url}/results"), str::to_string);
        let body = send_text(self.authorised(self.client.get(results), key)).await?;
        Ok(BatchProgress::Ended(
            batch::lines(&body)
                .iter()
                .filter_map(|line| Self::answer(context, line))
                .collect(),
        ))
    }

    async fn cancel_batch(&self, key: &ApiKey, batch_id: &str) -> Result<(), ProviderError> {
        let url = format!("{}/messages/batches/{batch_id}/cancel", self.base_url);
        send_json(self.authorised(self.client.post(url), key))
            .await
            .map(|_| ())
    }
}
