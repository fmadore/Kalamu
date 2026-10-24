//! Whisper model files: a fixed list, downloaded from a pinned Hugging Face revision and
//! checked against their SHA-256 before use. No arbitrary URLs or paths from the interface.
//!
//! Downloads are in `download.rs`. Ported from Live Translation's `whisper/models.rs`, with the larger models a batch
//! transcription app needs (Live Translation stops at Small for real-time captions).

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::OfflineError;

/// The `ggerganov/whisper.cpp` revision the speech models come from, as in Live Translation.
const WHISPER: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1";
/// Silero voice-activity detection for whisper.cpp. Version 5.1.2, the one whisper.cpp 1.8.3
/// (bundled by whisper-rs-sys 0.15) was released with.
const VAD_REPO: &str =
    "https://huggingface.co/ggml-org/whisper-vad/resolve/9ffd54a1e1ee413ddf265af9913beaf518d1639b";

/// One downloadable model. `id` is the model id in the catalogue.
#[derive(Debug, PartialEq, Eq)]
pub struct ModelFile {
    pub id: &'static str,
    pub file: &'static str,
    pub bytes: u64,
    pub(crate) sha256: &'static str,
    /// The pinned folder `file` is downloaded from.
    pub(crate) source: &'static str,
}

/// Quantised builds: a third of the full-precision size for a negligible loss in accuracy.
pub const MODELS: [ModelFile; 4] = [
    ModelFile {
        id: "whisper-large-v3-turbo",
        file: "ggml-large-v3-turbo-q5_0.bin",
        source: WHISPER,
        bytes: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
    },
    ModelFile {
        id: "whisper-large-v3",
        file: "ggml-large-v3-q5_0.bin",
        source: WHISPER,
        bytes: 1_081_140_203,
        sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1",
    },
    ModelFile {
        id: "whisper-small",
        file: "ggml-small-q5_1.bin",
        source: WHISPER,
        bytes: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
    },
    ModelFile {
        id: "whisper-base",
        file: "ggml-base-q5_1.bin",
        source: WHISPER,
        bytes: 59_707_625,
        sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
    },
];

/// Skips silence and music before transcription, where Whisper otherwise invents text. Tiny,
/// and downloaded along with the first speech model.
pub const VAD: ModelFile = ModelFile {
    id: "silero-vad",
    file: "ggml-silero-v5.1.2.bin",
    bytes: 885_098,
    sha256: "29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf",
    source: VAD_REPO,
};

pub fn find(id: &str) -> Option<&'static ModelFile> {
    MODELS.iter().find(|m| m.id == id)
}

impl ModelFile {
    /// The fixed name a download is written under until it has been verified, so the next
    /// download can find and delete what an interrupted one left behind.
    pub(crate) fn partial(&self) -> String {
        format!("{}.part", self.file)
    }
}

/// What the Settings view shows for each model.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export, rename = "OfflineModel")]
pub struct ModelStatus {
    pub id: String,
    #[ts(type = "number")]
    pub bytes: u64,
    pub installed: bool,
    pub downloading: bool,
    #[ts(type = "number")]
    pub downloaded_bytes: u64,
    /// Loaded by a running job, so it cannot be removed now.
    pub in_use: bool,
}

#[derive(Default)]
pub(crate) struct State {
    /// The model being downloaded, its cancel token and the bytes received so far.
    pub(crate) download: Option<(&'static str, CancellationToken, u64)>,
    pub(crate) leases: Vec<&'static str>,
}

/// Downloads, lists, removes and lends out model files in one folder.
#[derive(Clone)]
pub struct ModelManager {
    pub(crate) dir: PathBuf,
    pub(crate) state: Arc<Mutex<State>>,
}

pub(crate) fn lock(state: &Mutex<State>) -> std::sync::MutexGuard<'_, State> {
    state.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn io(e: impl std::fmt::Display) -> OfflineError {
    OfflineError::Io(e.to_string())
}

impl ModelManager {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            state: Arc::default(),
        }
    }

    pub fn is_installed(&self, model: &ModelFile) -> bool {
        fs::metadata(self.dir.join(model.file)).is_ok_and(|m| m.len() == model.bytes)
    }

    pub fn list(&self) -> Vec<ModelStatus> {
        let state = lock(&self.state);
        MODELS
            .iter()
            .map(|model| {
                let download = state.download.as_ref().filter(|d| d.0 == model.id);
                ModelStatus {
                    id: model.id.to_owned(),
                    bytes: model.bytes,
                    installed: self.is_installed(model),
                    downloading: download.is_some(),
                    downloaded_bytes: download.map_or(0, |d| d.2),
                    in_use: state.leases.contains(&model.id),
                }
            })
            .collect()
    }

    pub fn cancel_download(&self) {
        if let Some((_, token, _)) = &lock(&self.state).download {
            token.cancel();
        }
    }

    pub fn remove(&self, id: &str) -> Result<(), OfflineError> {
        let model = find(id).ok_or_else(|| OfflineError::UnknownModel(id.to_owned()))?;
        let state = lock(&self.state);
        if state.leases.contains(&model.id) {
            return Err(OfflineError::InUse);
        }
        if state.download.is_some() {
            return Err(OfflineError::Busy);
        }
        match fs::remove_file(self.dir.join(model.file)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io(e)),
        }
    }

    /// Pins the file against removal or replacement while a job uses it.
    pub fn lease(&self, model: &'static ModelFile) -> Result<ModelLease, OfflineError> {
        let mut state = lock(&self.state);
        if state.download.as_ref().is_some_and(|d| d.0 == model.id) || !self.is_installed(model) {
            return Err(OfflineError::NotInstalled(model.id.to_owned()));
        }
        state.leases.push(model.id);
        Ok(ModelLease {
            manager: self.clone(),
            model,
            path: self.dir.join(model.file),
        })
    }

    /// The voice-activity model's path, when it is installed and whisper.cpp can open it:
    /// whisper.cpp opens it by name with the C runtime, which cannot read non-ASCII paths on
    /// Windows, so a user folder with an accented name goes without it.
    pub fn vad_path(&self) -> Option<String> {
        let path = self.dir.join(VAD.file);
        let path = path.to_str()?;
        (self.is_installed(&VAD) && path.is_ascii()).then(|| path.to_owned())
    }
}

/// A model file lent to a job.
pub struct ModelLease {
    manager: ModelManager,
    pub model: &'static ModelFile,
    path: PathBuf,
}

impl ModelLease {
    /// The whole file, provided it is exactly the pinned model: a damaged or replaced file must
    /// never reach native inference. Read through Rust's Unicode-aware file API, since Windows
    /// user names may not be ASCII.
    pub fn read_verified(&self) -> Result<Vec<u8>, OfflineError> {
        let bytes = fs::metadata(&self.path).map_err(io)?.len();
        if bytes != self.model.bytes {
            return Err(OfflineError::Integrity);
        }
        let data = fs::read(&self.path).map_err(io)?;
        if hex::encode(Sha256::digest(&data)) != self.model.sha256 {
            return Err(OfflineError::Integrity);
        }
        Ok(data)
    }
}

impl Drop for ModelLease {
    fn drop(&mut self) {
        let mut state = lock(&self.manager.state);
        if let Some(i) = state.leases.iter().position(|id| *id == self.model.id) {
            state.leases.remove(i);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const BASE: &ModelFile = &MODELS[3];

    fn files(dir: &std::path::Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    #[tokio::test]
    async fn a_model_in_use_cannot_be_removed_and_a_damaged_file_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        // Right size, wrong content.
        fs::write(dir.path().join(BASE.file), vec![0u8; 16]).unwrap();
        let manager = ModelManager::new(dir.path().to_owned());
        assert!(!manager.is_installed(BASE));
        assert!(matches!(
            manager.lease(BASE),
            Err(OfflineError::NotInstalled(_))
        ));
        manager.remove(BASE.id).unwrap();
        assert!(files(dir.path()).is_empty());
    }
}
