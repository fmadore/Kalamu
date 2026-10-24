use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

use crate::{LanguageChoice, PieceRef, ProcessOutput, ProviderId, SetupOptions, Task};

/// An API key, kept out of `Debug` output and logs.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The key itself, for the one request header that needs it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

/// One piece converted to a form the provider accepts.
#[derive(Debug, Clone)]
pub struct PreparedInput {
    pub piece: PieceRef,
    /// `application/pdf`, `image/png`, `audio/mpeg`…
    pub mime: String,
    pub bytes: Vec<u8>,
    /// File name to give the provider where it asks for one.
    pub file_name: String,
}

/// Everything a provider needs for one call.
#[derive(Debug)]
pub struct ProcessRequest<'a> {
    pub task: Task,
    pub model: &'a str,
    pub prompt: &'a str,
    pub input: &'a PreparedInput,
    pub languages: &'a LanguageChoice,
    pub options: &'a SetupOptions,
}

/// Why a call failed, sorted by what the engine should do about it.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ProviderError {
    #[error("The API key was rejected: {0}")]
    Auth(String),
    #[error("The provider's rate limit was reached: {message}")]
    RateLimited {
        retry_after: Option<Duration>,
        message: String,
    },
    #[error("Temporary problem reaching the provider: {0}")]
    Transient(String),
    #[error("The provider refused to return this text: {0}")]
    Blocked(String),
    #[error("The provider rejected the request: {0}")]
    InvalidRequest(String),
    #[error("Not supported: {0}")]
    Unsupported(String),
    #[error("Unexpected response from the provider: {0}")]
    Other(String),
}

impl ProviderError {
    /// Worth trying again after a delay.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Transient(_))
    }

    /// The failure will not change on retry and affects every piece (stop the job).
    pub fn is_fatal_for_job(&self) -> bool {
        matches!(self, Self::Auth(_))
    }
}

/// A cloud provider or offline engine. The engine only knows providers through this trait and
/// never matches on a provider's name.
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;

    /// MIME types this provider accepts for a task, in order of preference. Media preparation
    /// converts each piece into the first one it can produce.
    fn accepted_inputs(&self, task: Task) -> &'static [&'static str];

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError>;

    /// Model ids the key can use, for discovering models not yet in the catalogue. Also used to
    /// check that a key works.
    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError>;
}
