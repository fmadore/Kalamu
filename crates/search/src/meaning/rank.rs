//! A question compared with every current chunk, documents listed by their best chunk; each
//! document's vector, the mean of its chunks'; and the documents most like another.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use kalamu_core::{
    DocumentId, EmbeddingSpace, JobId, PassageKey, ProjectId, SetupId, SourceId, TiedResult,
};

use super::anchor;
use super::cache::{Cache, dot};
use super::scope::{Pick, Piece, Scope, States, Status, size};
use super::{
    MAX_DOCUMENTS, MAX_NEIGHBOURS, MAX_PASSAGES, MeaningCoverage, MeaningDocument, MeaningPassage,
    MeaningRequest, MeaningView, Neighbour,
};
use crate::{ReviewFilter, Search, SearchError};

/// The mean of some vectors, each weighted, made of length 1 again; none without any weight.
pub fn document_vector<'a>(chunks: impl IntoIterator<Item = (&'a [f32], f32)>) -> Option<Vec<f32>> {
    let mut sum: Vec<f32> = Vec::new();
    for (v, weight) in chunks {
        if sum.is_empty() {
            sum = vec![0.0; v.len()];
        }
        if v.len() != sum.len() {
            continue;
        }
        for (s, x) in sum.iter_mut().zip(v) {
            *s += x * weight;
        }
    }
    let norm = sum.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 || !norm.is_finite() {
        return None;
    }
    sum.iter_mut().for_each(|x| *x /= norm);
    Some(sum)
}

/// Higher first; equal ones in the order given.
fn by_score(a: f32, b: f32) -> Ordering {
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

/// The pieces whose chunks are compared, by key.
fn usable(states: &States, review: ReviewFilter) -> HashMap<PassageKey, &Piece> {
    states
        .pieces
        .iter()
        .filter(|p| matches!(p.status, Status::Current { .. }))
        .filter(|p| match review {
            ReviewFilter::Any => true,
            ReviewFilter::Reviewed => p.reviewed,
            ReviewFilter::NotReviewed => !p.reviewed,
        })
        .map(|p| (p.key, p))
        .collect()
}

/// The rows of the cache whose piece is usable and whose stamp is its piece's.
fn current_rows<'a>(
    cache: &'a Cache,
    usable: &'a HashMap<PassageKey, &'a Piece>,
) -> impl Iterator<Item = usize> + 'a {
    cache.rows.iter().enumerate().filter_map(|(i, row)| {
        usable
            .get(&row.key)
            .is_some_and(|p| p.stamp == row.stamp)
            .then_some(i)
    })
}

type ResultKey = (JobId, SetupId, SourceId);
/// A chunk scored: its similarity, its row in the cache, and the result it was cut from.
type Scored<'a> = (f32, usize, &'a TiedResult);

fn results_by_key(scope: &Scope) -> HashMap<ResultKey, Vec<&TiedResult>> {
    let mut out: HashMap<ResultKey, Vec<&TiedResult>> = HashMap::new();
    for c in &scope.chosen {
        out.entry((c.job_id, c.setup_id, c.source_id))
            .or_default()
            .push(c);
    }
    out
}

/// What was covered, counted from the pieces' states and the documents compared.
fn coverage(scope: &Scope, states: &States, compared: &HashSet<DocumentId>) -> MeaningCoverage {
    let mut c = MeaningCoverage {
        documents: scope.documents.len() as u32,
        compared: compared.len() as u32,
        pieces: states.pieces.len() as u32,
        pieces_unread: states.unread,
        without_preference: scope.without_preference.len() as u32,
        ..MeaningCoverage::default()
    };
    for p in &states.pieces {
        match p.status {
            Status::Current { complete: true } => c.pieces_prepared += 1,
            Status::Current { complete: false } | Status::Missing => c.pieces_missing += 1,
            Status::Changed => c.pieces_changed += 1,
        }
    }
    let read: HashSet<DocumentId> = scope
        .chosen
        .iter()
        .filter(|r| states.read.contains(&(r.job_id, r.setup_id, r.source_id)))
        .map(|r| r.document_id)
        .collect();
    c.unread = c.documents.saturating_sub(read.len() as u32);
    c.not_prepared = read.difference(compared).count() as u32;
    c
}

/// Each document's vector from its chunks, with how many were averaged.
pub(crate) struct Documents {
    pub scope: Scope,
    pub states: States,
    pub vectors: Vec<(DocumentId, Vec<f32>, u32)>,
}

impl Search {
    /// The project's documents closest in meaning to the question whose vector is `query`,
    /// made by `space`'s model; chunks cut at most `max_chars` long.
    pub fn meaning_search(
        &self,
        project: ProjectId,
        request: &MeaningRequest,
        space: &EmbeddingSpace,
        max_chars: usize,
        query: &[f32],
        stop: &dyn Fn() -> bool,
    ) -> Result<MeaningView, SearchError> {
        if query.len() != space.dimensions as usize {
            return Err(SearchError::WrongLength);
        }
        let scope = self.meaning_scope(
            project,
            request.collection,
            request.tag.as_deref(),
            Pick::Results {
                policy: request.results,
                task: request.task,
            },
        )?;
        let mut states = self.states(space, size(max_chars), &scope, stop)?;
        let usable = usable(&states, request.review);
        let results = results_by_key(&scope);
        let jobs = self.project_jobs(project)?;
        let guard = self.cached(space, &jobs)?;
        let Some(cache) = guard.as_ref() else {
            return Err(SearchError::NoVectors);
        };

        // Every current chunk scored, gathered under each document that reads it.
        let mut by_document: HashMap<DocumentId, Vec<Scored>> = HashMap::new();
        for i in current_rows(cache, &usable) {
            let row = &cache.rows[i];
            let score = dot(query, cache.vector(i));
            let k = row.key;
            for r in results
                .get(&(k.job_id, k.setup_id, k.source_id))
                .into_iter()
                .flatten()
            {
                by_document
                    .entry(r.document_id)
                    .or_default()
                    .push((score, i, r));
            }
        }
        let compared: HashSet<DocumentId> = by_document.keys().copied().collect();
        let mut ranked: Vec<(DocumentId, Vec<Scored>)> = by_document.into_iter().collect();
        for (_, chunks) in &mut ranked {
            chunks.sort_by(|a, b| by_score(a.0, b.0).then(a.1.cmp(&b.1)));
        }
        ranked.sort_by(|a, b| {
            by_score(a.1[0].0, b.1[0].0).then_with(|| scope.title(a.0).cmp(scope.title(b.0)))
        });
        ranked.truncate(MAX_DOCUMENTS);

        // Each document's best passages, none overlapping another of the same piece.
        let mut picked: Vec<(DocumentId, f32, Vec<Scored>)> = Vec::new();
        for (document, chunks) in ranked {
            let best = chunks[0].0;
            let mut kept: Vec<Scored> = Vec::new();
            for c in chunks {
                let row = &cache.rows[c.1];
                let overlaps = kept.iter().any(|k| {
                    let other = &cache.rows[k.1];
                    other.key == row.key && other.start < row.end && row.start < other.end
                });
                if !overlaps {
                    kept.push(c);
                }
                if kept.len() == MAX_PASSAGES {
                    break;
                }
            }
            picked.push((document, best, kept));
        }
        let wanted: HashSet<PassageKey> = picked
            .iter()
            .flat_map(|(_, _, kept)| kept.iter().map(|k| cache.rows[k.1].key))
            .filter(|k| !states.texts.contains_key(k))
            .collect();
        let read = self.read_passages(&scope, &wanted, stop)?;
        states.texts.extend(read);

        let mut documents = Vec::with_capacity(picked.len());
        for (document_id, similarity, kept) in picked {
            let mut passages = Vec::new();
            for (score, i, r) in kept {
                let row = &cache.rows[i];
                let Some(passage) = states.texts.get(&row.key) else {
                    continue;
                };
                let job = scope.tied.job(r.job_id);
                let setup = job.and_then(|j| j.setups.iter().find(|s| s.id == r.setup_id));
                passages.push(MeaningPassage {
                    job_id: r.job_id,
                    job_name: job.map(|j| j.name.clone()).unwrap_or_default(),
                    setup_id: r.setup_id,
                    setup_label: setup.map(|s| s.label.clone()).unwrap_or_default(),
                    source_id: r.source_id,
                    task: r.task,
                    preferred: r.preferred,
                    similarity: score,
                    hit: anchor::hit(passage, row.start as usize, row.end as usize),
                });
            }
            documents.push(MeaningDocument {
                document_id,
                title: scope.title(document_id).to_string(),
                similarity,
                passages,
            });
        }
        let coverage = coverage(&scope, &states, &compared);
        Ok(MeaningView {
            space: space.clone(),
            documents,
            compared: compared.len() as u32,
            coverage,
        })
    }

    /// Each document's vector, from one reading of it: the mean of its current chunks'
    /// vectors, weighted by their length.
    pub(crate) fn document_vectors(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        max_chars: usize,
        stop: &dyn Fn() -> bool,
    ) -> Result<Documents, SearchError> {
        let scope = self.meaning_scope(project, None, None, Pick::OnePerDocument)?;
        let states = self.states(space, size(max_chars), &scope, stop)?;
        let usable = usable(&states, ReviewFilter::Any);
        let results = results_by_key(&scope);
        let jobs = self.project_jobs(project)?;
        let guard = self.cached(space, &jobs)?;
        let Some(cache) = guard.as_ref() else {
            return Err(SearchError::NoVectors);
        };
        let mut rows: HashMap<DocumentId, Vec<usize>> = HashMap::new();
        for i in current_rows(cache, &usable) {
            let k = cache.rows[i].key;
            for r in results
                .get(&(k.job_id, k.setup_id, k.source_id))
                .into_iter()
                .flatten()
            {
                rows.entry(r.document_id).or_default().push(i);
            }
        }
        let mut vectors = Vec::new();
        for d in &scope.documents {
            let Some(mine) = rows.get(&d.id) else {
                continue;
            };
            let chunks = mine.iter().map(|&i| {
                let row = &cache.rows[i];
                (
                    cache.vector(i),
                    row.end.saturating_sub(row.start).max(1) as f32,
                )
            });
            if let Some(v) = document_vector(chunks) {
                vectors.push((d.id, v, mine.len() as u32));
            }
        }
        drop(guard);
        Ok(Documents {
            scope,
            states,
            vectors,
        })
    }

    /// The documents most like `document`, by their documents' vectors, closest first.
    pub fn meaning_neighbours(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        max_chars: usize,
        document: DocumentId,
        stop: &dyn Fn() -> bool,
    ) -> Result<Vec<Neighbour>, SearchError> {
        let docs = self.document_vectors(project, space, max_chars, stop)?;
        let Some((_, mine, _)) = docs.vectors.iter().find(|(d, _, _)| *d == document) else {
            return Ok(Vec::new());
        };
        let mut out: Vec<Neighbour> = docs
            .vectors
            .iter()
            .filter(|(d, _, _)| *d != document)
            .map(|(d, v, _)| Neighbour {
                document_id: *d,
                title: docs.scope.title(*d).to_string(),
                similarity: dot(mine, v),
            })
            .collect();
        out.sort_by(|a, b| by_score(a.similarity, b.similarity).then(a.title.cmp(&b.title)));
        out.truncate(MAX_NEIGHBOURS);
        Ok(out)
    }
}
