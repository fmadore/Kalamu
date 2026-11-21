//! Analyses as structured output: the JSON schema every provider constrains an analysis's
//! answer to, built from the researcher's fields, and the reading of the answer back into
//! checked fields. Shared by the cloud providers and the offline language models, so a field
//! asks the same of every model and every answer is checked the same way.
//!
//! Structured output fixes the shape, not every rule: the answer is checked against the fields
//! again (`AnalysisSchema::read_answer`), and one that breaks them is an error the engine
//! retries, never an empty result.

use serde_json::{Map, Value, json};

use crate::analysis::cited::{QUOTES, TURN, WORDS};
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
            FieldKind::Cited { fields, .. } => json!({
                "type": "array",
                "items": cited_entry(fields),
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

/// An entry that cites the document: its own fields, built as any fields are, and its
/// quotations, each the number of a turn and the words said there.
fn cited_entry(fields: &[crate::Field]) -> Value {
    let mut entry = object(fields);
    let quotes = json!({
        "type": "array",
        "description": "One or more quotations of the text, each copied word for word from \
                        one numbered turn.",
        "items": {
            "type": "object",
            "properties": {
                TURN: {
                    "type": "integer",
                    "description": "The number of the turn the words come from, as given in \
                                    brackets.",
                },
                WORDS: {
                    "type": "string",
                    "description": "The words exactly as they stand in that turn.",
                },
            },
            "required": [TURN, WORDS],
            "additionalProperties": false,
        },
    });
    if let Some(properties) = entry["properties"].as_object_mut() {
        properties.insert(QUOTES.to_string(), quotes);
    }
    if let Some(required) = entry["required"].as_array_mut() {
        required.push(json!(QUOTES));
    }
    entry
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
    let (truncated, cut_reason) = (output.truncated, output.cut_reason);
    let rejected = |reason: String| {
        if truncated {
            ProviderError::CutOff {
                usage: output.usage.clone(),
                reason: cut_reason,
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

/// Words past which a text cannot plausibly give nothing for any field of an analysis that asks
/// for several: a page of prose names someone, somewhere or something, or says what it is about.
pub const LONG_TEXT_WORDS: u64 = 500;

/// Refuses, as a broken answer, an analysis's answer that left every one of its fields empty
/// for a text of `words` words (`None` when what was read is not a text: a scan, a picture),
/// so that the engine asks again and the piece fails with the reason if no answer passes,
/// instead of an empty answer kept as a success. GPT-6.2 Sol answered every list empty for a
/// newspaper issue of 4,800 words in the 1.10.0 test pass, where two other models found dozens
/// of names.
///
/// Kept as it is: an answer to a text of [`LONG_TEXT_WORDS`] or fewer, and an analysis of one
/// field (a dictionary's merges, a register's rows), for which nothing is a real answer.
pub fn refuse_empty(
    output: ProcessOutput,
    fields: &AnalysisSchema,
    words: Option<u64>,
) -> Result<ProcessOutput, ProviderError> {
    let Some(words) = words.filter(|w| *w > LONG_TEXT_WORDS) else {
        return Ok(output);
    };
    if fields.fields.len() < 2 || !output.fields.iter().all(|a| a.value.is_empty()) {
        return Ok(output);
    }
    Err(ProviderError::InvalidAnswer {
        reason: format!(
            "the model left every field empty, though the text runs to about {words} words: it \
             may not have read it. Try again, or choose another model"
        ),
        usage: output.usage,
    })
}

/// How many words a text holds, as [`refuse_empty`] counts them.
pub fn word_count(text: &str) -> u64 {
    text.split_whitespace().count() as u64
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

    #[test]
    fn an_answer_stopped_by_a_filter_says_so_and_not_the_limit() {
        let cut = |cut_reason| {
            finish(
                ProcessOutput {
                    text: r#"{"summary":"Une réunion"#.into(),
                    truncated: true,
                    cut_reason,
                    ..ProcessOutput::default()
                },
                &fields(),
            )
            .unwrap_err()
        };
        let filter = cut(crate::CutReason::Filter);
        assert!(
            matches!(
                filter,
                ProviderError::CutOff {
                    reason: crate::CutReason::Filter,
                    ..
                }
            ),
            "{filter}"
        );
        assert_eq!(filter.code(), "provider.cut_off_filter");
        assert!(filter.to_string().contains("filter"), "{filter}");
        assert!(!filter.to_string().contains("output limit"), "{filter}");
        assert!(!filter.is_retryable());

        let other = cut(crate::CutReason::Other);
        assert_eq!(other.code(), "provider.cut_off_other");
        assert_eq!(cut(crate::CutReason::Limit).code(), "provider.cut_off");
    }

    /// People and places, as the entities analysis asks for them.
    fn lists(labels: &[&str]) -> AnalysisSchema {
        let mut schema = AnalysisSchema {
            fields: labels
                .iter()
                .map(|label| Field {
                    key: String::new(),
                    label: (*label).into(),
                    instructions: String::new(),
                    kind: FieldKind::List { max: Some(30) },
                    reason: false,
                })
                .collect(),
        };
        schema.assign_keys();
        schema
    }

    fn answered(text: &str, schema: &AnalysisSchema) -> ProcessOutput {
        finish(
            ProcessOutput {
                text: text.into(),
                usage: crate::Usage {
                    input_tokens: 7_300,
                    output_tokens: 40,
                    ..crate::Usage::default()
                },
                ..ProcessOutput::default()
            },
            schema,
        )
        .unwrap()
    }

    #[test]
    fn every_field_left_empty_for_a_long_text_is_refused_and_billed() {
        let schema = lists(&["People", "Places"]);
        let empty = answered(r#"{"people":[],"places":[]}"#, &schema);
        let err = refuse_empty(empty.clone(), &schema, Some(4_800)).unwrap_err();
        assert!(matches!(err, ProviderError::InvalidAnswer { .. }), "{err}");
        // Asked again, as any broken answer is, and billed all the same.
        assert!(err.is_retryable());
        assert_eq!(err.billed().map(|u| u.input_tokens), Some(7_300));
        assert!(err.to_string().contains("4800 words"), "{err}");
        // Blank items are no answer either.
        let blank = answered(r#"{"people":[" "],"places":[]}"#, &schema);
        assert!(refuse_empty(blank, &schema, Some(4_800)).is_err());

        // A short text may name nobody and nowhere; a scan's length is not known.
        assert!(refuse_empty(empty.clone(), &schema, Some(LONG_TEXT_WORDS)).is_ok());
        assert!(refuse_empty(empty, &schema, None).is_ok());
        // One field answered is an answer.
        let one = answered(r#"{"people":[],"places":["Lincoln"]}"#, &schema);
        assert!(refuse_empty(one, &schema, Some(4_800)).is_ok());
        // An analysis of one field (a dictionary's merges) may rightly find nothing.
        let single = lists(&["Groups"]);
        let none = answered(r#"{"groups":[]}"#, &single);
        assert!(refuse_empty(none, &single, Some(40_000)).is_ok());
    }

    #[test]
    fn words_are_counted_between_spaces() {
        assert_eq!(word_count("  The Daily\nNebraskan,\t1911 "), 4);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn an_answer_stored_before_the_reason_was_kept_reads_as_the_limit() {
        let old: ProcessOutput = serde_json::from_str(
            r#"{"text":"Bamako","detected_language":null,"model_version":null,"truncated":true}"#,
        )
        .unwrap();
        assert_eq!(old.cut_reason, crate::CutReason::Limit);
        // And one stopped at the limit is written as before, without the reason.
        let written = serde_json::to_value(&old).unwrap();
        assert!(written.get("cut_reason").is_none(), "{written}");
        let filtered = ProcessOutput {
            cut_reason: crate::CutReason::Filter,
            ..old
        };
        assert_eq!(
            serde_json::to_value(&filtered).unwrap()["cut_reason"],
            "filter"
        );
    }
}
