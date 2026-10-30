//! Economy mode: a set-up's pieces gathered into batches for the provider's batch service, then
//! waited for and collected.
//!
//! A batch is recorded before it is sent and its pieces marked as submitted, so that closing the
//! app at any point loses nothing: a batch the provider accepted is collected when the job
//! resumes, and one never confirmed as sent (still `Sending`) puts its pieces back in the queue.
//! The one gap is a crash between the provider accepting a batch and its id being written,
//! which would send those pieces again; the window is one HTTP response.

mod collect;
mod send;

use kalamu_core::{
    ApiKey, BATCH_WINDOW_MS, BatchRecord, PieceState, PreparedInput, ProgressEvent, Setup,
    Timestamp,
};

use crate::{Engine, EngineError};

/// Pieces prepared for one set-up and not yet sent.
#[derive(Default)]
pub(crate) struct Gathered {
    pub(super) pieces: Vec<(PieceState, PreparedInput)>,
    pub(super) bytes: usize,
}

/// How batches ended for a job, for its final status.
#[derive(Default)]
pub(crate) struct BatchOutcome {
    pub failed: bool,
    /// Batches still with the provider: the job was paused while waiting.
    pub waiting: bool,
    /// The key or the account stopped working.
    pub fatal: Option<String>,
}

impl BatchOutcome {
    pub(crate) fn merge(&mut self, other: BatchOutcome) {
        self.failed |= other.failed;
        self.waiting |= other.waiting;
        if self.fatal.is_none() {
            self.fatal = other.fatal;
        }
    }
}

impl Engine {
    pub(super) fn api_key(&self, setup: &Setup) -> Result<ApiKey, EngineError> {
        self.keys
            .key(&setup.provider)
            .ok_or_else(|| EngineError::MissingKey(setup.provider.to_string()))
    }
}

pub(super) fn waiting(batch: &BatchRecord, finished: u32) -> ProgressEvent {
    ProgressEvent::BatchWaiting {
        job_id: batch.job_id,
        setup_id: batch.setup_id,
        pieces: batch.items.len() as u32,
        finished,
        submitted_at: batch.submitted_at,
        due_by: Timestamp(batch.submitted_at.0 + BATCH_WINDOW_MS),
    }
}

pub(super) fn stem(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".into())
}
