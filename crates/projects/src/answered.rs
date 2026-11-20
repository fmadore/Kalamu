//! What a project's analyses answered, by document: the choices of each analysis that has any
//! (whether a document speaks of the researcher's question, a document's tone), for the table of
//! documents to be filtered by them; and one analysis's answers as a table, one row per
//! document, with the researcher's tags beside the model's answers (the screening table of #51).
//!
//! Read from the job store as it stands, the researcher's corrections applied, never stored
//! twice and never by a preset's name: any analysis with a choice is offered, and any analysis
//! can be written as a table. Nothing here calls a model or changes a result.

use std::collections::HashMap;

use kalamu_core::analysis::cited;
use kalamu_core::{
    Correction, CsvCells, DocumentId, DocumentLink, Field, FieldAnswer, FieldKind, FieldValue, Job,
    JobId, PieceState, ProjectError, ProjectId, SetupId, SourceId, StoreError, Table, Task,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Projects, missing};

/// One choice field of one analysis (a job's set-up), and the answer it gave each document of
/// the project it read, as corrected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChoiceAnswers {
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub job_name: String,
    /// The prompt it ran with, by which the interface shows a default's choices in its
    /// language.
    pub preset_id: Option<String>,
    /// The researcher's question the analysis was asked with, when there was one.
    pub question: Option<String>,
    /// The field as the job froze it.
    pub field: Field,
    pub answers: Vec<DocumentAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentAnswer {
    pub document: DocumentId,
    pub value: String,
}

/// The jobs a project's links name, each with its pieces and corrections.
pub(crate) type LoadedJobs = HashMap<JobId, (Job, Vec<PieceState>, Vec<Correction>)>;

/// The answers of a set-up over one source, corrected: those of its first piece read (an
/// analysis reads a document whole, in one piece).
fn answers_of(
    projects: &Projects,
    pieces: &[PieceState],
    corrections: &[Correction],
    setup: SetupId,
    source: SourceId,
) -> Result<Option<Vec<FieldAnswer>>, StoreError> {
    let mut mine: Vec<&PieceState> = pieces
        .iter()
        .filter(|p| p.setup_id == setup && p.source_id == source && p.status.is_finished())
        .collect();
    mine.sort_by_key(|p| p.index);
    for piece in mine {
        if let Some(answers) = projects.jobs.answers(piece, corrections)? {
            return Ok(Some(answers));
        }
    }
    Ok(None)
}

/// A set-up's top-level choice fields.
fn choices(fields: &[Field]) -> impl Iterator<Item = &Field> {
    fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::Choice { .. }))
}

impl Projects {
    /// Every choice of the analyses tied to the project's documents, each with the documents it
    /// answered, newest job first; an analysis with no choice gives none.
    pub(crate) fn choice_answers(
        &self,
        loaded: &LoadedJobs,
        links: &[DocumentLink],
    ) -> Result<Vec<ChoiceAnswers>, ProjectError> {
        let mut jobs: Vec<&(Job, Vec<PieceState>, Vec<Correction>)> = loaded
            .values()
            .filter(|(job, _, _)| job.task == Task::Analysis)
            .collect();
        jobs.sort_by_key(|(job, _, _)| std::cmp::Reverse(job.created_at));
        let mut out = Vec::new();
        for (job, pieces, corrections) in jobs {
            for setup in &job.setups {
                let Some(schema) = &setup.prompt.schema else {
                    continue;
                };
                let fields: Vec<&Field> = choices(&schema.fields).collect();
                if fields.is_empty() {
                    continue;
                }
                let mut groups: Vec<Vec<DocumentAnswer>> = vec![Vec::new(); fields.len()];
                for link in links.iter().filter(|l| l.job_id == job.id) {
                    let Some(answers) =
                        answers_of(self, pieces, corrections, setup.id, link.source_id)?
                    else {
                        continue;
                    };
                    for (field, group) in fields.iter().zip(&mut groups) {
                        let value = answers
                            .iter()
                            .find(|a| a.key == field.key)
                            .map(|a| a.value.joined())
                            .unwrap_or_default();
                        if !value.trim().is_empty() {
                            group.push(DocumentAnswer {
                                document: link.document_id,
                                value,
                            });
                        }
                    }
                }
                for (field, answers) in fields.into_iter().zip(groups) {
                    if answers.is_empty() {
                        continue;
                    }
                    out.push(ChoiceAnswers {
                        job_id: job.id,
                        setup_id: setup.id,
                        job_name: job.name.clone(),
                        preset_id: setup.prompt.preset_id.clone(),
                        question: setup
                            .options
                            .question
                            .as_deref()
                            .map(str::trim)
                            .filter(|q| !q.is_empty())
                            .map(String::from),
                        field: field.clone(),
                        answers,
                    });
                }
            }
        }
        Ok(out)
    }

    /// One analysis's answers over the project's documents as a table for a spreadsheet: the
    /// document's title and file, each of its fields under the label the job froze (a choice's
    /// reason beside it when it asks for one, a field that cites the text as its quotations
    /// with where each stands), then the researcher's tags. One row per document it read, in
    /// title order; a document it could not read is left out. Guarded: the cells hold what a
    /// model wrote.
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
        let pieces = self.jobs.pieces(job_id)?;
        let corrections = self.jobs.corrections(job_id)?;
        let links = self.store.links(project)?;
        let mut documents = self.store.documents(project)?;
        documents.sort_by_key(|d| (d.title.to_lowercase(), d.title.clone()));

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
            let Some(link) = links
                .iter()
                .find(|l| l.document_id == document.id && l.job_id == job_id)
            else {
                continue;
            };
            let Some(answers) = answers_of(self, &pieces, &corrections, setup_id, link.source_id)?
            else {
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
