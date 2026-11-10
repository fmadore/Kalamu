//! Mistral's chat models (Large, Medium, Small) and the Z.ai GLM and DeepSeek models Mistral
//! hosts, for analyses, through Chat Completions with a strict JSON schema. Text extraction stays with
//! Mistral OCR, which Mistral's documentation points to for document parsing. A PDF sent here as
//! `document_url` is read by that OCR before the model sees it (Mistral's "Document QnA").
//!
//! `reasoning_effort` is the level set for the job, from the levels the catalogue lists for the
//! model, whose lowest is the suggestion: `none` for Mistral's own models, whose answer then
//! comes back as one string, and `low` for GLM and DeepSeek, which offer only `low`, `high` and
//! `max` and may answer in thinking and text chunks. A model the catalogue lacks is sent none,
//! and thinks as much as Mistral's default for it: no level is accepted by every model.
//! `max_tokens` is the model's documented output limit, and is left out where Mistral documents
//! none, so the model writes until it is done.

use kalamu_core::{
    AnalysisSchema, ProcessOutput, ProcessRequest, ProviderError, RequestSetting, Tuning, Usage,
};
use serde_json::{Value, json};

use crate::analysis;
use crate::http::data_url;

pub(super) fn request_settings(tuning: &Tuning) -> Vec<RequestSetting> {
    let mut settings = Vec::new();
    if let Some(effort) = &tuning.reasoning {
        settings.push(RequestSetting::new("reasoning_effort", effort.as_str()));
    }
    if let Some(max) = tuning.max_output_tokens {
        settings.push(RequestSetting::new("max_tokens", max));
    }
    settings
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
    let mut body = json!({
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
    });
    if let Some(effort) = &request.tuning.reasoning {
        body["reasoning_effort"] = json!(effort);
    }
    if let Some(max) = request.tuning.max_output_tokens {
        body["max_tokens"] = json!(max);
    }
    body
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
    let finish = choice
        .and_then(|c| c.get("finish_reason"))
        .and_then(Value::as_str);
    Ok(ProcessOutput {
        text,
        truncated: finish == Some("length"),
        usage: Usage {
            input_tokens: count("prompt_tokens"),
            output_tokens: count("completion_tokens"),
            // Part of `prompt_tokens`, billed at a tenth.
            cached_input_tokens: count("prompt_tokens_details/cached_tokens"),
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
        let cut = parse(&json!({
            "choices": [{ "index": 0, "finish_reason": "length",
                "message": { "role": "assistant", "content": "{\"summary\":\"Bamako, le" } }],
        }))
        .unwrap();
        assert!(cut.truncated, "kept, and marked as cut off");
        assert!(!out_of(&json!("{}")).truncated);
    }

    fn out_of(content: &Value) -> ProcessOutput {
        parse(&json!({
            "choices": [{ "index": 0, "finish_reason": "stop",
                "message": { "role": "assistant", "content": content } }],
        }))
        .unwrap()
    }
}
