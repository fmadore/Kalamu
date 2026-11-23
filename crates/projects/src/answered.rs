//! What a project's analyses answered, by document: the choices of each analysis that has any
//! (whether a document speaks of the researcher's question, a document's tone), for the table of
//! documents to be filtered by them and to show, in each row, the answer with its reason and its
//! passages; and the documents a question has not been asked of yet, for it to be asked again of
//! them alone (the screening of #51). One analysis's answers as a table are in `tabled`.
//!
//! Analyses that asked the same question with the same prompt and model are one question kept
//! with the project: their answers stand together, a document's newest answer counting, so asking
//! the question again of the documents added since adds to it. The same prompt is the prompt as
//! each froze it, not only its preset: once a preset is edited, a question asked again under it
//! is a group of its own (#241). The question is kept by the jobs
//! themselves (their set-up's `question`), so no table holds it twice.
//!
//! Read from the job store as it stands, the researcher's corrections applied, never stored
//! twice and never by a preset's name: any analysis with a choice is offered. Nothing here calls
//! a model or changes a result.

use std::collections::HashMap;

use kalamu_core::analysis::cited::Evidence;
use kalamu_core::{
    Correction, DocumentId, DocumentLink, Field, FieldAnswer, FieldKind, FieldValue, Job, JobId,
    LanguageChoice, PieceState, ProjectError, ProjectId, ProviderId, Setup, SetupId, SourceId,
    StoreError, Task, Timestamp,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Projects;
use crate::texts::TextOf;

/// One choice field of the analyses that asked a question alike (one job's set-up, or several
/// that asked the same question with the same prompt and model), and the answer given each
/// document of the project, as corrected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChoiceAnswers {
    /// The newest of the analyses: its name, its field, and the one its table is asked by.
    pub job_id: JobId,
    pub setup_id: SetupId,
    pub job_name: String,
    /// The prompt it ran with, by which the interface shows a default's choices in its
    /// language.
    pub preset_id: Option<String>,
    /// The researcher's question the analysis was asked with, when there was one.
    pub question: Option<String>,
    /// The model it was read with, and the languages of its answers, for the question to be
    /// asked again alike.
    pub provider: ProviderId,
    pub model: String,
    pub languages: LanguageChoice,
    /// The fingerprint of the prompt every analysis of the group froze
    /// ([`kalamu_core::PromptSnapshot::fingerprint`]): its instructions, its request and its
    /// fields. A question asked again under a prompt edited since is another group.
    pub prompt: String,
    /// When the oldest analysis of the group was made: what tells apart two groups of the same
    /// question asked under two versions of its prompt.
    pub first_asked: Timestamp,
    /// The field as the job froze it.
    pub field: Field,
    pub answers: Vec<DocumentAnswer>,
    /// For a question, the project's documents it has not been answered for yet that have a text
    /// to be read in: those added since, or one whose reading failed. Empty for an analysis
    /// asked no question.
    pub unread: Vec<DocumentId>,
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

/// Whether two set-ups asked the same question alike: the same question, preset, provider and
/// model, and the same prompt as each froze it ([`kalamu_core::PromptSnapshot::fingerprint`]):
/// a preset edited between two runs keeps its id but not its instructions or its fields, and
/// answers read under two wordings never stand together. Set-ups asked no question are never
/// alike: each analysis stands on its own.
pub(crate) fn alike(a: &Setup, b: &Setup) -> bool {
    question_of(a).is_some()
        && question_of(a) == question_of(b)
        && a.prompt.preset_id == b.prompt.preset_id
        && a.provider == b.provider
        && a.model == b.model
        && a.prompt.fingerprint() == b.prompt.fingerprint()
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
    /// answered, newest job first; an analysis with no choice gives none. Analyses that asked a
    /// question alike are one group, a document's newest answer counting.
    pub(crate) fn choice_answers(
        &self,
        project: ProjectId,
        loaded: &LoadedJobs,
        links: &[DocumentLink],
    ) -> Result<Vec<ChoiceAnswers>, ProjectError> {
        let mut jobs: Vec<&(Job, Vec<PieceState>, Vec<Correction>)> = loaded
            .values()
            .filter(|(job, _, _)| job.task == Task::Analysis)
            .collect();
        jobs.sort_by_key(|(job, _, _)| std::cmp::Reverse(job.created_at));
        // Each group with the set-up that heads it.
        let mut out: Vec<(ChoiceAnswers, &Setup)> = Vec::new();
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
                    // Asked alike before? The older answers fill in what the newer left.
                    if let Some((group, _)) = out
                        .iter_mut()
                        .find(|(g, head)| alike(head, setup) && g.field.key == field.key)
                    {
                        // Jobs come newest first: this one is the oldest yet.
                        group.first_asked = job.created_at;
                        for answer in answers {
                            if !group.answers.iter().any(|a| a.document == answer.document) {
                                group.answers.push(answer);
                            }
                        }
                        continue;
                    }
                    out.push((
                        ChoiceAnswers {
                            job_id: job.id,
                            setup_id: setup.id,
                            job_name: job.name.clone(),
                            preset_id: setup.prompt.preset_id.clone(),
                            question: question_of(setup),
                            provider: setup.provider.clone(),
                            model: setup.model.clone(),
                            languages: job.settings.languages.clone(),
                            prompt: setup.prompt.fingerprint(),
                            first_asked: job.created_at,
                            field: field.clone(),
                            answers,
                            unread: Vec::new(),
                        },
                        setup,
                    ));
                }
            }
        }
        let mut out: Vec<ChoiceAnswers> = out.into_iter().map(|(g, _)| g).collect();
        self.fill_unread(project, &mut out)?;
        Ok(out)
    }

    /// For each group asked a question, the project's documents it has no answer for that have
    /// a text to be read in, as the screening dialog would read them.
    fn fill_unread(
        &self,
        project: ProjectId,
        groups: &mut [ChoiceAnswers],
    ) -> Result<(), ProjectError> {
        if !groups.iter().any(|g| g.question.is_some()) {
            return Ok(());
        }
        let documents: Vec<DocumentId> = self
            .store
            .documents(project)?
            .into_iter()
            .map(|d| d.id)
            .collect();
        let unanswered: Vec<DocumentId> = documents
            .iter()
            .copied()
            .filter(|d| {
                groups
                    .iter()
                    .any(|g| g.question.is_some() && !g.answers.iter().any(|a| a.document == *d))
            })
            .collect();
        if unanswered.is_empty() {
            return Ok(());
        }
        let readable: Vec<DocumentId> = self
            .document_texts(project, &unanswered)?
            .into_iter()
            .filter(|t| matches!(t.text, TextOf::Result { .. } | TextOf::Own(_)))
            .map(|t| t.document)
            .collect();
        for group in groups.iter_mut().filter(|g| g.question.is_some()) {
            group.unread = readable
                .iter()
                .copied()
                .filter(|d| !group.answers.iter().any(|a| a.document == *d))
                .collect();
        }
        Ok(())
    }
}
