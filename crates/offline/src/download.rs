//! Downloading a model: one at a time, from its pinned URL, written under a temporary name and
//! installed only once its size and SHA-256 match. Ported from Live Translation.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::OfflineError;
use crate::models::{ModelFile, ModelManager, VAD, find, io, lock};

/// No overall deadline: Large is a gigabyte, and a slow or proxied network is still a network.
/// A connection that stops delivering is what fails, after this long without a byte.
const STALL: Duration = Duration::from_secs(60);

/// How much of the body is gathered before it is written: one blocking-pool round trip per
/// mebibyte instead of one per network chunk.
const WRITE_BATCH: usize = 1 << 20;

/// Where a download comes from and what it must turn out to be. Always the pinned Hugging Face
/// file in the app; the tests point it at a local server with a small payload.
struct Transfer {
    url: String,
    bytes: u64,
    sha256: String,
    stall: Duration,
}

impl Transfer {
    fn pinned(model: &ModelFile) -> Self {
        Self {
            url: format!("{}/{}", model.source, model.file),
            bytes: model.bytes,
            sha256: model.sha256.to_owned(),
            stall: STALL,
        }
    }
}

impl ModelManager {
    /// Downloads a speech model, and the voice-activity model first if it is missing.
    pub async fn download(&self, id: &str) -> Result<(), OfflineError> {
        let model = find(id).ok_or_else(|| OfflineError::UnknownModel(id.to_owned()))?;
        if !self.is_installed(&VAD) {
            self.download_from(&VAD, &Transfer::pinned(&VAD)).await?;
        }
        self.download_from(model, &Transfer::pinned(model)).await
    }

    async fn download_from(
        &self,
        model: &'static ModelFile,
        transfer: &Transfer,
    ) -> Result<(), OfflineError> {
        let token = CancellationToken::new();
        {
            let mut state = lock(&self.state);
            if state.download.is_some() {
                return Err(OfflineError::Busy);
            }
            if state.leases.contains(&model.id) {
                return Err(OfflineError::InUse);
            }
            state.download = Some((model.id, token.clone(), 0));
        }
        // Clears the download on every way out, including an error or an unwind; `Partial`
        // does the same for the file.
        let _operation = DownloadGuard(self.clone());
        fs::create_dir_all(&self.dir).map_err(io)?;
        let mut partial = Partial::create(&self.dir, model).await?;
        let client = reqwest::Client::builder()
            // Plain HTTP exists only for the local test server.
            .https_only(cfg!(not(test)))
            .connect_timeout(Duration::from_secs(15));
        #[cfg(test)]
        let client = client.no_proxy();
        let client = client
            .build()
            .map_err(|e| OfflineError::Download(e.to_string()))?;
        let mut response = tokio::select! {
            () = token.cancelled() => return Err(OfflineError::Cancelled),
            response = tokio::time::timeout(transfer.stall, client.get(&transfer.url).send()) =>
                response
                    .map_err(|_| OfflineError::Stalled)?
                    .and_then(reqwest::Response::error_for_status)
                    .map_err(|e| OfflineError::Download(e.to_string()))?,
        };
        if response
            .content_length()
            .is_some_and(|length| length > transfer.bytes)
        {
            return Err(OfflineError::Integrity);
        }
        let mut hash = Sha256::new();
        let mut total = 0u64;
        loop {
            // Cancellation also interrupts a stalled connection, not only a successful read.
            let chunk = tokio::select! {
                () = token.cancelled() => return Err(OfflineError::Cancelled),
                chunk = tokio::time::timeout(transfer.stall, response.chunk()) =>
                    chunk
                        .map_err(|_| OfflineError::Stalled)?
                        .map_err(|e| OfflineError::Download(e.to_string()))?,
            };
            let Some(chunk) = chunk else {
                break;
            };
            total += chunk.len() as u64;
            if total > transfer.bytes {
                return Err(OfflineError::Integrity);
            }
            hash.update(&chunk);
            partial.write(&chunk).await?;
            if let Some(download) = &mut lock(&self.state).download {
                download.2 = total;
            }
        }
        if token.is_cancelled() {
            return Err(OfflineError::Cancelled);
        }
        if total != transfer.bytes || hex::encode(hash.finalize()) != transfer.sha256 {
            return Err(OfflineError::Integrity);
        }
        partial.install(self.dir.join(model.file)).await
    }
}

struct DownloadGuard(ModelManager);

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        lock(&self.0.state).download = None;
    }
}

/// Delete what earlier downloads left in the folder: only `.part` files, which only this
/// manager writes, and only one download runs at a time.
fn remove_partials(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().ends_with(".part")
            && entry.file_type().is_ok_and(|t| t.is_file())
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// The download in progress, removed on every way out except a verified install. File work
/// runs on the blocking pool: writes and the final `sync_all` can stall for seconds on a slow
/// disk or under an antivirus scan.
struct Partial {
    path: PathBuf,
    file: Option<Arc<File>>,
    pending: Vec<u8>,
}

impl Partial {
    async fn create(dir: &Path, model: &ModelFile) -> Result<Self, OfflineError> {
        let dir = dir.to_owned();
        let name = model.partial();
        tokio::task::spawn_blocking(move || {
            remove_partials(&dir);
            let path = dir.join(name);
            let file = File::create(&path).map_err(io)?;
            Ok(Self {
                path,
                file: Some(Arc::new(file)),
                pending: Vec::with_capacity(WRITE_BATCH),
            })
        })
        .await
        .map_err(io)?
    }

    async fn write(&mut self, bytes: &[u8]) -> Result<(), OfflineError> {
        self.pending.extend_from_slice(bytes);
        if self.pending.len() >= WRITE_BATCH {
            self.flush().await?;
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), OfflineError> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let file = self.file.clone().ok_or_else(|| io("already closed"))?;
        let bytes = std::mem::replace(&mut self.pending, Vec::with_capacity(WRITE_BATCH));
        tokio::task::spawn_blocking(move || (&*file).write_all(&bytes))
            .await
            .map_err(io)?
            .map_err(io)
    }

    /// Write what is left, flush, close, then rename over the final name: a reader sees either
    /// no model or a complete, verified one.
    async fn install(mut self, destination: PathBuf) -> Result<(), OfflineError> {
        self.flush().await?;
        tokio::task::spawn_blocking(move || {
            let file = self.file.take().ok_or_else(|| io("already closed"))?;
            file.sync_all().map_err(io)?;
            drop(file);
            fs::rename(&self.path, destination).map_err(io)
        })
        .await
        .map_err(io)?
    }
}

impl Drop for Partial {
    fn drop(&mut self) {
        // Close first: Windows only finishes deleting a file once its last handle is gone.
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    use super::*;

    const PAYLOAD: &[u8] = b"a small stand-in for a model";

    /// A one-request HTTP server on a loopback port that sends `body` after `head`, then keeps
    /// the socket open and silent when `hang` is set.
    fn serve(head: &str, body: Vec<u8>, hang: bool) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let head = head.to_owned();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok_and(|n| n > 0) && line != "\r\n" {
                line.clear();
            }
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
            let _ = stream.flush();
            if hang {
                std::thread::sleep(Duration::from_secs(30));
            }
        });
        format!("http://{address}/model.bin")
    }

    fn ok(body: &[u8], hang: bool) -> String {
        serve(
            &format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                PAYLOAD.len()
            ),
            body.to_vec(),
            hang,
        )
    }

    fn transfer(url: String) -> Transfer {
        Transfer {
            url,
            bytes: PAYLOAD.len() as u64,
            sha256: hex::encode(Sha256::digest(PAYLOAD)),
            stall: Duration::from_millis(800),
        }
    }

    fn files(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    use crate::models::MODELS;

    const BASE: &ModelFile = &MODELS[3];

    #[test]
    fn production_downloads_are_pinned_https() {
        for model in MODELS.iter().chain([&VAD]) {
            let transfer = Transfer::pinned(model);
            assert!(transfer.url.starts_with("https://huggingface.co/"));
            // A full commit hash, never a branch name that moves.
            assert!(
                transfer.url.split('/').any(|part| part.len() == 40),
                "{}",
                transfer.url
            );
            assert_eq!(model.sha256.len(), 64);
        }
    }

    #[tokio::test]
    async fn a_verified_download_is_installed_under_its_final_name_only() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ModelManager::new(dir.path().to_owned());
        manager
            .download_from(BASE, &transfer(ok(PAYLOAD, false)))
            .await
            .unwrap();
        assert_eq!(files(dir.path()), [BASE.file]);
        assert!(manager.list().iter().all(|m| !m.downloading));
    }

    #[tokio::test]
    async fn a_hash_mismatch_installs_nothing_and_leaves_no_partial_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut corrupt = PAYLOAD.to_vec();
        corrupt[0] ^= 1;
        let error = ModelManager::new(dir.path().to_owned())
            .download_from(BASE, &transfer(ok(&corrupt, false)))
            .await
            .unwrap_err();
        assert!(matches!(error, OfflineError::Integrity), "{error}");
        assert!(files(dir.path()).is_empty());
    }

    #[tokio::test]
    async fn a_stalled_connection_fails_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let error = ModelManager::new(dir.path().to_owned())
            .download_from(BASE, &transfer(ok(&PAYLOAD[..5], true)))
            .await
            .unwrap_err();
        assert!(matches!(error, OfflineError::Stalled), "{error}");
        assert!(files(dir.path()).is_empty());
    }
}
