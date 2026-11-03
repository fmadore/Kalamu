//! A project's descriptions written through an export profile: the columns the researcher chose,
//! in their order and under their headings, lists, missing values and line breaks as the profile
//! says. From what is stored only: no model is called, and another profile only writes again.
//! Columns that read a document's result are filled by [`crate::resulted`].

use kalamu_core::{
    CollectionId, Document, ExportProfile, ProfileValue, ProjectError, ProjectId, Timestamp,
};
use serde::Serialize;
use serde::ser::SerializeMap;

use crate::Projects;
use crate::described::{described, failed};

/// What a column holds for one document.
pub(crate) enum Cell {
    Text(String),
    List(Vec<String>),
    Missing,
}

impl Cell {
    /// A text, or missing when empty.
    pub(crate) fn text(s: String) -> Self {
        if s.is_empty() {
            Self::Missing
        } else {
            Self::Text(s)
        }
    }

    /// A list, or missing when empty.
    pub(crate) fn list(items: Vec<String>) -> Self {
        if items.is_empty() {
            Self::Missing
        } else {
            Self::List(items)
        }
    }
}

fn cell(
    value: &ProfileValue,
    document: &Document,
    collections: &[String],
    project: &str,
    parts: &[crate::Described],
) -> Cell {
    let text = |s: &str| {
        if s.is_empty() {
            Cell::Missing
        } else {
            Cell::Text(s.to_string())
        }
    };
    let list = |items: &[String]| {
        if items.is_empty() {
            Cell::Missing
        } else {
            Cell::List(items.to_vec())
        }
    };
    let part = |field| parts.iter().find(|p| p.field == field);
    match value {
        ProfileValue::DocumentId => Cell::Text(document.id.to_string()),
        ProfileValue::Title => text(&document.title),
        ProfileValue::FileName => text(&document.source.file_name),
        ProfileValue::Path => text(&document.source.path),
        ProfileValue::Project => text(project),
        ProfileValue::Collections => list(collections),
        ProfileValue::Tags => list(&document.tags),
        ProfileValue::Part { field } => part(*field).map_or(Cell::Missing, |p| text(&p.value)),
        ProfileValue::From { field } => part(*field).map_or(Cell::Missing, |p| text(p.by.as_str())),
        ProfileValue::Fixed { text: fixed } => text(fixed),
        // Read from the document's result, by `Projects::result_cell`.
        ProfileValue::Status { .. }
        | ProfileValue::Model { .. }
        | ProfileValue::Text { .. }
        | ProfileValue::Answer { .. } => Cell::Missing,
    }
}

/// A value's line breaks, and the spaces around them, made one space.
fn one_line(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// One JSON row, its keys in the profile's order.
struct Row<'a>(Vec<(&'a str, serde_json::Value)>);

impl Serialize for Row<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (heading, value) in &self.0 {
            map.serialize_entry(heading, value)?;
        }
        map.end()
    }
}

#[derive(Serialize)]
struct ProfiledOut<'a> {
    format: &'static str,
    version: u32,
    profile: &'a str,
    project: String,
    project_name: &'a str,
    exported_at: Timestamp,
    columns: Vec<&'a str>,
    rows: Vec<Row<'a>>,
}

impl Projects {
    /// Each document shown, one row of the profile's cells.
    fn profiled_rows(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
        profile: &ExportProfile,
    ) -> Result<(crate::described::Listed, Vec<Vec<Cell>>), ProjectError> {
        let listed = self.listed(project, collection)?;
        // The job store is read only when a column needs a result.
        let results = if profile.columns.iter().any(|c| c.value.reads_result()) {
            let ids: Vec<_> = listed.documents.iter().map(|(d, _)| d.id).collect();
            Some(self.results_of(project, &ids)?)
        } else {
            None
        };
        let mut rows = Vec::with_capacity(listed.documents.len());
        for (d, names) in &listed.documents {
            let parts = described(d, &listed.project.source_defaults);
            let mut row = Vec::with_capacity(profile.columns.len());
            for c in &profile.columns {
                let read = match &results {
                    Some(results) => self.result_cell(results, &c.value, d.id)?,
                    None => None,
                };
                let cell =
                    read.unwrap_or_else(|| cell(&c.value, d, names, &listed.project.name, &parts));
                row.push(match cell {
                    Cell::Text(t) if profile.join_lines => Cell::Text(one_line(&t)),
                    Cell::List(items) if profile.join_lines => {
                        Cell::List(items.iter().map(|i| one_line(i)).collect())
                    }
                    other => other,
                });
            }
            rows.push(row);
        }
        Ok((listed, rows))
    }

    /// The documents of a project, or of a collection, through a profile, as CSV: its headings,
    /// then one row per document, lists joined with its separator and missing values written as
    /// it says.
    pub fn export_profiled_csv(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
        profile: &ExportProfile,
    ) -> Result<String, ProjectError> {
        let (_, rows) = self.profiled_rows(project, collection, profile)?;
        let mut out = csv::Writer::from_writer(Vec::new());
        out.write_record(profile.columns.iter().map(|c| c.heading.as_str()))
            .map_err(failed)?;
        for row in rows {
            let row: Vec<String> = row
                .into_iter()
                .map(|cell| match cell {
                    Cell::Text(t) => t,
                    Cell::List(items) => items.join(&profile.list_separator),
                    Cell::Missing => profile.missing.clone(),
                })
                .collect();
            out.write_record(&row).map_err(failed)?;
        }
        let bytes = out.into_inner().map_err(failed)?;
        String::from_utf8(bytes).map_err(failed)
    }

    /// The same as JSON: each row an object under the profile's headings, in its order, lists
    /// kept as arrays and missing values `null`.
    pub fn export_profiled_json(
        &self,
        project: ProjectId,
        collection: Option<CollectionId>,
        profile: &ExportProfile,
    ) -> Result<String, ProjectError> {
        let (listed, rows) = self.profiled_rows(project, collection, profile)?;
        let headings: Vec<&str> = profile.columns.iter().map(|c| c.heading.as_str()).collect();
        let out = ProfiledOut {
            format: "kalamu-profiled-descriptions",
            version: 1,
            profile: &profile.name,
            project: project.to_string(),
            project_name: &listed.project.name,
            exported_at: Timestamp::now(),
            columns: headings.clone(),
            rows: rows
                .into_iter()
                .map(|row| {
                    Row(headings
                        .iter()
                        .zip(row)
                        .map(|(h, cell)| {
                            let value = match cell {
                                Cell::Text(t) => serde_json::Value::String(t),
                                Cell::List(items) => items.into(),
                                Cell::Missing => serde_json::Value::Null,
                            };
                            (*h, value)
                        })
                        .collect())
                })
                .collect(),
        };
        serde_json::to_string_pretty(&out).map_err(failed)
    }
}
