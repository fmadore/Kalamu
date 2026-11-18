//! What can stop the search by meaning, each with the stable id the interface translates.

use kalamu_core::{ProviderError, StoreError};
use kalamu_search::SearchError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeaningError {
    #[error(transparent)]
    Search(#[from] SearchError),
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The project has no model chosen for the search by meaning yet.
    #[error("Choose a model for the search by meaning first")]
    NoModel,
    /// A model not in the list offered (no longer in the catalogue, or its embedder missing).
    #[error("This model is not offered: {0}")]
    UnknownModel(String),
    /// A cloud model the researcher has not agreed to send the project's text to.
    #[error("Agree to send the project's text to {0} first")]
    NoConsent(String),
    #[error("No API key saved for {0}")]
    MissingKey(String),
    /// A model run on this computer not yet downloaded.
    #[error("{0}")]
    NotReady(ProviderError),
    /// A call that failed after its retries, or that cannot succeed (a refused key).
    #[error("{0}")]
    Provider(ProviderError),
    #[error("The project's text is already being prepared")]
    AlreadyRunning,
    /// The question is empty.
    #[error("Type a question or a description")]
    EmptyQuery,
    /// Vectors of another length than the model's, or not one per text.
    #[error("The model's answer could not be read")]
    BadAnswer,
}

impl MeaningError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Search(e) => e.code(),
            Self::Store(StoreError::NotFound) => "project.not_found",
            Self::Store(_) => "job.database",
            Self::NoModel => "meaning.no_model",
            Self::UnknownModel(_) => "meaning.unknown_model",
            Self::NoConsent(_) => "meaning.no_consent",
            Self::MissingKey(_) => "provider.missing_key",
            Self::NotReady(e) | Self::Provider(e) => e.code(),
            Self::AlreadyRunning => "meaning.running",
            Self::EmptyQuery => "meaning.empty_query",
            Self::BadAnswer => "provider.other",
        }
    }
}
