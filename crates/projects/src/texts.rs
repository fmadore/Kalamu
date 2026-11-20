//! Which text each chosen document is read in, for an analysis of a project's documents (the
//! screening of #51): the corrected text of the result read from it, preferred or the only one,
//! or the document's own words when it is a text file nothing has read into text. The engine
//! then freezes each result's text as it does for *Use this text*; this crate only chooses, and
//! never chooses between two readings the researcher has not.

use kalamu_core::{
    DocumentId, InputKind, JobId, ProjectError, ProjectId, SetupId, SourceFile, SourceId, Task,
};

use crate::{Projects, missing};

/// The tasks whose text a document is read in, in the order they are tried: a correction is the
/// researcher's best text of a page, then the page or the recording read. A translation is not:
/// a passage quoted from it would not be the document's own words.
pub const TEXT_TASKS: [Task; 3] = [
    Task::TextCorrection,
    Task::TextExtraction,
    Task::AudioTranscription,
];

/// What a document is read in.
#[derive(Debug, Clone, PartialEq)]
pub enum TextOf {
    /// A result's corrected text, to be frozen before it is sent.
    Result {
        job_id: JobId,
        setup_id: SetupId,
        source_id: SourceId,
    },
    /// The document is a text file that no task has read: its own words, sent as they are.
    Own(Box<SourceFile>),
    /// Several results of the first task that has any, none preferred: the researcher chooses
    /// one in the document first.
    Undecided,
    /// Nothing has been read into text from it yet.
    Nothing,
}

/// One chosen document and what it is read in.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentText {
    pub document: DocumentId,
    pub title: String,
    pub text: TextOf,
}

impl Projects {
    /// What each of these documents of a project is read in, in the order given. A document not
    /// in the project is refused, as it is everywhere a selection is sent.
    pub fn document_texts(
        &self,
        project: ProjectId,
        ids: &[DocumentId],
    ) -> Result<Vec<DocumentText>, ProjectError> {
        self.store.project(project).map_err(missing)?;
        let all = self.store.documents(project)?;
        let results = self.results_of(project, ids)?;
        ids.iter()
            .map(|id| {
                let document = all
                    .iter()
                    .find(|d| d.id == *id)
                    .ok_or(ProjectError::NotFound)?;
                let mut text = TextOf::Nothing;
                for task in TEXT_TASKS {
                    match results.text_result(*id, task) {
                        Ok((job_id, setup_id, source_id)) => {
                            text = TextOf::Result {
                                job_id,
                                setup_id,
                                source_id,
                            };
                            break;
                        }
                        Err(true) => {
                            text = TextOf::Undecided;
                            break;
                        }
                        Err(false) => {}
                    }
                }
                if text == TextOf::Nothing && document.source.kind == InputKind::Text {
                    text = TextOf::Own(Box::new(document.source.clone()));
                }
                Ok(DocumentText {
                    document: document.id,
                    title: document.title.clone(),
                    text,
                })
            })
            .collect()
    }
}
