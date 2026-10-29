//! Economy mode: pieces sent together to a provider's batch service, which answers within a day
//! at about half the price. A batch outlives the app: its record is written before it is sent
//! and updated as it moves on, so a job closed mid-batch collects the results when resumed.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AnalysisSchema, JobId, ProcessOutput, ProviderError, ProviderId, SetupId, SourceId};
use crate::{RequestSetting, Task, Timestamp};

/// How long providers promise to take, at most; the interface shows the time it ends.
pub const BATCH_WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

/// One piece inside a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BatchItem {
    /// The id the provider returns the answer under, unique within the batch.
    pub id: String,
    pub source_id: SourceId,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BatchPhase {
    /// Recorded, about to be sent. A batch still here after a restart was never accepted (or
    /// its id was lost with the app): its pieces are sent again.
    Sending,
    /// Accepted by the provider, which is working on it.
    Waiting,
    /// Answers stored; nothing more to do.
    Collected,
    /// Cancelled, expired or refused as a whole; its unanswered pieces go back in the queue.
    Abandoned,
}

impl BatchPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sending => "sending",
            Self::Waiting => "waiting",
            Self::Collected => "collected",
            Self::Abandoned => "abandoned",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "sending" => Self::Sending,
            "waiting" => Self::Waiting,
            "collected" => Self::Collected,
            "abandoned" => Self::Abandoned,
            _ => return None,
        })
    }

    pub fn is_open(self) -> bool {
        matches!(self, Self::Sending | Self::Waiting)
    }
}

/// A batch as the store keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BatchRecord {
    /// Kalamu's own id, given before sending.
    pub id: String,
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub provider: ProviderId,
    /// The provider's id, once it has accepted the batch.
    pub remote_id: Option<String>,
    pub phase: BatchPhase,
    pub items: Vec<BatchItem>,
    /// The settings its requests carried, recorded when it was sent: the app may be updated
    /// before the answers are collected.
    #[serde(default)]
    pub request: Vec<RequestSetting>,
    pub submitted_at: Timestamp,
    pub updated_at: Timestamp,
}

impl BatchRecord {
    /// A fresh id for a batch about to be recorded.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// How much one batch may carry, in pieces and in bytes of prepared input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatchLimits {
    pub max_items: usize,
    pub max_bytes: usize,
}

impl Default for BatchLimits {
    fn default() -> Self {
        Self {
            max_items: 1_000,
            max_bytes: 100 * 1024 * 1024,
        }
    }
}

/// What the provider needs to read a batch's answers, which come back without their requests.
#[derive(Debug, Clone, Copy)]
pub struct BatchContext<'a> {
    pub task: Task,
    pub model: &'a str,
    pub analysis: Option<&'a AnalysisSchema>,
}

/// Where a batch stands at the provider.
#[derive(Debug)]
pub enum BatchProgress {
    /// Still working; `finished` of `total` answered so far, where the provider says.
    Working { finished: u32, total: u32 },
    /// Every answer is in, each under its item id; an item missing from the list was not
    /// answered and goes back in the queue.
    Ended(Vec<(String, Result<ProcessOutput, ProviderError>)>),
    /// Expired, cancelled or failed as a whole, with the provider's reason.
    Abandoned(String),
}
