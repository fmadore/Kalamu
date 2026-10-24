//! What every provider shares for summaries: the JSON shape asked for, the document as the
//! model reads it, and reading the answer back.
//!
//! The document goes first and the request last, as both Google's and Anthropic's long-context
//! guidance advise, and the answer is constrained to a schema with each provider's structured
//! output rather than asked for in prose.

use scriptorium_core::{PreparedInput, ProcessOutput, ProviderError};
use serde::Deserialize;
use serde_json::{Value, json};

/// `{ "summary": "...", "keywords": ["...", ...] }`. Every field required and no others, which
/// OpenAI's strict mode demands; no length limits, which Claude's structured output refuses.
pub(crate) fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "summary": {
                "type": "string",
                "description": "The summary, as plain prose without markdown."
            },
            "keywords": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Keywords and key phrases, most important first."
            }
        },
        "required": ["summary", "keywords"],
        "additionalProperties": false
    })
}

/// The text of a text input, decoded leniently: a stray byte must not lose the document.
pub(crate) fn document_text(input: &PreparedInput) -> String {
    String::from_utf8_lossy(&input.bytes).into_owned()
}

/// A plain-text document wrapped with its name, for providers that take text parts.
pub(crate) fn tagged_document(input: &PreparedInput) -> String {
    format!(
        "<document name=\"{}\">\n{}\n</document>",
        input.file_name.replace('"', "'"),
        document_text(input)
    )
}

#[derive(Deserialize)]
struct Answer {
    summary: String,
    #[serde(default)]
    keywords: Vec<String>,
}

/// Turns the model's JSON answer into the summary text and keywords. Tolerates a code fence
/// around the JSON, which some models add even under a schema.
pub(crate) fn finish(mut output: ProcessOutput) -> Result<ProcessOutput, ProviderError> {
    let raw = output.text.trim();
    let raw = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .map_or(raw, |r| r.trim_end_matches("```"))
        .trim();
    let answer: Answer = serde_json::from_str(raw)
        .map_err(|e| ProviderError::Other(format!("the summary was not valid JSON: {e}")))?;
    output.text = answer.summary.trim().to_string();
    output.keywords = answer
        .keywords
        .into_iter()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();
    Ok(output)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_answer_with_or_without_a_code_fence() {
        for text in [
            r#"{"summary":" Une réunion. ","keywords":["imam"," Bamako ",""]}"#,
            "```json\n{\"summary\":\"Une réunion.\",\"keywords\":[\"imam\",\"Bamako\"]}\n```",
        ] {
            let out = finish(ProcessOutput {
                text: text.into(),
                ..ProcessOutput::default()
            })
            .unwrap();
            assert_eq!(out.text, "Une réunion.");
            assert_eq!(out.keywords, ["imam", "Bamako"]);
        }
        assert!(
            finish(ProcessOutput {
                text: "Not JSON".into(),
                ..ProcessOutput::default()
            })
            .is_err()
        );
    }
}
