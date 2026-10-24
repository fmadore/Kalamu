//! Google Gemini through the `generateContent` REST endpoint, with the settings tested in
//! zmo- and iwac-ai-pipelines: high media resolution per part, the lowest thinking level the
//! models accept, safety filters off (historical documents trip them), and no temperature
//! (lower values make Gemini 3 loop).

use async_trait::async_trait;
use reqwest::Client;
use scriptorium_core::{
    ApiKey, ProcessOutput, ProcessRequest, Provider, ProviderError, ProviderId, Task, Usage,
};
use serde_json::{Value, json};

use crate::http::{base64, send_json};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Pages above this go through the Files API in the Python pipelines; one page rarely does.
const INLINE_LIMIT: usize = 18 * 1024 * 1024;

const SAFETY_CATEGORIES: &[&str] = &[
    "HARM_CATEGORY_HARASSMENT",
    "HARM_CATEGORY_HATE_SPEECH",
    "HARM_CATEGORY_SEXUALLY_EXPLICIT",
    "HARM_CATEGORY_DANGEROUS_CONTENT",
];

pub struct Gemini {
    client: Client,
    base_url: String,
}

impl Gemini {
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
        let mut media = json!({
            "inlineData": {
                "mimeType": request.input.mime,
                "data": base64(&request.input.bytes),
            }
        });
        // Media resolution applies to images and documents; ULTRA_HIGH is refused for PDFs.
        if !request.input.mime.starts_with("audio/") {
            media["mediaResolution"] = json!({ "level": "MEDIA_RESOLUTION_HIGH" });
        }
        json!({
            "systemInstruction": { "parts": [{ "text": request.prompt }] },
            "contents": [{
                "role": "user",
                "parts": [media, { "text": request.user_prompt }],
            }],
            "generationConfig": {
                "maxOutputTokens": 65_535,
                "responseMimeType": "text/plain",
                "thinkingConfig": { "thinkingLevel": "LOW" },
            },
            "safetySettings": SAFETY_CATEGORIES
                .iter()
                .map(|c| json!({ "category": c, "threshold": "BLOCK_NONE" }))
                .collect::<Vec<_>>(),
        })
    }

    fn parse(response: &Value) -> Result<ProcessOutput, ProviderError> {
        if let Some(reason) = response
            .pointer("/promptFeedback/blockReason")
            .and_then(Value::as_str)
        {
            return Err(ProviderError::Blocked(format!(
                "the request was blocked ({reason})"
            )));
        }
        let candidate = response
            .pointer("/candidates/0")
            .ok_or_else(|| ProviderError::Other("no candidate in the response".into()))?;
        let finish = candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .unwrap_or("STOP");
        // Thought summaries are not part of the transcription.
        let text: String = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
            .map(|parts| {
                parts
                    .iter()
                    .filter(|p| !p.get("thought").and_then(Value::as_bool).unwrap_or(false))
                    .filter_map(|p| p.get("text").and_then(Value::as_str))
                    .collect()
            })
            .unwrap_or_default();
        match finish {
            "STOP" | "FINISH_REASON_UNSPECIFIED" => {}
            // Keep what was transcribed; the engine marks the page as truncated.
            "MAX_TOKENS" if !text.is_empty() => {}
            "RECITATION" => {
                return Err(ProviderError::Blocked(
                    "Gemini's copyright filter stopped this page (RECITATION)".into(),
                ));
            }
            other if text.is_empty() => {
                return Err(ProviderError::Blocked(format!(
                    "no text returned ({other})"
                )));
            }
            _ => {}
        }
        let usage = response.get("usageMetadata");
        let count = |key: &str| {
            usage
                .and_then(|u| u.get(key))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        };
        Ok(ProcessOutput {
            text,
            segments: Vec::new(),
            detected_language: None,
            usage: Usage {
                input_tokens: count("promptTokenCount"),
                // Thinking tokens are billed as output.
                output_tokens: count("candidatesTokenCount") + count("thoughtsTokenCount"),
                ..Usage::default()
            },
            model_version: response
                .get("modelVersion")
                .and_then(Value::as_str)
                .map(str::to_string),
        })
    }
}

#[async_trait]
impl Provider for Gemini {
    fn id(&self) -> ProviderId {
        ProviderId::new("gemini")
    }

    fn accepted_inputs(&self, task: Task) -> &'static [&'static str] {
        match task {
            Task::TextExtraction => &[
                "application/pdf",
                "image/png",
                "image/jpeg",
                "image/webp",
                "image/heic",
                "image/heif",
            ],
            Task::AudioTranscription => &[
                "audio/mpeg",
                "audio/wav",
                "audio/mp4",
                "audio/flac",
                "audio/ogg",
                "audio/aac",
            ],
        }
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError> {
        if request.input.bytes.len() > INLINE_LIMIT {
            return Err(ProviderError::InvalidRequest(format!(
                "this piece is {} MB; Gemini accepts up to 18 MB per request",
                request.input.bytes.len() / (1024 * 1024)
            )));
        }
        let url = format!("{}/models/{}:generateContent", self.base_url, request.model);
        let response = send_json(
            self.client
                .post(url)
                .header("x-goog-api-key", key.expose())
                .json(&Self::body(&request)),
        )
        .await?;
        Self::parse(&response)
    }

    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError> {
        let response = send_json(
            self.client
                .get(format!("{}/models?pageSize=1000", self.base_url))
                .header("x-goog-api-key", key.expose()),
        )
        .await?;
        Ok(response
            .get("models")
            .and_then(Value::as_array)
            .map(|models| {
                models
                    .iter()
                    .filter_map(|m| m.get("name").and_then(Value::as_str))
                    .map(|n| n.trim_start_matches("models/").to_string())
                    .collect()
            })
            .unwrap_or_default())
    }
}
