//! What each result of a review queue needs (#85), from what is stored: no model is called.
//!
//! The project chooses the results ([`ReviewScope`]); here each of their pieces is looked at
//! once. A piece that failed or is not read yet is a processing problem, kept apart from the
//! answers. An answer is listed when it stopped at the model's limit, when another set-up of
//! its job read the piece differently (the words the comparison review highlights, or the
//! fields its sigla tell apart), when it read a frozen text that has changed since, and while
//! the researcher has not marked it reviewed. Marking it reviewed resolves it; nothing else
//! does, and agreement between models is never taken for a review.

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet};

use kalamu_core::{
    AnalysisSchema, Correction, DocumentId, FieldAnswer, FieldKind, FieldValue, Job, JobId,
    PieceRef, PieceState, PieceStatus, QueueCounts, QueueItem, ReviewQueue, ReviewScope,
    ReviewTarget, Setup, Signal, SourceId, StoreError, StoredResult, Task,
};

use crate::compare::{DiffKind, diff};
use crate::snapshot::Freshness;
use crate::{Engine, EngineError};

/// How many characters of a stored answer's fingerprint name its revision.
const REVISION: usize = 12;

/// What the queue reads of one job, loaded once however many of its results are in scope.
pub(crate) struct Loaded {
    pub job: Job,
    pub pieces: Vec<PieceState>,
    pub corrections: Vec<Correction>,
    /// The stored answers of its finished pieces, by fingerprint.
    pub results: HashMap<String, StoredResult>,
    /// Sources read from a frozen text, and how that text stands.
    /// With whether the input was a document unit's pages.
    pub freshness: HashMap<SourceId, (Freshness, bool)>,
}

impl Loaded {
    fn result(&self, piece: &PieceState) -> Option<&StoredResult> {
        if !piece.status.is_finished() {
            return None;
        }
        piece.cache_key.as_ref().and_then(|k| self.results.get(k))
    }
}

impl Engine {
    /// The review queue over the results a project chose: every piece that needs the
    /// researcher, with its reasons, and the coverage the counts are out of.
    pub fn review_queue(&self, scope: &ReviewScope) -> Result<ReviewQueue, EngineError> {
        let mut jobs: HashMap<JobId, Option<Loaded>> = HashMap::new();
        let mut queue = ReviewQueue {
            counts: QueueCounts {
                documents: scope.documents,
                ..QueueCounts::default()
            },
            items: Vec::new(),
        };
        let mut read: HashSet<DocumentId> = HashSet::new();
        for target in &scope.targets {
            let loaded = match jobs.entry(target.job_id) {
                Entry::Occupied(known) => known.into_mut(),
                Entry::Vacant(new) => new.insert(self.load(target.job_id)?),
            };
            // A job deleted meanwhile has nothing left to review.
            let Some(loaded) = loaded.as_ref() else {
                continue;
            };
            if assess(loaded, target, &mut queue) {
                read.insert(target.document_id);
            }
        }
        queue.counts.documents_read = read.len() as u32;
        let c = &mut queue.counts;
        for item in &queue.items {
            if item.flagged() {
                c.flagged += 1;
                c.unresolved += u32::from(item.open);
            }
            for signal in &item.signals {
                c.reasons.add(signal);
            }
        }
        Ok(queue)
    }

    fn load(&self, id: JobId) -> Result<Option<Loaded>, EngineError> {
        let job = match self.store.job(id) {
            Ok(job) => job,
            Err(StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let pieces = self.store.pieces(id)?;
        let mut results = HashMap::new();
        for piece in &pieces {
            if let Some(result) = self.store.finished_result(piece)? {
                results.insert(result.cache_key.clone(), result);
            }
        }
        let freshness = if job.sources.iter().any(|s| s.derived_from.is_some()) {
            self.derived_states(id)?
                .into_iter()
                .map(|d| (d.source_id, (d.freshness, d.from.unit.is_some())))
                .collect()
        } else {
            HashMap::new()
        };
        Ok(Some(Loaded {
            corrections: self.store.corrections(id)?,
            job,
            pieces,
            results,
            freshness,
        }))
    }
}

/// Adds one result's items and counts to the queue; whether any of its pieces was read.
pub(crate) fn assess(loaded: &Loaded, target: &ReviewTarget, queue: &mut ReviewQueue) -> bool {
    let job = &loaded.job;
    let Some(setup) = job.setups.iter().find(|s| s.id == target.setup_id) else {
        return false;
    };
    let mut pieces: Vec<&PieceState> = loaded
        .pieces
        .iter()
        .filter(|p| p.setup_id == setup.id && p.source_id == target.source_id)
        .collect();
    pieces.sort_by_key(|p| p.index);
    let counts = &mut queue.counts;
    counts.results += 1;
    counts.pieces += pieces.len() as u32;

    let item = |piece: Option<&PieceState>, signals: Vec<Signal>| {
        let result = piece.and_then(|p| loaded.result(p));
        let correction = piece.and_then(|p| Correction::of(&loaded.corrections, p));
        QueueItem {
            document_id: target.document_id,
            document: target.document.clone(),
            job_id: job.id,
            job_name: job.name.clone(),
            task: job.task,
            source_id: target.source_id,
            setup_id: setup.id,
            setup_label: setup.label.clone(),
            model: setup.model.clone(),
            preferred: target.preferred,
            piece: piece.map(|p| p.piece.clone()),
            index: piece.map(|p| p.index),
            revision: result.map(|r| r.cache_key.chars().take(REVISION).collect()),
            answered_at: result.map(|r| r.created_at),
            reviewed: false,
            corrected: correction.is_some_and(|c| c.text.is_some() || c.fields.is_some()),
            processing: signals.iter().any(Signal::is_processing),
            open: true,
            signals,
        }
    };

    if pieces.is_empty() {
        queue.items.push(item(None, vec![Signal::NotRead]));
        return false;
    }

    let stale = match loaded.freshness.get(&target.source_id) {
        Some(&(Freshness::Changed, unit)) => Some(Signal::Stale { gone: false, unit }),
        Some(&(Freshness::Gone, unit)) => Some(Signal::Stale { gone: true, unit }),
        Some((Freshness::Current, _)) | None => None,
    };
    // A result read in one piece carries its changed input itself; one of several pieces says
    // so once, for the whole.
    if let Some(signal) = &stale
        && pieces.len() > 1
    {
        queue.items.push(item(None, vec![signal.clone()]));
    }

    // A table of contents is reviewed as its combined entries: its pages are reviewed with them.
    let combined_reviewed = job.task == Task::Contents
        && pieces
            .iter()
            .find(|p| p.piece == PieceRef::Whole)
            .and_then(|p| Correction::of(&loaded.corrections, p))
            .is_some_and(|c| c.reviewed);

    let mut any_read = false;
    for piece in &pieces {
        let reviewable = job.task != Task::Contents || piece.piece == PieceRef::Whole;
        let reviewed = if reviewable {
            Correction::of(&loaded.corrections, piece).is_some_and(|c| c.reviewed)
        } else {
            combined_reviewed
        };
        let mut signals = Vec::new();
        match piece.status {
            PieceStatus::Failed => {
                queue.counts.failed += 1;
                signals.push(Signal::Failed {
                    error: piece.error.clone(),
                });
            }
            status if !status.is_finished() => {
                queue.counts.not_read += 1;
                signals.push(Signal::NotRead);
            }
            _ => {
                any_read = true;
                queue.counts.processed += 1;
                queue.counts.reviewed += u32::from(reviewed);
                if let Some(result) = loaded.result(piece) {
                    if result.output.truncated {
                        signals.push(Signal::Truncated);
                    }
                    if let Some(signal) = disagreement(loaded, setup, piece, result) {
                        signals.push(signal);
                    }
                }
                if pieces.len() == 1
                    && let Some(signal) = &stale
                {
                    signals.push(signal.clone());
                }
                if reviewable && !reviewed {
                    signals.push(Signal::NotReviewed);
                }
            }
        }
        if signals.is_empty() {
            continue;
        }
        let mut row = item(Some(piece), signals);
        row.reviewed = reviewed;
        row.open = row.processing
            || !reviewed
            || row
                .signals
                .iter()
                .any(|s| matches!(s, Signal::Stale { .. }));
        queue.items.push(row);
    }
    any_read
}

/// How the other set-ups of the job read the same piece, where they read it differently.
fn disagreement(
    loaded: &Loaded,
    setup: &Setup,
    piece: &PieceState,
    result: &StoredResult,
) -> Option<Signal> {
    let mut with = Vec::new();
    let mut words = 0;
    let mut fields: BTreeSet<String> = BTreeSet::new();
    for other in loaded.job.setups.iter().filter(|s| s.id != setup.id) {
        let Some(theirs) = loaded
            .pieces
            .iter()
            .find(|p| {
                p.setup_id == other.id && p.source_id == piece.source_id && p.index == piece.index
            })
            .and_then(|p| loaded.result(p))
        else {
            continue;
        };
        let (mine, their) = (&result.output, &theirs.output);
        let differs = if mine.fields.is_empty() && their.fields.is_empty() {
            let n = differing_words(&mine.text, &their.text);
            words = words.max(n);
            n > 0
        } else {
            let found = differing_fields(setup.prompt.schema.as_ref(), &mine.fields, &their.fields);
            let any = !found.is_empty();
            fields.extend(found);
            any
        };
        if differs {
            with.push(other.label.clone());
        }
    }
    (!with.is_empty()).then(|| Signal::Disagrees {
        with,
        words,
        fields: fields.into_iter().collect(),
    })
}

/// Words that only one of two texts has, as the comparison review highlights them.
pub(crate) fn differing_words(a: &str, b: &str) -> u32 {
    diff(a, b)
        .iter()
        .filter(|s| s.kind != DiffKind::Same)
        .map(|s| s.text.split_whitespace().count() as u32)
        .sum()
}

/// A field's values as a set, capitals and surrounding spaces set aside, as the analysis
/// review's sigla match them; a table of contents' entries by their titles. `None` for prose,
/// which two models never word alike.
fn values(kind: Option<&FieldKind>, value: &FieldValue) -> Option<BTreeSet<String>> {
    let norm = |s: &str| s.trim().to_lowercase();
    let set = |items: &[String]| {
        items
            .iter()
            .map(|s| norm(s))
            .filter(|s| !s.is_empty())
            .collect()
    };
    match (kind, value) {
        (Some(FieldKind::Text) | None, FieldValue::Text(_)) => None,
        (_, FieldValue::Text(t)) => Some(set(std::slice::from_ref(t))),
        (_, FieldValue::List(items)) => Some(set(items)),
        (_, FieldValue::Entries(entries)) => Some(
            entries
                .iter()
                .filter_map(|e| e.first().map(|a| norm(&a.value.joined())))
                .collect(),
        ),
    }
}

/// The labels of the fields two answers give differently, prose left aside.
pub(crate) fn differing_fields(
    schema: Option<&AnalysisSchema>,
    a: &[FieldAnswer],
    b: &[FieldAnswer],
) -> Vec<String> {
    let mut keys: Vec<&str> = Vec::new();
    for answer in a.iter().chain(b) {
        if !keys.contains(&answer.key.as_str()) {
            keys.push(&answer.key);
        }
    }
    let field = |key: &str| schema.and_then(|s| s.fields.iter().find(|f| f.key == key));
    let empty = FieldValue::List(Vec::new());
    keys.into_iter()
        .filter(|key| {
            let kind = field(key).map(|f| &f.kind);
            let of = |answers: &[FieldAnswer]| {
                let value = answers
                    .iter()
                    .find(|x| x.key == *key)
                    .map_or(&empty, |x| &x.value);
                values(kind, value)
            };
            match (of(a), of(b)) {
                (Some(x), Some(y)) => x != y,
                // One answered in prose what the other listed.
                (Some(x), None) | (None, Some(x)) => !x.is_empty(),
                (None, None) => false,
            }
        })
        .map(|key| field(key).map_or_else(|| key.to_string(), |f| f.label.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use kalamu_core::Field;

    use super::*;

    fn answer(key: &str, value: FieldValue) -> FieldAnswer {
        FieldAnswer {
            key: key.into(),
            value,
            reason: None,
        }
    }

    fn list(items: &[&str]) -> FieldValue {
        FieldValue::List(items.iter().map(|s| (*s).to_string()).collect())
    }

    #[test]
    fn texts_disagree_by_the_words_the_review_highlights() {
        assert_eq!(differing_words("le 3 mai 1957", "le 3 mai 1957"), 0);
        assert_eq!(differing_words("le 3 mai\n1957", "le 3 mai 1957"), 0);
        assert_eq!(differing_words("le 3 mai 1957", "le 5 mai 1957"), 2);
    }

    #[test]
    fn analyses_disagree_on_names_and_choices_not_on_prose() {
        let schema = AnalysisSchema {
            fields: vec![
                Field {
                    key: "summary".into(),
                    label: "Summary".into(),
                    instructions: String::new(),
                    kind: FieldKind::Text,
                    reason: false,
                },
                Field {
                    key: "people".into(),
                    label: "People".into(),
                    instructions: String::new(),
                    kind: FieldKind::List { max: None },
                    reason: false,
                },
                Field {
                    key: "tone".into(),
                    label: "Tone".into(),
                    instructions: String::new(),
                    kind: FieldKind::Choice {
                        values: vec!["positive".into(), "neutral".into()],
                    },
                    reason: false,
                },
            ],
        };
        let a = vec![
            answer("summary", FieldValue::Text("A letter to the imam.".into())),
            answer("people", list(&["Père Éric", "Imam Ouédraogo"])),
            answer("tone", FieldValue::Text("neutral".into())),
        ];
        // Other prose, the same names in another order and capitals, the same tone.
        let same = vec![
            answer("summary", FieldValue::Text("The father writes.".into())),
            answer("people", list(&["imam ouédraogo ", "Père Éric"])),
            answer("tone", FieldValue::Text("Neutral".into())),
        ];
        assert!(differing_fields(Some(&schema), &a, &same).is_empty());
        let other = vec![
            answer("summary", FieldValue::Text("A letter to the imam.".into())),
            answer("people", list(&["Père Éric"])),
            answer("tone", FieldValue::Text("positive".into())),
        ];
        assert_eq!(
            differing_fields(Some(&schema), &a, &other),
            vec!["People".to_string(), "Tone".to_string()]
        );
    }
}
