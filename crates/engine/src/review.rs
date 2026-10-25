//! What the review and comparison views read: each piece's AI output with the user's
//! correction on top.

use std::collections::BTreeMap;

use scriptorium_core::speakers::{labels, rename_segments, rename_text};
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
    /// What the model returned, never edited, with the researcher's speaker names applied.
    pub ai_text: Option<String>,
    /// The user's corrected text, if any, with speaker names applied.
    pub corrected_text: Option<String>,
    pub reviewed: bool,
    pub model_version: Option<String>,
    pub cost_usd: Option<f64>,
    /// Timed lines of a transcription, relative to the start of the piece.
    pub segments: Vec<scriptorium_core::Segment>,
    /// Keywords of a summary.
    pub keywords: Vec<String>,
    /// The model's speaker labels in this piece, when the set-up asked for them.
    pub speakers: Vec<String>,
    /// The names the researcher gave this set-up's speakers in this recording.
    pub speaker_names: BTreeMap<String, String>,
}

impl Engine {
    /// Every piece of one source, for the review and comparison views.
    pub fn source_pieces(
        &self,
        job_id: JobId,
        source_id: SourceId,
    ) -> Result<Vec<PieceView>, EngineError> {
        let job = self.store.job(job_id)?;
        let corrections = self.store.corrections(job_id)?;
        let names = self.store.speaker_names(job_id)?;
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
            let diarized = job
                .setups
                .iter()
                .any(|s| s.id == state.setup_id && s.options.diarize);
            let speaker_names = names
                .iter()
                .find(|n| n.setup_id == state.setup_id && n.source_id == source_id)
                .map(|n| n.names.clone())
                .unwrap_or_default();
            let speakers = match &result {
                Some(r) if diarized => labels(&r.output.text, &r.output.segments),
                _ => Vec::new(),
            };
            out.push(PieceView {
                setup_id: state.setup_id,
                source_id,
                index: state.index,
                piece: state.piece.clone(),
                status: state.status,
                error: state.error.clone(),
                ai_text: result
                    .as_ref()
                    .map(|r| rename_text(&r.output.text, &speaker_names)),
                corrected_text: correction
                    .and_then(|c| c.text.as_deref())
                    .map(|t| rename_text(t, &speaker_names)),
                reviewed: correction.is_some_and(|c| c.reviewed),
                model_version: result.as_ref().and_then(|r| r.output.model_version.clone()),
                cost_usd: result.as_ref().and_then(|r| r.cost_usd),
                keywords: result
                    .as_ref()
                    .map(|r| r.output.keywords.clone())
                    .unwrap_or_default(),
                segments: result
                    .map(|r| rename_segments(&r.output.segments, &speaker_names))
                    .unwrap_or_default(),
                speakers,
                speaker_names,
            });
        }
        out.sort_by_key(|p| (p.index, p.setup_id));
        Ok(out)
    }
}
