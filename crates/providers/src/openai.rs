//! OpenAI: the Responses API for text extraction and summaries (PDF pages as `input_file`,
//! images as `input_image` at high detail) and the transcriptions endpoint for audio: speaker
//! labels from the diarizing model (deprecated, shutting down 2027-02-26), plain text with
//! several language hints from `gpt-transcribe`.
//!
//! Neither Python pipeline uses OpenAI for these tasks, so this follows the API reference
//! rather than ported code.
//!
//! Prompt caching is automatic: the instructions and schema come first and are shared by every
//! page, and the response reports the input tokens read from or written to the cache. Economy
//! mode sends Responses requests through the Batch API ([`batch`]); the transcriptions endpoint
//! has no batch service, so recordings always go one by one.

mod batch;

use async_trait::async_trait;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use scriptorium_core::{
    ApiKey, BatchContext, BatchLimits, BatchProgress, LanguageChoice, ProcessOutput,
    ProcessRequest, Provider, ProviderError, ProviderId, RequestSetting, Segment, Task, Usage,
};
use serde_json::{Value, json};

use crate::analysis;
use crate::http::{data_url, ids, join_turns, secs_to_ms, send_json, speaker_label};

const BASE_URL: &str = "https://api.openai.com/v1";
const MAX_OUTPUT_TOKENS: u32 = 32_000;

pub struct OpenAi {
    client: Client,
    base_url: String,
}

impl OpenAi {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, BASE_URL)
    }

    pub fn with_base_url(client: Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn extraction_body(request: &ProcessRequest<'_>) -> Value {
        let input = &request.input;
        let media = if input.mime == "text/plain" {
            json!({ "type": "input_text", "text": analysis::tagged_document(input) })
        } else if input.mime == "application/pdf" {
            json!({
                "type": "input_file",
                "filename": input.file_name,
                "file_data": data_url(&input.mime, &input.bytes),
            })
        } else {
            json!({
                "type": "input_image",
                "image_url": data_url(&input.mime, &input.bytes),
                "detail": "high",
            })
        };
        let mut body = json!({
            "model": request.model,
            "instructions": request.prompt,
            "input": [{
                "role": "user",
                "content": [media, { "type": "input_text", "text": request.user_prompt }],
            }],
            "max_output_tokens": MAX_OUTPUT_TOKENS,
            "store": false,
        });
        if let Some(fields) = request.analysis {
            body["text"] = json!({
                "format": {
                    "type": "json_schema",
                    "name": "analysis",
                    "schema": analysis::schema(fields),
                    "strict": true,
                }
            });
        }
        body
    }

    fn parse_extraction(response: &Value) -> Result<ProcessOutput, ProviderError> {
        if response.get("status").and_then(Value::as_str) == Some("incomplete") {
            let reason = response
                .pointer("/incomplete_details/reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            if reason == "content_filter" {
                return Err(ProviderError::Blocked(
                    "OpenAI's content filter stopped this page".into(),
                ));
            }
        }
        let mut text = String::new();
        let mut refusal = None;
        for item in response
            .get("output")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if item.get("type").and_then(Value::as_str) != Some("message") {
                continue;
            }
            for part in item
                .get("content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                match part.get("type").and_then(Value::as_str) {
                    Some("output_text") => {
                        text.push_str(part.get("text").and_then(Value::as_str).unwrap_or(""));
                    }
                    Some("refusal") => {
                        refusal = part
                            .get("refusal")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                    }
                    _ => {}
                }
            }
        }
        if text.is_empty() {
            return Err(match refusal {
                Some(r) => ProviderError::Blocked(r),
                None => ProviderError::Other("OpenAI returned no text".into()),
            });
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
                cached_input_tokens: count("input_tokens_details/cached_tokens"),
                cache_write_tokens: count("input_tokens_details/cache_write_tokens"),
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

    /// A response as the task reads it: an analysis's answer is checked against its fields.
    fn read(context: BatchContext<'_>, response: &Value) -> Result<ProcessOutput, ProviderError> {
        let output = Self::parse_extraction(response)?;
        match (context.task, context.analysis) {
            (Task::Analysis, Some(fields)) => analysis::finish(output, fields),
            (Task::Analysis, None) => Err(ProviderError::InvalidRequest(
                "an analysis needs fields".into(),
            )),
            _ => Ok(output),
        }
    }

    async fn transcribe(
        &self,
        key: &ApiKey,
        request: &ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        let input = request.input;
        let file = Part::bytes(input.bytes.clone())
            .file_name(input.file_name.clone())
            .mime_str(&input.mime)
            .map_err(|e| ProviderError::InvalidRequest(e.to_string()))?;
        let diarize = request.model.contains("diarize");
        let mut form = Form::new()
            .part("file", file)
            .text("model", request.model.to_string())
            .text(
                "response_format",
                if diarize { "diarized_json" } else { "json" },
            );
        if diarize {
            // Required for recordings over 30 seconds with speaker labels.
            form = form.text("chunking_strategy", "auto");
        }
        if let LanguageChoice::Explicit(codes) = request.languages {
            let codes: Vec<&str> = codes
                .iter()
                .map(|c| c.as_str())
                .filter(|c| c.len() == 2)
                .collect();
            if diarize {
                // One language only; for mixed recordings detection works better.
                if let [only] = codes.as_slice() {
                    form = form.text("language", (*only).to_string());
                }
            } else {
                // `gpt-transcribe` takes several hints, which helps with code-switching; it
                // replaces `language`, and the two must not be sent together.
                for code in codes {
                    form = form.text("languages[]", code.to_string());
                }
            }
        }
        let response = send_json(
            self.client
                .post(format!("{}/audio/transcriptions", self.base_url))
                .bearer_auth(key.expose())
                .multipart(form),
        )
        .await?;
        let segments: Vec<Segment> = join_turns(
            response
                .get("segments")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|s| Segment {
                    start_ms: secs_to_ms(s.get("start")),
                    end_ms: secs_to_ms(s.get("end")),
                    speaker: s.get("speaker").and_then(Value::as_str).map(speaker_label),
                    text: s
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                })
                .collect(),
        );
        let usage = response.get("usage");
        Ok(ProcessOutput {
            text: response
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            segments,
            // `gpt-transcribe` reports the languages it heard.
            detected_language: response
                .pointer("/languages/0")
                .and_then(|l| {
                    l.as_str()
                        .or_else(|| l.get("language").and_then(Value::as_str))
                })
                .map(scriptorium_core::LanguageCode::new),
            usage: Usage {
                input_tokens: usage
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                output_tokens: usage
                    .and_then(|u| u.get("output_tokens"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                audio_seconds: usage
                    .and_then(|u| u.get("seconds"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                ..Usage::default()
            },
            model_version: Some(request.model.to_string()),
            fields: Vec::new(),
            regions: Vec::new(),
        })
    }
}

#[async_trait]
impl Provider for OpenAi {
    fn id(&self) -> ProviderId {
        ProviderId::new("openai")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            // The formats the speech-to-text guide lists (October 2026): not FLAC or Ogg.
            Task::AudioTranscription => &["audio/mpeg", "audio/wav", "audio/mp4", "audio/webm"],
            Task::Analysis => &[
                "text/plain",
                "application/pdf",
                "image/png",
                "image/jpeg",
                "image/webp",
            ],
        }
    }

    fn request_settings(&self, task: Task, _model: &str) -> Vec<RequestSetting> {
        match task {
            // No reasoning effort is sent: each model runs at its own default.
            Task::TextExtraction | Task::Analysis => vec![
                RequestSetting::new("max_output_tokens", MAX_OUTPUT_TOKENS),
                RequestSetting::new("store", false),
            ],
            Task::AudioTranscription => Vec::new(),
        }
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        match request.task {
            Task::TextExtraction | Task::Analysis => {
                let response = send_json(
                    self.client
                        .post(format!("{}/responses", self.base_url))
                        .bearer_auth(key.expose())
                        .json(&Self::extraction_body(&request)),
                )
                .await?;
                let context = BatchContext {
                    task: request.task,
                    model: request.model,
                    analysis: request.analysis,
                };
                Self::read(context, &response)
            }
            Task::AudioTranscription => self.transcribe(key, &request).await,
        }
    }

    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        let response = send_json(
            self.client
                .get(format!("{}/models", self.base_url))
                .bearer_auth(key.expose()),
        )
        .await?;
        Ok(ids(&response))
    }

    fn batches(&self, task: Task, _model: &str) -> bool {
        task != Task::AudioTranscription
    }

    /// 200 MB of JSON Lines a batch, with pages as base64 data URLs (a third larger).
    fn batch_limits(&self) -> BatchLimits {
        BatchLimits {
            max_items: 50_000,
            max_bytes: 140 * 1024 * 1024,
        }
    }

    async fn submit_batch(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        self.submit(key, requests).await
    }

    async fn poll_batch(
        &self,
        key: &ApiKey,
        batch_id: &str,
        context: BatchContext<'_>,
        _ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        self.poll(key, batch_id, context).await
    }

    async fn cancel_batch(&self, key: &ApiKey, batch_id: &str) -> Result<(), ProviderError> {
        send_json(
            self.client
                .post(format!("{}/batches/{batch_id}/cancel", self.base_url))
                .bearer_auth(key.expose()),
        )
        .await
        .map(|_| ())
    }
}
