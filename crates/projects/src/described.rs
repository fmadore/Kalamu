//! What the researcher says of each document's source (#55's first increment): its own
//! description, the parts its project's documents share, and both written out for reuse
//! outside Kalamu, each value with where it came from. Supplied by the researcher only: no
//! model reads or writes it, and no result changes it.

use kalamu_core::provenance::{ProvenanceDescribed, ProvenanceDocument};
use kalamu_core::{
    CollectionId, CsvCells, DescribedBy, DescriptionField, Document, DocumentId, JobId, Project,
    ProjectError, ProjectId, SourceDescription, SourceDocuments, SourceId, StoreError, Table,
    Timestamp, cleaned, same_tag, tag_name,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Projects;

/// A tag renamed: how many documents changed, and the name they now carry, spaces tidied or
/// taken from a tag in use, so the interface goes on filtering on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TagRenamed {
    pub changed: u32,
    pub name: String,
}

/// One part of a document's description as it stands, and where it comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Described {
    pub field: DescriptionField,
    pub value: String,
    pub by: DescribedBy,
}

/// A document's description as it stands: each part its own, or else the project's, in the
/// order of [`DescriptionField::ALL`].
pub fn described(document: &Document, defaults: &SourceDescription) -> Vec<Described> {
    DescriptionField::ALL
        .iter()
        .filter_map(|field| {
            let (value, by) = match document.source_description.get(field) {
                Some(v) => (v, DescribedBy::Document),
                None => (defaults.get(field)?, DescribedBy::Project),
            };
            Some(Described {
                field: *field,
                value: value.clone(),
                by,
            })
        })
        .collect()
}

/// The documents a job's source is, in every project, each with its description as it stands:
/// what the engine writes into the result's `provenance.json`.
impl SourceDocuments for Projects {
    fn documents(
        &self,
        job: JobId,
        source: SourceId,
    ) -> Result<Vec<ProvenanceDocument>, StoreError> {
        // Only this source's links are read, not every project's: each export asks.
        let links = self.store.links_of(job, source)?;
        if links.is_empty() {
            return Ok(Vec::new());
        }
        let mut documents = Vec::new();
        for link in &links {
            match self.store.document(link.document_id) {
                Ok(document) => documents.push(document),
                Err(StoreError::NotFound) => continue,
                Err(e) => return Err(e),
            }
        }
        // Project by project, in the order of the list of projects, each one's documents in
        // the order they were linked.
        let mut out = Vec::new();
        for project in self.store.projects()? {
            for document in documents.iter().filter(|d| d.project_id == project.id) {
                let document = document.clone();
                let description = described(&document, &project.source_defaults)
                    .into_iter()
                    .map(|d| {
                        let part = ProvenanceDescribed {
                            value: d.value,
                            from: d.by,
                        };
                        (d.field, part)
                    })
                    .collect();
                out.push(ProvenanceDocument {
                    project_id: project.id,
                    project_name: project.name.clone(),
                    document_id: document.id,
                    title: document.title,
                    description,
                });
            }
        }
        Ok(out)
    }
}

#[derive(Serialize)]
struct ValueOut<'a> {
    value: &'a str,
    from: &'static str,
}

#[derive(Serialize)]
struct DocumentOut<'a> {
    id: String,
    title: &'a str,
    file_name: &'a str,
    path: &'a str,
    collections: Vec<&'a str>,
    tags: &'a [String],
    description: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct DescriptionsOut<'a> {
    format: &'static str,
    version: u32,
    project: String,
    project_name: &'a str,
    exported_at: Timestamp,
    documents: Vec<DocumentOut<'a>>,
}

pub(crate) fn failed(e: impl std::fmt::Display) -> ProjectError {
    ProjectError::Export(e.to_string())
}

/// The documents written out and the names of the collections each is in.
pub(crate) struct Listed {
    pub(crate) project: Project,
    pub(crate) documents: Vec<(Document, Vec<String>)>,
}

impl Projects {
    /// The job sources tied to these documents of a project, or to all its documents: the
    /// results whose provenance names them.
    pub fn sources_of(
        &self,
        project: ProjectId,
        documents: Option<&[DocumentId]>,
    ) -> Result<Vec<(JobId, SourceId)>, ProjectError> {
        Ok(self
            .store
            .links(project)?
            .into_iter()
            .filter(|l| documents.is_none_or(|ids| ids.contains(&l.document_id)))
            .map(|l| (l.job_id, l.source_id))
            .collect())
    }

    /// Sets a document's own description, its parts trimmed and those left empty dropped (they
    /// take the project's).
    pub fn describe_document(
        &self,
        project: ProjectId,
        id: DocumentId,
        description: SourceDescription,
    ) -> Result<(), ProjectError> {
        let mut document = self.document_in(project, id)?;
        document.source_description = cleaned(description);
        self.store.put_document(&document)?;
        self.touch(project)
    }

    /// Sets the parts of the description a project's documents share.
    pub fn set_source_defaults(
        &self,
        id: ProjectId,
        defaults: SourceDescription,
    ) -> Result<Project, ProjectError> {
        let mut project = self.store.project(id).map_err(crate::missing)?;
        project.source_defaults = cleaned(defaults);
        project.updated_at = Timestamp::now();
        self.store.put_project(&project)?;
        Ok(project)
    }

    /// Renames a tag on every document of the project that carries it, whatever its capitals.
    /// A new name that differs only in capitals respells it; one the project already uses, in
    /// whatever capitals, merges the two.
    pub fn rename_tag(
        &self,
        project: ProjectId,
        from: &str,
        to: &str,
    ) -> Result<TagRenamed, ProjectError> {
        let to = tag_name(to).ok_or(ProjectError::NoTag)?;
        let documents = self.store.documents(project)?;
        let spelled = if same_tag(from, &to) {
            to
        } else {
            documents
                .iter()
                .flat_map(|d| &d.tags)
                .find(|t| same_tag(t, &to))
                .cloned()
                .unwrap_or(to)
        };
        let mut changed = 0;
        for mut document in documents.into_iter().filter(|d| d.has_tag(from)) {
            document.untag(from);
            // The other spelling of a merged tag gives way too.
            document.untag(&spelled);
            document.tag(&spelled);
            self.store.put_document(&document)?;
            changed += 1;
        }
        if changed > 0 {
            self.touch(project)?;
        }
        Ok(TagRenamed {
            changed,
            name: spelled,
        })
    }

    pub(crate) fn listed(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
    ) -> Result<Listed, ProjectError> {
        let found = self.store.project(project).map_err(crate::missing)?;
        if let Some(c) = collection {
            self.collection(project, c)?;
        }
        let collections = self.store.collections(project)?;
        let members = self.store.members(project)?;
        let documents = self
            .store
            .documents(project)?
            .into_iter()
            .filter(|d| collection.is_none_or(|c| members.contains(&(c, d.id))))
            .map(|d| {
                let names = collections
                    .iter()
                    .filter(|c| members.contains(&(c.id, d.id)))
                    .map(|c| c.name.clone())
                    .collect();
                (d, names)
            })
            .collect();
        Ok(Listed {
            project: found,
            documents,
        })
    }

    /// The documents of a project, or of one of its collections, with their descriptions as
    /// they stand, each part with where it came from, as JSON.
    pub fn export_descriptions_json(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
    ) -> Result<String, ProjectError> {
        let listed = self.listed(project, collection)?;
        let defaults = &listed.project.source_defaults;
        let out = DescriptionsOut {
            format: "kalamu-descriptions",
            version: 1,
            project: project.to_string(),
            project_name: &listed.project.name,
            exported_at: Timestamp::now(),
            documents: listed
                .documents
                .iter()
                .map(|(d, names)| DocumentOut {
                    id: d.id.to_string(),
                    title: &d.title,
                    file_name: &d.source.file_name,
                    path: &d.source.path,
                    collections: names.iter().map(String::as_str).collect(),
                    tags: &d.tags,
                    description: described(d, defaults)
                        .iter()
                        .filter_map(|p| {
                            let value = serde_json::to_value(ValueOut {
                                value: &p.value,
                                from: p.by.as_str(),
                            })
                            .ok()?;
                            Some((p.field.as_str().to_string(), value))
                        })
                        .collect(),
                })
                .collect(),
        };
        serde_json::to_string_pretty(&out).map_err(failed)
    }

    /// The same as a table for a spreadsheet: one row per document, each part of the
    /// description in a column with another beside it saying where it came from; lists joined
    /// with `; `. The `export` crate writes it as CSV.
    pub fn descriptions_table(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
    ) -> Result<Table, ProjectError> {
        let listed = self.listed(project, collection)?;
        let mut head = vec![
            "document_id".to_string(),
            "title".into(),
            // Not `file`, which is a part of the description (the box or folder).
            "file_name".into(),
            "path".into(),
            "collections".into(),
            "tags".into(),
        ];
        for field in DescriptionField::ALL {
            head.push(field.as_str().into());
            head.push(format!("{}_from", field.as_str()));
        }
        let mut rows = Vec::with_capacity(listed.documents.len());
        for (d, names) in &listed.documents {
            let parts = described(d, &listed.project.source_defaults);
            let mut row = vec![
                d.id.to_string(),
                d.title.clone(),
                d.source.file_name.clone(),
                d.source.path.clone(),
                names.join("; "),
                d.tags.join("; "),
            ];
            for field in DescriptionField::ALL {
                match parts.iter().find(|p| p.field == field) {
                    Some(p) => {
                        row.push(p.value.clone());
                        row.push(p.by.as_str().into());
                    }
                    None => row.extend([String::new(), String::new()]),
                }
            }
            rows.push(row);
        }
        // For a spreadsheet, beside the JSON that keeps every value exactly; another tool's
        // import has the export profiles, written as they are.
        Ok(Table {
            headings: head,
            rows,
            cells: CsvCells::Guarded,
        })
    }
}
