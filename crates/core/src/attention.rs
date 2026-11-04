//! The review queue ([#85](https://github.com/fmadore/Kalamu/issues/85)): across a project's
//! documents, the pages and answers that need the researcher, each with its reason.
//!
//! The queue is gathered from what is already stored (pieces that failed or are not read yet,
//! answers cut off at the model's limit, set-ups of a job that read a piece differently, inputs
//! frozen from a result that has changed since, answers not yet reviewed); it never calls a
//! model. Nothing in it approves anything: a piece is reviewed only when the researcher marks it
//! so, and agreement between models is no more than the absence of one reason.
//!
//! The project chooses which results the queue reads ([`ReviewScope`]); the engine, which knows
//! the stored pieces, says what each needs ([`ReviewQueue`]).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{DocumentId, JobId, PieceRef, SetupId, SourceId, Task, Timestamp};

/// One result the queue reads: a set-up of a job over the source that is a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewTarget {
    pub document_id: DocumentId,
    /// The document's title, to name it in the queue.
    pub document: String,
    pub job_id: JobId,
    pub source_id: SourceId,
    pub setup_id: SetupId,
    /// The researcher's preferred result for the document's task.
    pub preferred: bool,
}

/// What the queue covers: how many documents are in scope, and the results it reads from them.
/// A document with several results is still one document.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewScope {
    pub documents: u32,
    pub targets: Vec<ReviewTarget>,
}

/// Why a piece, or a result as a whole, is in the queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum Signal {
    /// Processing failed, with the reason stored; nothing was read to review.
    Failed { error: Option<String> },
    /// Not read yet: waiting, being read, with the provider in a batch, or never started.
    NotRead,
    /// The answer stopped at the model's output limit: the end may be missing.
    Truncated,
    /// Other set-ups of the same job read it differently: their labels, how many words
    /// differ from the one that differs most (texts), and the fields answered differently
    /// (analyses and tables of contents). Prose fields are not compared, since two models
    /// never word a summary alike.
    Disagrees {
        with: Vec<String>,
        words: u32,
        fields: Vec<String>,
    },
    /// Read from a text frozen from an earlier result that has been corrected or read further
    /// since (`gone` when that result's job was deleted), or, with `unit`, from a document
    /// unit's pages since changed (`gone` when the unit was deleted): this answer read an
    /// earlier version.
    Stale { gone: bool, unit: bool },
    /// Read, and not yet marked as reviewed by the researcher.
    NotReviewed,
}

impl Signal {
    /// Every kind of signal, as serialised, in the order the queue lists its reasons: the
    /// interface reads them from `src/lib/coreLists.json` rather than keeping its own copy.
    pub const KINDS: [&'static str; 6] = [
        "failed",
        "not_read",
        "truncated",
        "disagrees",
        "stale",
        "not_reviewed",
    ];

    /// A processing problem: nothing was read, so there is no answer to review.
    pub fn is_processing(&self) -> bool {
        matches!(self, Self::Failed { .. } | Self::NotRead)
    }
}

/// One line of the queue: a piece of a result, or the result as a whole, with its reasons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueueItem {
    pub document_id: DocumentId,
    pub document: String,
    pub job_id: JobId,
    pub job_name: String,
    pub task: Task,
    pub source_id: SourceId,
    pub setup_id: SetupId,
    pub setup_label: String,
    pub model: String,
    pub preferred: bool,
    /// The page, passage, segment or whole document read; `None` for the result as a whole:
    /// never started, or read from an input that has changed since.
    pub piece: Option<PieceRef>,
    /// The piece's position in its source, from 0, to open the review there.
    pub index: Option<u32>,
    /// The stored answer: the first characters of its fingerprint, and when it was stored. A
    /// new answer is a new revision, and its review starts afresh.
    pub revision: Option<String>,
    pub answered_at: Option<Timestamp>,
    /// Why it is listed; never empty.
    pub signals: Vec<Signal>,
    /// Marked as reviewed by the researcher; corrected by them.
    pub reviewed: bool,
    pub corrected: bool,
    /// A processing problem, kept apart from answers that need review.
    pub processing: bool,
    /// Still needs the researcher: a processing problem or a changed input, or an answer not
    /// reviewed. A reviewed item stays listed, as resolved, for as long as its reasons hold.
    pub open: bool,
}

impl QueueItem {
    /// Flagged for a reason beyond not being reviewed yet.
    pub fn flagged(&self) -> bool {
        self.signals.iter().any(|s| *s != Signal::NotReviewed)
    }
}

/// The queue's coverage: what the counts are out of is said with them, and filtering the items
/// never changes them.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueueCounts {
    /// Documents in scope, and those of them with at least one piece read in the results kept.
    pub documents: u32,
    pub documents_read: u32,
    /// Results kept (a set-up of a job over a document), and their pieces.
    pub results: u32,
    pub pieces: u32,
    /// Pieces read; failed; not read yet; marked as reviewed.
    pub processed: u32,
    pub failed: u32,
    pub not_read: u32,
    pub reviewed: u32,
    /// Items flagged for a reason beyond not being reviewed, and those of them still open.
    pub flagged: u32,
    pub unresolved: u32,
    /// Items carrying each reason.
    pub truncated: u32,
    pub disagrees: u32,
    pub stale: u32,
    pub not_reviewed: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReviewQueue {
    pub counts: QueueCounts,
    /// In the order of the scope's results, each result's pieces in their order.
    pub items: Vec<QueueItem>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn every_signal_kind_is_listed_once_in_order() {
        let one_of_each = [
            Signal::Failed { error: None },
            Signal::NotRead,
            Signal::Truncated,
            Signal::Disagrees {
                with: Vec::new(),
                words: 0,
                fields: Vec::new(),
            },
            Signal::Stale {
                gone: false,
                unit: false,
            },
            Signal::NotReviewed,
        ];
        let kinds: Vec<String> = one_of_each
            .iter()
            .map(|s| {
                serde_json::to_value(s).unwrap()["kind"]
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(kinds, Signal::KINDS);
    }
}
