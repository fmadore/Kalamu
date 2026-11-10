//! Analyses as structured output: the JSON schema every provider constrains an analysis's
//! answer to, built from the researcher's fields, and the reading of the answer back into
//! checked fields. Shared by the cloud providers and the offline language models, so a field
//! asks the same of every model and every answer is checked the same way.
//!
//! Structured output fixes the shape, not every rule: the answer is checked against the fields
//! again (`AnalysisSchema::read_answer`), and one that breaks them is an error the engine
//! retries, never an empty result.

use serde_json::{Map, Value, json};

use crate::analysis::reason_key;
use crate::{AnalysisSchema, FieldKind, PreparedInput, ProcessOutput, ProviderError};

/// One property per field, plus `<key>_reason` for fields that ask for a reason. Every property
/// required and no others, which OpenAI's strict mode demands; no length limits, which Claude's
/// structured output refuses (a list's maximum is in the prompt and applied on reading). An
/// entries field is an array of objects built the same way.
pub fn schema(fields: &AnalysisSchema) -> Value {
    object(&fields.fields)
}

fn object(fields: &[crate::Field]) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for field in fields {
        let description = if field.instructions.trim().is_empty() {
            field.label.trim().to_string()
        } else {
            format!("{}: {}", field.label.trim(), field.instructions.trim())
        };
        let property = match &field.kind {
            FieldKind::Text => json!({ "type": "string", "description": description }),
            FieldKind::Choice { values } => {
                json!({ "type": "string", "enum": values, "description": description })
            }
            FieldKind::List { .. } => json!({
                "type": "array",
                "items": { "type": "string" },
                "description": description,
            }),
            FieldKind::Choices { values } => json!({
                "type": "array",
                "items": { "type": "string", "enum": values },
                "description": description,
            }),
            FieldKind::Entries { fields } => json!({
                "type": "array",
                "items": object(fields),
                "description": description,
            }),
        };
        properties.insert(field.key.clone(), property);
        required.push(field.key.clone());
        if field.reason {
            let key = reason_key(&field.key);
            properties.insert(
                key.clone(),
                json!({
                    "type": "string",
                    "description": format!("Why, in one sentence: {}", field.label.trim()),
                }),
            );
            required.push(key);
        }
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
    })
}

/// The text of a text input, decoded leniently: a stray byte must not lose the document.
pub fn document_text(input: &PreparedInput) -> String {
    String::from_utf8_lossy(&input.bytes).into_owned()
}

/// A plain-text document wrapped with its name, for providers that take text parts.
pub fn tagged_document(input: &PreparedInput) -> String {
    format!(
        "<document name=\"{}\">\n{}\n</document>",
        input.file_name.replace('"', "'"),
        document_text(input)
    )
}

/// Turns the model's JSON answer into checked answers, with the plain-text rendering as the
/// output's text. Tolerates a code fence around the JSON, which some models add even under a
/// schema. An answer that breaks the fields is retried by the engine, never stored empty; one
/// the model stopped writing at its output limit fails once, since asking again would stop at
/// the same place and be billed again.
pub fn finish(
    mut output: ProcessOutput,
    fields: &AnalysisSchema,
) -> Result<ProcessOutput, ProviderError> {
    let raw = output.text.trim();
    let raw = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .map_or(raw, |r| r.trim_end_matches("```"))
        .trim();
    // Refused or not, the answer was written and billed: its usage goes with the error.
    let truncated = output.truncated;
    let rejected = |reason: String| {
        if truncated {
            ProviderError::CutOff {
                usage: output.usage.clone(),
            }
        } else {
            ProviderError::InvalidAnswer {
                reason,
                usage: output.usage.clone(),
            }
        }
    };
    let answer: Value = serde_json::from_str(raw)
        .map_err(|e| rejected(format!("the answer was not valid JSON: {e}")))?;
    output.fields = fields.read_answer(&answer).map_err(rejected)?;
    output.text = fields.render(&output.fields);
    Ok(output)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{Field, FieldValue};

    use super::*;

    fn fields() -> AnalysisSchema {
        let mut schema = AnalysisSchema {
            fields: vec![
                Field {
                    key: String::new(),
                    label: "Summary".into(),
                    instructions: String::new(),
                    kind: FieldKind::Text,
                    reason: false,
                },
                Field {
                    key: String::new(),
                    label: "Tone".into(),
                    instructions: "Towards the subject.".into(),
                    kind: FieldKind::Choice {
                        values: vec!["Positive".into(), "Negative".into()],
                    },
                    reason: true,
                },
            ],
        };
        schema.assign_keys();
        schema
    }

    #[test]
    fn the_schema_has_every_field_and_reason_required() {
        let schema = schema(&fields());
        assert_eq!(
            schema["required"],
            json!(["summary", "tone", "tone_reason"])
        );
        assert_eq!(
            schema["properties"]["tone"]["enum"],
            json!(["Positive", "Negative"])
        );
        assert_eq!(
            schema["properties"]["tone"]["description"],
            "Tone: Towards the subject."
        );
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn entries_are_arrays_of_strict_objects() {
        let schema = schema(&crate::contents::merge_schema(&fields()));
        let entries = &schema["properties"]["entries"];
        assert_eq!(entries["type"], "array");
        assert_eq!(
            entries["items"]["required"],
            json!(["title", "pages", "summary", "tone", "tone_reason"])
        );
        assert_eq!(entries["items"]["additionalProperties"], false);
    }

    #[test]
    fn reads_the_answer_with_or_without_a_code_fence() {
        for text in [
            r#"{"summary":" Une réunion. ","tone":"negative","tone_reason":"Critique."}"#,
            "```json\n{\"summary\":\"Une réunion.\",\"tone\":\"Negative\",\"tone_reason\":\"Critique.\"}\n```",
        ] {
            let out = finish(
                ProcessOutput {
                    text: text.into(),
                    ..ProcessOutput::default()
                },
                &fields(),
            )
            .unwrap();
            assert_eq!(out.fields[0].value, FieldValue::Text("Une réunion.".into()));
            assert_eq!(out.fields[1].value, FieldValue::Text("Negative".into()));
            assert_eq!(
                out.text,
                "Summary\nUne réunion.\n\nTone: Negative\nReason: Critique."
            );
        }
        for broken in [
            "Not JSON",
            r#"{"summary":"x","tone":"Mixed","tone_reason":""}"#,
        ] {
            let usage = crate::Usage {
                input_tokens: 1_200,
                output_tokens: 80,
                ..crate::Usage::default()
            };
            let err = finish(
                ProcessOutput {
                    text: broken.into(),
                    usage: usage.clone(),
                    ..ProcessOutput::default()
                },
                &fields(),
            )
            .unwrap_err();
            assert!(matches!(err, ProviderError::InvalidAnswer { .. }), "{err}");
            assert!(err.is_retryable());
            // The answer was billed though refused: its usage goes with the error.
            assert_eq!(err.billed(), Some(&usage));
        }
    }

    #[test]
    fn an_answer_cut_off_at_the_output_limit_fails_once_and_is_billed() {
        let usage = crate::Usage {
            input_tokens: 9_000,
            output_tokens: 32_000,
            ..crate::Usage::default()
        };
        let err = finish(
            ProcessOutput {
                text: r#"{"summary":"Une réunion du conseil où"#.into(),
                usage: usage.clone(),
                truncated: true,
                ..ProcessOutput::default()
            },
            &fields(),
        )
        .unwrap_err();
        assert!(matches!(err, ProviderError::CutOff { .. }), "{err}");
        assert!(!err.is_retryable());
        assert!(!err.is_fatal_for_job());
        assert_eq!(err.billed(), Some(&usage));
        assert!(err.to_string().contains("output limit"), "{err}");

        // A whole answer that happened to reach the limit is read as any other.
        let out = finish(
            ProcessOutput {
                text: r#"{"summary":"x","tone":"Positive","tone_reason":"y"}"#.into(),
                truncated: true,
                ..ProcessOutput::default()
            },
            &fields(),
        )
        .unwrap();
        assert_eq!(out.fields[0].value, FieldValue::Text("x".into()));
    }
}
