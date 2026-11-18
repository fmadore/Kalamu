//! What preparing a project's text for a model needs: the chunks of every piece not yet
//! prepared, prepared in part or prepared from a text since corrected, without those already
//! stored. Read before anything is sent, for the estimate; read again when a run starts, so a
//! run stopped part-way goes on from where it got to without embedding anything twice.

use std::collections::{HashMap, HashSet};

use kalamu_core::{ChunkVector, EmbeddingSpace, Passage, PassageKey, ProjectId};

use super::chunks::{TARGET, cut};
use super::scope::{Pick, Scope, Status, content, size, text_of};
use super::{PendingChunk, Preparation, PreparationCounts, estimated_tokens};
use crate::{ResultPolicy, Search, SearchError};

impl Search {
    /// The text of each piece `keys` names, as the review shows it.
    pub(crate) fn read_passages(
        &self,
        scope: &Scope,
        keys: &HashSet<PassageKey>,
        stop: &dyn Fn() -> bool,
    ) -> Result<HashMap<PassageKey, Passage>, SearchError> {
        let mut jobs: Vec<_> = keys.iter().map(|k| k.job_id).collect();
        jobs.sort();
        jobs.dedup();
        let mut out = HashMap::new();
        for id in jobs {
            if stop() {
                return Err(SearchError::Stopped);
            }
            let Some(job) = scope.tied.job(id) else {
                continue;
            };
            let stored = self.stored(job)?;
            for piece in self.jobs.pieces(id)? {
                let key = PassageKey {
                    job_id: piece.job_id,
                    setup_id: piece.setup_id,
                    source_id: piece.source_id,
                    index: piece.index,
                };
                if !keys.contains(&key) {
                    continue;
                }
                let Some(stamp) = stored.stamp(job.task, &piece) else {
                    continue;
                };
                if let Some(passage) = self.passage(job, &piece, &stored, stamp)? {
                    out.insert(key, passage);
                }
            }
        }
        Ok(out)
    }

    /// What preparing the project's text for `space` would embed, chunks of at most
    /// `max_chars` characters: each document's preferred reading of each task that reads text,
    /// or every reading where none is preferred. Nothing is sent.
    pub fn meaning_preparation(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        max_chars: usize,
        stop: &dyn Fn() -> bool,
    ) -> Result<Preparation, SearchError> {
        let scope = self.meaning_scope(
            project,
            None,
            None,
            Pick::Results {
                policy: ResultPolicy::Preferred,
                task: None,
            },
        )?;
        let size = size(max_chars);
        let mut states = self.states(space, size, &scope, stop)?;
        let mut counts = PreparationCounts {
            pieces: states.pieces.len() as u32,
            ..PreparationCounts::default()
        };
        let mut to_cut = Vec::new();
        for p in &states.pieces {
            match p.status {
                Status::Current { complete: true } => counts.prepared += 1,
                Status::Current { complete: false } | Status::Missing => counts.missing += 1,
                Status::Changed => counts.changed += 1,
            }
            if p.status != (Status::Current { complete: true }) {
                to_cut.push((p.key, p.stamp.clone()));
            }
        }
        let unread: HashSet<PassageKey> = to_cut
            .iter()
            .map(|(k, _)| *k)
            .filter(|k| !states.texts.contains_key(k))
            .collect();
        let read = self.read_passages(&scope, &unread, stop)?;
        states.texts.extend(read);

        let mut out = Preparation::default();
        for (key, stamp) in to_cut {
            let Some(passage) = states.texts.get(&key) else {
                continue;
            };
            let text = text_of(passage);
            let content = content(text, size);
            let held: HashSet<u32> = states
                .rows
                .get(&key)
                .into_iter()
                .flatten()
                .filter(|r| r.content == content)
                .map(|r| r.chunk)
                .collect();
            let pieces = cut(text, TARGET, max_chars);
            counts.skipped += pieces.skipped;
            let of = pieces.chunks.len() as u32;
            if of == 0 {
                if held.is_empty() {
                    out.empty.push(ChunkVector {
                        key,
                        chunk: 0,
                        of: 0,
                        stamp,
                        content,
                        start: 0,
                        end: 0,
                        vector: Vec::new(),
                    });
                }
                continue;
            }
            for (i, c) in pieces.chunks.into_iter().enumerate() {
                let chunk = i as u32;
                if held.contains(&chunk) {
                    continue;
                }
                counts.chunks += 1;
                counts.chars += c.text.chars().count() as u64;
                counts.tokens += estimated_tokens(&c.text);
                out.pending.push(PendingChunk {
                    key,
                    chunk,
                    of,
                    stamp: stamp.clone(),
                    content: content.clone(),
                    start: c.start as u32,
                    end: c.end as u32,
                    text: c.text,
                });
            }
        }
        out.pending.sort_by_key(|c| {
            (
                c.key.job_id,
                c.key.setup_id,
                c.key.source_id,
                c.key.index,
                c.chunk,
            )
        });
        out.counts = counts;
        Ok(out)
    }
}
