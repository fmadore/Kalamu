//! Which results of a project's documents are searched or mapped, and where each of their
//! pieces stands in the vector index: current, prepared in part, prepared from a text since
//! corrected, or never prepared.
//!
//! A piece's stamp (what the word search stamps a passage with) changes whenever its correction
//! is saved, even when only marked reviewed. Its chunks are then checked against its text: when
//! the text is the same, they take the new stamp at no cost; otherwise they are left out until
//! the researcher prepares them again.

use std::collections::{HashMap, HashSet};

use kalamu_core::fingerprint::sha256_hex;
use kalamu_core::{
    ChunkStamp, CollectionId, Correction, Document, DocumentId, EmbeddingSpace, JobId, Passage,
    PassageKey, ProjectId, SetupId, SourceId, Task, TiedResult, TiedResults, VectorIndex,
};

use super::chunks::{self, TARGET};
use crate::passages::searched;
use crate::{ResultPolicy, Search, SearchError};

/// Which results are taken.
#[derive(Clone, Copy)]
pub(crate) enum Pick {
    /// Each document's results that read text, of `task` when given: the preferred one of a
    /// task where chosen, every one where not, or every one.
    Results {
        policy: ResultPolicy,
        task: Option<Task>,
    },
    /// One reading per document, for its point on the map: its preferred one, of the first task
    /// in [`READING_ORDER`] that has one, else that task's first result.
    OnePerDocument,
}

/// The order in which a document's one reading is chosen: a model's correction of the text
/// first, the text read from the page or the recording next, a translation last.
const READING_ORDER: [Task; 4] = [
    Task::TextCorrection,
    Task::TextExtraction,
    Task::AudioTranscription,
    Task::Translation,
];

pub(crate) struct Scope {
    pub documents: Vec<Document>,
    pub tied: TiedResults,
    pub chosen: Vec<TiedResult>,
    pub without_preference: HashSet<DocumentId>,
}

impl Scope {
    pub fn title(&self, id: DocumentId) -> &str {
        self.documents
            .iter()
            .find(|d| d.id == id)
            .map_or("", |d| d.title.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Its chunks are current; `complete` when every one is there.
    Current {
        complete: bool,
    },
    /// Prepared from a text since corrected.
    Changed,
    Missing,
}

/// A piece read, in a result taken.
pub(crate) struct Piece {
    pub key: PassageKey,
    pub stamp: String,
    pub reviewed: bool,
    pub status: Status,
}

pub(crate) struct States {
    pub pieces: Vec<Piece>,
    /// Pieces of the results taken that are not read.
    pub unread: u32,
    /// The results with some piece read.
    pub read: HashSet<(JobId, SetupId, SourceId)>,
    /// The text of each piece read while checking it.
    pub texts: HashMap<PassageKey, Passage>,
    /// What the index holds of each piece.
    pub rows: HashMap<PassageKey, Vec<ChunkStamp>>,
}

/// The hash a passage's chunks are stored under: its text and how it is cut.
pub(crate) fn content(text: &str, size: usize) -> String {
    sha256_hex(format!("{}:{size}:{text}", chunks::VERSION).as_bytes())
}

/// The length chunks are cut at for a model taking `max_chars`.
pub(crate) fn size(max_chars: usize) -> usize {
    TARGET.min(max_chars).max(1)
}

/// The text of a passage of text read (one part).
pub(crate) fn text_of(passage: &Passage) -> &str {
    passage.parts.first().map_or("", |p| p.text.as_str())
}

impl Search {
    pub(crate) fn vector_index(&self) -> Result<&dyn VectorIndex, SearchError> {
        self.vectors.as_deref().ok_or(SearchError::NoVectors)
    }

    /// The results of the project's documents in the collection and tag given that `pick`
    /// takes.
    pub(crate) fn meaning_scope(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
        tag: Option<&str>,
        pick: Pick,
    ) -> Result<Scope, SearchError> {
        self.check_project(project)?;
        let members = self.projects.members(project)?;
        let documents: Vec<Document> = self
            .projects
            .documents(project)?
            .into_iter()
            .filter(|d| collection.is_none_or(|c| members.contains(&(c, d.id))))
            .filter(|d| tag.is_none_or(|t| d.has_tag(t)))
            .collect();
        let ids: HashSet<DocumentId> = documents.iter().map(|d| d.id).collect();
        let tied = TiedResults::read(
            &self.projects.links(project)?,
            &self.projects.preferred(project)?,
            |d| ids.contains(&d),
            |id| self.jobs.job(id),
        )
        .map_err(SearchError::Jobs)?;
        let mut chosen = Vec::new();
        let mut without_preference = HashSet::new();
        match pick {
            Pick::Results { policy, task } => {
                for ((document, _), results) in tied.by_document_and_task() {
                    let results: Vec<&TiedResult> = results
                        .into_iter()
                        .filter(|r| r.task.yields_text() && task.is_none_or(|t| t == r.task))
                        .collect();
                    let has_preferred = results.iter().any(|c| c.preferred);
                    if policy == ResultPolicy::Preferred && has_preferred {
                        chosen.extend(results.into_iter().filter(|c| c.preferred).copied());
                    } else {
                        if !has_preferred && results.len() > 1 {
                            without_preference.insert(document);
                        }
                        chosen.extend(results.into_iter().copied());
                    }
                }
            }
            Pick::OnePerDocument => {
                for d in &documents {
                    let Some(task) = READING_ORDER
                        .iter()
                        .find(|t| tied.of(d.id, **t).next().is_some())
                    else {
                        continue;
                    };
                    let results: Vec<&TiedResult> = tied.of(d.id, *task).collect();
                    if let Some(p) = results.iter().find(|r| r.preferred) {
                        chosen.push(**p);
                    } else if let Some(first) = results.first() {
                        if results.len() > 1 {
                            without_preference.insert(d.id);
                        }
                        chosen.push(**first);
                    }
                }
            }
        }
        Ok(Scope {
            documents,
            tied,
            chosen,
            without_preference,
        })
    }

    /// Every job tied to the project's documents.
    pub(crate) fn project_jobs(&self, project: ProjectId) -> Result<Vec<JobId>, SearchError> {
        self.check_project(project)?;
        let mut jobs: Vec<JobId> = self
            .projects
            .links(project)?
            .into_iter()
            .map(|l| l.job_id)
            .collect();
        jobs.sort();
        jobs.dedup();
        Ok(jobs)
    }

    /// Where each piece read of the results taken stands in `space`, chunks cut at `size`.
    /// Chunks of a text unchanged take their piece's new stamp.
    pub(crate) fn states(
        &self,
        space: &EmbeddingSpace,
        size: usize,
        scope: &Scope,
        stop: &dyn Fn() -> bool,
    ) -> Result<States, SearchError> {
        let index = self.vector_index()?;
        let mut wanted: HashMap<JobId, HashSet<(SetupId, SourceId)>> = HashMap::new();
        for c in &scope.chosen {
            wanted
                .entry(c.job_id)
                .or_default()
                .insert((c.setup_id, c.source_id));
        }
        let mut jobs: Vec<&JobId> = wanted.keys().collect();
        jobs.sort();
        let mut out = States {
            pieces: Vec::new(),
            unread: 0,
            read: HashSet::new(),
            texts: HashMap::new(),
            rows: HashMap::new(),
        };
        for id in jobs {
            if stop() {
                return Err(SearchError::Stopped);
            }
            let Some(job) = scope.tied.job(*id) else {
                continue;
            };
            let results = &wanted[id];
            let pieces = self.jobs.pieces(job.id)?;
            let stored = self.stored(job)?;
            let mut rows: HashMap<PassageKey, Vec<ChunkStamp>> = HashMap::new();
            for row in index.stamps(space, job.id)? {
                rows.entry(row.key).or_default().push(row);
            }
            for piece in &pieces {
                if !results.contains(&(piece.setup_id, piece.source_id))
                    || !searched(job.task, &piece.piece)
                {
                    continue;
                }
                let Some(stamp) = stored.stamp(job.task, piece) else {
                    out.unread += 1;
                    continue;
                };
                let key = PassageKey {
                    job_id: piece.job_id,
                    setup_id: piece.setup_id,
                    source_id: piece.source_id,
                    index: piece.index,
                };
                let held = rows.remove(&key).unwrap_or_default();
                let complete = |held: &[ChunkStamp]| {
                    held.first()
                        .is_some_and(|r| held.len() as u32 == r.of.max(1))
                };
                let status = if held.is_empty() {
                    Status::Missing
                } else if held.iter().all(|r| r.stamp == stamp) {
                    Status::Current {
                        complete: complete(&held),
                    }
                } else {
                    // Saved again since: the same text keeps its chunks.
                    let Some(passage) = self.passage(job, piece, &stored, stamp.clone())? else {
                        out.unread += 1;
                        continue;
                    };
                    let now = content(text_of(&passage), size);
                    let status = if held.iter().all(|r| r.content == now) {
                        index.restamp(space, &key, &now, &stamp)?;
                        Status::Current {
                            complete: complete(&held),
                        }
                    } else {
                        Status::Changed
                    };
                    out.texts.insert(key, passage);
                    status
                };
                let correction = Correction::of(&stored.corrections, piece);
                out.read.insert((key.job_id, key.setup_id, key.source_id));
                out.pieces.push(Piece {
                    key,
                    stamp,
                    reviewed: correction.is_some_and(|c| c.reviewed),
                    status,
                });
                out.rows.insert(key, held);
            }
        }
        Ok(out)
    }
}
