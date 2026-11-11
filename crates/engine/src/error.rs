//! The engine's errors, each with the stable id the interface translates.

use kalamu_core::{ProviderError, StoreError};
use kalamu_media::MediaError;
use kalamu_prompts::PromptError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error(transparent)]
    Prompt(#[from] PromptError),
    #[error("Unknown provider: {0}")]
    UnknownProvider(String),
    #[error("No API key saved for {0}")]
    MissingKey(String),
    /// An offline model not downloaded, or a processor that cannot run it.
    #[error("{0}")]
    NotReady(ProviderError),
    #[error("{0} cannot do this task")]
    UnsupportedModel(String),
    /// A model its provider no longer serves, named by an older pipeline or comparison.
    #[error("{0} is no longer served by its provider; choose the model that replaces it")]
    RetiredModel(String),
    /// A model given a source it cannot read: images to a model that reads text only, a whole
    /// PDF to one that reads no PDF. The reason is media's, as each piece would have failed.
    #[error("{model} cannot read {file}: {reason}")]
    CannotRead {
        model: String,
        file: String,
        reason: String,
    },
    /// A source of a kind the task does not read.
    #[error("{0} cannot be read for this task")]
    UnsupportedSource(String),
    #[error("The job is already running")]
    AlreadyRunning,
    #[error("The job needs at least one file and one set-up")]
    EmptyJob,
    /// A translation started without the language to translate into.
    #[error("Choose the language to translate into")]
    NoLanguage,
    /// The link as pasted; the interface says what is wrong with it.
    #[error("{0}")]
    NotYouTube(String),
    /// A YouTube video has no folder of its own to write beside.
    #[error("Choose a folder for the results of a YouTube video")]
    LinkNeedsFolder,
    #[error("Could not write the results: {0}")]
    Output(String),
    /// A table of contents or an analysis has no text for a later task to read.
    #[error("This task's results are not a text to analyse")]
    NotText,
    /// Nothing has been read yet under this set-up.
    #[error("Nothing has been read yet")]
    NothingRead,
    #[error("Could not keep the piece converted to listen to: {0}")]
    ListeningCopy(String),
    #[error(transparent)]
    Workflow(kalamu_core::WorkflowError),
    /// A workflow step that cannot run with these files or models.
    #[error("Step {step}: {source}")]
    Step { step: u32, source: Box<EngineError> },
    /// A workflow step approved before its job has finished.
    #[error("This step has not finished")]
    StepNotFinished,
    #[error(transparent)]
    Unit(#[from] kalamu_core::UnitError),
}

impl EngineError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Store(StoreError::NotFound) => "job.not_found",
            Self::Store(_) => "job.database",
            Self::Media(MediaError::Encrypted) => "media.encrypted",
            Self::Media(MediaError::UnsupportedKind) => "media.unsupported",
            Self::Media(_) => "media.read",
            Self::Prompt(e) => e.code(),
            Self::UnknownProvider(_) => "provider.unknown",
            Self::MissingKey(_) => "provider.missing_key",
            Self::NotReady(e) => e.code(),
            Self::UnsupportedModel(_) => "provider.unsupported",
            Self::RetiredModel(_) => "provider.retired",
            Self::CannotRead { .. } => "provider.cannot_read",
            Self::UnsupportedSource(_) => "source.unsupported",
            Self::AlreadyRunning => "job.running",
            Self::EmptyJob => "job.empty",
            Self::NoLanguage => "job.no_language",
            Self::NotYouTube(_) => "source.not_youtube",
            Self::LinkNeedsFolder => "job.link_needs_folder",
            Self::Output(_) => "output.write",
            Self::NotText => "job.not_text",
            Self::NothingRead => "job.nothing_read",
            Self::ListeningCopy(_) => "media.read",
            Self::Workflow(e) => e.code(),
            Self::Step { source, .. } => source.code(),
            Self::StepNotFinished => "workflow.not_finished",
            Self::Unit(e) => e.code(),
        }
    }
}
