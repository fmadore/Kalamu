//! Failures on their way to the interface, as in Live Translation: the core names the failure
//! with a stable id, the interface words it from its translation files. The detail is the
//! untranslated technical text (a provider's message, a path), shown after the sentence.

use kalamu_core::{ProviderError, WorkflowError};
use kalamu_engine::EngineError;
use kalamu_prompts::PromptError;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[ts(export)]
pub struct AppError {
    /// `job.not_found`, `provider.auth`…; the interface looks up the sentence.
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub detail: Option<String>,
    /// The step of a workflow at fault, numbered from 1: the interface names it before the
    /// sentence, and clears the message once that step is changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub step: Option<u32>,
}

impl AppError {
    pub fn new(id: &str, detail: impl std::fmt::Display) -> Self {
        Self {
            id: id.to_string(),
            detail: Some(format!("{detail:#}")),
            step: None,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(d) => write!(f, "{}: {d}", self.id),
            None => f.write_str(&self.id),
        }
    }
}

impl From<EngineError> for AppError {
    fn from(e: EngineError) -> Self {
        match e {
            // The step is the interface's to word; the detail is what went wrong in it.
            EngineError::Step { step, source } => Self {
                step: Some(step),
                ..Self::from(*source)
            },
            EngineError::Workflow(e) | EngineError::Prompt(PromptError::Workflow(e)) => e.into(),
            e => Self::new(e.code(), e),
        }
    }
}

/// A workflow refused: the sentence and the step say it all, so no English detail repeats them.
impl From<WorkflowError> for AppError {
    fn from(e: WorkflowError) -> Self {
        Self {
            id: e.code().to_string(),
            detail: None,
            step: e.step(),
        }
    }
}

impl From<ProviderError> for AppError {
    fn from(e: ProviderError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<kalamu_offline::OfflineError> for AppError {
    fn from(e: kalamu_offline::OfflineError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<PromptError> for AppError {
    fn from(e: PromptError) -> Self {
        match e {
            PromptError::Workflow(e) => e.into(),
            e => Self::new(e.code(), e),
        }
    }
}

impl From<kalamu_core::ProjectError> for AppError {
    fn from(e: kalamu_core::ProjectError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<kalamu_core::DictionaryError> for AppError {
    fn from(e: kalamu_core::DictionaryError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<kalamu_search::SearchError> for AppError {
    fn from(e: kalamu_search::SearchError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<kalamu_catalog::CatalogError> for AppError {
    fn from(e: kalamu_catalog::CatalogError) -> Self {
        Self::new("catalog.invalid", e)
    }
}

/// Every id the app can send besides the engine's and providers' own; the front end has a test
/// that each has a sentence.
pub mod id {
    pub const KEYCHAIN: &str = "keys.keychain";
    pub const KEY_FORMAT: &str = "keys.format";
    pub const SETTINGS: &str = "settings.write";
    pub const CATALOG_DOWNLOAD: &str = "catalog.download";
    pub const FILE: &str = "file.read";
    /// A result file that could not be opened: moved, deleted, or no app for it.
    pub const OPEN: &str = "file.open";
    /// A job's source file that is no longer where it was added, so its pages cannot be shown.
    pub const FILE_MISSING: &str = "file.missing";
    pub const INTERNAL: &str = "app.internal";
    /// A YouTube video that is private, so no one outside its owner can read it.
    pub const SOURCE_PRIVATE: &str = "source.private";
    /// A YouTube video that was removed, or a link to none.
    pub const SOURCE_MISSING: &str = "source.not_found";
    /// Work sent to a cloud provider the researcher has not agreed to send their files to.
    pub const CONSENT: &str = "provider.consent";
    /// Work that waits until no job runs, such as clearing unused results.
    pub const JOBS_RUNNING: &str = "app.jobs_running";
}

pub type CmdResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refused_workflow_names_its_step_once() {
        let refused: AppError = PromptError::Workflow(WorkflowError::NoLanguage { step: 2 }).into();
        assert_eq!(
            refused,
            AppError {
                id: "job.no_language".into(),
                detail: None,
                step: Some(2),
            }
        );
    }

    #[test]
    fn a_step_that_cannot_run_keeps_its_detail() {
        let refused: AppError = EngineError::Step {
            step: 3,
            source: Box::new(EngineError::MissingKey("gemini".into())),
        }
        .into();
        assert_eq!(refused.id, "provider.missing_key");
        assert_eq!(refused.step, Some(3));
        assert_eq!(
            refused.detail.as_deref(),
            Some("No API key saved for gemini")
        );
    }
}
