use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    AnalysisSchema, BatchContext, BatchLimits, BatchProgress, LanguageChoice, PieceRef,
    ProcessOutput, ProviderId, SetupOptions, Task,
};

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

/// Raw 16-bit little-endian PCM, 16 kHz, one channel: what offline speech engines read.
pub const PCM_16K_MONO: &str = "audio/x-pcm-s16le-16k";

/// A setting a provider sends with every request for a task and model, as a pipeline's record
/// reports it. `name` is the API parameter's path in the request (`output_config.effort`), so a
/// provider's tests can check that the request really carries `value`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RequestSetting {
    pub name: String,
    pub value: String,
}

impl RequestSetting {
    pub fn new(name: &str, value: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

/// Set by the engine when the user pauses or cancels a job. Cloud calls finish on their own;
/// long local work (an offline transcription) checks it between windows and stops early.
#[derive(Debug, Clone, Default)]
pub struct StopSignal(Arc<AtomicBool>);

impl StopSignal {
    pub fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.0.load(Ordering::Relaxed)
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
    /// Instructions, sent as the system prompt where the provider has one.
    pub prompt: &'a str,
    /// The per-piece request sent alongside the input.
    pub user_prompt: &'a str,
    pub input: &'a PreparedInput,
    pub languages: &'a LanguageChoice,
    pub options: &'a SetupOptions,
    /// For an analysis, the fields the answer must have.
    pub analysis: Option<&'a AnalysisSchema>,
    pub stop: &'a StopSignal,
}

/// Why a call failed, sorted by what the engine should do about it.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ProviderError {
    #[error("The API key was rejected: {0}")]
    Auth(String),
    /// Out of credit or over a daily quota: retrying the next piece will fail too.
    #[error("The provider account has no credit or quota left: {0}")]
    QuotaExhausted(String),
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
    /// An analysis answer that breaks its schema; another try usually gives a good one.
    #[error("The answer did not fit the analysis: {0}")]
    InvalidAnswer(String),
    /// An offline engine whose model has not been downloaded.
    #[error("Not installed: {0}")]
    NotInstalled(String),
    /// Stopped part-way because the job was paused or cancelled; the piece is done again later.
    #[error("Stopped before the end")]
    Interrupted,
}

impl ProviderError {
    /// A stable id the interface translates (`provider.rate_limited`…).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Auth(_) => "provider.auth",
            Self::QuotaExhausted(_) => "provider.quota",
            Self::RateLimited { .. } => "provider.rate_limited",
            Self::Transient(_) => "provider.transient",
            Self::Blocked(_) => "provider.blocked",
            Self::InvalidRequest(_) => "provider.invalid_request",
            Self::Unsupported(_) => "provider.unsupported",
            Self::Other(_) => "provider.other",
            Self::InvalidAnswer(_) => "provider.invalid_answer",
            Self::NotInstalled(_) => "provider.not_installed",
            Self::Interrupted => "provider.interrupted",
        }
    }

    /// Worth trying again after a delay.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Transient(_) | Self::InvalidAnswer(_)
        )
    }

    /// The failure will not change on retry and affects every piece (stop the job).
    pub fn is_fatal_for_job(&self) -> bool {
        matches!(
            self,
            Self::Auth(_) | Self::QuotaExhausted(_) | Self::NotInstalled(_)
        )
    }
}

/// A cloud provider or offline engine. The engine only knows providers through this trait and
/// never matches on a provider's name.
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;

    /// Cloud providers need an API key; offline engines do not.
    fn needs_key(&self) -> bool {
        true
    }

    /// Whether `model` can run now: an offline engine checks that its model was downloaded
    /// and that this processor can run it, before the job starts.
    fn check_ready(&self, model: &str) -> Result<(), ProviderError> {
        let _ = model;
        Ok(())
    }

    /// MIME types this provider accepts for a task, in order of preference. Media preparation
    /// converts each piece into the first one it can produce.
    fn accepted_inputs(&self, task: Task) -> &'static [&'static str];

    /// The fixed settings sent with a request for `task` and `model` (reasoning effort, output
    /// limit, media resolution) when it reads a page, for the record of a pipeline. Neither the
    /// prompt nor the researcher's own options, which the record states apart.
    fn request_settings(&self, task: Task, model: &str) -> Vec<RequestSetting> {
        let _ = (task, model);
        Vec::new()
    }

    /// The settings `request` itself carries, as provenance records them: they may depend on
    /// what it sends (a text, a page, a video) and on whether it goes through the batch service
    /// (`batched`). A provider whose settings vary overrides this, deciding them with the same
    /// code that builds the request, so the record cannot claim what was not sent.
    fn sent_settings(&self, request: &ProcessRequest<'_>, batched: bool) -> Vec<RequestSetting> {
        let _ = batched;
        self.request_settings(request.task, request.model)
    }

    async fn process(
        &self,
        key: &ApiKey,
        request: ProcessRequest<'_>,
    ) -> Result<ProcessOutput, ProviderError>;

    /// Model ids the key can use, for discovering models not yet in the catalogue. Also used to
    /// check that a key works.
    async fn list_models(&self, key: &ApiKey) -> Result<Vec<String>, ProviderError>;

    /// Whether `model` can take `task` through the provider's batch service (economy mode).
    /// Providers without one, and offline engines, keep the default.
    fn batches(&self, task: Task, model: &str) -> bool {
        let _ = (task, model);
        false
    }

    /// How much one batch may carry; the engine splits larger sets into several batches.
    fn batch_limits(&self) -> BatchLimits {
        BatchLimits::default()
    }

    /// Sends prepared pieces as one batch, each under its id, and returns the provider's batch
    /// id. Every request shares the task, model and prompt of one set-up.
    async fn submit_batch(
        &self,
        key: &ApiKey,
        requests: &[(String, ProcessRequest<'_>)],
    ) -> Result<String, ProviderError> {
        let _ = (key, requests);
        Err(ProviderError::Unsupported("batches".into()))
    }

    /// Where a batch stands; once it has ended, its answers, read as [`Provider::process`]
    /// reads one. `ids` are the items' ids in the order they were sent.
    async fn poll_batch(
        &self,
        key: &ApiKey,
        batch_id: &str,
        context: BatchContext<'_>,
        ids: &[String],
    ) -> Result<BatchProgress, ProviderError> {
        let _ = (key, batch_id, context, ids);
        Err(ProviderError::Unsupported("batches".into()))
    }

    /// Asks the provider to stop a batch. Answers already written may still be billed.
    async fn cancel_batch(&self, key: &ApiKey, batch_id: &str) -> Result<(), ProviderError> {
        let _ = (key, batch_id);
        Err(ProviderError::Unsupported("batches".into()))
    }
}
