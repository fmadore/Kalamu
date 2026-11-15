//! Model files: fixed lists, one per engine, each downloaded from a pinned revision and checked
//! against its SHA-256 before use. No arbitrary URLs or paths from the interface. Whisper's
//! speech models are here; Tesseract's language data is in `tessdata.rs`.
//!
//! Downloads are in `download.rs`. Ported from Live Translation's `whisper/models.rs`, with the larger models a batch
//! transcription app needs (Live Translation stops at Small for real-time captions).

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::OfflineError;
use crate::host::lock;

/// The `ggerganov/whisper.cpp` revision the speech models come from, as in Live Translation.
const WHISPER_REPO: &str =
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
    /// The languages a file of language data reads, as catalogue codes; empty for a model
    /// that is not chosen by language.
    pub languages: &'static [&'static str],
    /// A file of its own this model cannot run without (a vision model's image encoder):
    /// downloaded before it, counted in its size, and removed with it.
    pub pair: Option<&'static ModelFile>,
}

/// The files one engine downloads into its own folder.
#[derive(Debug)]
pub struct ModelSet {
    pub files: &'static [ModelFile],
    /// A file every other one needs, downloaded with the first of them.
    pub companion: Option<&'static ModelFile>,
}

/// Whisper's speech models, with the voice-activity model they share.
pub static WHISPER: ModelSet = ModelSet {
    files: &MODELS,
    companion: Some(&VAD),
};

/// Quantised builds: a third of the full-precision size for a negligible loss in accuracy.
pub const MODELS: [ModelFile; 4] = [
    ModelFile {
        id: "whisper-large-v3-turbo",
        file: "ggml-large-v3-turbo-q5_0.bin",
        source: WHISPER_REPO,
        bytes: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        languages: &[],
        pair: None,
    },
    ModelFile {
        id: "whisper-large-v3",
        file: "ggml-large-v3-q5_0.bin",
        source: WHISPER_REPO,
        bytes: 1_081_140_203,
        sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1",
        languages: &[],
        pair: None,
    },
    ModelFile {
        id: "whisper-small",
        file: "ggml-small-q5_1.bin",
        source: WHISPER_REPO,
        bytes: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        languages: &[],
        pair: None,
    },
    ModelFile {
        id: "whisper-base",
        file: "ggml-base-q5_1.bin",
        source: WHISPER_REPO,
        bytes: 59_707_625,
        sha256: "422f1ae452ade6f30a004d7e5c6a43195e4433bc370bf23fac9cc591f01a8898",
        languages: &[],
        pair: None,
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
    languages: &[],
    pair: None,
};

/// A Whisper speech model.
pub fn find(id: &str) -> Option<&'static ModelFile> {
    MODELS.iter().find(|m| m.id == id)
}

impl ModelSet {
    pub fn find(&self, id: &str) -> Option<&'static ModelFile> {
        self.files.iter().find(|m| m.id == id)
    }
}

impl ModelFile {
    /// The fixed name a download is written under until it has been verified, so the next
    /// download can continue what an interrupted one left behind.
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
    /// While downloading, the bytes received so far; otherwise, for a model not installed,
    /// what an interrupted download kept, from which the next one continues.
    #[ts(type = "number")]
    pub downloaded_bytes: u64,
    /// Loaded by a running job, so it cannot be removed now.
    pub in_use: bool,
    /// For language data, the catalogue codes of the languages it reads.
    pub languages: Vec<String>,
}

#[derive(Default)]
pub(crate) struct State {
    /// The model being downloaded, its cancel token and the bytes received so far.
    pub(crate) download: Option<(&'static str, CancellationToken, u64)>,
    pub(crate) leases: Vec<&'static str>,
}

/// An engine that keeps files loaded while it holds no lease on them (Tesseract's idle
/// engines), told when one is removed so it lets go of what it loaded from it.
pub(crate) trait Release: Send + Sync {
    fn release(&self, id: &'static str);
}

/// Downloads, lists, removes and lends out one engine's model files in one folder.
#[derive(Clone)]
pub struct ModelManager {
    pub(crate) dir: PathBuf,
    pub(crate) set: &'static ModelSet,
    pub(crate) state: Arc<Mutex<State>>,
    /// Told of each file removed; held weakly, so an engine dropped is forgotten.
    releasers: Arc<Mutex<Vec<Weak<dyn Release>>>>,
}

pub(crate) fn io(e: impl std::fmt::Display) -> OfflineError {
    OfflineError::Io(e.to_string())
}

impl ModelManager {
    pub fn new(dir: PathBuf, set: &'static ModelSet) -> Self {
        Self {
            dir,
            set,
            state: Arc::default(),
            releasers: Arc::default(),
        }
    }

    /// Tells `releaser` of every file removed from now on.
    pub(crate) fn on_remove(&self, releaser: Weak<dyn Release>) {
        let mut releasers = lock(&self.releasers);
        releasers.retain(|r| r.strong_count() > 0);
        releasers.push(releaser);
    }

    /// One of this engine's files.
    pub fn find(&self, id: &str) -> Option<&'static ModelFile> {
        self.set.find(id)
    }

    /// The folder the files are kept in.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    pub fn is_installed(&self, model: &ModelFile) -> bool {
        fs::metadata(self.dir.join(model.file)).is_ok_and(|m| m.len() == model.bytes)
    }

    /// The bytes of `model` an interrupted download kept under its `.part` name; none when the
    /// part is longer than the file, which the next download drops.
    pub(crate) fn kept(&self, model: &ModelFile) -> u64 {
        let kept = fs::metadata(self.dir.join(model.partial())).map_or(0, |m| m.len());
        if kept > model.bytes { 0 } else { kept }
    }

    /// The model and the file it is paired with, both downloaded.
    pub fn is_ready(&self, model: &ModelFile) -> bool {
        self.is_installed(model) && model.pair.is_none_or(|pair| self.is_installed(pair))
    }

    pub fn list(&self) -> Vec<ModelStatus> {
        let state = lock(&self.state);
        self.set
            .files
            .iter()
            .map(|model| {
                let pair = model.pair.map_or(0, |p| p.bytes);
                // A paired file is downloaded first, so the model's own bytes come after it.
                let downloaded = state.download.as_ref().and_then(|d| {
                    if d.0 == model.id {
                        Some(pair + d.2)
                    } else {
                        model.pair.filter(|p| p.id == d.0).map(|_| d.2)
                    }
                });
                let installed = self.is_ready(model);
                let kept = || {
                    if installed {
                        return 0;
                    }
                    let pair = model.pair.map_or(0, |p| {
                        if self.is_installed(p) {
                            p.bytes
                        } else {
                            self.kept(p)
                        }
                    });
                    pair + self.kept(model)
                };
                ModelStatus {
                    id: model.id.to_owned(),
                    bytes: model.bytes + pair,
                    installed,
                    downloading: downloaded.is_some(),
                    downloaded_bytes: downloaded.unwrap_or_else(kept),
                    in_use: state.leases.contains(&model.id),
                    languages: model.languages.iter().map(|&l| l.to_owned()).collect(),
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
        let model = self
            .find(id)
            .ok_or_else(|| OfflineError::UnknownModel(id.to_owned()))?;
        let state = lock(&self.state);
        if state.leases.contains(&model.id)
            || model.pair.is_some_and(|p| state.leases.contains(&p.id))
        {
            return Err(OfflineError::InUse);
        }
        if state.download.is_some() {
            return Err(OfflineError::Busy);
        }
        // What an unfinished download kept goes with it, to give the space back.
        for file in std::iter::once(model).chain(model.pair) {
            for name in [file.file.to_owned(), file.partial()] {
                match fs::remove_file(self.dir.join(name)) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(io(e)),
                }
            }
        }
        drop(state);
        // Engines that loaded the files without holding them let go of them, outside the
        // locks: what they drop may lock them again.
        let releasers: Vec<Arc<dyn Release>> = lock(&self.releasers)
            .iter()
            .filter_map(Weak::upgrade)
            .collect();
        for file in std::iter::once(model).chain(model.pair) {
            for releaser in &releasers {
                releaser.release(file.id);
            }
        }
        Ok(())
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
    /// Windows, so a user folder with an accented name goes without it there. macOS opens
    /// UTF-8 paths.
    pub fn vad_path(&self) -> Option<String> {
        let path = self.dir.join(VAD.file);
        let path = path.to_str()?;
        let readable = !cfg!(windows) || path.is_ascii();
        (self.is_installed(&VAD) && readable).then(|| path.to_owned())
    }
}

/// A model file lent to a job.
pub struct ModelLease {
    manager: ModelManager,
    pub model: &'static ModelFile,
    path: PathBuf,
}

impl ModelLease {
    /// Where the file is, for an engine that opens it itself.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Checks the file is exactly the pinned model without holding it in memory: for engines
    /// that open the file themselves, a gigabyte at a time.
    pub fn verify(&self) -> Result<(), OfflineError> {
        let bytes = fs::metadata(&self.path).map_err(io)?.len();
        if bytes != self.model.bytes {
            return Err(OfflineError::Integrity);
        }
        if sha256_of_file(&self.path)? != self.model.sha256 {
            return Err(OfflineError::Integrity);
        }
        Ok(())
    }

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

/// The SHA-256 of the file at `path` in hexadecimal, read a few kilobytes at a time: the same
/// as hashing its bytes at once (`hex::encode(Sha256::digest(bytes))`) without holding them.
pub(crate) fn sha256_of_file(path: &std::path::Path) -> Result<String, OfflineError> {
    let mut file = fs::File::open(path).map_err(io)?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash).map_err(io)?;
    Ok(hex::encode(hash.finalize()))
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
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        assert!(!manager.is_installed(BASE));
        assert!(matches!(
            manager.lease(BASE),
            Err(OfflineError::NotInstalled(_))
        ));
        manager.remove(BASE.id).unwrap();
        assert!(files(dir.path()).is_empty());
    }

    /// The hash read a few kilobytes at a time is the hash of the bytes at once, byte for byte:
    /// it is compared with the pinned SHA-256 of every model file, so any difference would
    /// refuse files that are right. Sizes around `std::io::copy`'s 8 KiB buffer, and one of
    /// several megabytes.
    #[test]
    fn the_streamed_hash_is_the_hash_of_the_whole_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        for size in [0usize, 1, 8191, 8192, 8193, 65_537, 3 * 1024 * 1024 + 7] {
            let bytes: Vec<u8> = (0..size)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed as u8
                })
                .collect();
            let path = dir.path().join(format!("{size}.bin"));
            fs::write(&path, &bytes).unwrap();
            assert_eq!(
                sha256_of_file(&path).unwrap(),
                hex::encode(Sha256::digest(&bytes)),
                "{size} bytes"
            );
        }
        // And a known value: the SHA-256 of "abc" (FIPS 180-2).
        let abc = dir.path().join("abc");
        fs::write(&abc, b"abc").unwrap();
        assert_eq!(
            sha256_of_file(&abc).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    struct Released(Mutex<Vec<&'static str>>);

    impl Release for Released {
        fn release(&self, id: &'static str) {
            lock(&self.0).push(id);
        }
    }

    #[test]
    fn removing_a_file_tells_the_engines_that_loaded_it() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        let released = Arc::new(Released(Mutex::new(Vec::new())));
        let weak: Weak<Released> = Arc::downgrade(&released);
        manager.clone().on_remove(weak);
        manager.remove(BASE.id).unwrap();
        assert_eq!(*lock(&released.0), [BASE.id]);
        // A dropped engine is forgotten.
        drop(released);
        manager.remove(BASE.id).unwrap();
        assert!(
            lock(&manager.releasers)
                .iter()
                .all(|r| r.strong_count() == 0)
        );
    }
}
