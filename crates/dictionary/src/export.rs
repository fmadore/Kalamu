//! The dictionary written out for reuse outside Kalamu: its entries in JSON, with stable ids,
//! their forms as written, review status and the pairs kept apart; and every form a result
//! listed in CSV, as written beside the entry and label it maps to.

use kalamu_core::{DictionaryError, ProjectId, Timestamp};
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
    /// written beside its entry's id and label (empty when no entry holds it).
    pub fn export_csv(&self, project: ProjectId) -> Result<String, DictionaryError> {
        let gathered = self.gather(project, None)?;
        let entries = self.store.entries(project)?;
        let mut out = csv::Writer::from_writer(Vec::new());
        out.write_record([
            "document_id",
            "document",
            "job",
            "setup",
            "field",
            "original",
            "entry_id",
            "label",
            "reviewed",
        ])
        .map_err(failed)?;
        for reading in &gathered.readings {
            let title = gathered
                .titles
                .get(&reading.document)
                .map_or("", String::as_str);
            for (field, forms) in &reading.fields {
                for form in forms {
                    let entry = holder(&entries, field, form);
                    out.write_record([
                        reading.document.to_string().as_str(),
                        title,
                        reading.job_name.as_str(),
                        reading.setup_label.as_str(),
                        field.as_str(),
                        form.as_str(),
                        entry.map(|e| e.id.to_string()).unwrap_or_default().as_str(),
                        entry.map_or("", |e| e.label.as_str()),
                        if entry.is_some_and(|e| e.reviewed) {
                            "yes"
                        } else {
                            "no"
                        },
                    ])
                    .map_err(failed)?;
                }
            }
        }
        let bytes = out.into_inner().map_err(failed)?;
        String::from_utf8(bytes).map_err(failed)
    }
}
