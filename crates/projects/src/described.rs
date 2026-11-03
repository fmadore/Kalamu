//! What the researcher says of each document's source (#55's first increment): its own
//! description, the parts its project's documents share, and both written out for reuse
//! outside Kalamu, each value with where it came from. Supplied by the researcher only: no
//! model reads or writes it, and no result changes it.

use kalamu_core::{
    CollectionId, DescriptionField, Document, DocumentId, Project, ProjectError, ProjectId,
    SourceDescription, Timestamp, cleaned, same_tag, tag_name,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Projects;

/// Where a part of a document's description comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DescribedBy {
    /// The document's own description.
    Document,
    /// The project's, shared by its documents.
    Project,
}

impl DescribedBy {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Project => "project",
        }
    }
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
    /// whatever capitals, merges the two. Returns how many documents changed.
    pub fn rename_tag(
        &self,
        project: ProjectId,
        from: &str,
        to: &str,
    ) -> Result<u32, ProjectError> {
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
        Ok(changed)
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

    /// The same as CSV: one row per document, each part of the description in a column with
    /// another beside it saying where it came from; lists joined with `; `.
    pub fn export_descriptions_csv(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
    ) -> Result<String, ProjectError> {
        let listed = self.listed(project, collection)?;
        let mut out = csv::Writer::from_writer(Vec::new());
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
        out.write_record(&head).map_err(failed)?;
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
            out.write_record(&row).map_err(failed)?;
        }
        let bytes = out.into_inner().map_err(failed)?;
        String::from_utf8(bytes).map_err(failed)
    }
}
