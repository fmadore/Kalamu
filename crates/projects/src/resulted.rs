//! A document's result for a task, as an export profile's columns read it (#55): the one the
//! researcher preferred or, where none is preferred, the only one there is. With several and
//! none preferred, none: the app never chooses between readings. Read from the job store with
//! the researcher's corrections and speakers' names applied; nothing is read again.

use std::collections::{BTreeMap, HashMap, HashSet};

use kalamu_core::speakers::rename_text;
use kalamu_core::{
    Correction, DocumentId, DocumentLink, FieldValue, Job, JobId, JobStore, Labels, PieceState,
    PreferredResult, ProfileValue, ProjectError, ProjectId, SetupId, SourceId, StoreError, Task,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Projects;
use crate::profiled::Cell;

/// A job read once for every document of an export.
struct JobRead {
    job: Job,
    pieces: Vec<PieceState>,
    corrections: Vec<Correction>,
    /// The researcher's names for each recording's speakers, by set-up and source.
    names: HashMap<(SetupId, SourceId), BTreeMap<String, String>>,
}

/// What an export's documents were read into, gathered once.
pub(crate) struct Results {
    jobs: HashMap<JobId, JobRead>,
    links: Vec<DocumentLink>,
    preferred: Vec<PreferredResult>,
}

/// One result: a set-up of a job over the document's source.
#[derive(Clone, Copy)]
struct Chosen<'a> {
    read: &'a JobRead,
    setup_id: SetupId,
    source_id: SourceId,
}

impl Chosen<'_> {
    /// Its pages or segments, in reading order.
    fn pieces(&self) -> Vec<&PieceState> {
        let mut out: Vec<&PieceState> = self
            .read
            .pieces
            .iter()
            .filter(|p| p.setup_id == self.setup_id && p.source_id == self.source_id)
            .collect();
        out.sort_by_key(|p| p.index);
        out
    }

    fn status(&self) -> &'static str {
        let pieces = self.pieces();
        let reviewed = pieces
            .iter()
            .filter(|p| Correction::of(&self.read.corrections, p).is_some_and(|c| c.reviewed))
            .count();
        if reviewed == 0 {
            "not_reviewed"
        } else if reviewed == pieces.len() {
            "reviewed"
        } else {
            "partly_reviewed"
        }
    }

    fn model(&self) -> String {
        self.read
            .job
            .setups
            .iter()
            .find(|s| s.id == self.setup_id)
            .map(|s| s.model.clone())
            .unwrap_or_default()
    }

    /// The text of the pieces read, corrected, apart by a blank line.
    fn text(&self, jobs: &dyn JobStore) -> Result<String, StoreError> {
        let names = self
            .read
            .names
            .get(&(self.setup_id, self.source_id))
            .cloned()
            .unwrap_or_default();
        let mut parts = Vec::new();
        for piece in self.pieces() {
            let corrected =
                Correction::of(&self.read.corrections, piece).and_then(|c| c.text.clone());
            let text = match corrected {
                Some(text) => text,
                None => match jobs.finished_result(piece)? {
                    Some(result) => result.output.text,
                    None => continue,
                },
            };
            let text = rename_text(&text, &names);
            if !text.trim().is_empty() {
                parts.push(text.trim().to_string());
            }
        }
        Ok(parts.join("\n\n"))
    }

    /// The answers given to a field, corrected, from every piece read.
    fn answers(&self, jobs: &dyn JobStore, key: &str) -> Result<Vec<FieldValue>, StoreError> {
        let mut out = Vec::new();
        for piece in self.pieces() {
            if let Some(answers) = jobs.answers(piece, &self.read.corrections)? {
                out.extend(
                    answers
                        .into_iter()
                        .filter(|a| a.key == key)
                        .map(|a| a.value),
                );
            }
        }
        Ok(out)
    }
}

/// An answer's values as one cell: a list when any is a list (entries by their first field),
/// else the texts apart by a blank line.
fn answer_cell(values: Vec<FieldValue>) -> Cell {
    let mut texts = Vec::new();
    let mut items = Vec::new();
    let mut listed = false;
    for value in values {
        match value {
            FieldValue::Text(t) => {
                if !t.trim().is_empty() {
                    texts.push(t.trim().to_string());
                }
            }
            FieldValue::List(list) => {
                listed = true;
                items.extend(list);
            }
            FieldValue::Entries(entries) => {
                listed = true;
                items.extend(
                    entries
                        .iter()
                        .filter_map(|e| e.first().map(|a| a.value.joined())),
                );
            }
        }
    }
    if listed {
        items.extend(texts);
        Cell::list(items)
    } else {
        Cell::text(texts.join("\n\n"))
    }
}

impl Results {
    /// The result a profile's columns read for a document and a task.
    fn chosen(&self, document: DocumentId, task: Task) -> Option<Chosen<'_>> {
        let mut seen = HashSet::new();
        let candidates: Vec<Chosen> = self
            .links
            .iter()
            .filter(|l| l.document_id == document)
            .filter_map(|l| self.jobs.get(&l.job_id).map(|read| (l, read)))
            .filter(|(_, read)| read.job.task == task)
            .flat_map(|(l, read)| {
                read.job.setups.iter().map(move |s| Chosen {
                    read,
                    setup_id: s.id,
                    source_id: l.source_id,
                })
            })
            .filter(|c| seen.insert((c.read.job.id, c.setup_id)))
            .filter(|c| c.pieces().iter().any(|p| p.status.is_finished()))
            .collect();
        let preferred = self
            .preferred
            .iter()
            .find(|p| p.document_id == document && p.task == task);
        match preferred {
            Some(p) => candidates
                .into_iter()
                .find(|c| c.read.job.id == p.job_id && c.setup_id == p.setup_id),
            None => match candidates.as_slice() {
                [only] => Some(*only),
                _ => None,
            },
        }
    }
}

impl Projects {
    /// The results of these documents of a project, read once for an export.
    pub(crate) fn results_of(
        &self,
        project: ProjectId,
        documents: &[DocumentId],
    ) -> Result<Results, ProjectError> {
        let links: Vec<DocumentLink> = self
            .store
            .links(project)?
            .into_iter()
            .filter(|l| documents.contains(&l.document_id))
            .collect();
        let mut jobs = HashMap::new();
        for link in &links {
            if jobs.contains_key(&link.job_id) {
                continue;
            }
            let job = match self.jobs.job(link.job_id) {
                Ok(job) => job,
                Err(StoreError::NotFound) => continue,
                Err(e) => return Err(e.into()),
            };
            let read = JobRead {
                pieces: self.jobs.pieces(job.id)?,
                corrections: self.jobs.corrections(job.id)?,
                names: self
                    .jobs
                    .speaker_names(job.id)?
                    .into_iter()
                    .map(|n| ((n.setup_id, n.source_id), n.names))
                    .collect(),
                job,
            };
            jobs.insert(link.job_id, read);
        }
        Ok(Results {
            jobs,
            links,
            preferred: self.store.preferred(project)?,
        })
    }

    /// A result column's cell for a document; `None` for a column that reads no result.
    pub(crate) fn result_cell(
        &self,
        results: &Results,
        value: &ProfileValue,
        document: DocumentId,
        labels: &Labels,
    ) -> Result<Option<Cell>, ProjectError> {
        let task = match value {
            ProfileValue::Status { task }
            | ProfileValue::Model { task }
            | ProfileValue::Text { task } => *task,
            ProfileValue::Answer { .. } | ProfileValue::Labelled { .. } => Task::Analysis,
            _ => return Ok(None),
        };
        let Some(chosen) = results.chosen(document, task) else {
            return Ok(Some(Cell::Missing));
        };
        let jobs = self.jobs.as_ref();
        Ok(Some(match value {
            ProfileValue::Status { .. } => Cell::text(chosen.status().to_string()),
            ProfileValue::Model { .. } => Cell::text(chosen.model()),
            ProfileValue::Text { .. } => Cell::text(chosen.text(jobs)?),
            ProfileValue::Answer { key } => answer_cell(chosen.answers(jobs, key)?),
            ProfileValue::Labelled { key } => match answer_cell(chosen.answers(jobs, key)?) {
                Cell::List(items) => Cell::list(labels.applied(key, &items)),
                other => other,
            },
            _ => Cell::Missing,
        }))
    }
}

/// A field the project's analyses answered, for a profile's column to name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnswerField {
    pub key: String,
    /// What the most recent analysis asking it calls it.
    pub label: String,
}

impl Projects {
    /// The fields the analyses tied to a project's documents asked, by key, in the order of
    /// their labels.
    pub fn answer_fields(&self, project: ProjectId) -> Result<Vec<AnswerField>, ProjectError> {
        self.store.project(project).map_err(crate::missing)?;
        let mut jobs: Vec<JobId> = self
            .store
            .links(project)?
            .into_iter()
            .map(|l| l.job_id)
            .collect();
        jobs.sort();
        jobs.dedup();
        let mut fields: HashMap<String, (kalamu_core::Timestamp, String)> = HashMap::new();
        for id in jobs {
            let job = match self.jobs.job(id) {
                Ok(job) => job,
                Err(StoreError::NotFound) => continue,
                Err(e) => return Err(e.into()),
            };
            if job.task != Task::Analysis {
                continue;
            }
            for schema in job.setups.iter().filter_map(|s| s.prompt.schema.as_ref()) {
                for field in &schema.fields {
                    let known = fields.get(&field.key);
                    if known.is_none_or(|(at, _)| *at <= job.created_at) {
                        fields.insert(field.key.clone(), (job.created_at, field.label.clone()));
                    }
                }
            }
        }
        let mut out: Vec<AnswerField> = fields
            .into_iter()
            .map(|(key, (_, label))| AnswerField { key, label })
            .collect();
        out.sort_by_key(|f| (f.label.to_lowercase(), f.key.clone()));
        Ok(out)
    }
}
