//! Planning a source's work: the pieces it is cut into for the job's task, and each piece ×
//! set-up recorded in the store before anything is sent, so totals and the resume screen are
//! right at once and a piece already answered under the same key is not sent again.

use std::collections::HashMap;
use std::sync::Arc;

use kalamu_core::{
    CacheKey, Job, PieceRef, PieceState, PieceStatus, Setup, SetupId, SourceFile, SourceId, Task,
    Timestamp,
};
use kalamu_media::OpenSource;

use crate::{Engine, EngineError};

/// The pieces a job has recorded, by set-up, source and index.
pub(crate) type Existing = HashMap<(SetupId, SourceId, u32), PieceState>;

/// A piece still to read for one set-up, recorded as pending, with the key its answer is
/// stored under.
pub(crate) type Work = (Setup, PieceState, CacheKey);

impl Engine {
    /// The pieces a source is cut into for the job's task. An error is the source's own (a
    /// correction of a file with no text), reported and skipped by the caller.
    pub(crate) async fn plan(
        &self,
        job: &Job,
        source: &SourceFile,
        opened: &Arc<OpenSource>,
        existing: &Existing,
    ) -> Result<Result<Vec<PieceRef>, String>, EngineError> {
        // A recording resumed keeps the pieces planned the first time: planning decodes the
        // whole file, and the cuts must not move under results already stored.
        let stored = stored_pieces(existing, job, source);
        let mut pieces = if job.task == Task::Analysis {
            // An analysis reads the whole document at once.
            vec![PieceRef::Whole]
        } else if job.task == Task::TextCorrection {
            // A correction reads the text there is: a PDF's pages, a text file's passages.
            let (opened, sample) = (opened.clone(), job.settings.sample_pages);
            let planned = tokio::task::spawn_blocking(move || opened.text_pieces(sample))
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?;
            match planned {
                Ok(pieces) => pieces,
                Err(e) => return Ok(Err(e.to_string())),
            }
        } else if stored.is_empty() {
            let (opened, split, sample, length) = (
                opened.clone(),
                source.split_spreads,
                job.settings.sample_pages,
                self.piece_length(job),
            );
            tokio::task::spawn_blocking(move || opened.pieces(split, sample, length))
                .await
                .map_err(|e| EngineError::Output(e.to_string()))?
        } else {
            stored
        };
        // A table of contents ends with the step that combines its pages.
        if job.task == Task::Contents && pieces.last() != Some(&PieceRef::Whole) {
            pieces.push(PieceRef::Whole);
        }
        Ok(Ok(pieces))
    }

    /// Records every piece × set-up of a source, all at once, and returns those still to read:
    /// a piece finished under the same key is skipped, one in a batch the provider still holds
    /// is left to be collected.
    pub(crate) fn record_pieces(
        &self,
        job: &Job,
        source: &SourceFile,
        pieces: &[PieceRef],
        existing: &Existing,
    ) -> Result<Vec<Work>, EngineError> {
        let mut work = Vec::new();
        // Written together, so a close part-way never leaves a recording's plan recorded in
        // part, which a resume would take for the whole plan.
        let mut rows = Vec::new();
        for setup in &job.setups {
            // Pages read several at a time are answered in another context than alone: their
            // keys say so, and an answer read one way is never reused for the other.
            let mut fixes = crate::output::preprocessing(source);
            let together = self.pages_per_call(job, source, setup);
            if together > 1 {
                fixes.push(format!("pages_per_call:{together}"));
            }
            for (index, piece) in pieces.iter().enumerate() {
                let index = index as u32;
                // The combining step's key depends on the pages' results: it is worked out
                // once they are read (`merge_pages`).
                if job.task == Task::Contents && *piece == PieceRef::Whole {
                    if !existing.contains_key(&(setup.id, source.id, index)) {
                        rows.push(pending(job, setup, source, index, piece, None));
                    }
                    continue;
                }
                let key = CacheKey::compute(
                    job.task,
                    &source.sha256,
                    piece,
                    setup,
                    &job.settings.languages,
                    &fixes,
                );
                let state = match existing.get(&(setup.id, source.id, index)) {
                    Some(s)
                        if s.status.is_finished()
                            && s.cache_key.as_deref() == Some(key.as_str()) =>
                    {
                        continue;
                    }
                    // In a batch the provider still holds: collected later.
                    Some(s) if s.status == PieceStatus::Submitted => continue,
                    Some(s) => PieceState {
                        status: PieceStatus::Pending,
                        cache_key: Some(key.0.clone()),
                        piece: piece.clone(),
                        ..s.clone()
                    },
                    None => pending(job, setup, source, index, piece, Some(key.0.clone())),
                };
                rows.push(state.clone());
                work.push((setup.clone(), state, key));
            }
        }
        self.store.upsert_pieces(&rows)?;
        Ok(work)
    }
}

/// A piece recorded for the first time, not yet sent.
fn pending(
    job: &Job,
    setup: &Setup,
    source: &SourceFile,
    index: u32,
    piece: &PieceRef,
    cache_key: Option<String>,
) -> PieceState {
    PieceState {
        job_id: job.id,
        setup_id: setup.id,
        source_id: source.id,
        index,
        piece: piece.clone(),
        status: PieceStatus::Pending,
        cache_key,
        error: None,
        attempts: 0,
        updated_at: Timestamp::now(),
    }
}

/// The pieces already recorded for a recording, in order; empty for documents, whose pages are
/// cheap to list again, for a recording not started yet, and for a plan that does not run
/// without a gap to the recording's end (recorded in part by an earlier version), which is
/// planned again.
fn stored_pieces(existing: &Existing, job: &Job, source: &SourceFile) -> Vec<PieceRef> {
    let audio = matches!(
        source.kind,
        kalamu_core::InputKind::Audio | kalamu_core::InputKind::Video
    );
    let Some(first) = job.setups.first() else {
        return Vec::new();
    };
    if !audio {
        return Vec::new();
    }
    let mut mine: Vec<&PieceState> = existing
        .values()
        .filter(|p| p.setup_id == first.id && p.source_id == source.id)
        .collect();
    mine.sort_by_key(|p| p.index);
    let gapless = mine.iter().enumerate().all(|(i, p)| p.index == i as u32);
    let to_the_end = match mine.last().map(|p| &p.piece) {
        Some(PieceRef::Segment { end_ms, .. }) => *end_ms >= source.duration_ms.unwrap_or(0),
        Some(_) => true,
        None => false,
    };
    if !gapless || !to_the_end {
        return Vec::new();
    }
    mine.into_iter().map(|p| p.piece.clone()).collect()
}
