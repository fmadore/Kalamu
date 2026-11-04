//! Which results the review queue reads (#85): the documents in scope, and for each, the result
//! the researcher preferred for its task or, where none is preferred, every result they have
//! not set aside. A document read by several set-ups stays one document; its alternatives are
//! listed, so the researcher sees what the queue reads and can narrow it.

use std::collections::HashSet;

use kalamu_core::{
    CollectionId, DocumentId, JobId, ProjectError, ProjectId, ReviewScope, ReviewTarget, SetupId,
    Task, TiedResults,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Projects, missing};

/// The documents a queue covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum QueueSelection {
    Project,
    Collection { id: CollectionId },
    Documents { ids: Vec<DocumentId> },
}

/// A result, named by its job and set-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResultKey {
    pub job_id: JobId,
    pub setup_id: SetupId,
}

/// A set-up of a job that reads documents with no preferred result for its task: the queue
/// reads it unless the researcher sets it aside.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResultChoice {
    pub key: ResultKey,
    pub job_name: String,
    pub task: Task,
    pub setup_label: String,
    pub model: String,
    /// Documents in scope it read that have no preferred result.
    pub documents: u32,
    pub included: bool,
}

/// What the queue reads, with the choices that shaped it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct QueuePlan {
    pub scope: ReviewScope,
    /// Results that stand for documents without a preferred one, newest job first.
    pub choices: Vec<ResultChoice>,
    /// Documents and tasks read through the researcher's preferred result.
    pub preferred: u32,
}

impl Projects {
    /// The results a review queue over `selection` reads: for each document and task, the
    /// preferred result where the researcher chose one (never set aside), else every result not
    /// in `excluded`.
    pub fn queue_plan(
        &self,
        project: ProjectId,
        selection: &QueueSelection,
        excluded: &[ResultKey],
    ) -> Result<QueuePlan, ProjectError> {
        self.store.project(project).map_err(missing)?;
        let all = self.store.documents(project)?;
        let documents: Vec<_> = match selection {
            QueueSelection::Project => all,
            QueueSelection::Collection { id } => {
                self.collection(project, *id)?;
                let members: HashSet<DocumentId> = self
                    .store
                    .members(project)?
                    .into_iter()
                    .filter(|(c, _)| c == id)
                    .map(|(_, d)| d)
                    .collect();
                all.into_iter()
                    .filter(|d| members.contains(&d.id))
                    .collect()
            }
            QueueSelection::Documents { ids } => {
                if let Some(stray) = ids.iter().find(|id| !all.iter().any(|d| d.id == **id)) {
                    self.document_in(project, *stray)?;
                }
                all.into_iter().filter(|d| ids.contains(&d.id)).collect()
            }
        };
        let in_scope: HashSet<DocumentId> = documents.iter().map(|d| d.id).collect();
        let tied = TiedResults::read(
            &self.store.links(project)?,
            &self.store.preferred(project)?,
            |d| in_scope.contains(&d),
            |id| self.jobs.job(id),
        )?;
        let excluded: HashSet<ResultKey> = excluded.iter().copied().collect();

        let mut targets = Vec::new();
        let mut choices: Vec<ResultChoice> = Vec::new();
        let mut through_preferred: HashSet<(DocumentId, Task)> = HashSet::new();
        for document in &documents {
            for task in tied.tasks(document.id) {
                // The preferred result, while its job and set-up still exist.
                if let Some(chosen) = tied.preferred(document.id, task) {
                    through_preferred.insert((document.id, task));
                    targets.push(ReviewTarget {
                        document_id: document.id,
                        document: document.title.clone(),
                        job_id: chosen.job_id,
                        source_id: chosen.source_id,
                        setup_id: chosen.setup_id,
                        preferred: true,
                    });
                    continue;
                }
                let mut alternatives: Vec<_> = tied
                    .of(document.id, task)
                    .filter_map(|r| Some((r, tied.job(r.job_id)?)))
                    .collect();
                alternatives.sort_by_key(|(_, job)| std::cmp::Reverse(job.created_at));
                for (result, job) in alternatives {
                    let Some(setup) = job.setups.iter().find(|s| s.id == result.setup_id) else {
                        continue;
                    };
                    let key = ResultKey {
                        job_id: job.id,
                        setup_id: setup.id,
                    };
                    let included = !excluded.contains(&key);
                    match choices.iter_mut().find(|c| c.key == key) {
                        Some(choice) => choice.documents += 1,
                        None => choices.push(ResultChoice {
                            key,
                            job_name: job.name.clone(),
                            task: job.task,
                            setup_label: setup.label.clone(),
                            model: setup.model.clone(),
                            documents: 1,
                            included,
                        }),
                    }
                    if included {
                        targets.push(ReviewTarget {
                            document_id: document.id,
                            document: document.title.clone(),
                            job_id: job.id,
                            source_id: result.source_id,
                            setup_id: setup.id,
                            preferred: false,
                        });
                    }
                }
            }
        }
        choices.sort_by_key(|c| {
            let job = tied.job(c.key.job_id);
            let created = job.map(|j| j.created_at);
            let order = job.and_then(|j| j.setups.iter().position(|s| s.id == c.key.setup_id));
            (std::cmp::Reverse(created), order)
        });
        Ok(QueuePlan {
            scope: ReviewScope {
                documents: documents.len() as u32,
                targets,
            },
            choices,
            preferred: through_preferred.len() as u32,
        })
    }
}
