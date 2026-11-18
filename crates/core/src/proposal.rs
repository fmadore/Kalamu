//! What every request the researcher makes from the review goes through, whatever it asks (a
//! passage corrected, a region read again): recorded before its call and after it, then
//! accepted, discarded or undone. Each kind keeps its own request and answer; this is the part
//! of its record those steps change, alike for all of them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{RequestSetting, Timestamp};

/// Where a proposal stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProposalStatus {
    /// Recorded before the call; still waiting for the model.
    Asked,
    /// The model answered and the answer is offered.
    Answered,
    /// The call failed, or its answer was refused.
    Failed,
    /// The researcher stopped waiting, or Kalamu closed while it waited.
    Cancelled,
    /// Written into the researcher's corrections.
    Accepted,
    Discarded,
    /// Accepted, then put back as it was.
    Undone,
}

impl ProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asked => "asked",
            Self::Answered => "answered",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Accepted => "accepted",
            Self::Discarded => "discarded",
            Self::Undone => "undone",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Asked,
            Self::Answered,
            Self::Failed,
            Self::Cancelled,
            Self::Accepted,
            Self::Discarded,
            Self::Undone,
        ]
        .into_iter()
        .find(|v| v.as_str() == s)
    }
}

/// A request made from the review, as its record is kept: what its lifecycle changes.
pub trait Proposal {
    fn status(&self) -> ProposalStatus;
    /// The fields the steps every kind goes through change.
    fn lifecycle(&mut self) -> Lifecycle<'_>;
}

/// The fields of a proposal's record that the steps every kind goes through change; the
/// request and the answer stay each kind's own.
pub struct Lifecycle<'a> {
    pub status: &'a mut ProposalStatus,
    /// What the request carried besides the prompt.
    pub request: &'a mut Vec<RequestSetting>,
    pub cost_usd: &'a mut Option<f64>,
    /// The answer was one already stored.
    pub cached: &'a mut bool,
    pub error: &'a mut Option<String>,
    /// The text written: the answer, or the researcher's edit of it.
    pub accepted: &'a mut Option<String>,
    pub answered_at: &'a mut Option<Timestamp>,
    pub decided_at: &'a mut Option<Timestamp>,
}

impl Lifecycle<'_> {
    /// The call failed, or its answer was refused.
    pub fn fail(&mut self, error: String) {
        *self.status = ProposalStatus::Failed;
        *self.error = Some(error);
    }

    pub fn cancel(&mut self) {
        *self.status = ProposalStatus::Cancelled;
    }

    /// Written, as `text`, where the proposal goes.
    pub fn accept(&mut self, text: String, at: Timestamp) {
        *self.status = ProposalStatus::Accepted;
        *self.accepted = Some(text);
        *self.decided_at = Some(at);
    }

    pub fn discard(&mut self, at: Timestamp) {
        *self.status = ProposalStatus::Discarded;
        *self.decided_at = Some(at);
    }

    /// Accepted, then put back as it was.
    pub fn undo(&mut self, at: Timestamp) {
        *self.status = ProposalStatus::Undone;
        *self.decided_at = Some(at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_is_kept_as_the_interface_reads_it() -> Result<(), serde_json::Error> {
        for status in [
            ProposalStatus::Asked,
            ProposalStatus::Answered,
            ProposalStatus::Failed,
            ProposalStatus::Cancelled,
            ProposalStatus::Accepted,
            ProposalStatus::Discarded,
            ProposalStatus::Undone,
        ] {
            assert_eq!(ProposalStatus::parse(status.as_str()), Some(status));
            assert_eq!(serde_json::to_value(status)?, status.as_str());
        }
        assert_eq!(ProposalStatus::parse("waiting"), None);
        Ok(())
    }
}
