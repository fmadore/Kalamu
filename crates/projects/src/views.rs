//! What the project screens show: the documents with what has been read from each, and one
//! document with all its results. Read from the job store as it stands, never stored twice.

use std::collections::HashMap;

use kalamu_core::{
    CacheKey, Collection, CollectionId, Correction, Document, DocumentId, DocumentLink, Job, JobId,
    JobStatus, PieceState, PieceStatus, Project, ProjectError, ProjectId, ProviderId, SetupId,
    SourceId, Task, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::documents::is_missing;
use crate::{Projects, missing};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectView {
    pub project: Project,
    pub collections: Vec<CollectionView>,
    pub documents: Vec<DocumentRow>,
    /// The jobs that read its documents, newest first: the project's activity.
    pub jobs: Vec<JobId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CollectionView {
    pub collection: Collection,
    pub documents: u32,
}

/// A document's line in the project's table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentRow {
    pub document: Document,
    pub collections: Vec<CollectionId>,
    /// What has been read from it, task by task.
    pub tasks: Vec<TaskBrief>,
    /// The file is no longer where it was added.
    pub missing: bool,
    /// Other documents of the project with the same content.
    pub same_content: u32,
    /// When a job that read it was last started.
    pub last_run: Option<Timestamp>,
}

/// One task's results for a document, counted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TaskBrief {
    pub task: Task,
    /// Set-ups of the jobs tied to the document: each is one result.
    pub results: u32,
    /// Results with every piece read.
    pub finished: u32,
    /// Results with every piece reviewed by the researcher.
    pub reviewed: u32,
    /// The researcher chose one of them as preferred.
    pub preferred: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentBrief {
    pub id: DocumentId,
    pub title: String,
    pub path: String,
}

impl DocumentBrief {
    pub(crate) fn of(d: &Document) -> Self {
        Self {
            id: d.id,
            title: d.title.clone(),
            path: d.source.path.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentDetail {
    pub document: Document,
    pub collections: Vec<CollectionId>,
    pub missing: bool,
    pub same_content: Vec<DocumentBrief>,
    /// Every result, newest job first.
    pub results: Vec<DocumentResult>,
}

/// One set-up of one job over the document: a result, with where it stands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentResult {
    pub job_id: JobId,
    pub job_name: String,
    pub job_status: JobStatus,
    pub task: Task,
    pub created_at: Timestamp,
    /// The job's source that is this document, to open its review.
    pub source_id: SourceId,
    pub setup_id: SetupId,
    pub setup_label: String,
    pub provider: ProviderId,
    pub model: String,
    pub pieces: u32,
    pub finished: u32,
    pub failed: u32,
    /// Pieces the researcher marked as reviewed, and those corrected.
    pub reviewed: u32,
    pub corrected: u32,
    /// What its own calls cost (not results reused from the cache).
    pub cost_usd: Option<f64>,
    pub preferred: bool,
}

/// The jobs a project's links name, each read once, with its pieces and corrections. A job
/// deleted meanwhile is left out.
struct Loaded {
    jobs: HashMap<JobId, (Job, Vec<PieceState>, Vec<Correction>)>,
}

impl Loaded {
    fn read(projects: &Projects, links: &[DocumentLink]) -> Result<Self, ProjectError> {
        let mut jobs = HashMap::new();
        for link in links {
            if jobs.contains_key(&link.job_id) {
                continue;
            }
            match projects.jobs.job(link.job_id) {
                Ok(job) => {
                    let pieces = projects.jobs.pieces(job.id)?;
                    let corrections = projects.jobs.corrections(job.id)?;
                    jobs.insert(job.id, (job, pieces, corrections));
                }
                Err(kalamu_core::StoreError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(Self { jobs })
    }

    /// Each set-up's result over the linked source, without its cost.
    fn results(&self, link: &DocumentLink) -> Vec<DocumentResult> {
        let Some((job, pieces, corrections)) = self.jobs.get(&link.job_id) else {
            return Vec::new();
        };
        job.setups
            .iter()
            .map(|setup| {
                let mine: Vec<&PieceState> = pieces
                    .iter()
                    .filter(|p| p.setup_id == setup.id && p.source_id == link.source_id)
                    .collect();
                let fixes: Vec<&Correction> = corrections
                    .iter()
                    .filter(|c| c.setup_id == setup.id && c.source_id == link.source_id)
                    .collect();
                DocumentResult {
                    job_id: job.id,
                    job_name: job.name.clone(),
                    job_status: job.status,
                    task: job.task,
                    created_at: job.created_at,
                    source_id: link.source_id,
                    setup_id: setup.id,
                    setup_label: setup.label.clone(),
                    provider: setup.provider.clone(),
                    model: setup.model.clone(),
                    pieces: mine.len() as u32,
                    finished: mine.iter().filter(|p| p.status.is_finished()).count() as u32,
                    failed: mine
                        .iter()
                        .filter(|p| p.status == PieceStatus::Failed)
                        .count() as u32,
                    reviewed: fixes.iter().filter(|c| c.reviewed).count() as u32,
                    corrected: fixes
                        .iter()
                        .filter(|c| c.text.is_some() || c.fields.is_some())
                        .count() as u32,
                    cost_usd: None,
                    preferred: false,
                }
            })
            .collect()
    }
}

/// The collections each document is in.
fn memberships(members: &[(CollectionId, DocumentId)], document: DocumentId) -> Vec<CollectionId> {
    members
        .iter()
        .filter(|(_, d)| *d == document)
        .map(|(c, _)| *c)
        .collect()
}

impl Projects {
    pub fn view(&self, id: ProjectId) -> Result<ProjectView, ProjectError> {
        let project = self.store.project(id).map_err(missing)?;
        let collections = self.store.collections(id)?;
        let documents = self.store.documents(id)?;
        let members = self.store.members(id)?;
        let links = self.store.links(id)?;
        let preferred = self.store.preferred(id)?;
        let loaded = Loaded::read(self, &links)?;

        let rows = documents
            .iter()
            .map(|document| {
                let mut tasks: Vec<TaskBrief> = Vec::new();
                let mut last_run: Option<Timestamp> = None;
                for link in links.iter().filter(|l| l.document_id == document.id) {
                    for result in loaded.results(link) {
                        last_run =
                            Some(last_run.map_or(result.created_at, |t| t.max(result.created_at)));
                        let brief = match tasks.iter_mut().find(|t| t.task == result.task) {
                            Some(brief) => brief,
                            None => {
                                tasks.push(TaskBrief {
                                    task: result.task,
                                    results: 0,
                                    finished: 0,
                                    reviewed: 0,
                                    preferred: preferred.iter().any(|p| {
                                        p.document_id == document.id && p.task == result.task
                                    }),
                                });
                                let last = tasks.len() - 1;
                                &mut tasks[last]
                            }
                        };
                        brief.results += 1;
                        if result.pieces > 0 && result.finished == result.pieces {
                            brief.finished += 1;
                        }
                        if result.pieces > 0 && result.reviewed == result.pieces {
                            brief.reviewed += 1;
                        }
                    }
                }
                tasks.sort_by_key(|t| Task::ALL.iter().position(|x| *x == t.task));
                DocumentRow {
                    collections: memberships(&members, document.id),
                    tasks,
                    missing: is_missing(&document.source),
                    same_content: documents
                        .iter()
                        .filter(|d| {
                            d.id != document.id && d.source.sha256 == document.source.sha256
                        })
                        .count() as u32,
                    last_run,
                    document: document.clone(),
                }
            })
            .collect();

        let mut jobs: Vec<(Timestamp, JobId)> = loaded
            .jobs
            .values()
            .map(|(job, _, _)| (job.created_at, job.id))
            .collect();
        jobs.sort_by(|a, b| b.cmp(a));
        Ok(ProjectView {
            collections: collections
                .into_iter()
                .map(|collection| CollectionView {
                    documents: members.iter().filter(|(c, _)| *c == collection.id).count() as u32,
                    collection,
                })
                .collect(),
            documents: rows,
            jobs: jobs.into_iter().map(|(_, id)| id).collect(),
            project,
        })
    }

    pub fn document(
        &self,
        project: ProjectId,
        id: DocumentId,
    ) -> Result<DocumentDetail, ProjectError> {
        let document = self.document_in(project, id)?;
        let links: Vec<DocumentLink> = self
            .store
            .links(project)?
            .into_iter()
            .filter(|l| l.document_id == id)
            .collect();
        let preferred = self.store.preferred(project)?;
        let loaded = Loaded::read(self, &links)?;
        let mut results = Vec::new();
        for link in &links {
            let Some((_, pieces, _)) = loaded.jobs.get(&link.job_id) else {
                continue;
            };
            for mut result in loaded.results(link) {
                result.cost_usd = self.cost(pieces, result.setup_id, link.source_id)?;
                result.preferred = preferred.iter().any(|p| {
                    p.document_id == id
                        && p.job_id == result.job_id
                        && p.setup_id == result.setup_id
                });
                results.push(result);
            }
        }
        results.sort_by_key(|r| std::cmp::Reverse(r.created_at));
        let others = self.store.documents(project)?;
        Ok(DocumentDetail {
            collections: memberships(&self.store.members(project)?, id),
            missing: is_missing(&document.source),
            same_content: others
                .iter()
                .filter(|d| d.id != id && d.source.sha256 == document.source.sha256)
                .map(DocumentBrief::of)
                .collect(),
            results,
            document,
        })
    }

    /// What one result's own calls cost, as the job's cost counts them: the pieces it stored,
    /// not those taken from the cache.
    fn cost(
        &self,
        pieces: &[PieceState],
        setup: SetupId,
        source: SourceId,
    ) -> Result<Option<f64>, ProjectError> {
        let mut total: Option<f64> = None;
        for piece in pieces.iter().filter(|p| {
            p.setup_id == setup && p.source_id == source && p.status == PieceStatus::Done
        }) {
            let Some(key) = &piece.cache_key else {
                continue;
            };
            if let Some(cost) = self
                .jobs
                .cached_result(&CacheKey(key.clone()))?
                .and_then(|r| r.cost_usd)
            {
                total = Some(total.unwrap_or(0.0) + cost);
            }
        }
        Ok(total)
    }
}
