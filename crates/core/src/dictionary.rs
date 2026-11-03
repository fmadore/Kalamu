//! A project's dictionary: the names and terms its analyses found, gathered into entries, each
//! one person, place or term under its spellings (#48).
//!
//! The analyses' answers are never changed: an entry maps the forms the models wrote, kept as
//! they were, to the label the researcher prefers. Every entry is the researcher's decision; a
//! form no entry holds is its own, not yet reviewed. Names of one field (a namespace: people,
//! places) are never mixed with another's, and the same spelling is not taken as evidence that
//! two mentions are one person.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    AnalysisSchema, CollectionId, EntryId, Field, FieldAnswer, FieldKind, FieldValue, JobId,
    ProjectId, StoreError, Timestamp,
};

/// A form as an analysis wrote it, its spaces trimmed and runs of spaces made one; capitals and
/// accents are kept, since they may tell two names apart. `None` when nothing is left.
pub fn form_of(written: &str) -> Option<String> {
    let form = written.split_whitespace().collect::<Vec<_>>().join(" ");
    (!form.is_empty()).then_some(form)
}

/// Spellings gathered as one person, place or term, under the label the researcher prefers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Entry {
    pub id: EntryId,
    pub project_id: ProjectId,
    /// The analysis field its forms come from (`people`): entries never cross fields.
    pub namespace: String,
    /// The researcher's preferred label, which need not be one of the forms.
    pub label: String,
    /// The forms the analyses wrote, as written, each in one entry of the namespace at most.
    pub forms: Vec<String>,
    /// The researcher has checked it.
    pub reviewed: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// The labels of a project's entries, by namespace and form, for writing an answer under the
/// researcher's labels without changing it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels(HashMap<(String, String), String>);

impl Labels {
    pub fn of(entries: &[Entry]) -> Self {
        Self(
            entries
                .iter()
                .flat_map(|e| {
                    e.forms
                        .iter()
                        .map(|f| ((e.namespace.clone(), f.clone()), e.label.clone()))
                })
                .collect(),
        )
    }

    /// The forms of a list answer to a field, each under its entry's label, a form no entry
    /// holds as written; a label two forms reach is written once, where it first appears.
    pub fn applied(&self, namespace: &str, written: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::with_capacity(written.len());
        for item in written {
            let Some(form) = form_of(item) else { continue };
            let label = self
                .0
                .get(&(namespace.to_string(), form.clone()))
                .cloned()
                .unwrap_or(form);
            if !out.contains(&label) {
                out.push(label);
            }
        }
        out
    }
}

/// Two entries the researcher says are not the same, however alike: never merged, and later
/// proposals will not offer them again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KeptApart {
    pub a: EntryId,
    pub b: EntryId,
}

impl KeptApart {
    /// The pair in a fixed order, so it is stored once whichever way it was given.
    pub fn new(a: EntryId, b: EntryId) -> Self {
        if a <= b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }

    pub fn holds(&self, id: EntryId) -> bool {
        self.a == id || self.b == id
    }
}

/// A request for merges proposed by a model: the analysis job that asks, and the rows each of
/// its sources holds, so the answers can be read back against them. The job runs as any other,
/// through the engine; nothing it answers is applied without the researcher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalRequest {
    pub id: String,
    pub project_id: ProjectId,
    pub namespace: String,
    /// The collection the rows were drawn from; the whole project when absent.
    pub collection: Option<CollectionId>,
    pub job_id: JobId,
    /// One per source of the job, in order.
    pub groups: Vec<ProposalGroup>,
    /// Rows of the namespace when it was sent; every one is in at least one group.
    pub rows: u32,
    pub created_at: Timestamp,
}

/// The rows one source of a proposal job sent, numbered from 1 as `R1`, `R2` in its text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProposalGroup {
    /// SHA-256 of the text sent: the source it is.
    pub sha256: String,
    /// Each row by the forms it held then, so an answer still finds them after the entries
    /// have been merged or taken apart.
    pub rows: Vec<Vec<String>>,
}

impl ProposalRequest {
    /// A fresh id for a request about to be kept.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// The analysis a proposal job answers: groups of rows, each one person, place or term.
pub mod proposal {
    use super::*;

    /// The field holding the groups.
    pub const GROUPS: &str = "groups";
    /// A group's rows, as `R1`, `R7`.
    pub const ROWS: &str = "rows";
    pub const LABEL: &str = "label";
    pub const REASON: &str = "reason";
    pub const CERTAINTY: &str = "certainty";
    /// How sure the model says it is, in words: not a calibrated probability.
    pub const CERTAINTIES: [&str; 3] = ["certain", "likely", "possible"];

    fn field(key: &str, label: &str, instructions: &str, kind: FieldKind) -> Field {
        Field {
            key: key.to_string(),
            label: label.to_string(),
            instructions: instructions.to_string(),
            kind,
            reason: false,
        }
    }

    pub fn schema() -> AnalysisSchema {
        AnalysisSchema {
            fields: vec![field(
                GROUPS,
                "Groups",
                "Every group of rows that are one person, place, organisation or term; none \
                 when there is no such group.",
                FieldKind::Entries {
                    fields: vec![
                        field(
                            ROWS,
                            "Rows",
                            "The references of the rows that are one, as given (R1, R7): at \
                             least two.",
                            FieldKind::List { max: None },
                        ),
                        field(
                            LABEL,
                            "Label",
                            "The preferred label for them: the fullest correct form the rows \
                             give.",
                            FieldKind::Text,
                        ),
                        field(
                            REASON,
                            "Reason",
                            "One sentence: why they are one, from the forms and the documents.",
                            FieldKind::Text,
                        ),
                        field(
                            CERTAINTY,
                            "Certainty",
                            "How sure you are.",
                            FieldKind::Choice {
                                values: CERTAINTIES.iter().map(|c| c.to_string()).collect(),
                            },
                        ),
                    ],
                },
            )],
        }
    }

    /// One group as the model answered it, before its references are checked.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Answered {
        pub rows: Vec<String>,
        pub label: String,
        pub reason: String,
        pub certainty: String,
    }

    /// The groups in an answer; a missing field reads as no group.
    pub fn groups(answers: &[FieldAnswer]) -> Vec<Answered> {
        let Some(groups) = answers.iter().find(|a| a.key == GROUPS) else {
            return Vec::new();
        };
        groups
            .value
            .entries()
            .iter()
            .map(|entry| Answered {
                rows: match entry.iter().find(|a| a.key == ROWS).map(|a| &a.value) {
                    Some(FieldValue::List(rows)) => rows.clone(),
                    Some(FieldValue::Text(row)) => vec![row.clone()],
                    _ => Vec::new(),
                },
                label: crate::analysis::answer_text(entry, LABEL),
                reason: crate::analysis::answer_text(entry, REASON),
                certainty: crate::analysis::answer_text(entry, CERTAINTY),
            })
            .collect()
    }

    /// The row a reference names, numbered from 1 (`R7`, `r7` or `7`); `None` for anything
    /// else.
    pub fn row_number(reference: &str) -> Option<usize> {
        let r = reference.trim();
        let digits = r
            .strip_prefix('R')
            .or_else(|| r.strip_prefix('r'))
            .unwrap_or(r);
        digits.parse::<usize>().ok().filter(|n| *n > 0)
    }
}

#[derive(Debug, Error)]
pub enum DictionaryError {
    #[error("Project not found")]
    NotFound,
    /// An entry of another project, or one that no longer exists.
    #[error("This entry is not in the project's dictionary")]
    NoEntry,
    #[error("An entry needs a label")]
    NoLabel,
    #[error("Entries of different fields cannot be merged")]
    Namespaces,
    /// Merging two entries the researcher kept apart.
    #[error("You kept these apart")]
    KeptApart,
    /// Fewer than two things to merge or keep apart.
    #[error("Choose at least two")]
    TooFew,
    /// Keeping apart two forms of one entry.
    #[error("These are in one entry: take one out of it first")]
    SameEntry,
    /// Fewer than two rows in the field to compare.
    #[error("This field has fewer than two rows to compare")]
    NothingToCompare,
    #[error("The dictionary could not be written out: {0}")]
    Export(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl DictionaryError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound | Self::Store(StoreError::NotFound) => "project.not_found",
            Self::NoEntry => "dictionary.no_entry",
            Self::NoLabel => "dictionary.no_label",
            Self::Namespaces => "dictionary.namespaces",
            Self::KeptApart => "dictionary.kept_apart",
            Self::TooFew => "dictionary.too_few",
            Self::SameEntry => "dictionary.same_entry",
            Self::NothingToCompare => "dictionary.nothing_to_compare",
            Self::Export(_) => "file.read",
            Self::Store(_) => "job.database",
        }
    }
}

/// Persistence of the researcher's dictionary decisions, beside the project's rows.
pub trait DictionaryStore: Send + Sync {
    fn put_entry(&self, entry: &Entry) -> Result<(), StoreError>;
    /// The project's entries in the order they were made.
    fn entries(&self, project: ProjectId) -> Result<Vec<Entry>, StoreError>;
    /// Removes the entry and the pairs it was kept apart in; its forms are their own again.
    fn delete_entry(&self, id: EntryId) -> Result<(), StoreError>;
    fn put_apart(&self, project: ProjectId, pair: &KeptApart) -> Result<(), StoreError>;
    fn remove_apart(&self, pair: &KeptApart) -> Result<(), StoreError>;
    fn apart(&self, project: ProjectId) -> Result<Vec<KeptApart>, StoreError>;
    fn put_request(&self, request: &ProposalRequest) -> Result<(), StoreError>;
    /// The project's proposal requests, oldest first.
    fn requests(&self, project: ProjectId) -> Result<Vec<ProposalRequest>, StoreError>;
    fn delete_request(&self, id: &str) -> Result<(), StoreError>;
    /// A proposal the researcher turned down, by the forms it would join.
    fn put_dismissed(
        &self,
        project: ProjectId,
        namespace: &str,
        key: &str,
    ) -> Result<(), StoreError>;
    fn remove_dismissed(
        &self,
        project: ProjectId,
        namespace: &str,
        key: &str,
    ) -> Result<(), StoreError>;
    /// `(namespace, key)` of every proposal turned down.
    fn dismissed(&self, project: ProjectId) -> Result<Vec<(String, String)>, StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_replace_the_forms_an_entry_holds_in_its_namespace_only() {
        let entry = |namespace: &str, label: &str, forms: &[&str]| Entry {
            id: EntryId::new(),
            project_id: ProjectId::new(),
            namespace: namespace.into(),
            label: label.into(),
            forms: forms.iter().map(|f| f.to_string()).collect(),
            reviewed: true,
            created_at: Timestamp(0),
            updated_at: Timestamp(0),
        };
        let labels = Labels::of(&[
            entry(
                "people",
                "Moussa Traoré",
                &["Moussa Traore", "Traoré Moussa"],
            ),
            entry("places", "Bobo-Dioulasso", &["Bobo"]),
        ]);
        let written: Vec<String> = ["Traoré  Moussa", "Bobo", "Moussa Traore", "Awa Diallo", " "]
            .map(String::from)
            .into();
        assert_eq!(
            labels.applied("people", &written),
            ["Moussa Traoré", "Bobo", "Awa Diallo"],
            "a place's entry leaves a person's form alone, and one label is written once"
        );
        assert_eq!(Labels::default().applied("people", &written).len(), 4);
    }

    #[test]
    fn a_form_keeps_its_capitals_and_accents() {
        assert_eq!(
            form_of("  El Hadj   Oumar  ").as_deref(),
            Some("El Hadj Oumar")
        );
        assert_eq!(form_of("Père Éric").as_deref(), Some("Père Éric"));
        assert_eq!(form_of(" \n "), None);
    }

    #[test]
    fn references_name_rows_from_one() {
        assert_eq!(proposal::row_number("R7"), Some(7));
        assert_eq!(proposal::row_number(" r12 "), Some(12));
        assert_eq!(proposal::row_number("3"), Some(3));
        assert_eq!(proposal::row_number("R0"), None);
        assert_eq!(proposal::row_number("Moussa"), None);
        assert!(proposal::schema().validate().is_ok());
    }

    #[test]
    fn a_pair_kept_apart_is_one_whichever_way_it_is_given() {
        let (a, b) = (EntryId::new(), EntryId::new());
        assert_eq!(KeptApart::new(a, b), KeptApart::new(b, a));
        assert!(KeptApart::new(a, b).holds(b));
    }
}
