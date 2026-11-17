//! The review manifest (#85): every item of a project's review queue in one scope, the sample
//! drawn at random included, with what identifies its source and result, why it is listed, and
//! what the researcher decided. Written as JSON and as a CSV beside it; read from what is
//! stored, so writing it calls no model.

use kalamu_core::{
    PieceRef, ProjectError, ProjectId, QueueCounts, QueueItem, ReviewQueue, SampleCounts, Signal,
    Table, Timestamp,
};
use serde::Serialize;

use crate::{Projects, QueueSelection, missing};

impl Projects {
    /// The manifest of `queue`, the review queue over `selection` as the engine gathered it.
    pub fn review_manifest(
        &self,
        project: ProjectId,
        selection: &QueueSelection,
        queue: &ReviewQueue,
    ) -> Result<ReviewManifest, ProjectError> {
        let named = self.store.project(project).map_err(missing)?;
        Ok(ReviewManifest::new(
            project,
            &named.name,
            selection.key(),
            queue,
            Timestamp::now(),
        ))
    }
}

/// The manifest of one scope of a project's queue.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewManifest {
    pub format: &'static str,
    pub version: u32,
    pub project: ProjectId,
    pub project_name: String,
    /// Which documents: `project`, `collection:<id>` or `documents:<digest>` (see
    /// [`crate::QueueSelection::key`]).
    pub scope: String,
    pub exported_at: Timestamp,
    /// The queue's coverage, over everything in scope whatever the interface showed.
    pub counts: QueueCounts,
    /// The sample drawn at random in this scope, if one was.
    pub sample: Option<SampleCounts>,
    pub items: Vec<ManifestRow>,
}

/// One item, as the manifest writes it.
#[derive(Debug, Clone, Serialize)]
pub struct ManifestRow {
    #[serde(flatten)]
    pub item: QueueItem,
    /// The place in the document as exports label it (`3`, `3 (left)`, `00:20:00`, `§2`);
    /// empty for a whole document or the result as a whole.
    pub place: String,
    /// The seed of the sample it was drawn into.
    pub sample_seed: Option<u32>,
}

/// The CSV's columns, in order.
const COLUMNS: [&str; 29] = [
    "document_id",
    "document",
    "source_id",
    "job_id",
    "job_name",
    "task",
    "setup_id",
    "setup_label",
    "model",
    "preferred",
    "piece",
    "place",
    "position",
    "revision",
    "answered_at",
    "reasons",
    "reason_details",
    "flagged",
    "processing",
    "sample",
    "sample_seed",
    "reviewed",
    "corrected",
    "decided_at",
    "open",
    "kept_aside",
    "kept_note",
    "kept_at",
    "listed_for",
];

impl ReviewManifest {
    pub fn new(
        project: ProjectId,
        project_name: &str,
        scope: String,
        queue: &ReviewQueue,
        exported_at: Timestamp,
    ) -> Self {
        let seed = queue.sample.map(|s| s.seed);
        Self {
            format: "kalamu-review-manifest",
            version: 1,
            project,
            project_name: project_name.to_string(),
            scope,
            exported_at,
            counts: queue.counts.clone(),
            sample: queue.sample,
            items: queue
                .items
                .iter()
                .map(|item| ManifestRow {
                    place: item.piece.as_ref().map(PieceRef::label).unwrap_or_default(),
                    sample_seed: if item.sample { seed } else { None },
                    item: item.clone(),
                })
                .collect(),
        }
    }

    pub fn json(&self) -> Result<String, ProjectError> {
        serde_json::to_string_pretty(self).map_err(|e| ProjectError::Export(e.to_string()))
    }

    /// One row per item, for a spreadsheet.
    pub fn table(&self) -> Table {
        let yes = |b: bool| if b { "yes" } else { "no" }.to_string();
        let when = |t: Option<Timestamp>| t.map(Timestamp::utc_date_time).unwrap_or_default();
        let rows = self
            .items
            .iter()
            .map(|row| {
                let i = &row.item;
                let kept = i.kept.as_ref();
                vec![
                    i.document_id.to_string(),
                    i.document.clone(),
                    i.source_id.to_string(),
                    i.job_id.to_string(),
                    i.job_name.clone(),
                    i.task.as_str().to_string(),
                    i.setup_id.to_string(),
                    i.setup_label.clone(),
                    i.model.clone(),
                    yes(i.preferred),
                    i.piece
                        .as_ref()
                        .map(PieceRef::descriptor)
                        .unwrap_or_else(|| "result".into()),
                    row.place.clone(),
                    i.index.map(|n| (n + 1).to_string()).unwrap_or_default(),
                    i.revision.clone().unwrap_or_default(),
                    when(i.answered_at),
                    i.signals.iter().map(kind).collect::<Vec<_>>().join("; "),
                    i.signals
                        .iter()
                        .filter_map(detail)
                        .collect::<Vec<_>>()
                        .join("; "),
                    yes(i.flagged()),
                    yes(i.processing),
                    yes(i.sample),
                    row.sample_seed.map(|s| s.to_string()).unwrap_or_default(),
                    yes(i.reviewed),
                    yes(i.corrected),
                    when(i.decided_at),
                    yes(i.open),
                    match kept {
                        Some(k) if k.current => "yes".into(),
                        Some(_) => "for an earlier answer".into(),
                        None => "no".into(),
                    },
                    kept.map(|k| k.note.clone()).unwrap_or_default(),
                    when(kept.map(|k| k.kept_at)),
                    listed_for(i).into(),
                ]
            })
            .collect();
        Table {
            headings: COLUMNS.iter().map(|c| (*c).to_string()).collect(),
            rows,
            cells: Default::default(),
        }
    }
}

/// A reason's kind, as [`Signal`] serialises it.
fn kind(signal: &Signal) -> String {
    match signal {
        Signal::Failed { .. } => "failed",
        Signal::NotRead => "not_read",
        Signal::Truncated { .. } => "truncated",
        Signal::Disagrees { .. } => "disagrees",
        Signal::Stale { .. } => "stale",
        Signal::NotReviewed => "not_reviewed",
    }
    .into()
}

/// What a reason found, where it says more than its kind.
fn detail(signal: &Signal) -> Option<String> {
    match signal {
        Signal::Failed { error: Some(e) } => Some(format!("failed: {e}")),
        Signal::Truncated { reason } => Some(format!(
            "truncated: {}",
            match reason {
                kalamu_core::CutReason::Limit => "limit",
                kalamu_core::CutReason::Filter => "filter",
                kalamu_core::CutReason::Other => "other",
            }
        )),
        Signal::Disagrees {
            with,
            words,
            fields,
        } => Some(if fields.is_empty() {
            format!("disagrees with {}: {words} words", with.join(", "))
        } else {
            format!("disagrees with {}: {}", with.join(", "), fields.join(", "))
        }),
        Signal::Stale { gone, unit } => Some(
            match (unit, gone) {
                (false, false) => "stale: input text corrected since",
                (false, true) => "stale: input job deleted",
                (true, false) => "stale: item's pages changed",
                (true, true) => "stale: item deleted",
            }
            .into(),
        ),
        Signal::Failed { error: None } | Signal::NotRead | Signal::NotReviewed => None,
    }
}

/// Why it is in the manifest: a processing problem, a flagged answer, an answer only not yet
/// reviewed, or a page drawn at random that nothing flags.
fn listed_for(item: &QueueItem) -> &'static str {
    if item.processing {
        "processing"
    } else if item.flagged() {
        "flagged"
    } else if item.sample {
        "sample"
    } else {
        "not_reviewed"
    }
}
