//! Failures on their way to the interface, as in Live Translation: the core names the failure
//! with a stable id, the interface words it from its translation files. The detail is the
//! untranslated technical text (a provider's message, a path), shown after the sentence.

use std::collections::BTreeMap;

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
    /// The values the sentence names, by name (`server`, `host`, `model`): a server of the
    /// researcher's own, the address it was looked for at. The interface puts them in the
    /// sentence it looks up, so no English reaches it.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub params: Option<BTreeMap<String, String>>,
}

impl AppError {
    pub fn new(id: &str, detail: impl std::fmt::Display) -> Self {
        Self {
            id: id.to_string(),
            detail: Some(format!("{detail:#}")),
            step: None,
            params: None,
        }
    }

    /// A failure the interface words whole from `id` and the values `params` gives, with no
    /// English detail; none given leaves `params` out.
    pub fn with(id: &str, params: impl IntoIterator<Item = (&'static str, String)>) -> Self {
        let params: BTreeMap<String, String> = params
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        Self {
            id: id.to_string(),
            detail: None,
            step: None,
            params: (!params.is_empty()).then_some(params),
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
            // A server's id names no provider once the server is removed; saying "unknown
            // provider" would leave the researcher guessing which.
            EngineError::UnknownProvider(provider) if kalamu_catalog::is_own_id(&provider) => {
                Self::with(id::SERVER_REMOVED, [])
            }
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
            params: None,
        }
    }
}

/// An address the researcher typed for a server, refused: the sentence says why.
impl From<kalamu_core::address::AddressError> for AppError {
    fn from(e: kalamu_core::address::AddressError) -> Self {
        Self::with(e.code(), [])
    }
}

/// No server found at an address: the sentence names the host; only a failure the server
/// worded itself carries its words as the detail.
impl From<kalamu_providers::own::DiscoverError> for AppError {
    fn from(e: kalamu_providers::own::DiscoverError) -> Self {
        let mut error = Self::with(e.code(), e.host().map(|host| ("host", host.to_string())));
        if let kalamu_providers::own::DiscoverError::Other { message, .. } = e {
            error.detail = Some(message);
        }
        error
    }
}

/// A server not saved: the sentence names the server, the model or the name at fault.
impl From<kalamu_catalog::DraftError> for AppError {
    fn from(e: kalamu_catalog::DraftError) -> Self {
        Self::with(e.code(), e.params())
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

impl From<kalamu_core::ProjectFileError> for AppError {
    fn from(e: kalamu_core::ProjectFileError) -> Self {
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

impl From<kalamu_engine::MeaningError> for AppError {
    fn from(e: kalamu_engine::MeaningError) -> Self {
        Self::new(e.code(), e)
    }
}

impl From<kalamu_catalog::CatalogError> for AppError {
    fn from(e: kalamu_catalog::CatalogError) -> Self {
        match e {
            kalamu_catalog::CatalogError::Refused(_) => Self::new(id::CATALOG_REFUSED, e),
            e => Self::new("catalog.invalid", e),
        }
    }
}

/// Every id the app can send besides the engine's and providers' own; the front end has a test
/// that each has a sentence.
pub mod id {
    pub const KEYCHAIN: &str = "keys.keychain";
    pub const KEY_FORMAT: &str = "keys.format";
    pub const SETTINGS: &str = "settings.write";
    pub const CATALOG_DOWNLOAD: &str = "catalog.download";
    /// A model list read but failing its checks of sense, or too large: set aside.
    pub const CATALOG_REFUSED: &str = "catalog.refused";
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
    /// The list of the researcher's servers could not be written.
    pub const SERVERS_WRITE: &str = "server.write";
    /// *This server asks for a key* ticked, with no key typed and none saved; names `server`.
    pub const SERVER_KEY_MISSING: &str = "server.key_missing";
    /// A server's key is set in Settings › Other servers, with the server, not among the
    /// providers' keys.
    pub const SERVER_KEY_ELSEWHERE: &str = "server.key_elsewhere";
    /// A trial stopped by the researcher before it ended; names `model`.
    pub const TRIAL_STOPPED: &str = "server.trial_stopped";
    /// A server changed (another address or way of speaking, a model in use unticked) or
    /// removed while a job or workflow run that sends to it runs; names `server`.
    pub const SERVER_IN_USE: &str = "server.in_use";
    /// Work that names a server removed in Settings › Other servers: a server added again is a
    /// new one, under a new id, which the work does not name.
    pub const SERVER_REMOVED: &str = "server.removed";
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
                params: None,
            }
        );
    }

    /// A server's failures reach the interface as ids with their values, never as English.
    #[test]
    fn a_server_s_failures_carry_their_values_not_sentences() {
        use kalamu_providers::own::DiscoverError;

        let nothing: AppError = DiscoverError::NothingAnswers {
            host: "192.168.1.20:11434".into(),
        }
        .into();
        assert_eq!(nothing.id, "server.nothing_answers");
        assert_eq!(nothing.detail, None);
        assert_eq!(
            nothing.params,
            Some(BTreeMap::from([(
                "host".to_string(),
                "192.168.1.20:11434".to_string()
            )]))
        );
        let other: AppError = DiscoverError::Other {
            host: "h".into(),
            message: "TLS handshake failed".into(),
        }
        .into();
        assert_eq!(other.detail.as_deref(), Some("TLS handshake failed"));

        let address: AppError = kalamu_core::address::AddressError::Scheme.into();
        assert_eq!(address.id, "server.address_scheme");
        assert_eq!(address.params, None);
        let json = serde_json::to_value(&address).unwrap_or_default();
        assert_eq!(json, serde_json::json!({ "id": "server.address_scheme" }));

        let untried: AppError = kalamu_catalog::DraftError::NotTried {
            model: "gemma3:12b".into(),
        }
        .into();
        assert_eq!(untried.id, "server.not_tried");
        assert_eq!(
            untried.params,
            Some(BTreeMap::from([(
                "model".to_string(),
                "gemma3:12b".to_string()
            )]))
        );
    }

    #[test]
    fn a_removed_server_is_said_as_removed_and_any_other_provider_as_unknown() {
        let removed: AppError = EngineError::UnknownProvider("own-0000abcd".into()).into();
        assert_eq!(removed, AppError::with(id::SERVER_REMOVED, []));
        let unknown: AppError = EngineError::UnknownProvider("acme".into()).into();
        assert_eq!(unknown.id, "provider.unknown");
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
