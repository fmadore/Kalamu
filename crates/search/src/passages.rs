//! Passages made from the store, and the index brought up to date with it.
//!
//! A passage is stamped with what it was made from: the stored answer's key, when the
//! researcher last corrected or reviewed the piece, and the names given to its speakers. The
//! stamps are read without reading any answer, so checking a project costs little, and only
//! the pieces whose stamp changed are read and indexed again.

use std::collections::{HashMap, HashSet};

use kalamu_core::analysis::answer_text;
use kalamu_core::contents::TITLE;
use kalamu_core::fingerprint::sha256_hex;
use kalamu_core::speakers::{rename_segments, text_as_left};
use kalamu_core::{
    Correction, FieldAnswer, FieldValue, Job, Passage, PassageKey, PassagePart, PieceRef,
    PieceState, SetupId, SourceId, SpeakerIndex, Task,
};

use crate::fold::folded;
use crate::{Search, SearchError};

/// Raised whenever what a passage holds, or how it is folded, changes: every passage is then
/// made again at the next search.
const VERSION: u32 = 2;

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
    speakers: SpeakerIndex,
}

impl Stored {
    fn stamp(&self, task: Task, piece: &PieceState) -> Option<String> {
        let key = piece.cache_key.as_deref()?;
        if !piece.status.is_finished() {
            return None;
        }
        let corrected = Correction::of(&self.corrections, piece).map_or(0, |c| c.updated_at.0);
        let names = self.speakers.of(piece.setup_id, piece.source_id);
        let names = if names.is_empty() {
            String::new()
        } else {
            sha256_hex(serde_json::to_string(names).unwrap_or_default().as_bytes())
        };
        Some(format!(
            "{VERSION}:{}:{key}:{corrected}:{names}",
            task.as_str()
        ))
    }
}

/// The answers as parts: each field's answer and reason, and each field of a table of
/// contents' entries under the entry's title.
fn answer_parts(answers: &[FieldAnswer], entry: Option<(&str, u32)>, out: &mut Vec<PassagePart>) {
    for answer in answers {
        let part = |text: String, reason: bool| PassagePart {
            field: Some(answer.key.clone()),
            entry: entry
                .map(|(title, _)| title)
                .filter(|t| !t.is_empty())
                .map(str::to_string),
            entry_number: entry.map(|(_, n)| n),
            reason,
            text,
            lines: Vec::new(),
        };
        match &answer.value {
            FieldValue::Entries(entries) => {
                for (i, e) in entries.iter().enumerate() {
                    let title = answer_text(e, TITLE);
                    answer_parts(e, Some((title.as_str(), i as u32 + 1)), out);
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
            let names = stored.speakers.of(piece.setup_id, piece.source_id);
            let corrected = correction.and_then(|c| c.text.as_deref());
            // A correction replaces the text; the model's timings no longer match it.
            let lines = match corrected {
                Some(_) => Vec::new(),
                None => rename_segments(&result.output.segments, names)
                    .into_iter()
                    .map(|s| (s.start_ms, s.text))
                    .collect(),
            };
            vec![PassagePart {
                field: None,
                entry: None,
                entry_number: None,
                reason: false,
                text: text_as_left(corrected, &result.output.text, names),
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

    /// What bringing the index up to date with one job for the results `wanted` (set-ups over
    /// sources) needs: passages of pieces no longer read are removed at once, and those whose
    /// stamp changed are listed, to be made again by [`Search::apply`].
    fn plan(&self, job: &Job, wanted: &HashSet<(SetupId, SourceId)>) -> Result<Plan, SearchError> {
        let pieces = self.jobs.pieces(job.id)?;
        let stored = Stored {
            corrections: self.jobs.corrections(job.id)?,
            speakers: SpeakerIndex::new(self.jobs.speaker_names(job.id)?),
        };
        let indexed: HashMap<PassageKey, String> = self.index.stamps(job.id)?.into_iter().collect();
        let mut current = HashMap::new();
        for (i, piece) in pieces.iter().enumerate() {
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
            current.insert(key, (i, stamp));
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

        let mut changed: Vec<(usize, String)> = current
            .iter()
            .filter(|(k, _)| wanted.contains(&(k.setup_id, k.source_id)))
            .filter_map(|(k, (i, stamp))| {
                let stamp = stamp.as_ref()?;
                (indexed.get(k) != Some(stamp)).then(|| (*i, stamp.clone()))
            })
            .collect();
        changed.sort_by_key(|(i, _)| {
            let p = &pieces[*i];
            (p.setup_id, p.source_id, p.index)
        });
        Ok(Plan {
            pieces,
            stored,
            changed,
        })
    }

    /// Makes again the passages a plan lists, in batches, telling `prepared` how many after
    /// each.
    fn apply(
        &self,
        job: &Job,
        plan: &Plan,
        stop: &dyn Fn() -> bool,
        prepared: &mut dyn FnMut(u32),
    ) -> Result<(), SearchError> {
        for batch in plan.changed.chunks(BATCH) {
            if stop() {
                return Err(SearchError::Stopped);
            }
            let mut passages = Vec::with_capacity(batch.len());
            for (i, stamp) in batch {
                let piece = &plan.pieces[*i];
                if let Some(passage) = self.passage(job, piece, &plan.stored, stamp.clone())? {
                    let text = index_text(&passage);
                    passages.push((passage, text));
                }
            }
            self.index.put_passages(&passages)?;
            prepared(batch.len() as u32);
        }
        Ok(())
    }

    /// Brings the index up to date with each job for its results wanted, telling `progress`
    /// how many pieces of how many have been prepared, after each batch and once before the
    /// first when there is any. Returns each job's pieces, in the same order.
    pub(crate) fn refresh(
        &self,
        jobs: &[(&Job, &HashSet<(SetupId, SourceId)>)],
        stop: &dyn Fn() -> bool,
        progress: &dyn Fn(u32, u32),
    ) -> Result<Vec<Vec<PieceState>>, SearchError> {
        let mut plans = Vec::with_capacity(jobs.len());
        for (job, wanted) in jobs {
            if stop() {
                return Err(SearchError::Stopped);
            }
            plans.push(self.plan(job, wanted)?);
        }
        let total: u32 = plans.iter().map(|p| p.changed.len() as u32).sum();
        let mut done = 0;
        if total > 0 {
            progress(done, total);
        }
        for ((job, _), plan) in jobs.iter().zip(&plans) {
            self.apply(job, plan, stop, &mut |n| {
                done += n;
                progress(done, total);
            })?;
        }
        Ok(plans.into_iter().map(|p| p.pieces).collect())
    }
}

/// What bringing the index up to date with one job needs.
struct Plan {
    pieces: Vec<PieceState>,
    stored: Stored,
    /// The pieces whose passage must be made again, by position in `pieces`, with their stamp.
    changed: Vec<(usize, String)>,
}
