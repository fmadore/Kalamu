//! Project workspaces: a researcher's documents, grouped in collections, with every result
//! read from each, whatever job produced it.
//!
//! A document is a research item, not a file: two files with the same bytes are not taken for
//! the same item unless the researcher says so, and a document is tied to the jobs that read
//! it by explicit links. Organising (collections, titles, links) never calls a model and never
//! touches the files or the results; a job keeps the sources it was started with, so later
//! changes to a collection leave past runs as they were.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    CollectionId, DocumentId, JobId, ProjectId, SetupId, SourceFile, SourceId, StoreError, Task,
    Timestamp,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Put away: kept whole, but out of the list of projects until restored.
    #[serde(default)]
    pub archived: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// A flat grouping of documents; a document can be in several.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Collection {
    pub id: CollectionId,
    pub project_id: ProjectId,
    pub name: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Document {
    pub id: DocumentId,
    pub project_id: ProjectId,
    /// The researcher's name for it; the file's name until changed.
    pub title: String,
    /// The original, referenced where it lies on the PC: a project never moves or copies it.
    pub source: SourceFile,
    pub created_at: Timestamp,
    /// The researcher's labels, filters beside the collections; spelled as first given.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Document {
    /// Adds a tag, unless it holds one that differs only in capitals. Returns whether it changed.
    pub fn tag(&mut self, tag: &str) -> bool {
        if self.has_tag(tag) {
            return false;
        }
        self.tags.push(tag.to_string());
        self.tags.sort_by_key(|t| (t.to_lowercase(), t.clone()));
        true
    }

    /// Removes a tag, whatever its capitals. Returns whether it changed.
    pub fn untag(&mut self, tag: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|t| !same_tag(t, tag));
        self.tags.len() != before
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| same_tag(t, tag))
    }
}

/// Two tags are one when they differ only in capitals.
pub fn same_tag(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// A tag as typed, its spaces trimmed and runs of spaces made one; `None` when nothing is left.
pub fn tag_name(typed: &str) -> Option<String> {
    let tag = typed.split_whitespace().collect::<Vec<_>>().join(" ");
    (!tag.is_empty()).then_some(tag)
}

/// A document read by one source of a job: every set-up of that job over that source is a
/// result of the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentLink {
    pub document_id: DocumentId,
    pub job_id: JobId,
    pub source_id: SourceId,
    pub linked_at: Timestamp,
}

/// The result the researcher prefers for a task among a document's alternatives. Never chosen
/// by the app: a newer run does not replace it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PreferredResult {
    pub document_id: DocumentId,
    pub task: Task,
    pub job_id: JobId,
    pub setup_id: SetupId,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    /// A collection, document or job of another project, or one that no longer exists.
    #[error("{0} does not belong to this project")]
    Elsewhere(String),
    #[error("A project needs a name")]
    NoName,
    #[error("A tag needs a name")]
    NoTag,
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ProjectError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound | Self::Store(StoreError::NotFound) => "project.not_found",
            Self::Elsewhere(_) => "project.elsewhere",
            Self::NoName => "project.no_name",
            Self::NoTag => "project.no_tag",
            Self::Store(_) => "job.database",
        }
    }
}

/// Persistence of projects, kept apart from the job store the engine uses: organising work
/// does not run it.
pub trait ProjectStore: Send + Sync {
    fn put_project(&self, project: &Project) -> Result<(), StoreError>;
    fn projects(&self) -> Result<Vec<Project>, StoreError>;
    fn project(&self, id: ProjectId) -> Result<Project, StoreError>;
    /// Removes the project with its collections, documents, links and preferences; the jobs
    /// and their results stay.
    fn delete_project(&self, id: ProjectId) -> Result<(), StoreError>;

    fn put_collection(&self, collection: &Collection) -> Result<(), StoreError>;
    fn collections(&self, project: ProjectId) -> Result<Vec<Collection>, StoreError>;
    /// Removes the collection and its memberships; its documents stay in the project.
    fn delete_collection(&self, id: CollectionId) -> Result<(), StoreError>;

    fn put_document(&self, document: &Document) -> Result<(), StoreError>;
    fn documents(&self, project: ProjectId) -> Result<Vec<Document>, StoreError>;
    fn document(&self, id: DocumentId) -> Result<Document, StoreError>;
    /// Removes the document, its memberships, links and preferences; the jobs that read it and
    /// their results stay.
    fn delete_document(&self, id: DocumentId) -> Result<(), StoreError>;

    fn add_member(&self, collection: CollectionId, document: DocumentId) -> Result<(), StoreError>;
    fn remove_member(
        &self,
        collection: CollectionId,
        document: DocumentId,
    ) -> Result<(), StoreError>;
    /// Every `(collection, document)` pair of a project.
    fn members(&self, project: ProjectId) -> Result<Vec<(CollectionId, DocumentId)>, StoreError>;

    fn put_link(&self, link: &DocumentLink) -> Result<(), StoreError>;
    fn remove_link(
        &self,
        document: DocumentId,
        job: JobId,
        source: SourceId,
    ) -> Result<(), StoreError>;
    fn links(&self, project: ProjectId) -> Result<Vec<DocumentLink>, StoreError>;

    /// Sets, or with `None` clears, the preferred result of a task for a document.
    fn set_preferred(
        &self,
        document: DocumentId,
        task: Task,
        choice: Option<(JobId, SetupId)>,
    ) -> Result<(), StoreError>;
    fn preferred(&self, project: ProjectId) -> Result<Vec<PreferredResult>, StoreError>;
}
