//! What project search keeps: the corrected text of each piece of each result, as the review
//! shows it, in a local index that is only ever derived from the store.
//!
//! The index is a cache, never a source of truth. Each passage carries a stamp of what it was
//! made from (the stored answer, the researcher's correction, the speakers' names); a passage
//! whose stamp no longer matches is made again before anything is searched, and a lost or
//! damaged index is rebuilt from the results and corrections, which it never changes.

use serde::{Deserialize, Serialize};

use crate::{JobId, PieceRef, SetupId, SourceId, StoreError};

/// One piece of one source under one set-up of a job: what a passage is the text of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PassageKey {
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub source_id: SourceId,
    /// The piece's position within its source, from 0.
    pub index: u32,
}

/// The text of one piece as the review shows it: corrections applied and speakers named, or an
/// analysis's answers as the researcher left them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Passage {
    pub key: PassageKey,
    /// What the passage was made from; another stamp means it must be made again.
    pub stamp: String,
    pub piece: PieceRef,
    pub reviewed: bool,
    pub corrected: bool,
    /// The text, one part; or an analysis's answers and reasons, a part each.
    pub parts: Vec<PassagePart>,
}

/// A stretch of a passage searched on its own, so that a match never runs from one answer
/// into the next.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassagePart {
    /// For an answer, its field's key; none for the text of a page or a recording.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// For a field of a table of contents' entry, the entry's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    /// For a field of a table of contents' entry, the entry's place in the table, from 1, as
    /// the review numbers them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_number: Option<u32>,
    /// The model's reason for the answer rather than the answer.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reason: bool,
    pub text: String,
    /// A recording's timed lines, each with its start from the start of the piece in
    /// milliseconds, while the text is the model's own; empty once corrected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<(u64, String)>,
}

/// How big the index is, to show and to measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexSize {
    pub passages: u64,
    pub bytes: u64,
}

/// The search index, kept apart from the job store: losing it loses nothing, since every
/// passage is made again from the store. Implemented with SQLite's full-text search.
pub trait SearchIndex: Send + Sync {
    /// The stamp of every passage indexed for a job.
    fn stamps(&self, job: JobId) -> Result<Vec<(PassageKey, String)>, StoreError>;
    /// Indexes passages, each with its text folded for matching, replacing any under the same
    /// key, all at once.
    fn put_passages(&self, passages: &[(Passage, String)]) -> Result<(), StoreError>;
    fn remove_passages(&self, keys: &[PassageKey]) -> Result<(), StoreError>;
    /// The jobs that have passages, so that those of deleted jobs can be forgotten.
    fn indexed_jobs(&self) -> Result<Vec<JobId>, StoreError>;
    fn forget_job(&self, job: JobId) -> Result<(), StoreError>;
    /// Every passage whose folded text holds each of `terms`, every one folded and three
    /// characters at least.
    fn find(&self, terms: &[String]) -> Result<Vec<Passage>, StoreError>;
    /// Empties the index, to be made again from the store.
    fn clear(&self) -> Result<(), StoreError>;
    fn size(&self) -> Result<IndexSize, StoreError>;
}
