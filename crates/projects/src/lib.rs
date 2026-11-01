//! Project workspaces: a researcher's documents, grouped in collections, with every result read
//! from each, whatever job produced it.
//!
//! This crate holds the rules; the store keeps the rows ([`ProjectStore`]) and the jobs stay in
//! the job store, read here, never changed. Organising never calls a model and never touches a
//! file or a result: adding a document references the file where it lies, a collection is a
//! set of links, and a job is tied to a document by an explicit link that a researcher can
//! review. A job keeps the sources it was started with, so a later change to a collection does
//! not rewrite a past run or start new work.

use std::sync::Arc;

use kalamu_core::{
    Collection, CollectionId, JobId, JobStore, Project, ProjectError, ProjectId, ProjectStore,
    StoreError, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod attach;
mod documents;
mod views;

pub use attach::{AttachChoice, SourceMatch};
pub use views::{
    CollectionView, DocumentBrief, DocumentDetail, DocumentResult, DocumentRow, ProjectView,
    TagView, TaskBrief,
};

/// A project as the list of projects shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectSummary {
    pub project: Project,
    pub documents: u32,
    pub collections: u32,
    /// Jobs that read at least one of its documents.
    pub jobs: u32,
}

pub struct Projects {
    pub(crate) store: Arc<dyn ProjectStore>,
    pub(crate) jobs: Arc<dyn JobStore>,
}

/// A store's "not found" as the project's own.
pub(crate) fn missing(e: StoreError) -> ProjectError {
    match e {
        StoreError::NotFound => ProjectError::NotFound,
        other => ProjectError::Store(other),
    }
}

/// A name with its spaces trimmed, refused when nothing is left.
fn named(name: &str) -> Result<String, ProjectError> {
    let name = name.trim();
    if name.is_empty() {
        Err(ProjectError::NoName)
    } else {
        Ok(name.to_string())
    }
}

impl Projects {
    /// `store` and `jobs` are usually the same database.
    pub fn new(store: Arc<dyn ProjectStore>, jobs: Arc<dyn JobStore>) -> Self {
        Self { store, jobs }
    }

    pub fn list(&self) -> Result<Vec<ProjectSummary>, ProjectError> {
        let mut out = Vec::new();
        for project in self.store.projects()? {
            let links = self.store.links(project.id)?;
            let mut jobs: Vec<JobId> = links.iter().map(|l| l.job_id).collect();
            jobs.sort();
            jobs.dedup();
            out.push(ProjectSummary {
                documents: self.store.documents(project.id)?.len() as u32,
                collections: self.store.collections(project.id)?.len() as u32,
                jobs: jobs.len() as u32,
                project,
            });
        }
        Ok(out)
    }

    pub fn create(&self, name: &str, description: &str) -> Result<Project, ProjectError> {
        let now = Timestamp::now();
        let project = Project {
            id: ProjectId::new(),
            name: named(name)?,
            description: description.trim().to_string(),
            archived: false,
            created_at: now,
            updated_at: now,
        };
        self.store.put_project(&project)?;
        Ok(project)
    }

    /// Renames and describes a project.
    pub fn update(
        &self,
        id: ProjectId,
        name: &str,
        description: &str,
    ) -> Result<Project, ProjectError> {
        let mut project = self.store.project(id).map_err(missing)?;
        project.name = named(name)?;
        project.description = description.trim().to_string();
        project.updated_at = Timestamp::now();
        self.store.put_project(&project)?;
        Ok(project)
    }

    /// Puts a project away, or brings it back; nothing in it changes.
    pub fn set_archived(&self, id: ProjectId, archived: bool) -> Result<Project, ProjectError> {
        let mut project = self.store.project(id).map_err(missing)?;
        project.archived = archived;
        project.updated_at = Timestamp::now();
        self.store.put_project(&project)?;
        Ok(project)
    }

    /// Deletes the project and how it organised its documents; the jobs, their results and the
    /// files stay.
    pub fn delete(&self, id: ProjectId) -> Result<(), ProjectError> {
        self.store.delete_project(id).map_err(missing)
    }

    pub fn create_collection(
        &self,
        project: ProjectId,
        name: &str,
    ) -> Result<Collection, ProjectError> {
        self.store.project(project).map_err(missing)?;
        let collection = Collection {
            id: CollectionId::new(),
            project_id: project,
            name: named(name)?,
            created_at: Timestamp::now(),
        };
        self.store.put_collection(&collection)?;
        self.touch(project)?;
        Ok(collection)
    }

    pub fn rename_collection(
        &self,
        project: ProjectId,
        id: CollectionId,
        name: &str,
    ) -> Result<(), ProjectError> {
        let mut collection = self.collection(project, id)?;
        collection.name = named(name)?;
        self.store.put_collection(&collection)?;
        self.touch(project)
    }

    /// Deletes a collection; its documents stay in the project.
    pub fn delete_collection(
        &self,
        project: ProjectId,
        id: CollectionId,
    ) -> Result<(), ProjectError> {
        self.collection(project, id)?;
        self.store.delete_collection(id)?;
        self.touch(project)
    }

    /// One of the project's collections.
    pub(crate) fn collection(
        &self,
        project: ProjectId,
        id: CollectionId,
    ) -> Result<Collection, ProjectError> {
        self.store
            .collections(project)?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| ProjectError::Elsewhere("The collection".into()))
    }

    /// Marks the project as just changed, so the list shows it first.
    pub(crate) fn touch(&self, id: ProjectId) -> Result<(), ProjectError> {
        let mut project = self.store.project(id).map_err(missing)?;
        project.updated_at = Timestamp::now();
        self.store.put_project(&project)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
