//! One analysis's answers over a project's documents as a table for a spreadsheet, one row per
//! document, with the researcher's tags beside the model's answers (the screening table of
//! #51). An analysis whose question was asked again of documents added since is written with
//! the answers of every analysis that asked it alike, a document's newest answer counting, as
//! the table of documents shows them (`answered`).

use std::collections::HashSet;

use kalamu_core::analysis::cited;
use kalamu_core::{
    CsvCells, Field, FieldAnswer, FieldValue, Job, JobId, ProjectError, ProjectId, SetupId,
    StoreError, Table, Task,
};

use crate::answered::{alike, answers_of};
use crate::{Projects, missing};

impl Projects {
    /// One analysis's answers over the project's documents as a table for a spreadsheet: the
    /// document's title and file, each of its fields under the label the job froze (a choice's
    /// reason beside it when it asks for one, a field that cites the text as its quotations
    /// with where each stands), then the researcher's tags. One row per document it read, in
    /// title order, with the documents the same question was asked of again; a document none
    /// could read is left out. Guarded: the cells hold what a model wrote.
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

        // The analyses read, newest first: this one, and those that asked its question alike.
        // Every job looked at is remembered, an analysis or not, so each is read once however
        // many documents link to it.
        let mut jobs: Vec<Job> = vec![job.clone()];
        let mut seen: HashSet<JobId> = HashSet::from([job.id]);
        for link in &links {
            if !seen.insert(link.job_id) {
                continue;
            }
            match self.jobs.job(link.job_id) {
                Ok(other) if other.task == Task::Analysis => jobs.push(other),
                Ok(_) | Err(StoreError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
        }
        jobs.sort_by_key(|j| std::cmp::Reverse(j.created_at));
        let mut read = Vec::new();
        for other in &jobs {
            for s in &other.setups {
                if s.id == setup_id || alike(s, setup) {
                    read.push((
                        other.id,
                        s.id,
                        self.jobs.pieces(other.id)?,
                        self.jobs.corrections(other.id)?,
                    ));
                }
            }
        }

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
