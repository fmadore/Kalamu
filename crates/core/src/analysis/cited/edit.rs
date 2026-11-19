//! The researcher's changes to a cited field, a layer over the model's answer like every
//! correction: an entry's fields typed anew, an entry removed, one quotation of an entry
//! removed. Quotations are never added or changed this way: a correction's quotations must be
//! some of the model's, exactly ([`keeps_evidence`]).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use super::{CitedValue, Evidence};
use crate::{Field, FieldAnswer, FieldKind, FieldValue, RowError};

/// One change to a cited field's entries, as the review makes it: the answers as the researcher
/// left them, with this change, become the piece's corrected answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum CitedEdit {
    /// One of an entry's own fields typed anew: the text of a text or choice field (empty for
    /// none), a list's items separated by semicolons.
    Answer {
        entry: u32,
        key: String,
        value: String,
    },
    /// An entry taken out, with its quotations.
    RemoveEntry { entry: u32 },
    /// One quotation taken out of an entry; the entry stays, also with none left.
    RemoveQuotation { entry: u32, quotation: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CitedError {
    #[error("\"{0}\" is not a field of entries that cite the text")]
    NotCited(String),
    #[error("The field has no entry {0}")]
    NoEntry(u32),
    #[error("The entry has no quotation {0}")]
    NoQuotation(u32),
    #[error("The entries have no field \"{0}\"")]
    NoField(String),
    #[error("\"{value}\" is not one of the choices for \"{label}\"")]
    NotAChoice { value: String, label: String },
    /// A correction holding a quotation the model's answer does not: quotations are kept only
    /// as found in the text.
    #[error("A quotation in \"{0}\" is not one found in the text")]
    Invented(String),
}

impl CitedError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotCited(_) => "cited.not_cited",
            Self::NoEntry(_) => "cited.no_entry",
            Self::NoQuotation(_) => "cited.no_quotation",
            Self::NoField(_) => "cited.no_field",
            Self::NotAChoice { .. } => "cited.not_a_choice",
            Self::Invented(_) => "cited.invented",
        }
    }
}

/// The answers with one change to the cited field `key`. `answers` are the piece's as the
/// researcher left them (corrected, else the model's); every other field's answer is kept as it
/// is.
pub fn edit(
    fields: &[Field],
    answers: &[FieldAnswer],
    key: &str,
    change: &CitedEdit,
) -> Result<Vec<FieldAnswer>, CitedError> {
    let field = fields
        .iter()
        .find(|f| f.key == key)
        .ok_or_else(|| CitedError::NotCited(key.to_string()))?;
    let FieldKind::Cited { fields: own, .. } = &field.kind else {
        return Err(CitedError::NotCited(field.label.trim().to_string()));
    };
    let mut out = answers.to_vec();
    let answer = out
        .iter_mut()
        .find(|a| a.key == key)
        .ok_or(CitedError::NoEntry(entry_of(change)))?;
    let FieldValue::Cited(value) = &mut answer.value else {
        return Err(CitedError::NotCited(field.label.trim().to_string()));
    };
    let at = entry_of(change);
    if at as usize >= value.entries.len() {
        return Err(CitedError::NoEntry(at));
    }
    match change {
        CitedEdit::Answer {
            key, value: typed, ..
        } => {
            let column = own
                .iter()
                .find(|f| &f.key == key)
                .ok_or_else(|| CitedError::NoField(key.clone()))?;
            let written = crate::records::cell(column, typed).map_err(|e| match e {
                RowError::NotAChoice { value, label } => CitedError::NotAChoice { value, label },
                _ => CitedError::NoField(key.clone()),
            })?;
            let entry = &mut value.entries[at as usize];
            match entry.answers.iter_mut().find(|a| &a.key == key) {
                Some(answer) => answer.value = written,
                None => entry.answers.push(FieldAnswer {
                    key: key.clone(),
                    value: written,
                    reason: None,
                }),
            }
        }
        CitedEdit::RemoveEntry { .. } => {
            value.entries.remove(at as usize);
        }
        CitedEdit::RemoveQuotation { quotation, .. } => {
            let evidence = &mut value.entries[at as usize].evidence;
            if *quotation as usize >= evidence.len() {
                return Err(CitedError::NoQuotation(*quotation));
            }
            evidence.remove(*quotation as usize);
        }
    }
    Ok(out)
}

fn entry_of(change: &CitedEdit) -> u32 {
    match change {
        CitedEdit::Answer { entry, .. }
        | CitedEdit::RemoveEntry { entry }
        | CitedEdit::RemoveQuotation { entry, .. } => *entry,
    }
}

/// Checks a correction's cited fields against the model's answer before it is kept: each
/// quotation must be one the model's answer holds for that field, exactly (its place, its
/// words, its turn, speaker and times), and how many were left out stays the model's count.
/// A cited field whose value is not entries is refused.
pub fn keeps_evidence(
    fields: &[Field],
    model: &[FieldAnswer],
    corrected: &mut [FieldAnswer],
) -> Result<(), CitedError> {
    for field in fields {
        if !matches!(field.kind, FieldKind::Cited { .. }) {
            continue;
        }
        let Some(answer) = corrected.iter_mut().find(|a| a.key == field.key) else {
            continue;
        };
        let label = || field.label.trim().to_string();
        let FieldValue::Cited(value) = &mut answer.value else {
            return Err(CitedError::NotCited(label()));
        };
        let empty = CitedValue::default();
        let theirs = model
            .iter()
            .find(|a| a.key == field.key)
            .and_then(|a| a.value.cited())
            .unwrap_or(&empty);
        let found: Vec<&Evidence> = theirs.entries.iter().flat_map(|e| &e.evidence).collect();
        let invented = value
            .entries
            .iter()
            .flat_map(|e| &e.evidence)
            .any(|e| !found.contains(&e));
        if invented {
            return Err(CitedError::Invented(label()));
        }
        value.left_out = theirs.left_out;
        value.dropped = theirs.dropped;
        for entry in &mut value.entries {
            entry.claims.clear();
        }
    }
    Ok(())
}
