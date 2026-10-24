//! Mistral: the OCR endpoint for text extraction (no prompt; returns Markdown) and Voxtral for
//! transcription, as in iwac-ai-pipelines (`common/mistral_ocr.py`,
//! `02b_AI_transcribe_audio_voxtral.py`). Pages go inline as data URLs rather than through the
//! file upload the Python code uses, since each request carries a single page.

use async_trait::async_trait;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use scriptorium_core::{
    ApiKey, LanguageChoice, LanguageCode, ProcessOutput, ProcessRequest, Provider, ProviderError,
    ProviderId, Segment, Task, Usage,
};
use serde_json::{Value, json};

use crate::http::{data_url, ids, secs_to_ms, send_json};

const BASE_URL: &str = "https://api.mistral.ai/v1";

pub struct Mistral {
    client: Client,
    base_url: String,
}

impl Mistral {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, BASE_URL)
    }

    pub fn with_base_url(client: Client, base_url: &str) -> Self {
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn ocr_body(request: &ProcessRequest<'_>) -> Value {
        let url = data_url(&request.input.mime, &request.input.bytes);
        let document = if request.input.mime == "application/pdf" {
            json!({ "type": "document_url", "document_url": url })
        } else {
            json!({ "type": "image_url", "image_url": url })
        };
        json!({
            "model": request.model,
            "document": document,
            "include_image_base64": false,
            "table_format": "html",
        })
    }

    fn parse_ocr(response: &Value) -> Result<ProcessOutput, ProviderError> {
        let pages = response
            .get("pages")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::Other("no pages in the OCR response".into()))?;
        let text = pages
            .iter()
            .filter_map(|p| p.get("markdown").and_then(Value::as_str))
            .map(strip_image_links)
            .collect::<Vec<_>>()
            .join("\n\n");
        let processed = response
            .pointer("/usage_info/pages_processed")
            .and_then(Value::as_u64)
            .unwrap_or(pages.len() as u64);
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                pages: processed as u32,
                ..Usage::default()
            },
            model_version: response
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
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
        let mut form = Form::new()
            .part("file", file)
            .text("model", request.model.to_string());
        // Voxtral takes either a language or timestamps, not both: timestamps win when asked.
        let single_language = match request.languages {
            LanguageChoice::Explicit(codes) if codes.len() == 1 => Some(codes[0].clone()),
            _ => None,
        };
        let wants_times = request.options.timestamps || request.options.diarize;
        match (wants_times, single_language) {
            (false, Some(code)) => form = form.text("language", code.as_str().to_string()),
            _ => {
                form = form.text("timestamp_granularities", "segment");
                if request.options.diarize {
                    form = form.text("diarize", "true");
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
        let segments = response
            .get("segments")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|s| Segment {
                start_ms: secs_to_ms(s.get("start")),
                end_ms: secs_to_ms(s.get("end")),
                speaker: s
                    .get("speaker_id")
                    .and_then(|v| {
                        v.as_str()
                            .map(str::to_string)
                            .or_else(|| v.as_u64().map(|n| n.to_string()))
                    })
                    .map(|id| format!("Speaker {id}")),
                text: s
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            })
            .collect();
        Ok(ProcessOutput {
            text: response
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            segments,
            detected_language: response
                .get("language")
                .and_then(Value::as_str)
                .map(LanguageCode::new),
            usage: Usage {
                audio_seconds: response
                    .pointer("/usage/prompt_audio_seconds")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                input_tokens: response
                    .pointer("/usage/prompt_tokens")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                ..Usage::default()
            },
            model_version: response
                .get("model")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

/// Mistral OCR leaves `![img-0.jpeg](img-0.jpeg)` where pictures were; they point nowhere.
fn strip_image_links(markdown: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in markdown.lines() {
        let t = line.trim();
        if t.starts_with("![") && t.ends_with(')') && t.contains("](") {
            continue;
        }
        // Removing a picture leaves two blank lines; keep one.
        if t.is_empty() && out.last().is_some_and(|l| l.trim().is_empty()) {
            continue;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

#[async_trait]
impl Provider for Mistral {
    fn id(&self) -> ProviderId {
        ProviderId::new("mistral")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            Task::AudioTranscription => &[
                "audio/mpeg",
                "audio/wav",
                "audio/mp4",
                "audio/flac",
                "audio/ogg",
                "audio/webm",
            ],
        }
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        match request.task {
            Task::TextExtraction => {
                let response = send_json(
                    self.client
                        .post(format!("{}/ocr", self.base_url))
                        .bearer_auth(key.expose())
                        .json(&Self::ocr_body(&request)),
                )
                .await?;
                Self::parse_ocr(&response)
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
}
