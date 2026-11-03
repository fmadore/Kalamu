//! A project's dictionary (#48): the names and terms its analyses found, gathered from every
//! document's answers, and the researcher's entries that map the forms the models wrote to one
//! preferred label.
//!
//! The inventory is read from the job store each time, through the corrected view of each
//! answer, so it cannot drift from the results; the entries and the pairs kept apart are the
//! researcher's decisions, kept by a [`DictionaryStore`]. Nothing here calls a model or changes
//! a result. A list answer says a document names someone, not where: the inventory counts
//! documents, and how many of the results that read each listed the form, never mentions.

use std::sync::Arc;

use kalamu_core::{
    DictionaryError, DictionaryStore, EntryId, JobStore, Labels, ProjectId, ProjectStore,
    StoreError,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

mod decisions;
mod export;
mod inventory;
mod proposals;
mod requests;

pub use inventory::{DictionaryRow, DictionaryView, Evidence, FormView, Namespace};
pub use proposals::{Proposal, ProposalMember, ProposalsView, RequestView};
pub use requests::{INPUT_TOKENS, Prepared, PreparedGroup};

/// One line of the dictionary as the researcher picks it: an entry, or a form no entry holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum Item {
    Entry { id: EntryId },
    Form { form: String },
}

pub struct Dictionary {
    pub(crate) projects: Arc<dyn ProjectStore>,
    pub(crate) jobs: Arc<dyn JobStore>,
    pub(crate) store: Arc<dyn DictionaryStore>,
}

impl Dictionary {
    /// The three stores are usually the same database.
    pub fn new(
        projects: Arc<dyn ProjectStore>,
        jobs: Arc<dyn JobStore>,
        store: Arc<dyn DictionaryStore>,
    ) -> Self {
        Self {
            projects,
            jobs,
            store,
        }
    }

    /// The labels of the project's entries, for an export to write answers under them.
    pub fn labels(&self, project: ProjectId) -> Result<Labels, DictionaryError> {
        self.check_project(project)?;
        Ok(Labels::of(&self.store.entries(project)?))
    }

    /// The project, or [`DictionaryError::NotFound`].
    pub(crate) fn check_project(&self, project: ProjectId) -> Result<(), DictionaryError> {
        match self.projects.project(project) {
            Ok(_) => Ok(()),
            Err(StoreError::NotFound) => Err(DictionaryError::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

/// A form folded for comparison only: capitals, accents and punctuation set aside, as
/// iwac-ai-pipelines folds names before matching them. Two forms with one key are offered as
/// possibly the same, never merged on that account.
pub(crate) fn folded(form: &str) -> String {
    let plain: String = form
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The folded words in alphabetical order, so `Traoré Moussa` meets `Moussa Traore`; `None`
/// for one word, which the plain key already compares.
pub(crate) fn word_order_key(form: &str) -> Option<String> {
    let folded = folded(form);
    let mut words: Vec<&str> = folded.split(' ').collect();
    if words.len() < 2 {
        return None;
    }
    words.sort_unstable();
    Some(words.join(" "))
}

#[cfg(test)]
mod tests;
