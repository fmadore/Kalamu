//! Failures on their way to the interface, as in Live Translation: the core names the failure
//! with a stable id, the interface words it from its translation files. The detail is the
//! untranslated technical text (a provider's message, a path), shown after the sentence.

use kalamu_core::ProviderError;
use kalamu_engine::EngineError;
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
}

impl AppError {
    pub fn new(id: &str, detail: impl std::fmt::Display) -> Self {
        Self {
            id: id.to_string(),
            detail: Some(format!("{detail:#}")),
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
        Self::new(e.code(), e)
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

impl From<kalamu_prompts::PromptError> for AppError {
    fn from(e: kalamu_prompts::PromptError) -> Self {
        use kalamu_core::SchemaError;
        use kalamu_prompts::PromptError;
        let id = match &e {
            PromptError::Schema(SchemaError::Empty) => "prompt.no_fields",
            PromptError::Schema(SchemaError::Unnamed) => "prompt.unnamed_field",
            PromptError::Schema(SchemaError::Duplicate(_)) => "prompt.duplicate_field",
            PromptError::Schema(SchemaError::TooFewValues(_)) => "prompt.few_values",
            PromptError::IncompletePipeline => "prompt.unnamed_pipeline",
            PromptError::InvalidImport(_) => "prompt.invalid_file",
            _ => "prompt.invalid",
        };
        Self::new(id, e)
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
    pub const INTERNAL: &str = "app.internal";
    /// A YouTube video that is private, so no one outside its owner can read it.
    pub const SOURCE_PRIVATE: &str = "source.private";
    /// A YouTube video that was removed, or a link to none.
    pub const SOURCE_MISSING: &str = "source.not_found";
}

pub type CmdResult<T> = Result<T, AppError>;
