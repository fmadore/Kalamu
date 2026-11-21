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

pub mod cited;
pub use cited::{CitedEntry, CitedValue, Claim, Evidence};

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
    /// a title, pages and a summary each, or the rows of a register page, its columns the
    /// entries' fields. The field editor offers it as a repeating group ("one of these per
    /// entry"), an analysis's only field ([`crate::records`]).
    Entries { fields: Vec<Field> },
    /// Entries that cite the document ([`cited`]): each with its own small fields (a theme's
    /// name and description, a quotation's reason) and one or more quotations of the text,
    /// found in it word for word before they are kept. Unlike a repeating group it sits beside
    /// other fields and changes nothing in how the source is read.
    Cited {
        fields: Vec<Field>,
        /// At most this many entries, the most important first.
        #[serde(default)]
        max: Option<u32>,
    },
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
    /// Entries that cite the document, each with its quotations found in the text, and how
    /// many of the model's quotations were left out ([`CitedValue`]).
    Cited(CitedValue),
}

impl FieldValue {
    /// The value on one line: list items joined with semicolons, as spreadsheets expect;
    /// entries by their first field (a title), and entries that are quotations alone (a
    /// document's passages) by their quotations, each between quotation marks.
    pub fn joined(&self) -> String {
        match self {
            Self::Text(t) => t.clone(),
            Self::List(items) => items.join("; "),
            Self::Entries(entries) => entries
                .iter()
                .filter_map(|e| e.first().map(|a| a.value.joined()))
                .collect::<Vec<_>>()
                .join("; "),
            Self::Cited(cited) => cited
                .entries
                .iter()
                .filter_map(cited::CitedEntry::name)
                .collect::<Vec<_>>()
                .join("; "),
        }
    }

    /// The entries of an entries field; none for any other value.
    /// Nothing answered: a blank text, or no item, entry or quotation.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(t) => t.trim().is_empty(),
            Self::List(items) => items.iter().all(|i| i.trim().is_empty()),
            Self::Entries(entries) => entries.is_empty(),
            Self::Cited(cited) => cited.entries.is_empty(),
        }
    }

    pub fn entries(&self) -> &[Vec<FieldAnswer>] {
        match self {
            Self::Entries(entries) => entries,
            _ => &[],
        }
    }

    /// The entries of a field that cites the document; none for any other value.
    pub fn cited(&self) -> Option<&CitedValue> {
        match self {
            Self::Cited(cited) => Some(cited),
            _ => None,
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
    /// A repeating group with no column.
    #[error("\"{0}\" needs at least one column")]
    EmptyGroup(String),
    /// A repeating group among another's columns.
    #[error("\"{0}\" cannot hold another repeating group")]
    NestedGroup(String),
    #[error("An analysis holds one repeating group at most")]
    TwoGroups,
    /// A repeating group beside other fields: an analysis is either answered once per document
    /// or row by row, page by page.
    #[error("A repeating group is the analysis's only field: put \"{0}\" in it as a column")]
    GroupNotAlone(String),
    /// A field of entries that cite the document named as their quotations are.
    #[error("\"{0}\" is the name Kalamu gives the quotations: choose another")]
    ReservedName(String),
}

/// The property that holds a field's reason in the model's answer.
pub fn reason_key(key: &str) -> String {
    format!("{key}_reason")
}

impl AnalysisSchema {
    /// Gives new fields a key from their label, unique among the fields beside them (a
    /// repeating group's columns among themselves); existing keys stay.
    pub fn assign_keys(&mut self) {
        assign_keys(&mut self.fields);
    }

    /// Checks a schema before it is saved or run: names given and distinct, choices to choose
    /// from, and a repeating group with columns and no group inside it.
    pub fn validate(&self) -> Result<(), SchemaError> {
        validate_fields(&self.fields, false)
    }

    /// Checks a researcher's analysis: [`AnalysisSchema::validate`], and at most one repeating
    /// group, which is then the only field ([`crate::records`]).
    pub fn validate_analysis(&self) -> Result<(), SchemaError> {
        self.validate()?;
        let groups: Vec<&Field> = self
            .fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::Entries { .. }))
            .collect();
        match groups.as_slice() {
            [] => Ok(()),
            [_] => match self
                .fields
                .iter()
                .find(|f| !matches!(f.kind, FieldKind::Entries { .. }))
            {
                Some(other) => Err(SchemaError::GroupNotAlone(other.label.trim().to_string())),
                None => Ok(()),
            },
            _ => Err(SchemaError::TwoGroups),
        }
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

fn assign_keys(fields: &mut [Field]) {
    let mut taken: Vec<String> = fields
        .iter()
        .filter(|f| !f.key.is_empty())
        .map(|f| f.key.clone())
        .collect();
    for (i, field) in fields.iter_mut().enumerate() {
        if let FieldKind::Entries { fields } | FieldKind::Cited { fields, .. } = &mut field.kind {
            assign_keys(fields);
        }
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

/// `inside` when the fields are a repeating group's columns.
fn validate_fields(fields: &[Field], inside: bool) -> Result<(), SchemaError> {
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
            if inside {
                return Err(SchemaError::NestedGroup(label.to_string()));
            }
            if fields.is_empty() {
                return Err(SchemaError::EmptyGroup(label.to_string()));
            }
            validate_fields(fields, true)?;
        }
        if let FieldKind::Cited { fields, .. } = &field.kind {
            if inside {
                return Err(SchemaError::NestedGroup(label.to_string()));
            }
            cited::validate(fields)?;
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
            FieldKind::Cited { fields, max } => {
                FieldValue::Cited(cited::read(raw, field, fields, *max)?)
            }
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
            (FieldKind::Cited { fields, .. }, value) => cited::render(field, fields, value),
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

/// Each field's label as the most recent of these analyses asking it gives it, by key, with when
/// that analysis was made: what the dictionary's namespaces and a profile's columns call a
/// field. `keep` chooses the fields (the dictionary counts lists only). Analyses made at the
/// same moment are settled by the order given.
pub fn latest_labels<'a>(
    jobs: impl IntoIterator<Item = &'a crate::Job>,
    keep: impl Fn(&Field) -> bool,
) -> std::collections::BTreeMap<String, (crate::Timestamp, String)> {
    let mut labels: std::collections::BTreeMap<String, (crate::Timestamp, String)> =
        std::collections::BTreeMap::new();
    for job in jobs.into_iter().filter(|j| j.task == crate::Task::Analysis) {
        for schema in job.setups.iter().filter_map(|s| s.prompt.schema.as_ref()) {
            for field in schema.fields.iter().filter(|f| keep(f)) {
                let known = labels.get(&field.key);
                if known.is_none_or(|(at, _)| *at <= job.created_at) {
                    labels.insert(field.key.clone(), (job.created_at, field.label.clone()));
                }
            }
        }
    }
    labels
}

#[cfg(test)]
mod tests;
