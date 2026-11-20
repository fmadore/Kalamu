//! What a project's analyses answered, by document: the choices of each analysis that has any
//! (whether a document speaks of the researcher's question, a document's tone), for the table of
//! documents to be filtered by them and to show, in each row, the answer with its reason and its
//! passages (the screening of #51). One analysis's answers as a table are in `tabled`.
//!
//! Read from the job store as it stands, the researcher's corrections applied, never stored
//! twice and never by a preset's name: any analysis with a choice is offered. Nothing here calls
//! a model or changes a result.

use std::collections::HashMap;

use kalamu_core::analysis::cited::Evidence;
use kalamu_core::{
    Correction, DocumentId, DocumentLink, Field, FieldAnswer, FieldKind, FieldValue, Job, JobId,
    PieceState, ProjectError, Setup, SetupId, SourceId, StoreError, Task,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Projects;

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

/// One document's answer, with what the row shows beside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DocumentAnswer {
    pub document: DocumentId,
    pub value: String,
    /// The analysis that gave it.
    pub job_id: JobId,
    /// Why: the choice's own reason when it asks for one, else the analysis's first text
    /// answered (a screening's *Reason*); absent when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reason: Option<String>,
    /// The passages the analysis quoted for it, each found in the text with its page, in the
    /// order given.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(optional, as = "Option<Vec<Evidence>>")]
    pub passages: Vec<Evidence>,
}

/// The jobs a project's links name, each with its pieces and corrections.
pub(crate) type LoadedJobs = HashMap<JobId, (Job, Vec<PieceState>, Vec<Correction>)>;

/// The answers of a set-up over one source, corrected: those of its first piece read (an
/// analysis reads a document whole, in one piece; a long one in parts, whose answers stand only
/// through the step that brought them together, after them).
pub(crate) fn answers_of(
    projects: &Projects,
    pieces: &[PieceState],
    corrections: &[Correction],
    setup: SetupId,
    source: SourceId,
) -> Result<Option<Vec<FieldAnswer>>, StoreError> {
    let mut mine: Vec<&PieceState> = pieces
        .iter()
        .filter(|p| {
            p.setup_id == setup
                && p.source_id == source
                && p.status.is_finished()
                && !p.piece.is_window()
        })
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

/// The question a set-up was asked, trimmed; `None` when it was asked none.
pub(crate) fn question_of(setup: &Setup) -> Option<String> {
    setup
        .options
        .question
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(String::from)
}

/// What a row shows beside a choice's answer: its reason, and the passages quoted.
fn beside(
    field: &Field,
    fields: &[Field],
    answers: &[FieldAnswer],
) -> (Option<String>, Vec<Evidence>) {
    let own = answers
        .iter()
        .find(|a| a.key == field.key)
        .and_then(|a| a.reason.clone())
        .filter(|r| field.reason && !r.trim().is_empty());
    let text = || {
        fields
            .iter()
            .filter(|f| matches!(f.kind, FieldKind::Text))
            .find_map(|f| {
                answers
                    .iter()
                    .find(|a| a.key == f.key)
                    .map(|a| a.value.joined())
                    .filter(|t| !t.trim().is_empty())
            })
    };
    let passages = fields
        .iter()
        .filter(|f| matches!(f.kind, FieldKind::Cited { .. }))
        .filter_map(|f| answers.iter().find(|a| a.key == f.key))
        .filter_map(|a| match &a.value {
            FieldValue::Cited(value) => Some(value),
            _ => None,
        })
        .flat_map(|value| {
            value
                .entries
                .iter()
                .flat_map(|e| e.evidence.iter().cloned())
        })
        .collect();
    (own.or_else(text), passages)
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
        let mut out: Vec<ChoiceAnswers> = Vec::new();
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
                        if value.trim().is_empty() {
                            continue;
                        }
                        let (reason, passages) = beside(field, &schema.fields, &answers);
                        group.push(DocumentAnswer {
                            document: link.document_id,
                            value,
                            job_id: job.id,
                            reason,
                            passages,
                        });
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
                        question: question_of(setup),
                        field: field.clone(),
                        answers,
                    });
                }
            }
        }
        Ok(out)
    }
}
