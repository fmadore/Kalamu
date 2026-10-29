//! Google Gemini through the `generateContent` REST endpoint, with the settings tested in
//! zmo- and iwac-ai-pipelines: high media resolution, a low thinking level, safety filters off
//! (historical documents trip them), and no temperature (lower values make Gemini 3 loop).
//! Summaries come back as JSON under a response schema.
//!
//! Google now recommends medium resolution for PDFs; high is kept because Kalamu's
//! material is dense historical print and handwriting, where iwac-ai-pipelines measured the
//! difference. Resolution is set for the whole request, the stable form, since each request
//! carries one piece.
//!
//! A YouTube video is sent as its link (`fileData.fileUri`), which Google fetches itself, at
//! low media resolution: the words are in the sound track, and low resolution reads about
//! 100 tokens a second of video instead of 300. YouTube links are in preview.
//!
//! Prompt caching is implicit: the system instruction comes first, and the response reports
//! the part of the prompt read from Google's cache. Economy mode sends pages and documents
//! through batch mode ([`batch`]); recordings and YouTube links, which a batch is not documented
//! to take, always go one by one.

mod batch;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, BatchContext, BatchLimits, BatchProgress, ProcessOutput, ProcessRequest, Provider,
    ProviderError, ProviderId, RequestSetting, Task, Usage, YOUTUBE_LINK,
};
use reqwest::Client;
use serde_json::{Value, json};

use crate::analysis;
use crate::http::{base64, send_json};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Pages above this go through the Files API in the Python pipelines; one page rarely does.
const INLINE_LIMIT: usize = 18 * 1024 * 1024;

const MAX_OUTPUT_TOKENS: u32 = 65_535;
const THINKING_LEVEL: &str = "low";
const MEDIA_RESOLUTION: &str = "MEDIA_RESOLUTION_HIGH";
const VIDEO_RESOLUTION: &str = "MEDIA_RESOLUTION_LOW";

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
        let input = request.input;
        let youtube = input.mime == YOUTUBE_LINK;
        let media = if input.mime == "text/plain" {
            json!({ "text": analysis::tagged_document(input) })
        } else if youtube {
            json!({ "fileData": { "fileUri": String::from_utf8_lossy(&input.bytes) } })
        } else {
            json!({
                "inlineData": {
                    "mimeType": input.mime,
                    "data": base64(&input.bytes),
                }
            })
        };
        let mut generation = json!({
            "maxOutputTokens": MAX_OUTPUT_TOKENS,
            "responseMimeType": "text/plain",
            "thinkingConfig": { "thinkingLevel": THINKING_LEVEL },
        });
        // Media resolution applies to images and documents; ULTRA_HIGH is refused for PDFs.
        if input.mime == "application/pdf" || input.mime.starts_with("image/") {
            generation["mediaResolution"] = json!(MEDIA_RESOLUTION);
        } else if youtube {
            generation["mediaResolution"] = json!(VIDEO_RESOLUTION);
        }
        if let Some(fields) = request.analysis {
            generation["responseMimeType"] = json!("application/json");
            generation["responseJsonSchema"] = analysis::schema(fields);
        }
        json!({
            "systemInstruction": { "parts": [{ "text": request.prompt }] },
            "contents": [{
                "role": "user",
                "parts": [media, { "text": request.user_prompt }],
            }],
            "generationConfig": generation,
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
                // Part of `promptTokenCount`.
                cached_input_tokens: count("cachedContentTokenCount"),
                ..Usage::default()
            },
            model_version: response
                .get("modelVersion")
                .and_then(Value::as_str)
                .map(str::to_string),
            fields: Vec::new(),
            regions: Vec::new(),
        })
    }

    /// A response as the task reads it: an analysis's answer is checked against its fields.
    fn read(context: BatchContext<'_>, response: &Value) -> Result<ProcessOutput, ProviderError> {
        let output = Self::parse(response)?;
        match (context.task, context.analysis) {
            (Task::Analysis | Task::Contents | Task::TextCorrection, Some(fields)) => {
                analysis::finish(output, fields)
            }
            (Task::Analysis | Task::Contents | Task::TextCorrection, None) => Err(
                ProviderError::InvalidRequest("an analysis needs fields".into()),
            ),
            _ => Ok(output),
        }
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
                "audio/webm",
                YOUTUBE_LINK,
            ],
            Task::Analysis | Task::Contents | Task::TextCorrection => &[
                "text/plain",
                "application/pdf",
                "image/png",
                "image/jpeg",
                "image/webp",
                "image/heic",
                "image/heif",
            ],
        }
    }

    fn request_settings(&self, task: Task, _model: &str) -> Vec<RequestSetting> {
        let mut settings = vec![
            RequestSetting::new(
                "generationConfig.thinkingConfig.thinkingLevel",
                THINKING_LEVEL,
            ),
            RequestSetting::new("generationConfig.maxOutputTokens", MAX_OUTPUT_TOKENS),
            RequestSetting::new("safetySettings.*.threshold", "BLOCK_NONE"),
        ];
        // For pages and pictures; a recording or a text file has no resolution to set.
        if task != Task::AudioTranscription {
            settings.push(RequestSetting::new(
                "generationConfig.mediaResolution",
                MEDIA_RESOLUTION,
            ));
        }
        settings
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
        .await;
        let youtube = request.input.mime == YOUTUBE_LINK;
        let response = response.map_err(|e| if youtube { youtube_error(e) } else { e })?;
        let output = Self::parse(&response)?;
        if youtube && truncated(&response) {
            return Err(ProviderError::InvalidRequest(
                "the transcript filled the longest answer Gemini can write before the video                  ended: this video is too long to transcribe in one go"
                    .into(),
            ));
        }
        if request.task.request_task() == Task::Analysis {
            analysis::finish(output, analysis::fields(&request)?)
        } else {
            Ok(output)
        }
    }

    fn batches(&self, task: Task, _model: &str) -> bool {
        task != Task::AudioTranscription
    }

    /// Requests sent inline must total under 20 MB, with pages as base64 (a third larger).
    fn batch_limits(&self) -> BatchLimits {
        BatchLimits {
            max_items: 1_000,
            max_bytes: 13 * 1024 * 1024,
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
        ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        self.poll(key, batch_id, context, ids).await
    }

    async fn cancel_batch(&self, key: &ApiKey, batch_id: &str) -> Result<(), ProviderError> {
        send_json(
            self.client
                .post(format!("{}/{batch_id}:cancel", self.base_url))
                .header("x-goog-api-key", key.expose()),
        )
        .await
        .map(|_| ())
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

/// The answer stopped at the output limit. A page keeps what was read, but a video's transcript
/// would end part-way with nothing to show where.
fn truncated(response: &Value) -> bool {
    response
        .pointer("/candidates/0/finishReason")
        .and_then(Value::as_str)
        == Some("MAX_TOKENS")
}

/// Gemini's refusals of a YouTube link, in words a researcher can act on. A private video comes
/// back as "permission denied", which must not read as a rejected key and stop the job.
fn youtube_error(err: ProviderError) -> ProviderError {
    let message = match &err {
        ProviderError::Auth(m) | ProviderError::InvalidRequest(m) | ProviderError::Other(m) => m,
        ProviderError::QuotaExhausted(m) => {
            let lower = m.to_lowercase();
            return if lower.contains("youtube") || lower.contains("video") {
                ProviderError::QuotaExhausted(
                    "the free tier's daily allowance of YouTube video (8 hours) is used up; try                      again tomorrow or use a key on a paid plan"
                        .into(),
                )
            } else {
                err
            };
        }
        _ => return err,
    };
    let lower = message.to_lowercase();
    if lower.contains("api key") || lower.contains("api_key") {
        return err;
    }
    if lower.contains("private") || lower.contains("unlisted") || lower.contains("permission") {
        ProviderError::InvalidRequest(
            "this video is private or unlisted; Gemini reads public videos only".into(),
        )
    } else if lower.contains("not found")
        || lower.contains("unavailable")
        || lower.contains("removed")
        || lower.contains("does not exist")
    {
        ProviderError::InvalidRequest(
            "this video is not available on YouTube; it may have been removed".into(),
        )
    } else if lower.contains("token") && (lower.contains("exceed") || lower.contains("maximum")) {
        ProviderError::InvalidRequest(
            "this video is too long for the model to read in one request".into(),
        )
    } else if let ProviderError::Auth(m) = err {
        // A 403 that does not mention the key concerns the video, not the account.
        ProviderError::InvalidRequest(m)
    } else {
        err
    }
}
