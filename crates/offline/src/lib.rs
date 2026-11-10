//! Offline engines: Whisper for transcription, Tesseract and small OCR vision models for text
//! extraction, and small language models for analyses, tables of contents and corrections. They
//! implement the same [`kalamu_core::Provider`] trait as the cloud providers, so the engine
//! treats them alike; they need no key, and their models and language data are downloaded from
//! the app. The OCR and language models run in llama.cpp's programs, which they share
//! ([`LlamaHost`]).

mod cpu;
mod device;
mod diagnostics;
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
    /// Too little free space on the models' drive for what is still to download.
    #[error(
        "Not enough free space on the drive that holds the models: {} needed, {} available",
        gigabytes(*needed),
        gigabytes(*available)
    )]
    NoSpace { needed: u64, available: u64 },
}

/// Bytes as gigabytes with one decimal, as the systems' own file managers count them (10^9).
fn gigabytes(bytes: u64) -> String {
    let tenths = bytes / 100_000_000;
    format!("{}.{} GB", tenths / 10, tenths % 10)
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
            Self::NoSpace { .. } => "offline.no_space",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_space_is_said_in_gigabytes() {
        let error = OfflineError::NoSpace {
            needed: 6_408_000_000,
            available: 950_000_000,
        };
        assert_eq!(
            error.to_string(),
            "Not enough free space on the drive that holds the models: 6.4 GB needed, 0.9 GB available"
        );
    }
}
