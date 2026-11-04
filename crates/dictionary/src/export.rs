//! The dictionary written out for reuse outside Kalamu: its entries in JSON, with stable ids,
//! their forms as written, review status and the pairs kept apart; and every form a result
//! listed in CSV, as written beside the entry and label it maps to.

use kalamu_core::{DictionaryError, ProjectId, Table, Timestamp};
use serde::Serialize;

use crate::Dictionary;
use crate::inventory::holder;

#[derive(Serialize)]
struct EntryOut<'a> {
    id: String,
    namespace: &'a str,
    label: &'a str,
    forms: &'a [String],
    reviewed: bool,
    kept_apart_from: Vec<String>,
}

#[derive(Serialize)]
struct DictionaryOut<'a> {
    format: &'static str,
    version: u32,
    project: String,
    exported_at: Timestamp,
    entries: Vec<EntryOut<'a>>,
}

fn failed(e: impl std::fmt::Display) -> DictionaryError {
    DictionaryError::Export(e.to_string())
}

impl Dictionary {
    /// The entries as JSON: what the researcher decided, without the documents.
    pub fn export_json(&self, project: ProjectId) -> Result<String, DictionaryError> {
        self.check_project(project)?;
        let entries = self.store.entries(project)?;
        let apart = self.store.apart(project)?;
        let out = DictionaryOut {
            format: "kalamu-dictionary",
            version: 1,
            project: project.to_string(),
            exported_at: Timestamp::now(),
            entries: entries
                .iter()
                .map(|e| EntryOut {
                    id: e.id.to_string(),
                    namespace: &e.namespace,
                    label: &e.label,
                    forms: &e.forms,
                    reviewed: e.reviewed,
                    kept_apart_from: apart
                        .iter()
                        .filter(|p| p.holds(e.id))
                        .map(|p| if p.a == e.id { p.b } else { p.a }.to_string())
                        .collect(),
                })
                .collect(),
        };
        serde_json::to_string_pretty(&out).map_err(failed)
    }

    /// Every form each result listed, one row per document, result and form: the original as
    /// written beside its entry's id and label (empty when no entry holds it). A table for a
    /// spreadsheet, which the `export` crate writes as CSV.
    pub fn forms_table(&self, project: ProjectId) -> Result<Table, DictionaryError> {
        let gathered = self.gather(project, None)?;
        let entries = self.store.entries(project)?;
        let headings = [
            "document_id",
            "document",
            "job",
            "setup",
            "field",
            "original",
            "entry_id",
            "label",
            "reviewed",
        ]
        .map(str::to_string)
        .to_vec();
        let mut rows = Vec::new();
        for reading in &gathered.readings {
            let title = gathered
                .titles
                .get(&reading.document)
                .map_or("", String::as_str);
            for (field, forms) in &reading.fields {
                for form in forms {
                    let entry = holder(&entries, field, form);
                    rows.push(vec![
                        reading.document.to_string(),
                        title.to_string(),
                        reading.job_name.clone(),
                        reading.setup_label.clone(),
                        field.clone(),
                        form.clone(),
                        entry.map(|e| e.id.to_string()).unwrap_or_default(),
                        entry.map_or(String::new(), |e| e.label.clone()),
                        if entry.is_some_and(|e| e.reviewed) {
                            "yes"
                        } else {
                            "no"
                        }
                        .to_string(),
                    ]);
                }
            }
        }
        Ok(Table { headings, rows })
    }
}
