//! A project's dictionary: the names and terms its analyses found, gathered into entries, each
//! one person, place or term under its spellings (#48).
//!
//! The analyses' answers are never changed: an entry maps the forms the models wrote, kept as
//! they were, to the label the researcher prefers. Every entry is the researcher's decision; a
//! form no entry holds is its own, not yet reviewed. Names of one field (a namespace: people,
//! places) are never mixed with another's, and the same spelling is not taken as evidence that
//! two mentions are one person.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{EntryId, ProjectId, StoreError, Timestamp};

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
    #[error("The dictionary could not be written out: {0}")]
    Export(String),
    #[error(transparent)]
    Store(#[from] StoreError),
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_pair_kept_apart_is_one_whichever_way_it_is_given() {
        let (a, b) = (EntryId::new(), EntryId::new());
        assert_eq!(KeptApart::new(a, b), KeptApart::new(b, a));
        assert!(KeptApart::new(a, b).holds(b));
    }
}
