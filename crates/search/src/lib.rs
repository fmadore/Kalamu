//! Project search (#86): every passage of a project's documents that holds some words, found
//! in the text as the researcher corrected it, each opening its review where it stands.
//!
//! The text searched is the review's: a correction in place of the model's text, the speakers
//! named, an analysis's corrected answers. It is kept in a [`SearchIndex`] that is only ever
//! derived from the job store: before each search the passages of the results searched are
//! checked against what they were made from, and those that changed are made again, so a
//! correction, a new preference or a deleted job shows at once, and an index lost or damaged
//! is made again whole. Nothing here calls a model or changes a result.

use std::sync::Arc;

use kalamu_core::{
    CollectionId, DocumentId, JobId, JobStore, PieceRef, ProjectId, ProjectStore, SearchIndex,
    SetupId, SourceId, StoreError, Task,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

mod fold;
mod passages;
mod query;

pub use fold::{MIN_QUERY, folded, searchable};

/// The most matches listed; all are counted.
pub const MAX_LISTED: usize = 1000;

/// Which results of each document are searched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ResultPolicy {
    /// The preferred result of each task where the researcher chose one, every result where
    /// they did not.
    #[default]
    Preferred,
    /// Every result, preferred or not.
    All,
}

/// Which pieces are searched by whether the researcher has reviewed them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ReviewFilter {
    #[default]
    Any,
    Reviewed,
    NotReviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchRequest {
    /// Words found as one literal phrase.
    pub query: String,
    pub collection: Option<CollectionId>,
    pub tag: Option<String>,
    pub task: Option<Task>,
    pub review: ReviewFilter,
    pub results: ResultPolicy,
    /// Also search what models wrote about the documents: analyses' answers and reasons, and
    /// tables of contents' entries. Off, only the text read from them is searched.
    pub generated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SearchView {
    /// Documents with matches, in title order.
    pub documents: Vec<DocumentHits>,
    /// Every match found.
    pub matches: u32,
    /// The matches listed, at most [`MAX_LISTED`].
    pub listed: u32,
    pub coverage: Coverage,
}

/// What was searched, so that finding nothing is not read as absence.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Coverage {
    /// Documents in the collection or tag searched, or the whole project.
    pub documents: u32,
    /// Documents with some text searched.
    pub searched: u32,
    /// Documents with nothing read yet of the kinds searched.
    pub unread: u32,
    /// Pieces of the results searched that are not read: still to do, or failed.
    pub pieces_unread: u32,
    /// Documents where a task has several results and none is preferred, all searched.
    pub without_preference: u32,
    /// Pieces searched.
    pub pieces: u32,
}

/// One document's matches, result by result: two readings of it stay two results of one
/// document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentHits {
    pub document_id: DocumentId,
    pub title: String,
    pub results: Vec<ResultHits>,
}

/// The matches in one result: a set-up of a job over the document's source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResultHits {
    pub job_id: JobId,
    pub job_name: String,
    pub setup_id: SetupId,
    pub setup_label: String,
    pub source_id: SourceId,
    pub task: Task,
    /// The researcher's preferred result of its task for this document.
    pub preferred: bool,
    /// What a model wrote about the document (an analysis, a table of contents), not text read
    /// from it.
    pub generated: bool,
    /// An analysis of text corrected since it was read: the app fills it in from the engine.
    pub stale: bool,
    pub hits: Vec<Hit>,
}

/// One place the words stand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Hit {
    /// The piece it is in, from 0, as the review numbers them.
    pub index: u32,
    pub piece: PieceRef,
    /// Which match it is in its piece's text, from 0, as the review's search counts them.
    pub nth: u32,
    /// For an answer, its field: key and label.
    pub field: Option<FieldRef>,
    /// For a field of a table of contents' entry, the entry's title.
    pub entry: Option<String>,
    /// Found in the model's reason for the answer.
    pub reason: bool,
    pub before: String,
    pub words: String,
    pub after: String,
    /// In a recording, when its line begins, in milliseconds; none where the text was
    /// corrected into other words.
    #[ts(type = "number | null")]
    pub at_ms: Option<u64>,
    pub reviewed: bool,
    pub corrected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FieldRef {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Project not found")]
    NotFound,
    #[error("Search needs three letters at least")]
    TooShort,
    /// A newer search began before this one ended.
    #[error("Search stopped")]
    Stopped,
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl SearchError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "project.not_found",
            Self::TooShort => "search.too_short",
            Self::Stopped => "search.stopped",
            Self::Store(_) => "search.index",
        }
    }
}

pub struct Search {
    pub(crate) projects: Arc<dyn ProjectStore>,
    pub(crate) jobs: Arc<dyn JobStore>,
    pub(crate) index: Arc<dyn SearchIndex>,
}

impl Search {
    /// The project and job stores are usually one database; the index is a file of its own.
    pub fn new(
        projects: Arc<dyn ProjectStore>,
        jobs: Arc<dyn JobStore>,
        index: Arc<dyn SearchIndex>,
    ) -> Self {
        Self {
            projects,
            jobs,
            index,
        }
    }

    /// Forgets the passages of a deleted job.
    pub fn forget_job(&self, job: JobId) -> Result<(), SearchError> {
        Ok(self.index.forget_job(job)?)
    }

    /// Forgets the passages of every job no longer in the store: those deleted while the index
    /// could not be told.
    pub fn prune(&self) -> Result<u32, SearchError> {
        let mut forgotten = 0;
        for job in self.index.indexed_jobs()? {
            if matches!(self.jobs.job(job), Err(StoreError::NotFound)) {
                self.index.forget_job(job)?;
                forgotten += 1;
            }
        }
        Ok(forgotten)
    }

    /// Empties the index; the next search makes again what it needs from the store.
    pub fn rebuild(&self) -> Result<(), SearchError> {
        Ok(self.index.clear()?)
    }

    /// The project, or [`SearchError::NotFound`].
    pub(crate) fn check_project(&self, project: ProjectId) -> Result<(), SearchError> {
        match self.projects.project(project) {
            Ok(_) => Ok(()),
            Err(StoreError::NotFound) => Err(SearchError::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests;
