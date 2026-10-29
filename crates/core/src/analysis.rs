//! Analysis: what a researcher asks of each document, as named fields, and the answers.
//!
//! A schema belongs to an analysis preset and is frozen into the job with its prompt. Each
//! provider turns it into its own structured-output format, so the model can only answer with
//! these fields; the answer is checked again here before it is stored, and an answer that breaks
//! the schema is an error, never an empty result.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use ts_rs::TS;

/// The fields an analysis fills in for each document, in the order they are shown.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnalysisSchema {
    pub fields: Vec<Field>,
}

/// One thing to find in a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Field {
    /// The field's name in answers and exported data (`places`). Given once, from the label,
    /// and kept when the label changes so earlier exports stay comparable.
    #[serde(default)]
    pub key: String,
    /// What the researcher calls it: `Places`.
    pub label: String,
    /// What belongs in it, in the researcher's words; passed to the model.
    #[serde(default)]
    pub instructions: String,
    pub kind: FieldKind,
    /// Ask for a one-sentence reason with the answer.
    #[serde(default)]
    pub reason: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum FieldKind {
    /// Free prose: a summary, the main subject, a date.
    Text,
    /// Names or terms found in the document, spelled as there: people, places, keywords.
    List {
        /// At most this many, the most important first.
        #[serde(default)]
        max: Option<u32>,
    },
    /// Exactly one value from a fixed set: tone, relevance.
    Choice { values: Vec<String> },
    /// Any of a fixed set: themes from the researcher's own list.
    Choices { values: Vec<String> },
    /// A list of entries, each with its own fields: the articles of a table of contents, with
    /// a title, pages and a summary each. Built by Kalamu for a table of contents; the field
    /// editor does not offer it.
    Entries { fields: Vec<Field> },
}

/// A field's answer: prose or one choice as text, names or choices as a list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum FieldValue {
    Text(String),
    List(Vec<String>),
    /// Entries, each its fields' answers. An empty list reads back as `List`, which
    /// [`FieldValue::entries`] takes as no entries.
    Entries(Vec<Vec<FieldAnswer>>),
}

impl FieldValue {
    /// The value on one line: list items joined with semicolons, as spreadsheets expect;
    /// entries by their first field (a title).
    pub fn joined(&self) -> String {
        match self {
            Self::Text(t) => t.clone(),
            Self::List(items) => items.join("; "),
            Self::Entries(entries) => entries
                .iter()
                .filter_map(|e| e.first().map(|a| a.value.joined()))
                .collect::<Vec<_>>()
                .join("; "),
        }
    }

    /// The entries of an entries field; none for any other value.
    pub fn entries(&self) -> &[Vec<FieldAnswer>] {
        match self {
            Self::Entries(entries) => entries,
            _ => &[],
        }
    }
}

/// The text of one answer in a set of answers, by key; empty when it is missing.
pub fn answer_text(answers: &[FieldAnswer], key: &str) -> String {
    answers
        .iter()
        .find(|a| a.key == key)
        .map(|a| a.value.joined())
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldAnswer {
    pub key: String,
    pub value: FieldValue,
    /// The model's reason, when the field asks for one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SchemaError {
    #[error("An analysis needs at least one field")]
    Empty,
    #[error("Every field needs a name")]
    Unnamed,
    #[error("Two fields have the same name: {0}")]
    Duplicate(String),
    #[error("\"{0}\" needs at least two values to choose from")]
    TooFewValues(String),
}

/// The property that holds a field's reason in the model's answer.
pub fn reason_key(key: &str) -> String {
    format!("{key}_reason")
}

impl AnalysisSchema {
    /// Gives new fields a key from their label, unique within the schema; existing keys stay.
    pub fn assign_keys(&mut self) {
        let mut taken: Vec<String> = self
            .fields
            .iter()
            .filter(|f| !f.key.is_empty())
            .map(|f| f.key.clone())
            .collect();
        for (i, field) in self.fields.iter_mut().enumerate() {
            if !field.key.is_empty() {
                continue;
            }
            let base = key_for(&field.label).unwrap_or_else(|| format!("field_{}", i + 1));
            let mut key = base.clone();
            let mut n = 2;
            while taken.contains(&key) {
                key = format!("{base}_{n}");
                n += 1;
            }
            taken.push(key.clone());
            field.key = key;
        }
    }

    /// Checks a schema before it is saved or run.
    pub fn validate(&self) -> Result<(), SchemaError> {
        validate_fields(&self.fields)
    }

    /// Reads the model's JSON answer into one answer per field, tidied: text trimmed, list items
    /// trimmed and without repeats (ignoring case), lists cut to their maximum, choices spelled
    /// as in the schema. A missing field or a value outside its choices is an error.
    pub fn read_answer(&self, answer: &Value) -> Result<Vec<FieldAnswer>, String> {
        read_fields(&self.fields, answer)
    }

    /// The answers as plain text, for text files and the review: prose fields under their
    /// label, the others on one line each, reasons below them, entries one block each.
    pub fn render(&self, answers: &[FieldAnswer]) -> String {
        render_fields(&self.fields, answers)
    }
}

fn validate_fields(fields: &[Field]) -> Result<(), SchemaError> {
    if fields.is_empty() {
        return Err(SchemaError::Empty);
    }
    let mut seen: Vec<String> = Vec::new();
    for field in fields {
        let label = field.label.trim();
        if label.is_empty() || field.key.is_empty() {
            return Err(SchemaError::Unnamed);
        }
        let folded = label.to_lowercase();
        if seen.contains(&folded) || seen.contains(&field.key) {
            return Err(SchemaError::Duplicate(label.to_string()));
        }
        seen.push(folded);
        seen.push(field.key.clone());
        if let FieldKind::Choice { values } | FieldKind::Choices { values } = &field.kind {
            let distinct = values
                .iter()
                .map(|v| v.trim().to_lowercase())
                .filter(|v| !v.is_empty())
                .collect::<std::collections::BTreeSet<_>>();
            if distinct.len() < 2 {
                return Err(SchemaError::TooFewValues(label.to_string()));
            }
        }
        if let FieldKind::Entries { fields } = &field.kind {
            validate_fields(fields)?;
        }
    }
    Ok(())
}

fn read_fields(fields: &[Field], answer: &Value) -> Result<Vec<FieldAnswer>, String> {
    let object = answer
        .as_object()
        .ok_or_else(|| "the answer is not a JSON object".to_string())?;
    let mut out = Vec::new();
    for field in fields {
        let raw = object
            .get(&field.key)
            .ok_or_else(|| format!("the answer has no \"{}\"", field.label))?;
        let value = match &field.kind {
            FieldKind::Text => FieldValue::Text(text(raw, field)?.trim().to_string()),
            FieldKind::Choice { values } => {
                FieldValue::Text(choose(text(raw, field)?, values, field)?)
            }
            FieldKind::List { max } => {
                let mut items = distinct(strings(raw, field)?);
                if let Some(max) = max {
                    items.truncate(*max as usize);
                }
                FieldValue::List(items)
            }
            FieldKind::Choices { values } => FieldValue::List(distinct(
                strings(raw, field)?
                    .into_iter()
                    .map(|item| choose(&item, values, field))
                    .collect::<Result<_, _>>()?,
            )),
            FieldKind::Entries { fields } => FieldValue::Entries(
                raw.as_array()
                    .ok_or_else(|| format!("\"{}\" is not a list", field.label))?
                    .iter()
                    .map(|entry| read_fields(fields, entry))
                    .collect::<Result<_, _>>()?,
            ),
        };
        let reason = field
            .reason
            .then(|| object.get(&reason_key(&field.key)))
            .flatten()
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(str::to_string);
        out.push(FieldAnswer {
            key: field.key.clone(),
            value,
            reason,
        });
    }
    Ok(out)
}

fn render_fields(fields: &[Field], answers: &[FieldAnswer]) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for field in fields {
        let Some(answer) = answers.iter().find(|a| a.key == field.key) else {
            continue;
        };
        let mut block = match (&field.kind, &answer.value) {
            (FieldKind::Entries { fields }, value) => value
                .entries()
                .iter()
                .map(|entry| render_fields(fields, entry))
                .collect::<Vec<_>>()
                .join("\n\n"),
            (FieldKind::Text, value) => format!("{}\n{}", field.label, value.joined()),
            (_, value) => format!("{}: {}", field.label, value.joined()),
        };
        if let Some(reason) = &answer.reason {
            block.push_str(&format!("\nReason: {reason}"));
        }
        if !block.is_empty() {
            blocks.push(block);
        }
    }
    blocks.join("\n\n")
}

fn text<'a>(raw: &'a Value, field: &Field) -> Result<&'a str, String> {
    raw.as_str()
        .ok_or_else(|| format!("\"{}\" is not text", field.label))
}

fn strings(raw: &Value, field: &Field) -> Result<Vec<String>, String> {
    raw.as_array()
        .ok_or_else(|| format!("\"{}\" is not a list", field.label))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.trim().to_string())
                .ok_or_else(|| format!("\"{}\" holds something other than text", field.label))
        })
        .collect()
}

/// The schema's spelling of a value, matched ignoring case and surrounding space.
fn choose(value: &str, values: &[String], field: &Field) -> Result<String, String> {
    let wanted = value.trim().to_lowercase();
    values
        .iter()
        .find(|v| v.trim().to_lowercase() == wanted)
        .map(|v| v.trim().to_string())
        .ok_or_else(|| {
            format!(
                "\"{}\" is not one of the choices for \"{}\"",
                value.trim(),
                field.label
            )
        })
}

fn distinct(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        if !item.is_empty() && !out.iter().any(|o| o.to_lowercase() == item.to_lowercase()) {
            out.push(item);
        }
    }
    out
}

/// A key from a label: lower case ASCII letters, digits and underscores (`Lieux cités` →
/// `lieux_cites`), which every provider accepts as a property name.
fn key_for(label: &str) -> Option<String> {
    let mut key = String::new();
    for c in label.trim().to_lowercase().chars() {
        let folded = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
            'ç' => "c",
            'è' | 'é' | 'ê' | 'ë' => "e",
            'ì' | 'í' | 'î' | 'ï' => "i",
            'ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => "o",
            'ù' | 'ú' | 'û' | 'ü' => "u",
            'ý' | 'ÿ' => "y",
            'œ' => "oe",
            'æ' => "ae",
            'ß' => "ss",
            c if c.is_ascii_alphanumeric() => {
                key.push(c);
                continue;
            }
            _ => "_",
        };
        key.push_str(folded);
    }
    let key = key
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    match key.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => Some(key),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;

    fn field(label: &str, kind: FieldKind, reason: bool) -> Field {
        Field {
            key: String::new(),
            label: label.into(),
            instructions: String::new(),
            kind,
            reason,
        }
    }

    fn schema() -> AnalysisSchema {
        let mut schema = AnalysisSchema {
            fields: vec![
                field("Résumé", FieldKind::Text, false),
                field("Lieux cités", FieldKind::List { max: Some(2) }, false),
                field(
                    "Tone",
                    FieldKind::Choice {
                        values: vec!["Positive".into(), "Negative".into()],
                    },
                    true,
                ),
                field(
                    "Themes",
                    FieldKind::Choices {
                        values: vec!["Education".into(), "Pilgrimage".into()],
                    },
                    false,
                ),
            ],
        };
        schema.assign_keys();
        schema
    }

    #[test]
    fn keys_come_from_labels_and_stay_unique() {
        let keys: Vec<_> = schema().fields.into_iter().map(|f| f.key).collect();
        assert_eq!(keys, ["resume", "lieux_cites", "tone", "themes"]);
        let mut twice = AnalysisSchema {
            fields: vec![
                field("Places", FieldKind::Text, false),
                field("places!", FieldKind::Text, false),
                field("2024", FieldKind::Text, false),
            ],
        };
        twice.assign_keys();
        let keys: Vec<_> = twice.fields.iter().map(|f| f.key.as_str()).collect();
        assert_eq!(keys, ["places", "places_2", "field_3"]);
        assert_eq!(twice.validate(), Ok(()));
        twice.fields[1].label = "places".into();
        assert_eq!(
            twice.validate(),
            Err(SchemaError::Duplicate("places".into()))
        );
    }

    #[test]
    fn choices_need_two_values() {
        let mut one = AnalysisSchema {
            fields: vec![field(
                "Tone",
                FieldKind::Choice {
                    values: vec!["Positive".into(), " positive ".into()],
                },
                false,
            )],
        };
        one.assign_keys();
        assert_eq!(
            one.validate(),
            Err(SchemaError::TooFewValues("Tone".into()))
        );
        assert_eq!(schema().validate(), Ok(()));
        assert_eq!(
            AnalysisSchema::default().validate(),
            Err(SchemaError::Empty)
        );
    }

    #[test]
    fn answers_are_tidied_and_checked() {
        let schema = schema();
        let answers = schema
            .read_answer(&json!({
                "resume": "  Une réunion à Bamako. ",
                "lieux_cites": ["Bamako", "bamako", " Dakar ", "Ségou"],
                "tone": "negative",
                "tone_reason": "Critical of the decision.",
                "themes": ["pilgrimage", "Education", "Pilgrimage"]
            }))
            .unwrap();
        assert_eq!(
            answers[0].value,
            FieldValue::Text("Une réunion à Bamako.".into())
        );
        assert_eq!(
            answers[1].value,
            FieldValue::List(vec!["Bamako".into(), "Dakar".into()])
        );
        assert_eq!(answers[2].value, FieldValue::Text("Negative".into()));
        assert_eq!(
            answers[2].reason.as_deref(),
            Some("Critical of the decision.")
        );
        assert_eq!(
            answers[3].value,
            FieldValue::List(vec!["Pilgrimage".into(), "Education".into()])
        );
        assert_eq!(
            schema.render(&answers),
            "Résumé\nUne réunion à Bamako.\n\nLieux cités: Bamako; Dakar\n\nTone: Negative\n\
             Reason: Critical of the decision.\n\nThemes: Pilgrimage; Education"
        );
    }

    #[test]
    fn broken_answers_are_errors_not_empty_results() {
        let schema = schema();
        let base = json!({ "resume": "x", "lieux_cites": [], "tone": "Positive", "themes": [] });
        assert!(schema.read_answer(&base).is_ok());
        let mut missing = base.clone();
        missing.as_object_mut().unwrap().remove("lieux_cites");
        assert!(
            schema
                .read_answer(&missing)
                .unwrap_err()
                .contains("Lieux cités")
        );
        let mut outside = base.clone();
        outside["tone"] = json!("Mixed");
        assert!(schema.read_answer(&outside).unwrap_err().contains("Mixed"));
        assert!(schema.read_answer(&json!(["x"])).is_err());
    }
}
