//! Mistral: the OCR endpoint for text extraction (no prompt; returns Markdown) and Voxtral for
//! transcription, as in iwac-ai-pipelines (`common/mistral_ocr.py`,
//! `02b_AI_transcribe_audio_voxtral.py`), and the chat models for analyses ([`chat`]). Pages go
//! inline as data URLs rather than through the file upload the Python code uses, since each
//! request carries a single page.
//!
//! Economy mode sends OCR and analysis requests as one batch job ([`batch`]), at half price.
//! Recordings go one by one: Mistral's batch service lists the transcriptions endpoint, but not
//! how a recording travels inside a batch.

mod batch;
mod chat;

use async_trait::async_trait;
use kalamu_core::{
    ApiKey, BatchContext, BatchLimits, BatchProgress, LanguageChoice, LanguageCode, ProcessOutput,
    ProcessRequest, Provider, ProviderError, ProviderId, RequestSetting, Segment, Task, TextRegion,
    Usage,
};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use serde_json::{Value, json};

use crate::analysis;
use crate::http::{data_url, ids, join_turns, secs_to_ms, send_json, speaker_label};

const BASE_URL: &str = "https://api.mistral.ai/v1";
const OCR_TABLE_FORMAT: &str = "html";

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
            "table_format": OCR_TABLE_FORMAT,
            // Paragraph boxes, for searchable PDFs (OCR 4 and later).
            "include_blocks": true,
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
        let regions = pages.first().map(page_regions).unwrap_or_default();
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
            fields: Vec::new(),
            regions,
            truncated: false,
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
        let segments = join_turns(
            response
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
                        .map(|id| speaker_label(&id)),
                    text: s
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                })
                .collect(),
        );
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
            fields: Vec::new(),
            regions: Vec::new(),
            truncated: false,
        })
    }
}

/// The page's text blocks with their boxes, as fractions of the page. Pictures and signatures
/// carry no text to search.
fn page_regions(page: &Value) -> Vec<TextRegion> {
    let size = |key: &str| {
        page.pointer(&format!("/dimensions/{key}"))
            .and_then(Value::as_f64)
            .filter(|v| *v > 0.0)
    };
    let (Some(width), Some(height)) = (size("width"), size("height")) else {
        return Vec::new();
    };
    page.get("blocks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|b| {
            !matches!(
                b.get("type").and_then(Value::as_str),
                Some("image" | "signature")
            )
        })
        .filter_map(|b| {
            let n = |key: &str| b.get(key).and_then(Value::as_f64);
            let (x0, y0, x1, y1) = (
                n("top_left_x")?,
                n("top_left_y")?,
                n("bottom_right_x")?,
                n("bottom_right_y")?,
            );
            let text = plain_text(b.get("content").and_then(Value::as_str)?);
            (!text.is_empty() && x1 > x0 && y1 > y0).then(|| TextRegion {
                x: (x0 / width) as f32,
                y: (y0 / height) as f32,
                width: ((x1 - x0) / width) as f32,
                height: ((y1 - y0) / height) as f32,
                text,
            })
        })
        .collect()
}

/// A block's Markdown or HTML (tables) as the words a reader would search for.
fn plain_text(content: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    for c in content.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let name = tag.trim_start_matches('/').to_ascii_lowercase();
                if name.starts_with("td") || name.starts_with("th") {
                    out.push(' ');
                } else if name.starts_with("tr") || name.starts_with("br") || name.starts_with('p')
                {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(c),
            '*' | '_' | '`' => {}
            _ => out.push(c),
        }
    }
    out.lines()
        .map(|l| {
            l.trim()
                .trim_start_matches('#')
                .trim_start_matches(['-', '>'])
                .trim()
        })
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
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
        // Markdown's two-space line breaks mean nothing in a text file.
        out.push(line.trim_end());
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
            Task::Analysis | Task::Contents | Task::TextCorrection => &[
                "text/plain",
                "application/pdf",
                "image/png",
                "image/jpeg",
                "image/webp",
            ],
        }
    }

    fn request_settings(&self, task: Task, model: &str) -> Vec<RequestSetting> {
        match task {
            Task::TextExtraction => vec![
                RequestSetting::new("include_image_base64", false),
                RequestSetting::new("table_format", OCR_TABLE_FORMAT),
                RequestSetting::new("include_blocks", true),
            ],
            Task::Analysis | Task::Contents | Task::TextCorrection => chat::request_settings(model),
            Task::AudioTranscription => Vec::new(),
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
            Task::Analysis | Task::Contents | Task::TextCorrection => {
                let fields = analysis::fields(&request)?;
                let response = send_json(
                    self.client
                        .post(format!("{}/chat/completions", self.base_url))
                        .bearer_auth(key.expose())
                        .json(&chat::body(&request, fields)),
                )
                .await?;
                analysis::finish(chat::parse(&response)?, fields)
            }
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

    /// Inline batches take fewer than 10,000 requests; Mistral gives no size limit for them, so
    /// a batch is kept to a size its other limits allow.
    fn batch_limits(&self) -> BatchLimits {
        BatchLimits {
            max_items: 1_000,
            max_bytes: 50 * 1024 * 1024,
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
                .post(format!("{}/batch/jobs/{batch_id}/cancel", self.base_url))
                .bearer_auth(key.expose()),
        )
        .await
        .map(|_| ())
    }
}
