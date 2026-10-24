//! What the review and comparison views read: each piece's AI output with the user's
//! correction on top.

use scriptorium_core::{CacheKey, JobId, PieceRef, PieceStatus, SetupId, SourceId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// One piece as the review view shows it: the AI output, the correction on top, and status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PieceView {
    pub setup_id: SetupId,
    pub source_id: SourceId,
    pub index: u32,
    pub piece: PieceRef,
    pub status: PieceStatus,
    pub error: Option<String>,
    /// What the model returned, never edited.
    pub ai_text: Option<String>,
    /// The user's corrected text, if any.
    pub corrected_text: Option<String>,
    pub reviewed: bool,
    pub model_version: Option<String>,
    pub cost_usd: Option<f64>,
    /// Timed lines of a transcription, relative to the start of the piece.
    pub segments: Vec<scriptorium_core::Segment>,
    /// Keywords of a summary.
    pub keywords: Vec<String>,
}

impl Engine {
    /// Every piece of one source, for the review and comparison views.
    pub fn source_pieces(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<Vec<PieceView>, EngineError> {
        let corrections = self.store.corrections(job_id)?;
        let mut out = Vec::new();
        for state in self.store.pieces(job_id)? {
            if state.source_id != source_id {
                continue;
            }
            let result = match &state.cache_key {
                Some(k) if state.status.is_finished() => {
                    self.store.cached_result(&CacheKey(k.clone()))?
                }
                _ => None,
            };
            let correction = corrections.iter().find(|c| {
                c.setup_id == state.setup_id && c.source_id == source_id && c.index == state.index
            });
            out.push(PieceView {
                setup_id: state.setup_id,
                source_id,
                index: state.index,
                piece: state.piece.clone(),
                status: state.status,
                error: state.error.clone(),
                ai_text: result.as_ref().map(|r| r.output.text.clone()),
                corrected_text: correction.and_then(|c| c.text.clone()),
                reviewed: correction.is_some_and(|c| c.reviewed),
                model_version: result.as_ref().and_then(|r| r.output.model_version.clone()),
                cost_usd: result.as_ref().and_then(|r| r.cost_usd),
                keywords: result
                    .as_ref()
                    .map(|r| r.output.keywords.clone())
                    .unwrap_or_default(),
                segments: result.map(|r| r.output.segments).unwrap_or_default(),
            });
        }
        out.sort_by_key(|p| (p.index, p.setup_id));
        Ok(out)
    }
}
