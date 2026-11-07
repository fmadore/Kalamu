//! Offline engines: Whisper for transcription, Tesseract and small OCR vision models for text
//! extraction, and small language models for analyses, tables of contents and corrections. They
//! implement the same [`kalamu_core::Provider`] trait as the cloud providers, so the engine
//! treats them alike; they need no key, and their models and language data are downloaded from
//! the app. The OCR and language models run in llama.cpp's programs, which they share
//! ([`LlamaHost`]).

mod cpu;
mod device;
mod download;
mod host;
mod llama;
mod llm;
mod models;
mod tessdata;
mod tesseract;
mod vision;
mod whisper;

use kalamu_core::ProviderError;
use thiserror::Error;

pub use cpu::missing_features;
pub use device::{Backend, Placement};
pub use host::LlamaHost;
pub use llama::{Device, Llama};
pub use llm::{LLM, LLM_ID, LLM_MODELS, LocalLlm};
pub use models::{MODELS, ModelFile, ModelManager, ModelSet, ModelStatus, VAD, WHISPER, find};
pub use tessdata::{TESSDATA, for_language as tessdata_for_language};
pub use tesseract::{MODEL as TESSERACT_MODEL, TESSERACT_ID, Tesseract};
pub use vision::{VISION, VISION_ID, VISION_MODELS, Vision};
pub use whisper::{PROVIDER_ID, Whisper};

#[derive(Debug, Error)]
pub enum OfflineError {
    #[error("Unknown offline model: {0}")]
    UnknownModel(String),
    #[error("The {0} model has not been downloaded")]
    NotInstalled(String),
    #[error("Another model download is in progress")]
    Busy,
    #[error("The model is being used by a job")]
    InUse,
    #[error("The download was cancelled")]
    Cancelled,
    #[error("The download stopped responding")]
    Stalled,
    #[error("The model could not be downloaded: {0}")]
    Download(String),
    #[error("The model file is damaged; remove it and download it again")]
    Integrity,
    #[error("Could not read or write the model file: {0}")]
    Io(String),
}

impl OfflineError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownModel(_) => "offline.unknown_model",
            Self::NotInstalled(_) => "provider.not_installed",
            Self::Busy => "offline.busy",
            Self::InUse => "offline.in_use",
            Self::Cancelled => "offline.cancelled",
            Self::Stalled | Self::Download(_) => "offline.download",
            Self::Integrity => "offline.integrity",
            Self::Io(_) => "offline.io",
        }
    }
}

impl From<OfflineError> for ProviderError {
    fn from(e: OfflineError) -> Self {
        match e {
            OfflineError::NotInstalled(model) => Self::NotInstalled(model),
            other => Self::Other(other.to_string()),
        }
    }
}
