//! One analysis's answers over a project's documents as a table for a spreadsheet, one row per
//! document, with the researcher's tags beside the model's answers (the screening table of
//! #51).

use kalamu_core::analysis::cited;
use kalamu_core::{
    CsvCells, Field, FieldAnswer, FieldValue, JobId, ProjectError, ProjectId, SetupId, Table,
};

use crate::answered::answers_of;
use crate::{Projects, missing};

impl Projects {
    /// One analysis's answers over the project's documents as a table for a spreadsheet: the
    /// document's title and file, each of its fields under the label the job froze (a choice's
    /// reason beside it when it asks for one, a field that cites the text as its quotations
    /// with where each stands), then the researcher's tags. One row per document it read, in
    /// title order; a document it could not read is left out. Guarded: the cells hold what a model wrote.
    pub fn answers_table(
        &self,
        project: ProjectId,
        job_id: JobId,
        setup_id: SetupId,
    ) -> Result<Table, ProjectError> {
        self.store.project(project).map_err(missing)?;
        let job = self.jobs.job(job_id).map_err(missing)?;
        let setup = job
            .setups
            .iter()
            .find(|s| s.id == setup_id)
            .ok_or(ProjectError::NotFound)?;
        let fields: &[Field] = setup
            .prompt
            .schema
            .as_ref()
            .map_or(&[], |s| s.fields.as_slice());
        let links = self.store.links(project)?;
        let mut documents = self.store.documents(project)?;
        documents.sort_by_key(|d| (d.title.to_lowercase(), d.title.clone()));

        let read = vec![(
            job_id,
            setup_id,
            self.jobs.pieces(job_id)?,
            self.jobs.corrections(job_id)?,
        )];

        let mut headings = vec!["document".to_string(), "file".to_string()];
        for field in fields {
            headings.push(field.label.trim().to_string());
            if field.reason {
                headings.push(format!("{} (reason)", field.label.trim()));
            }
        }
        headings.push("tags".into());

        let mut rows = Vec::new();
        for document in &documents {
            let mut found = None;
            for (job, setup, pieces, corrections) in &read {
                let Some(link) = links
                    .iter()
                    .find(|l| l.document_id == document.id && l.job_id == *job)
                else {
                    continue;
                };
                if let Some(answers) =
                    answers_of(self, pieces, corrections, *setup, link.source_id)?
                {
                    found = Some(answers);
                    break;
                }
            }
            let Some(answers) = found else {
                continue;
            };
            let mut row = vec![document.title.clone(), document.source.file_name.clone()];
            for field in fields {
                let answer = answers.iter().find(|a| a.key == field.key);
                row.push(answer.map(cell).unwrap_or_default());
                if field.reason {
                    row.push(answer.and_then(|a| a.reason.clone()).unwrap_or_default());
                }
            }
            row.push(document.tags.join("; "));
            rows.push(row);
        }
        Ok(Table {
            headings,
            rows,
            cells: CsvCells::Guarded,
        })
    }
}

/// An answer as one cell: a field that cites the text as its quotations, one per line, each
/// after its entry's first field when it has one and before where it stands (`page 2`, or a
/// speaker and a time); any other as on one line.
fn cell(answer: &FieldAnswer) -> String {
    let FieldValue::Cited(value) = &answer.value else {
        return answer.value.joined();
    };
    let mut lines = Vec::new();
    for entry in &value.entries {
        let name = entry.answers.first().map(|a| a.value.joined());
        for evidence in &entry.evidence {
            let mut line = match &name {
                Some(name) if !name.trim().is_empty() => format!("{name}: \"{}\"", evidence.quote),
                _ => format!("\"{}\"", evidence.quote),
            };
            let by = cited::said_by(evidence);
            if !by.is_empty() {
                line.push_str(&format!(" ({by})"));
            }
            lines.push(line);
        }
    }
    lines.join("\n")
}
