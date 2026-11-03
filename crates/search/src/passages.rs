//! Passages made from the store, and the index brought up to date with it.
//!
//! A passage is stamped with what it was made from: the stored answer's key, when the
//! researcher last corrected or reviewed the piece, and the names given to its speakers. The
//! stamps are read without reading any answer, so checking a project costs little, and only
//! the pieces whose stamp changed are read and indexed again.

use std::collections::{BTreeMap, HashMap, HashSet};

use kalamu_core::analysis::answer_text;
use kalamu_core::contents::TITLE;
use kalamu_core::fingerprint::sha256_hex;
use kalamu_core::speakers::{rename_segments, rename_text};
use kalamu_core::{
    Correction, FieldAnswer, FieldValue, Job, Passage, PassageKey, PassagePart, PieceRef,
    PieceState, SetupId, SourceId, Task,
};

use crate::fold::folded;
use crate::{Search, SearchError};

/// Raised whenever what a passage holds, or how it is folded, changes: every passage is then
/// made again at the next search.
const VERSION: u32 = 1;

/// How many passages are indexed together, between which a newer search can stop this one and
/// the review can use the store.
const BATCH: usize = 64;

/// Stands between the parts of a passage in the index, so that no phrase runs from one answer
/// into the next. A private-use character, which no query holds.
const BETWEEN: char = '\u{E000}';

/// Whether a piece is searched: a table of contents is searched in its combined entries, every
/// other task in each of its pieces.
pub(crate) fn searched(task: Task, piece: &PieceRef) -> bool {
    task != Task::Contents || *piece == PieceRef::Whole
}

/// What one job holds that its passages are made from.
struct Stored {
    corrections: Vec<Correction>,
    names: HashMap<(SetupId, SourceId), BTreeMap<String, String>>,
}

impl Stored {
    fn stamp(&self, task: Task, piece: &PieceState) -> Option<String> {
        let key = piece.cache_key.as_deref()?;
        if !piece.status.is_finished() {
            return None;
        }
        let corrected = Correction::of(&self.corrections, piece).map_or(0, |c| c.updated_at.0);
        let names = match self.names.get(&(piece.setup_id, piece.source_id)) {
            Some(names) if !names.is_empty() => {
                sha256_hex(serde_json::to_string(names).unwrap_or_default().as_bytes())
            }
            _ => String::new(),
        };
        Some(format!(
            "{VERSION}:{}:{key}:{corrected}:{names}",
            task.as_str()
        ))
    }
}

/// The answers as parts: each field's answer and reason, and each field of a table of
/// contents' entries under the entry's title.
fn answer_parts(answers: &[FieldAnswer], entry: Option<&str>, out: &mut Vec<PassagePart>) {
    for answer in answers {
        let part = |text: String, reason: bool| PassagePart {
            field: Some(answer.key.clone()),
            entry: entry.map(str::to_string),
            reason,
            text,
            lines: Vec::new(),
        };
        match &answer.value {
            FieldValue::Entries(entries) => {
                for e in entries {
                    let title = answer_text(e, TITLE);
                    answer_parts(e, (!title.is_empty()).then_some(title.as_str()), out);
                }
            }
            value => {
                let text = value.joined();
                if !text.trim().is_empty() {
                    out.push(part(text, false));
                }
            }
        }
        if let Some(reason) = answer.reason.as_ref().filter(|r| !r.trim().is_empty()) {
            out.push(part(reason.clone(), true));
        }
    }
}

/// A passage's text as the index holds it.
pub(crate) fn index_text(passage: &Passage) -> String {
    passage
        .parts
        .iter()
        .map(|p| folded(&p.text))
        .collect::<Vec<_>>()
        .join(&BETWEEN.to_string())
}

impl Search {
    /// The piece as the review shows it; none when its answer is no longer stored.
    fn passage(
        &self,
        job: &Job,
        piece: &PieceState,
        stored: &Stored,
        stamp: String,
    ) -> Result<Option<Passage>, SearchError> {
        let Some(result) = self.jobs.finished_result(piece)? else {
            return Ok(None);
        };
        let correction = Correction::of(&stored.corrections, piece);
        let parts = if job.task.yields_text() {
            let names = stored
                .names
                .get(&(piece.setup_id, piece.source_id))
                .cloned()
                .unwrap_or_default();
            let corrected = correction.and_then(|c| c.text.as_deref());
            // A correction replaces the text; the model's timings no longer match it.
            let lines = match corrected {
                Some(_) => Vec::new(),
                None => rename_segments(&result.output.segments, &names)
                    .into_iter()
                    .map(|s| (s.start_ms, s.text))
                    .collect(),
            };
            vec![PassagePart {
                field: None,
                entry: None,
                reason: false,
                text: rename_text(corrected.unwrap_or(&result.output.text), &names),
                lines,
            }]
        } else {
            let answers = correction
                .and_then(|c| c.fields.clone())
                .unwrap_or(result.output.fields);
            let mut parts = Vec::new();
            answer_parts(&answers, None, &mut parts);
            parts
        };
        Ok(Some(Passage {
            key: PassageKey {
                job_id: piece.job_id,
                setup_id: piece.setup_id,
                source_id: piece.source_id,
                index: piece.index,
            },
            stamp,
            piece: piece.piece.clone(),
            reviewed: correction.is_some_and(|c| c.reviewed),
            corrected: correction.is_some_and(|c| c.text.is_some() || c.fields.is_some()),
            parts,
        }))
    }

    /// Brings the index up to date with one job for the results `wanted` (set-ups over
    /// sources): passages made again where their stamp changed, and those of pieces no longer
    /// read removed. Returns the job's pieces.
    pub(crate) fn refresh(
        &self,
        job: &Job,
        wanted: &HashSet<(SetupId, SourceId)>,
        stop: &dyn Fn() -> bool,
    ) -> Result<Vec<PieceState>, SearchError> {
        let pieces = self.jobs.pieces(job.id)?;
        let stored = Stored {
            corrections: self.jobs.corrections(job.id)?,
            names: self
                .jobs
                .speaker_names(job.id)?
                .into_iter()
                .map(|n| ((n.setup_id, n.source_id), n.names))
                .collect(),
        };
        let indexed: HashMap<PassageKey, String> = self.index.stamps(job.id)?.into_iter().collect();
        let mut current = HashMap::new();
        for piece in &pieces {
            let key = PassageKey {
                job_id: piece.job_id,
                setup_id: piece.setup_id,
                source_id: piece.source_id,
                index: piece.index,
            };
            let stamp = if searched(job.task, &piece.piece) {
                stored.stamp(job.task, piece)
            } else {
                None
            };
            current.insert(key, (piece, stamp));
        }

        // Passages of pieces gone, failed again or no longer searched; those of results not
        // wanted now are kept for another search.
        let gone: Vec<PassageKey> = indexed
            .keys()
            .filter(|k| current.get(k).is_none_or(|(_, stamp)| stamp.is_none()))
            .copied()
            .collect();
        if !gone.is_empty() {
            self.index.remove_passages(&gone)?;
        }

        let mut changed: Vec<(&PieceState, String)> = current
            .iter()
            .filter(|(k, _)| wanted.contains(&(k.setup_id, k.source_id)))
            .filter_map(|(k, (piece, stamp))| {
                let stamp = stamp.as_ref()?;
                (indexed.get(k) != Some(stamp)).then(|| (*piece, stamp.clone()))
            })
            .collect();
        changed.sort_by_key(|(p, _)| (p.setup_id, p.source_id, p.index));
        for batch in changed.chunks(BATCH) {
            if stop() {
                return Err(SearchError::Stopped);
            }
            let mut passages = Vec::with_capacity(batch.len());
            for (piece, stamp) in batch {
                if let Some(passage) = self.passage(job, piece, &stored, stamp.clone())? {
                    let text = index_text(&passage);
                    passages.push((passage, text));
                }
            }
            self.index.put_passages(&passages)?;
        }
        Ok(pieces)
    }
}
