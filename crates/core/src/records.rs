//! Records: an analysis whose only field is a repeating group ("one of these per entry"), its
//! columns text, choice and list fields (#171). A directory, a register or a membership list
//! holds dozens of entries of the same shape on a page, so such an analysis reads each page on
//! its own and answers it with the page's rows, in reading order; each row is one entry.
//!
//! What makes an analysis records is its fields, never its name: [`group`]. The rows the
//! researcher corrects, adds or removes are a page's corrected answers, kept apart from the
//! model's as every analysis's are; [`edit`] makes one change to them.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{AnalysisSchema, Field, FieldAnswer, FieldKind, FieldValue};

/// The repeating group of a records analysis: its only field, when that is a group. `None`
/// for an analysis answered once per document.
pub fn group(schema: &AnalysisSchema) -> Option<&Field> {
    match schema.fields.as_slice() {
        [field] if matches!(field.kind, FieldKind::Entries { .. }) => Some(field),
        _ => None,
    }
}

/// Whether an analysis of these fields reads page by page, one row per entry.
pub fn is_records(schema: &AnalysisSchema) -> bool {
    group(schema).is_some()
}

/// The group's columns, in order; none for an analysis that is not records.
pub fn columns(schema: &AnalysisSchema) -> &[Field] {
    match group(schema).map(|g| &g.kind) {
        Some(FieldKind::Entries { fields }) => fields,
        _ => &[],
    }
}

/// A page's rows in its answers, in reading order; none when the page has none, or the
/// analysis is not records.
pub fn rows<'a>(schema: &AnalysisSchema, answers: &'a [FieldAnswer]) -> &'a [Vec<FieldAnswer>] {
    let Some(group) = group(schema) else {
        return &[];
    };
    answers
        .iter()
        .find(|a| a.key == group.key)
        .map(|a| a.value.entries())
        .unwrap_or(&[])
}

/// One change to a page's rows, as the review makes it: the page's answers as the researcher
/// left them, with this change, become its corrected answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum RowEdit {
    /// A cell typed anew: the text of a text or choice column (empty for none), a list's items
    /// separated by semicolons.
    Cell {
        row: u32,
        key: String,
        value: String,
    },
    /// An empty row put in at `at`, from 0; at the number of rows, after the last.
    Add {
        at: u32,
    },
    Remove {
        row: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RowError {
    #[error("This analysis has no rows: its fields hold no repeating group")]
    NotRecords,
    #[error("The page has no row {0}")]
    NoRow(u32),
    #[error("The rows have no column \"{0}\"")]
    NoColumn(String),
    #[error("\"{value}\" is not one of the choices for \"{label}\"")]
    NotAChoice { value: String, label: String },
}

impl RowError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotRecords => "records.not_records",
            Self::NoRow(_) => "records.no_row",
            Self::NoColumn(_) => "records.no_column",
            Self::NotAChoice { .. } => "records.not_a_choice",
        }
    }
}

/// A page's answers with one change to its rows. `answers` are the page's as the researcher
/// left them (corrected, else the model's); the result has the group's answer alone, its rows
/// in their new order, each cell written as its column reads it: text trimmed, a choice spelled
/// as the column lists it, a list's items trimmed and without repeats.
pub fn edit(
    schema: &AnalysisSchema,
    answers: &[FieldAnswer],
    change: &RowEdit,
) -> Result<Vec<FieldAnswer>, RowError> {
    let group = group(schema).ok_or(RowError::NotRecords)?;
    let columns = columns(schema);
    let mut rows: Vec<Vec<FieldAnswer>> = rows(schema, answers)
        .iter()
        .map(|row| complete(columns, row))
        .collect();
    match change {
        RowEdit::Cell { row, key, value } => {
            let column = columns
                .iter()
                .find(|c| &c.key == key)
                .ok_or_else(|| RowError::NoColumn(key.clone()))?;
            let cells = rows.get_mut(*row as usize).ok_or(RowError::NoRow(*row))?;
            let written = cell(column, value)?;
            if let Some(answer) = cells.iter_mut().find(|a| &a.key == key) {
                answer.value = written;
            }
        }
        RowEdit::Add { at } => {
            let at = (*at as usize).min(rows.len());
            rows.insert(at, complete(columns, &[]));
        }
        RowEdit::Remove { row } => {
            if (*row as usize) >= rows.len() {
                return Err(RowError::NoRow(*row));
            }
            rows.remove(*row as usize);
        }
    }
    let reason = answers
        .iter()
        .find(|a| a.key == group.key)
        .and_then(|a| a.reason.clone());
    Ok(vec![FieldAnswer {
        key: group.key.clone(),
        value: FieldValue::Entries(rows),
        reason,
    }])
}

/// A row with one answer per column, in the columns' order: a column the row lacks is empty.
fn complete(columns: &[Field], row: &[FieldAnswer]) -> Vec<FieldAnswer> {
    columns
        .iter()
        .map(|column| {
            row.iter()
                .find(|a| a.key == column.key)
                .cloned()
                .unwrap_or_else(|| FieldAnswer {
                    key: column.key.clone(),
                    value: empty(column),
                    reason: None,
                })
        })
        .collect()
}

fn empty(column: &Field) -> FieldValue {
    match column.kind {
        FieldKind::List { .. } | FieldKind::Choices { .. } => FieldValue::List(Vec::new()),
        _ => FieldValue::Text(String::new()),
    }
}

/// A cell as typed, as its column reads it.
pub(crate) fn cell(column: &Field, typed: &str) -> Result<FieldValue, RowError> {
    let choose = |value: &str, values: &[String]| {
        values
            .iter()
            .find(|v| v.trim().to_lowercase() == value.trim().to_lowercase())
            .map(|v| v.trim().to_string())
            .ok_or_else(|| RowError::NotAChoice {
                value: value.trim().to_string(),
                label: column.label.trim().to_string(),
            })
    };
    let items = || -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for item in typed.split([';', '\n']).map(str::trim) {
            if !item.is_empty() && !out.iter().any(|o| o.to_lowercase() == item.to_lowercase()) {
                out.push(item.to_string());
            }
        }
        out
    };
    Ok(match &column.kind {
        FieldKind::Text | FieldKind::Entries { .. } | FieldKind::Cited { .. } => {
            FieldValue::Text(typed.trim().to_string())
        }
        FieldKind::Choice { .. } if typed.trim().is_empty() => FieldValue::Text(String::new()),
        FieldKind::Choice { values } => FieldValue::Text(choose(typed, values)?),
        FieldKind::List { max } => {
            let mut items = items();
            if let Some(max) = max {
                items.truncate(*max as usize);
            }
            FieldValue::List(items)
        }
        FieldKind::Choices { values } => FieldValue::List(
            items()
                .iter()
                .map(|item| choose(item, values))
                .collect::<Result<_, _>>()?,
        ),
    })
}

#[cfg(test)]
mod tests;
