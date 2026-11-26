//! An OpenAI-style server (vLLM, llama.cpp, LM Studio, a university's gateway, OpenRouter),
//! `POST {base}/chat/completions`, with only what every such server reads: system and user
//! messages, a page as an `image_url` object, `max_tokens` (Ollama's `/v1` ignores
//! `max_completion_tokens`), and for a structured answer a strict `json_schema`. No reasoning
//! level. OpenRouter's compression of long texts is turned off.
//!
//! Such a server may cut a long text without a word: after an answer to a text, the tokens it
//! says it read are compared with the characters sent, and fewer than one token for eight
//! characters is refused as a reading in part. An answer without `usage` is taken on its word.

use kalamu_core::{
    AnalysisSchema, CutReason, ProcessOutput, ProcessRequest, ProviderError, RequestSetting, Usage,
};
use serde_json::{Value, json};

use super::{Input, Limits, input_kind, without_thinking};
use crate::analysis;
use crate::http::data_url;

/// Fewer tokens read than one for this many characters sent is a reading in part: no language
/// is written so compactly.
const MAX_CHARS_PER_TOKEN: u64 = 8;

/// What every request carries beyond the prompt and the input, as provenance records it: the
/// value [`body`] writes, from the same [`Limits`].
pub(super) fn settings(limits: &Limits) -> Vec<RequestSetting> {
    vec![RequestSetting::new("max_tokens", limits.answer)]
}

/// A request's body, and for one that carries a text, the characters of text its messages hold.
pub(super) struct Sent {
    pub body: Value,
    pub text_chars: Option<usize>,
}

pub(super) fn body(
    request: &ProcessRequest<'_>,
    fields: Option<&AnalysisSchema>,
    limits: &Limits,
    openrouter: bool,
    name: &str,
) -> Result<Sent, ProviderError> {
    let input = request.input;
    let mut chars = request.prompt.chars().count();
    let (content, text) = match input_kind(request, name)? {
        Input::Text => {
            let text = format!(
                "{}\n\n{}",
                analysis::tagged_document(input),
                request.user_prompt
            );
            chars += text.chars().count();
            (json!(text), true)
        }
        Input::Image => (
            json!([
                { "type": "image_url", "image_url": { "url": data_url(&input.mime, &input.bytes) } },
                { "type": "text", "text": request.user_prompt },
            ]),
            false,
        ),
    };
    let mut messages = Vec::new();
    if !request.prompt.trim().is_empty() {
        messages.push(json!({ "role": "system", "content": request.prompt }));
    }
    messages.push(json!({ "role": "user", "content": content }));
    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": false,
        "max_tokens": limits.answer,
    });
    if let Some(fields) = fields {
        body["response_format"] = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "analysis",
                "schema": analysis::schema(fields),
                "strict": true,
            },
        });
    }
    if openrouter {
        body["plugins"] = json!([{ "id": "context-compression", "enabled": false }]);
    }
    Ok(Sent {
        body,
        text_chars: text.then_some(chars),
    })
}

/// The answer in `choices[0].message.content`, never `reasoning` or `reasoning_content`;
/// `text_chars` as [`body`] counted them.
pub(super) fn parse(
    response: &Value,
    name: &str,
    text_chars: Option<usize>,
) -> Result<ProcessOutput, ProviderError> {
    let usage = |key: &str| {
        response
            .pointer(&format!("/usage/{key}"))
            .and_then(Value::as_u64)
    };
    if let (Some(chars), Some(read)) = (text_chars, usage("prompt_tokens"))
        && read.saturating_mul(MAX_CHARS_PER_TOKEN) < chars as u64
    {
        return Err(ProviderError::InvalidRequest(format!(
            "{name} read only part of the text: it counted {read} tokens for {chars} characters \
             sent, fewer than one token for {MAX_CHARS_PER_TOKEN} characters. The server may cut \
             a text longer than its model's context; send shorter pieces, or raise the model's \
             context on the server."
        )));
    }
    let choice = response.pointer("/choices/0");
    let content = match choice.and_then(|c| c.pointer("/message/content")) {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(chunks)) => chunks
            .iter()
            .filter(|c| c.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|c| c.get("text").and_then(Value::as_str))
            .collect(),
        _ => String::new(),
    };
    let text = without_thinking(&content);
    let finish = choice
        .and_then(|c| c.get("finish_reason"))
        .and_then(Value::as_str);
    if text.trim().is_empty() {
        return Err(ProviderError::Other(format!(
            "{name} returned no text (finish reason: {})",
            finish.unwrap_or("unknown")
        )));
    }
    Ok(ProcessOutput {
        text: text.to_string(),
        truncated: !matches!(finish, None | Some("stop" | "tool_calls")),
        cut_reason: match finish {
            Some("content_filter") => CutReason::Filter,
            Some("length" | "model_length") | None => CutReason::Limit,
            Some(_) => CutReason::Other,
        },
        usage: Usage {
            input_tokens: usage("prompt_tokens").unwrap_or(0),
            output_tokens: usage("completion_tokens").unwrap_or(0),
            cached_input_tokens: usage("prompt_tokens_details/cached_tokens").unwrap_or(0),
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

    fn answer(content: Value, finish: &str) -> Value {
        json!({
            "model": "qwen3-vl",
            "choices": [{ "index": 0, "finish_reason": finish,
                "message": { "role": "assistant", "content": content,
                    "reasoning_content": "Thinking aloud." } }],
            "usage": { "prompt_tokens": 100, "completion_tokens": 5 },
        })
    }

    #[test]
    fn reads_the_answer_from_a_string_or_text_chunks_without_the_thinking() {
        let out = parse(&answer(json!("<think>a</think>4729"), "stop"), "Lab", None).unwrap();
        assert_eq!(out.text, "4729");
        assert!(!out.truncated);
        let chunks = json!([{ "type": "text", "text": "4729" }]);
        assert_eq!(
            parse(&answer(chunks, "stop"), "Lab", None).unwrap().text,
            "4729"
        );
        let filtered = parse(&answer(json!("Bam"), "content_filter"), "Lab", None).unwrap();
        assert!(filtered.truncated);
        assert_eq!(filtered.cut_reason, CutReason::Filter);
        let empty = parse(&answer(json!(""), "length"), "Lab", None);
        assert!(matches!(empty, Err(ProviderError::Other(m)) if m.starts_with("Lab ")));
    }

    #[test]
    fn a_reading_in_part_is_refused() {
        // 100 tokens for 800 characters is one for eight: accepted; for 801, refused.
        assert!(parse(&answer(json!("ok"), "stop"), "Lab", Some(800)).is_ok());
        let part = parse(&answer(json!("ok"), "stop"), "Lab", Some(801));
        assert!(
            matches!(&part, Err(ProviderError::InvalidRequest(m)) if m.contains("Lab read only part")),
            "{part:?}"
        );
        assert!(!part.as_ref().unwrap_err().is_retryable());
        assert!(!part.unwrap_err().is_fatal_for_job());
    }
}
