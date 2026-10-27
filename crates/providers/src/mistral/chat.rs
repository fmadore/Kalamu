//! Mistral's chat models (Large, Medium, Small) and the Z.ai GLM models Mistral hosts, for
//! analyses, through Chat Completions with a strict JSON schema. Text extraction stays with
//! Mistral OCR, which Mistral's documentation points to for document parsing. A PDF sent here as
//! `document_url` is read by that OCR before the model sees it (Mistral's "Document QnA").
//!
//! `reasoning_effort` is the lowest setting, as the other providers run at their lowest effort:
//! `none` for Mistral's own models, whose answer then comes back as one string, and `low` for
//! GLM, which rejects `none` and may answer in thinking and text chunks.

use scriptorium_core::{
    AnalysisSchema, ProcessOutput, ProcessRequest, ProviderError, RequestSetting, Usage,
};
use serde_json::{Value, json};

use crate::analysis;
use crate::http::data_url;

const MAX_TOKENS: u32 = 16_000;

pub(super) fn request_settings(model: &str) -> Vec<RequestSetting> {
    vec![
        RequestSetting::new("reasoning_effort", lowest_effort(model)),
        RequestSetting::new("max_tokens", MAX_TOKENS),
    ]
}

pub(super) fn body(request: &ProcessRequest<'_>, fields: &AnalysisSchema) -> Value {
    let input = request.input;
    let media = if input.mime == "text/plain" {
        json!({ "type": "text", "text": analysis::tagged_document(input) })
    } else if input.mime == "application/pdf" {
        json!({
            "type": "document_url",
            "document_url": data_url(&input.mime, &input.bytes),
            "document_name": input.file_name,
        })
    } else {
        json!({ "type": "image_url", "image_url": data_url(&input.mime, &input.bytes) })
    };
    json!({
        "model": request.model,
        "messages": [
            { "role": "system", "content": request.prompt },
            {
                "role": "user",
                "content": [media, { "type": "text", "text": request.user_prompt }],
            },
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "analysis",
                "schema": analysis::schema(fields),
                "strict": true,
            },
        },
        "reasoning_effort": lowest_effort(request.model),
        "max_tokens": MAX_TOKENS,
    })
}

/// Z.ai's GLM accepts `low`, `high` and `max` only.
fn lowest_effort(model: &str) -> &'static str {
    if model.starts_with("zai-glm") {
        "low"
    } else {
        "none"
    }
}

pub(super) fn parse(response: &Value) -> Result<ProcessOutput, ProviderError> {
    let choice = response.pointer("/choices/0");
    let text = match choice.and_then(|c| c.pointer("/message/content")) {
        Some(Value::String(text)) => text.clone(),
        // With reasoning on, thinking and text come as chunks; only the text is the answer.
        Some(Value::Array(chunks)) => chunks
            .iter()
            .filter(|c| c.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|c| c.get("text").and_then(Value::as_str))
            .collect(),
        _ => String::new(),
    };
    if text.is_empty() {
        let reason = choice
            .and_then(|c| c.get("finish_reason"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return Err(ProviderError::Other(format!(
            "Mistral returned no text (finish reason: {reason})"
        )));
    }
    let count = |key: &str| {
        response
            .pointer(&format!("/usage/{key}"))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    };
    Ok(ProcessOutput {
        text,
        usage: Usage {
            input_tokens: count("prompt_tokens"),
            output_tokens: count("completion_tokens"),
            ..Usage::default()
        },
        model_version: response
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string),
        ..ProcessOutput::default()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_answer_from_a_string_or_from_text_chunks() {
        for content in [
            json!("{\"summary\":\"x\"}"),
            json!([
                { "type": "thinking", "thinking": [{ "type": "text", "text": "Hmm." }] },
                { "type": "text", "text": "{\"summary\":\"x\"}" },
            ]),
        ] {
            let out = parse(&json!({
                "model": "mistral-small-2603",
                "choices": [{ "index": 0, "finish_reason": "stop",
                    "message": { "role": "assistant", "content": content } }],
                "usage": { "prompt_tokens": 120, "completion_tokens": 8, "total_tokens": 128 },
            }))
            .unwrap();
            assert_eq!(out.text, "{\"summary\":\"x\"}");
            assert_eq!((out.usage.input_tokens, out.usage.output_tokens), (120, 8));
            assert_eq!(out.model_version.as_deref(), Some("mistral-small-2603"));
        }
        let empty = parse(&json!({
            "choices": [{ "index": 0, "finish_reason": "length",
                "message": { "role": "assistant", "content": "" } }],
        }));
        assert!(matches!(empty, Err(ProviderError::Other(m)) if m.contains("length")));
    }
}
