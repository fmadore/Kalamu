//! Tying jobs to documents: an existing job attached after review of the proposed matches, a
//! job started in the project linked at once, a link undone.

use kalamu_core::{
    CollectionId, Document, DocumentId, DocumentLink, Job, JobId, ProjectError, ProjectId,
    SourceId, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::documents::{default_title, same_file};
use crate::views::DocumentBrief;
use crate::{Projects, missing};

/// One source of a job about to be attached, and the documents it might be.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SourceMatch {
    pub source_id: SourceId,
    pub file_name: String,
    pub path: String,
    /// Already tied to this document of the project.
    pub linked: Option<DocumentId>,
    /// The document it is taken for unless the researcher says otherwise: the very source a
    /// document was added with, the document of the result its text was frozen from, or the
    /// same file at the same place. `None`: a new document.
    pub proposed: Option<DocumentId>,
    /// Other documents with the same content, perhaps another copy of the same item: offered,
    /// never chosen for the researcher.
    pub same_content: Vec<DocumentBrief>,
}

/// What the researcher chose for one source: an existing document, or a new one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AttachChoice {
    pub source_id: SourceId,
    pub document: Option<DocumentId>,
}

/// The document a source is: the one added with this very source, the one already tied to the
/// result it was frozen from, or the same file at the same place.
fn proposed(
    documents: &[Document],
    links: &[DocumentLink],
    source: &kalamu_core::SourceFile,
) -> Option<DocumentId> {
    documents
        .iter()
        .find(|d| d.source.id == source.id)
        .map(|d| d.id)
        .or_else(|| {
            let from = source.derived_from.as_ref()?;
            links
                .iter()
                .find(|l| l.job_id == from.job_id && l.source_id == from.source_id)
                .map(|l| l.document_id)
        })
        .or_else(|| {
            documents
                .iter()
                .find(|d| same_file(&d.source, source))
                .map(|d| d.id)
        })
}

/// How each source of `job` would be attached, given the project's documents and links.
fn matches(job: &Job, documents: &[Document], links: &[DocumentLink]) -> Vec<SourceMatch> {
    job.sources
        .iter()
        .map(|source| {
            let proposed = proposed(documents, links, source);
            SourceMatch {
                source_id: source.id,
                file_name: source.file_name.clone(),
                path: source.path.clone(),
                linked: links
                    .iter()
                    .find(|l| l.job_id == job.id && l.source_id == source.id)
                    .map(|l| l.document_id),
                proposed,
                same_content: documents
                    .iter()
                    .filter(|d| d.source.sha256 == source.sha256 && Some(d.id) != proposed)
                    .map(DocumentBrief::of)
                    .collect(),
            }
        })
        .collect()
}

/// What attaching writes, gathered before it is written at once: new documents, their places
/// in a collection, and the links of sources to documents.
#[derive(Default)]
struct Attachment {
    documents: Vec<Document>,
    members: Vec<(CollectionId, DocumentId)>,
    links: Vec<DocumentLink>,
}

impl Projects {
    /// How each source of a job would be attached to the project, for the researcher to check.
    pub fn attach_plan(
        &self,
        project: ProjectId,
        job: JobId,
    ) -> Result<Vec<SourceMatch>, ProjectError> {
        self.store.project(project).map_err(missing)?;
        let job = self.jobs.job(job).map_err(missing)?;
        let documents = self.store.documents(project)?;
        let links = self.store.links(project)?;
        Ok(matches(&job, &documents, &links))
    }

    /// Attaches a job's sources as the researcher chose: each to an existing document of the
    /// project, or to a new document made from the source (and put in `collection`, if given).
    /// A source left out of `choices` is not attached. Nothing is read again: the job's results
    /// become the documents' results as they stand.
    pub fn attach(
        &self,
        project: ProjectId,
        job: JobId,
        choices: &[AttachChoice],
        collection: Option<CollectionId>,
    ) -> Result<Vec<DocumentId>, ProjectError> {
        self.store.project(project).map_err(missing)?;
        if let Some(c) = collection {
            self.collection(project, c)?;
        }
        let job = self.jobs.job(job).map_err(missing)?;
        let mut plan = Attachment::default();
        let out = self.plan(project, &job, choices, collection, &mut plan)?;
        self.store
            .attach_documents(&plan.documents, &plan.members, &plan.links)?;
        self.touch(project)?;
        Ok(out)
    }

    /// Links a job started from the project to its documents: each source to the document it
    /// was added as, or to a new document. Its files were chosen in the project, so they are
    /// its documents; nothing is asked.
    pub fn link_job(
        &self,
        project: ProjectId,
        job: JobId,
        collection: Option<CollectionId>,
    ) -> Result<Vec<DocumentId>, ProjectError> {
        self.link_jobs(project, &[job], collection)
    }

    /// Links several jobs to their documents as [`Projects::link_job`] links one, all or none:
    /// every job is checked and planned before anything is written, and what is written is
    /// written at once. A later job's source the same as an earlier one's joins the document
    /// made for it. The documents, job after job and source after source.
    pub fn link_jobs(
        &self,
        project: ProjectId,
        jobs: &[JobId],
        collection: Option<CollectionId>,
    ) -> Result<Vec<DocumentId>, ProjectError> {
        self.store.project(project).map_err(missing)?;
        if let Some(c) = collection {
            self.collection(project, c)?;
        }
        let mut documents = self.store.documents(project)?;
        let mut links = self.store.links(project)?;
        let mut plan = Attachment::default();
        let mut out = Vec::new();
        for &id in jobs {
            let job = self.jobs.job(id).map_err(missing)?;
            let choices: Vec<AttachChoice> = matches(&job, &documents, &links)
                .into_iter()
                .map(|m| AttachChoice {
                    source_id: m.source_id,
                    document: m.linked.or(m.proposed),
                })
                .collect();
            let (made, tied) = (plan.documents.len(), plan.links.len());
            out.extend(self.plan(project, &job, &choices, collection, &mut plan)?);
            documents.extend_from_slice(&plan.documents[made..]);
            links.extend_from_slice(&plan.links[tied..]);
        }
        self.store
            .attach_documents(&plan.documents, &plan.members, &plan.links)?;
        self.touch(project)?;
        Ok(out)
    }

    /// Adds to `plan` what attaching `job`'s sources as chosen writes, checking every choice
    /// first; a document chosen may be one the plan already makes. The documents, in order.
    fn plan(
        &self,
        project: ProjectId,
        job: &Job,
        choices: &[AttachChoice],
        collection: Option<CollectionId>,
        plan: &mut Attachment,
    ) -> Result<Vec<DocumentId>, ProjectError> {
        let mut planned = Vec::new();
        for choice in choices {
            let source = job
                .sources
                .iter()
                .find(|s| s.id == choice.source_id)
                .ok_or_else(|| ProjectError::Elsewhere("The file".into()))?;
            if let Some(document) = choice.document
                && !plan.documents.iter().any(|d| d.id == document)
            {
                self.document_in(project, document)?;
            }
            planned.push((source, choice.document));
        }
        let now = Timestamp::now();
        let mut out = Vec::new();
        for (source, document) in planned {
            let id = match document {
                Some(id) => id,
                None => {
                    let document = Document {
                        id: DocumentId::new(),
                        project_id: project,
                        title: default_title(source),
                        source: source.clone(),
                        created_at: now,
                        tags: Vec::new(),
                        source_description: Default::default(),
                    };
                    let id = document.id;
                    plan.documents.push(document);
                    if let Some(c) = collection {
                        plan.members.push((c, id));
                    }
                    id
                }
            };
            plan.links.push(DocumentLink {
                document_id: id,
                job_id: job.id,
                source_id: source.id,
                linked_at: now,
            });
            out.push(id);
        }
        Ok(out)
    }

    /// Unties one source of a job from a document; the job and its results stay.
    pub fn detach(
        &self,
        project: ProjectId,
        document: DocumentId,
        job: JobId,
        source: SourceId,
    ) -> Result<(), ProjectError> {
        self.document_in(project, document)?;
        self.store.remove_link(document, job, source)?;
        // A preferred result of a job no longer tied to the document is no longer its own.
        let still = self
            .store
            .links(project)?
            .iter()
            .any(|l| l.document_id == document && l.job_id == job);
        if !still {
            for choice in self.store.preferred(project)? {
                if choice.document_id == document && choice.job_id == job {
                    self.store.set_preferred(document, choice.task, None)?;
                }
            }
        }
        self.touch(project)
    }
}
