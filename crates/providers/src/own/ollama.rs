//! Ollama at its own address, `POST /api/chat` (docs.ollama.com/api/chat), never its `/v1`:
//! only here can the context be set and a cut refused. Every request carries the model's
//! context (`options.num_ctx`), the longest answer (`options.num_predict`), `truncate: false`
//! and `shift: false`, so a text longer than the context is an error 400 rather than a reading
//! in part, and `think: false`. A structured answer is asked for with `format`, the schema.

use kalamu_core::{
    AnalysisSchema, CutReason, ProcessOutput, ProcessRequest, ProviderError, RequestSetting, Usage,
};
use serde_json::{Value, json};

use super::{Input, Limits, input_kind, without_thinking};
use crate::analysis;
use crate::http::base64;

/// What every request carries beyond the prompt and the input, as provenance records it: the
/// values [`body`] writes, from the same [`Limits`], each named by its path in the body.
pub(super) fn settings(limits: &Limits) -> Vec<RequestSetting> {
    vec![
        RequestSetting::new("options.num_ctx", limits.context),
        RequestSetting::new("options.num_predict", limits.answer),
        RequestSetting::new("truncate", false),
        RequestSetting::new("shift", false),
        RequestSetting::new("think", false),
    ]
}

pub(super) fn body(
    request: &ProcessRequest<'_>,
    fields: Option<&AnalysisSchema>,
    limits: &Limits,
    name: &str,
) -> Result<Value, ProviderError> {
    let input = request.input;
    let user = match input_kind(request, name)? {
        Input::Text => json!({
            "role": "user",
            "content": format!("{}\n\n{}", analysis::tagged_document(input), request.user_prompt),
        }),
        Input::Image => json!({
            "role": "user",
            "content": request.user_prompt,
            "images": [base64(&input.bytes)],
        }),
    };
    let mut messages = Vec::new();
    if !request.prompt.trim().is_empty() {
        messages.push(json!({ "role": "system", "content": request.prompt }));
    }
    messages.push(user);
    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": false,
        "think": false,
        "truncate": false,
        "shift": false,
        "options": {
            "num_ctx": limits.context,
            "num_predict": limits.answer,
        },
    });
    if let Some(fields) = fields {
        body["format"] = analysis::schema(fields);
    }
    Ok(body)
}

/// The answer in `message.content`, never `message.thinking`.
pub(super) fn parse(response: &Value, name: &str) -> Result<ProcessOutput, ProviderError> {
    let content = response
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let text = without_thinking(content);
    let done = response.get("done_reason").and_then(Value::as_str);
    if text.trim().is_empty() {
        return Err(ProviderError::Other(format!(
            "{name} returned no text (done reason: {})",
            done.unwrap_or("unknown")
        )));
    }
    let count = |key: &str| response.get(key).and_then(Value::as_u64).unwrap_or(0);
    Ok(ProcessOutput {
        text: text.to_string(),
        truncated: done == Some("length"),
        cut_reason: CutReason::Limit,
        usage: Usage {
            input_tokens: count("prompt_eval_count"),
            output_tokens: count("eval_count"),
            ..Usage::default()
        },
        ..ProcessOutput::default()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_answer_is_the_content_and_never_the_thinking() {
        let out = parse(
            &json!({
                "model": "qwen3:8b",
                "message": { "role": "assistant", "content": "<think>x</think>Bamako",
                    "thinking": "Let me think." },
                "done": true, "done_reason": "stop",
                "prompt_eval_count": 26, "eval_count": 3,
            }),
            "Office Mac",
        )
        .unwrap();
        assert_eq!(out.text, "Bamako");
        assert!(!out.truncated);
        assert_eq!((out.usage.input_tokens, out.usage.output_tokens), (26, 3));
        assert_eq!(out.model_version, None);
        let empty = parse(
            &json!({ "message": { "role": "assistant", "content": "", "thinking": "Long." },
                "done": true, "done_reason": "length" }),
            "Office Mac",
        );
        assert!(
            matches!(empty, Err(ProviderError::Other(m)) if m.contains("Office Mac") && m.contains("length"))
        );
    }
}
