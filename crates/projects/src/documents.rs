//! Documents in a project: added without copies, grouped, tagged, renamed, merged and removed.

use std::path::Path;

use kalamu_core::{
    CollectionId, Document, DocumentId, DocumentLink, InputKind, JobId, ProjectError, ProjectId,
    SetupId, SourceFile, Task, Timestamp, same_tag, tag_name,
};

use crate::{Projects, missing, named};

/// The document's name before the researcher gives one: the file's, or a video's title.
pub(crate) fn default_title(source: &SourceFile) -> String {
    if let Some(title) = source.title.as_deref().filter(|t| !t.trim().is_empty()) {
        return title.trim().to_string();
    }
    Path::new(&source.file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| source.file_name.clone())
}

/// The same file at the same place: the one case taken for the same document without asking.
/// Equal bytes elsewhere may be another copy of another item, and are only pointed out.
pub(crate) fn same_file(a: &SourceFile, b: &SourceFile) -> bool {
    a.sha256 == b.sha256 && a.path == b.path
}

/// A file no longer where it was added (a YouTube video has no file).
pub(crate) fn is_missing(source: &SourceFile) -> bool {
    source.kind != InputKind::YouTube && !Path::new(&source.path).exists()
}

impl Projects {
    /// Adds files to a project, each as a document, and to `collection` if given. A file the
    /// project already holds, at the same place with the same content, is not added twice: its
    /// document is returned (and put in the collection). Returns the documents in the order
    /// of `sources`.
    pub fn add_documents(
        &self,
        project: ProjectId,
        sources: Vec<SourceFile>,
        collection: Option<CollectionId>,
    ) -> Result<Vec<DocumentId>, ProjectError> {
        self.store.project(project).map_err(missing)?;
        if let Some(c) = collection {
            self.collection(project, c)?;
        }
        let mut documents = self.store.documents(project)?;
        let mut out = Vec::new();
        for source in sources {
            let id = match documents.iter().find(|d| same_file(&d.source, &source)) {
                Some(existing) => existing.id,
                None => {
                    let document = Document {
                        id: DocumentId::new(),
                        project_id: project,
                        title: default_title(&source),
                        source,
                        created_at: Timestamp::now(),
                        tags: Vec::new(),
                    };
                    self.store.put_document(&document)?;
                    let id = document.id;
                    documents.push(document);
                    id
                }
            };
            if let Some(c) = collection {
                self.store.add_member(c, id)?;
            }
            out.push(id);
        }
        self.touch(project)?;
        Ok(out)
    }

    /// One of the project's documents.
    pub(crate) fn document_in(
        &self,
        project: ProjectId,
        id: DocumentId,
    ) -> Result<Document, ProjectError> {
        let document = self.store.document(id).map_err(missing)?;
        if document.project_id != project {
            return Err(ProjectError::Elsewhere("The document".into()));
        }
        Ok(document)
    }

    pub fn rename_document(
        &self,
        project: ProjectId,
        id: DocumentId,
        title: &str,
    ) -> Result<(), ProjectError> {
        let mut document = self.document_in(project, id)?;
        document.title = named(title)?;
        self.store.put_document(&document)?;
        self.touch(project)
    }

    /// Removes documents from the project, with their memberships and links; the files, the
    /// jobs that read them and their results stay.
    pub fn delete_documents(
        &self,
        project: ProjectId,
        ids: &[DocumentId],
    ) -> Result<(), ProjectError> {
        for id in ids {
            self.document_in(project, *id)?;
        }
        for id in ids {
            self.store.delete_document(*id)?;
        }
        self.touch(project)
    }

    pub fn add_to_collection(
        &self,
        project: ProjectId,
        collection: CollectionId,
        ids: &[DocumentId],
    ) -> Result<(), ProjectError> {
        self.collection(project, collection)?;
        for id in ids {
            self.document_in(project, *id)?;
        }
        for id in ids {
            self.store.add_member(collection, *id)?;
        }
        self.touch(project)
    }

    /// Takes documents out of a collection; they stay in the project.
    pub fn remove_from_collection(
        &self,
        project: ProjectId,
        collection: CollectionId,
        ids: &[DocumentId],
    ) -> Result<(), ProjectError> {
        self.collection(project, collection)?;
        for id in ids {
            self.store.remove_member(collection, *id)?;
        }
        self.touch(project)
    }

    /// Tags documents. A tag that differs only in capitals from one the project already uses
    /// takes that spelling, so one label is never split in two.
    pub fn tag_documents(
        &self,
        project: ProjectId,
        ids: &[DocumentId],
        tag: &str,
    ) -> Result<(), ProjectError> {
        let tag = tag_name(tag).ok_or(ProjectError::NoTag)?;
        let documents = self.store.documents(project)?;
        let spelled = documents
            .iter()
            .flat_map(|d| &d.tags)
            .find(|t| same_tag(t, &tag))
            .cloned()
            .unwrap_or(tag);
        let mut chosen = Vec::new();
        for id in ids {
            chosen.push(self.document_in(project, *id)?);
        }
        for mut document in chosen {
            if document.tag(&spelled) {
                self.store.put_document(&document)?;
            }
        }
        self.touch(project)
    }

    /// Takes a tag off documents, whatever its capitals; they stay where they are.
    pub fn untag_documents(
        &self,
        project: ProjectId,
        ids: &[DocumentId],
        tag: &str,
    ) -> Result<(), ProjectError> {
        let mut chosen = Vec::new();
        for id in ids {
            chosen.push(self.document_in(project, *id)?);
        }
        for mut document in chosen {
            if document.untag(tag) {
                self.store.put_document(&document)?;
            }
        }
        self.touch(project)
    }

    /// Makes two documents one, when the researcher decides they are the same item: `from`'s
    /// links, collections and tags move to `into`, which keeps its own title and preferred results
    /// (taking `from`'s where it has none), and `from` is removed. Nothing is read again.
    pub fn merge_documents(
        &self,
        project: ProjectId,
        from: DocumentId,
        into: DocumentId,
    ) -> Result<(), ProjectError> {
        if from == into {
            return Ok(());
        }
        let merged = self.document_in(project, from)?;
        let mut kept = self.document_in(project, into)?;
        let mut changed = false;
        for tag in &merged.tags {
            changed |= kept.tag(tag);
        }
        if changed {
            self.store.put_document(&kept)?;
        }
        for link in self.store.links(project)? {
            if link.document_id == from {
                self.store.put_link(&DocumentLink {
                    document_id: into,
                    ..link
                })?;
            }
        }
        for (collection, document) in self.store.members(project)? {
            if document == from {
                self.store.add_member(collection, into)?;
            }
        }
        let preferred = self.store.preferred(project)?;
        for choice in preferred.iter().filter(|p| p.document_id == from) {
            let taken = preferred
                .iter()
                .any(|p| p.document_id == into && p.task == choice.task);
            if !taken {
                self.store.set_preferred(
                    into,
                    choice.task,
                    Some((choice.job_id, choice.setup_id)),
                )?;
            }
        }
        self.store.delete_document(from)?;
        self.touch(project)
    }

    /// Chooses, or with `None` clears, the result the researcher prefers for a task: one of the
    /// document's own, from a job of that task.
    pub fn set_preferred(
        &self,
        project: ProjectId,
        document: DocumentId,
        task: Task,
        choice: Option<(JobId, SetupId)>,
    ) -> Result<(), ProjectError> {
        self.document_in(project, document)?;
        if let Some((job_id, setup_id)) = choice {
            let linked = self
                .store
                .links(project)?
                .iter()
                .any(|l| l.document_id == document && l.job_id == job_id);
            let job = self.jobs.job(job_id).map_err(missing)?;
            if !linked || job.task != task || !job.setups.iter().any(|s| s.id == setup_id) {
                return Err(ProjectError::Elsewhere("The result".into()));
            }
        }
        self.store.set_preferred(document, task, choice)?;
        Ok(())
    }
}
