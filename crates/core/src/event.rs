use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{JobId, JobStatus, PieceRef, PieceStatus, SetupId, SourceId, Timestamp};

/// What the engine reports while a job runs. Sent to the front end as Tauri events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum ProgressEvent {
    JobStarted {
        job_id: JobId,
        pieces_total: u32,
        pieces_finished: u32,
    },
    /// A source was split into pieces; its total is now known.
    SourcePrepared {
        job_id: JobId,
        source_id: SourceId,
        pieces: u32,
    },
    PieceUpdated {
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
        index: u32,
        piece: PieceRef,
        status: PieceStatus,
        error: Option<String>,
        /// The start of the text, for the live preview.
        preview: Option<String>,
    },
    /// Outputs for one source and set-up were written.
    SourceExported {
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
        files: Vec<String>,
    },
    /// A source could not be prepared or exported.
    SourceFailed {
        job_id: JobId,
        source_id: SourceId,
        error: String,
    },
    CostUpdated {
        job_id: JobId,
        spent_usd: f64,
    },
    /// A set-up's pieces are with the provider's batch service (economy mode).
    BatchWaiting {
        job_id: JobId,
        setup_id: SetupId,
        /// Pieces in the batch, and how many the provider has answered so far.
        pieces: u32,
        finished: u32,
        submitted_at: Timestamp,
        /// The latest the provider promises to answer by.
        due_by: Timestamp,
    },
    JobFinished {
        job_id: JobId,
        status: JobStatus,
    },
}

impl ProgressEvent {
    pub fn job_id(&self) -> JobId {
        match self {
            Self::JobStarted { job_id, .. }
            | Self::SourcePrepared { job_id, .. }
            | Self::PieceUpdated { job_id, .. }
            | Self::SourceExported { job_id, .. }
            | Self::SourceFailed { job_id, .. }
            | Self::CostUpdated { job_id, .. }
            | Self::BatchWaiting { job_id, .. }
            | Self::JobFinished { job_id, .. } => *job_id,
        }
    }
}
