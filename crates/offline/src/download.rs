//! Downloading a model: one at a time, from its pinned URL, written under a temporary name and
//! installed only once its size and SHA-256 match. Ported from Live Translation.
//!
//! What was received is kept under that name when a download stops (a stalled or dropped
//! connection, a cancel, the app closed), and the next download of the same file asks only for
//! the rest (`Range: bytes=N-`): Gemma 4 E4B is 5.15 GB with a 0.99 GB encoder, and starting
//! over from zero after a cut at 90 % on a slow connection is hours lost. Every URL is pinned to
//! a commit, so the rest belongs to the same file; the bytes kept are hashed again before the
//! rest is added, so the whole file is checked exactly as a download in one go is. A part
//! longer than the file, a server that answers with something else, or a whole file whose hash
//! fails, and the part is deleted and the file downloaded once more from the start.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_RANGE, RANGE};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::OfflineError;
use crate::host::lock;
use crate::models::{ModelFile, ModelManager, io};

/// No overall deadline: Large is a gigabyte, and a slow or proxied network is still a network.
/// A connection that stops delivering is what fails, after this long without a byte.
const STALL: Duration = Duration::from_secs(60);

/// How much of the body is gathered before it is written: one blocking-pool round trip per
/// mebibyte instead of one per network chunk.
const WRITE_BATCH: usize = 1 << 20;

/// Free space left on the models' drive once a download is done, beyond the files themselves:
/// a drive filled to its last byte breaks the system's own writes, and the app's.
const RESERVE: u64 = 256 << 20;

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

/// How one attempt at a file ended, short of an error to report.
enum Attempt {
    Installed,
    /// What was kept could not be continued (the server sent another file's length, refused
    /// the range, or the whole file's hash failed after a resume): deleted, to start over.
    StartOver,
}

/// Free space on the drive holding `dir`, as the system reports it to the user.
fn system_free_space(dir: &Path) -> std::io::Result<u64> {
    fs4::available_space(dir)
}

impl ModelManager {
    /// Downloads a model, and first the file every model of its engine needs if it is missing
    /// (Whisper's voice-activity model) and the file it is paired with (a vision model's image
    /// encoder). What earlier downloads of these files kept is continued.
    pub async fn download(&self, id: &str) -> Result<(), OfflineError> {
        let model = self
            .find(id)
            .ok_or_else(|| OfflineError::UnknownModel(id.to_owned()))?;
        let companion = self.set.companion.filter(|c| !self.is_installed(c));
        let pair = model.pair.filter(|p| !self.is_installed(p));
        let files: Vec<(&'static ModelFile, Transfer)> = companion
            .into_iter()
            .chain(pair)
            .chain([model])
            .map(|file| (file, Transfer::pinned(file)))
            .collect();
        self.download_all(&files, system_free_space).await
    }

    /// Downloads `files` in turn, once the drive is known to have room for what is still to
    /// come of all of them (`free_space` asks the system; the tests answer for it).
    async fn download_all(
        &self,
        files: &[(&'static ModelFile, Transfer)],
        free_space: fn(&Path) -> std::io::Result<u64>,
    ) -> Result<(), OfflineError> {
        fs::create_dir_all(&self.dir).map_err(io)?;
        let to_come: u64 = files
            .iter()
            .map(|(model, transfer)| {
                let kept = fs::metadata(self.dir.join(model.partial())).map_or(0, |m| m.len());
                let kept = if kept > transfer.bytes { 0 } else { kept };
                transfer.bytes - kept
            })
            .sum();
        let needed = to_come + RESERVE;
        match free_space(&self.dir) {
            Ok(available) if available < needed => {
                return Err(OfflineError::NoSpace { needed, available });
            }
            Ok(_) => {}
            // A drive the system cannot measure (some network shares) is not refused: a full
            // drive still fails the download, with the system's own error, and keeps the part.
            Err(e) => tracing::warn!(error = %e, "free space on the models' drive unknown"),
        }
        for (model, transfer) in files {
            self.download_from(model, transfer).await?;
        }
        Ok(())
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
        // Clears the download on every way out, including an error or an unwind.
        let _operation = DownloadGuard(self.clone());
        fs::create_dir_all(&self.dir).map_err(io)?;
        let client = reqwest::Client::builder()
            // Plain HTTP exists only for the local test server.
            .https_only(cfg!(not(test)))
            .connect_timeout(Duration::from_secs(15));
        #[cfg(test)]
        let client = client.no_proxy();
        let client = client
            .build()
            .map_err(|e| OfflineError::Download(e.to_string()))?;
        let dir = self.dir.clone();
        let set = self.set;
        let partial = tokio::task::spawn_blocking(move || {
            remove_stale_partials(&dir, set);
            Partial::open(&dir, model)
        })
        .await
        .map_err(io)??;
        match self.attempt(&client, partial, transfer, &token).await? {
            Attempt::Installed => Ok(()),
            // Once only: a fresh download that fails is reported.
            Attempt::StartOver => {
                let partial = Partial::create(&self.dir, model).await?;
                match self.attempt(&client, partial, transfer, &token).await? {
                    Attempt::Installed => Ok(()),
                    Attempt::StartOver => Err(OfflineError::Integrity),
                }
            }
        }
    }

    /// One attempt at the file, from what `partial` holds. An error keeps the part (a stall, a
    /// network error, a cancel) unless it is the file's own (too long, the wrong hash).
    async fn attempt(
        &self,
        client: &reqwest::Client,
        mut partial: Partial,
        transfer: &Transfer,
        token: &CancellationToken,
    ) -> Result<Attempt, OfflineError> {
        let mut kept = partial.len().await?;
        if kept > transfer.bytes {
            partial.discard().await;
            return Ok(Attempt::StartOver);
        }
        self.progress(kept);
        // The bytes kept, hashed again so the whole file is checked in one pass.
        let mut hash = partial.hash(token).await?;
        if kept == transfer.bytes {
            // Whole already (the app closed between the last byte and the install): checked
            // and installed without asking for anything.
            if hex::encode(hash.finalize()) == transfer.sha256 {
                let destination = self.dir.join(&partial.final_name);
                partial.install(destination).await?;
                return Ok(Attempt::Installed);
            }
            partial.discard().await;
            return Ok(Attempt::StartOver);
        }
        let mut request = client.get(&transfer.url);
        if kept > 0 {
            request = request.header(RANGE, format!("bytes={kept}-"));
        }
        let response = tokio::select! {
            () = token.cancelled() => return Err(OfflineError::Cancelled),
            response = tokio::time::timeout(transfer.stall, request.send()) =>
                response
                    .map_err(|_| OfflineError::Stalled)?
                    .map_err(|e| OfflineError::Download(e.to_string()))?,
        };
        let status = response.status();
        if kept > 0 && status == StatusCode::RANGE_NOT_SATISFIABLE {
            // The file is shorter than what was kept: not the file that was being received.
            partial.discard().await;
            return Ok(Attempt::StartOver);
        }
        let mut response = response
            .error_for_status()
            .map_err(|e| OfflineError::Download(e.to_string()))?;
        if kept > 0 {
            if status == StatusCode::PARTIAL_CONTENT {
                let (start, total) = content_range(
                    response
                        .headers()
                        .get(CONTENT_RANGE)
                        .and_then(|v| v.to_str().ok()),
                );
                if start != Some(kept) || total.is_some_and(|t| t != transfer.bytes) {
                    partial.discard().await;
                    return Ok(Attempt::StartOver);
                }
            } else {
                // A server that ignores ranges sends the whole file: from the start.
                partial.truncate().await?;
                hash = Sha256::new();
                kept = 0;
                self.progress(0);
            }
        }
        if response
            .content_length()
            .is_some_and(|length| kept + length > transfer.bytes)
        {
            partial.discard().await;
            return if kept > 0 {
                Ok(Attempt::StartOver)
            } else {
                Err(OfflineError::Integrity)
            };
        }
        let resumed = kept > 0;
        let mut total = kept;
        let received = loop {
            // Cancellation also interrupts a stalled connection, not only a successful read.
            let chunk = tokio::select! {
                () = token.cancelled() => break Err(OfflineError::Cancelled),
                chunk = tokio::time::timeout(transfer.stall, response.chunk()) => chunk,
            };
            let chunk = match chunk {
                Err(_) => break Err(OfflineError::Stalled),
                Ok(Err(e)) => break Err(OfflineError::Download(e.to_string())),
                Ok(Ok(None)) => break Ok(()),
                Ok(Ok(Some(chunk))) => chunk,
            };
            total += chunk.len() as u64;
            if total > transfer.bytes {
                partial.discard().await;
                return if resumed {
                    Ok(Attempt::StartOver)
                } else {
                    Err(OfflineError::Integrity)
                };
            }
            hash.update(&chunk);
            if let Err(e) = partial.write(&chunk).await {
                break Err(e);
            }
            self.progress(total);
        };
        if let Err(e) = received {
            // What arrived is kept for the next download, written out first.
            let _ = partial.flush().await;
            return Err(e);
        }
        if token.is_cancelled() {
            let _ = partial.flush().await;
            return Err(OfflineError::Cancelled);
        }
        if total < transfer.bytes {
            // The connection closed early without an error: the rest next time.
            let _ = partial.flush().await;
            return Err(OfflineError::Download(format!(
                "the connection closed after {total} of {} bytes",
                transfer.bytes
            )));
        }
        if hex::encode(hash.finalize()) != transfer.sha256 {
            partial.discard().await;
            return if resumed {
                Ok(Attempt::StartOver)
            } else {
                Err(OfflineError::Integrity)
            };
        }
        let destination = self.dir.join(&partial.final_name);
        partial.install(destination).await?;
        Ok(Attempt::Installed)
    }

    /// The bytes of the file being downloaded that are on disk, for the interface: on a
    /// resumed download it starts at what was kept, not at zero.
    fn progress(&self, bytes: u64) {
        if let Some(download) = &mut lock(&self.state).download {
            download.2 = bytes;
        }
    }
}

/// The first byte and the whole length of a `Content-Range: bytes 100-999/1000` answer.
fn content_range(header: Option<&str>) -> (Option<u64>, Option<u64>) {
    let Some(range) = header.and_then(|h| h.trim().strip_prefix("bytes ")) else {
        return (None, None);
    };
    let (span, total) = range.split_once('/').unwrap_or((range, "*"));
    let start = span
        .split_once('-')
        .and_then(|(s, _)| s.trim().parse().ok());
    (start, total.trim().parse().ok())
}

struct DownloadGuard(ModelManager);

impl Drop for DownloadGuard {
    fn drop(&mut self) {
        lock(&self.0.state).download = None;
    }
}

/// Deletes the `.part` files no file of this engine is downloaded under, those of models
/// dropped from the list by an update; only this manager writes `.part` files, and only one
/// download runs at a time. The others are kept, to be continued.
fn remove_stale_partials(dir: &Path, set: &crate::models::ModelSet) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let ours: Vec<String> = set
        .files
        .iter()
        .flat_map(|m| std::iter::once(m).chain(m.pair))
        .chain(set.companion)
        .map(ModelFile::partial)
        .collect();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".part")
            && !ours.contains(&name)
            && entry.file_type().is_ok_and(|t| t.is_file())
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// The download in progress, under the file's `.part` name: kept when the download stops,
/// deleted when what it holds turns out wrong, renamed once verified. File work runs on the
/// blocking pool: writes and the final `sync_all` can stall for seconds on a slow disk or
/// under an antivirus scan.
struct Partial {
    path: PathBuf,
    final_name: String,
    file: Option<Arc<File>>,
    pending: Vec<u8>,
}

impl Partial {
    /// The part an earlier download left, its end ready for what follows, or a new one.
    fn open(dir: &Path, model: &ModelFile) -> Result<Self, OfflineError> {
        let path = dir.join(model.partial());
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(io)?;
        file.seek(SeekFrom::End(0)).map_err(io)?;
        Ok(Self {
            path,
            final_name: model.file.to_owned(),
            file: Some(Arc::new(file)),
            pending: Vec::with_capacity(WRITE_BATCH),
        })
    }

    /// An empty part, whatever was there.
    async fn create(dir: &Path, model: &'static ModelFile) -> Result<Self, OfflineError> {
        let dir = dir.to_owned();
        tokio::task::spawn_blocking(move || {
            let mut partial = Self::open(&dir, model)?;
            partial.truncate_now()?;
            Ok(partial)
        })
        .await
        .map_err(io)?
    }

    fn handle(&self) -> Result<Arc<File>, OfflineError> {
        self.file.clone().ok_or_else(|| io("already closed"))
    }

    /// The bytes on disk.
    async fn len(&self) -> Result<u64, OfflineError> {
        let file = self.handle()?;
        tokio::task::spawn_blocking(move || file.metadata().map(|m| m.len()))
            .await
            .map_err(io)?
            .map_err(io)
    }

    /// The SHA-256 state after the bytes on disk, read from the start; a gigabyte takes a few
    /// seconds, and a cancel ends it.
    async fn hash(&self, token: &CancellationToken) -> Result<Sha256, OfflineError> {
        let path = self.path.clone();
        let token = token.clone();
        tokio::task::spawn_blocking(move || {
            let mut hash = Sha256::new();
            let mut file = File::open(&path).map_err(io)?;
            let mut buffer = vec![0u8; WRITE_BATCH];
            loop {
                if token.is_cancelled() {
                    return Err(OfflineError::Cancelled);
                }
                let n = file.read(&mut buffer).map_err(io)?;
                if n == 0 {
                    return Ok(hash);
                }
                hash.update(&buffer[..n]);
            }
        })
        .await
        .map_err(io)?
    }

    fn truncate_now(&mut self) -> Result<(), OfflineError> {
        self.pending.clear();
        let file = self.handle()?;
        file.set_len(0).map_err(io)?;
        (&*file).seek(SeekFrom::Start(0)).map_err(io)?;
        Ok(())
    }

    /// Empties the part, to receive the file from its start.
    async fn truncate(&mut self) -> Result<(), OfflineError> {
        self.pending.clear();
        let file = self.handle()?;
        tokio::task::spawn_blocking(move || {
            file.set_len(0)?;
            (&*file).seek(SeekFrom::Start(0)).map(|_| ())
        })
        .await
        .map_err(io)?
        .map_err(io)
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
        let file = self.handle()?;
        let bytes = std::mem::replace(&mut self.pending, Vec::with_capacity(WRITE_BATCH));
        tokio::task::spawn_blocking(move || (&*file).write_all(&bytes))
            .await
            .map_err(io)?
            .map_err(io)
    }

    /// Deletes the part: what it holds cannot be part of the file.
    async fn discard(mut self) {
        let file = self.file.take();
        let path = self.path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            // Close first: Windows only finishes deleting a file once its last handle is gone.
            drop(file);
            let _ = fs::remove_file(path);
        })
        .await;
    }

    /// Write what is left, flush, close, then rename over the final name: a reader sees either
    /// no model or a complete, verified one.
    async fn install(mut self, destination: PathBuf) -> Result<(), OfflineError> {
        self.flush().await?;
        let file = self.file.take();
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let file = file.ok_or_else(|| io("already closed"))?;
            file.sync_all().map_err(io)?;
            drop(file);
            fs::rename(&path, destination).map_err(io)
        })
        .await
        .map_err(io)?
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::sync::Mutex;

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

    /// How the file server answers.
    #[derive(Clone, Copy)]
    enum Server {
        /// Ranges honoured; the first answer cut after this many bytes.
        CutFirstAfter(usize),
        /// Ranges honoured.
        Ranges,
        /// The whole file, whatever was asked.
        NoRanges,
    }

    /// A server of `PAYLOAD` for any number of requests, recording the `Range` each asked for
    /// (empty for none).
    fn file_server(server: Server) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let asked = Arc::new(Mutex::new(Vec::new()));
        let record = asked.clone();
        std::thread::spawn(move || {
            for (n, stream) in listener.incoming().enumerate() {
                let Ok(mut stream) = stream else {
                    return;
                };
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                let mut range = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 0) && line != "\r\n" {
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("range:") {
                        range = value.trim().to_owned();
                    }
                    line.clear();
                }
                record.lock().unwrap().push(range.clone());
                let start = range
                    .strip_prefix("bytes=")
                    .and_then(|r| r.trim_end_matches('-').parse::<usize>().ok());
                let (head, body) = match (server, start) {
                    (Server::NoRanges, _) | (_, None) => (
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            PAYLOAD.len()
                        ),
                        PAYLOAD.to_vec(),
                    ),
                    (_, Some(start)) => (
                        format!(
                            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\n\
                             Content-Range: bytes {start}-{}/{}\r\nConnection: close\r\n\r\n",
                            PAYLOAD.len() - start,
                            PAYLOAD.len() - 1,
                            PAYLOAD.len()
                        ),
                        PAYLOAD[start..].to_vec(),
                    ),
                };
                let body = match server {
                    Server::CutFirstAfter(cut) if n == 0 => body[..cut].to_vec(),
                    _ => body,
                };
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        });
        (format!("http://{address}/model.bin"), asked)
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

    use crate::models::{MODELS, VAD, WHISPER};
    use crate::tessdata::TESSDATA;

    const BASE: &ModelFile = &MODELS[3];

    #[test]
    fn production_downloads_are_pinned_https() {
        for model in MODELS.iter().chain([&VAD]).chain(TESSDATA.files) {
            let transfer = Transfer::pinned(model);
            assert!(
                transfer.url.starts_with("https://huggingface.co/")
                    || transfer
                        .url
                        .starts_with("https://raw.githubusercontent.com/tesseract-ocr/"),
                "{}",
                transfer.url
            );
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
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
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
        let error = ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(ok(&corrupt, false)))
            .await
            .unwrap_err();
        assert!(matches!(error, OfflineError::Integrity), "{error}");
        assert!(files(dir.path()).is_empty());
    }

    #[tokio::test]
    async fn a_stalled_connection_fails_and_keeps_what_arrived() {
        let dir = tempfile::tempdir().unwrap();
        let error = ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(ok(&PAYLOAD[..5], true)))
            .await
            .unwrap_err();
        assert!(matches!(error, OfflineError::Stalled), "{error}");
        assert_eq!(files(dir.path()), [BASE.partial()]);
        assert_eq!(
            fs::read(dir.path().join(BASE.partial())).unwrap(),
            &PAYLOAD[..5]
        );
    }

    #[tokio::test]
    async fn a_cut_download_continues_where_it_stopped() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        let (url, asked) = file_server(Server::CutFirstAfter(10));
        let error = manager
            .download_from(BASE, &transfer(url.clone()))
            .await
            .unwrap_err();
        assert!(matches!(error, OfflineError::Download(_)), "{error}");
        assert_eq!(files(dir.path()), [BASE.partial()]);
        manager.download_from(BASE, &transfer(url)).await.unwrap();
        assert_eq!(files(dir.path()), [BASE.file]);
        assert_eq!(fs::read(dir.path().join(BASE.file)).unwrap(), PAYLOAD);
        assert_eq!(*asked.lock().unwrap(), ["", "bytes=10-"]);
    }

    #[tokio::test]
    async fn a_resumed_download_shows_what_was_kept_and_a_cancel_keeps_what_arrived() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), &PAYLOAD[..10]).unwrap();
        let url = serve(
            &format!(
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\n\
                 Content-Range: bytes 10-{}/{}\r\nConnection: close\r\n\r\n",
                PAYLOAD.len() - 10,
                PAYLOAD.len() - 1,
                PAYLOAD.len()
            ),
            PAYLOAD[10..15].to_vec(),
            true,
        );
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        let running = manager.clone();
        let mut long = transfer(url);
        long.stall = Duration::from_secs(20);
        let download = tokio::spawn(async move { running.download_from(BASE, &long).await });
        let mut shown = 0;
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let status = manager
                .list()
                .into_iter()
                .find(|m| m.id == BASE.id)
                .unwrap();
            shown = status.downloaded_bytes;
            if shown == 15 {
                break;
            }
        }
        assert_eq!(shown, 15, "the bytes kept and those received since");
        manager.cancel_download();
        let error = download.await.unwrap().unwrap_err();
        assert!(matches!(error, OfflineError::Cancelled), "{error}");
        assert_eq!(
            fs::read(dir.path().join(BASE.partial())).unwrap(),
            &PAYLOAD[..15]
        );
        // Stopped, the list still shows what was kept, for the next download to continue.
        let status = manager
            .list()
            .into_iter()
            .find(|m| m.id == BASE.id)
            .unwrap();
        assert!(!status.downloading && !status.installed);
        assert_eq!(status.downloaded_bytes, 15);
    }

    /// Not downloading, the list shows the part kept, and a paired encoder already whole; a part
    /// longer than its file counts for nothing, as the next download drops it.
    #[test]
    fn a_stopped_download_shows_what_the_next_one_continues_from() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        let shown = |manager: &ModelManager| {
            manager
                .list()
                .into_iter()
                .find(|m| m.id == BASE.id)
                .unwrap()
                .downloaded_bytes
        };
        assert_eq!(shown(&manager), 0);
        fs::write(dir.path().join(BASE.partial()), &PAYLOAD[..10]).unwrap();
        assert_eq!(shown(&manager), 10);
        // Lengthened without writing it.
        fs::File::options()
            .write(true)
            .open(dir.path().join(BASE.partial()))
            .unwrap()
            .set_len(BASE.bytes + 1)
            .unwrap();
        assert_eq!(shown(&manager), 0);

        // A vision model whose encoder arrived whole before the download stopped.
        let model = &crate::vision::VISION_MODELS[0];
        let encoder = model.pair.unwrap();
        let manager = ModelManager::new(dir.path().to_owned(), &crate::vision::VISION);
        fs::File::create(dir.path().join(encoder.file))
            .unwrap()
            .set_len(encoder.bytes)
            .unwrap();
        fs::write(dir.path().join(model.partial()), &PAYLOAD[..10]).unwrap();
        let status = manager
            .list()
            .into_iter()
            .find(|m| m.id == model.id)
            .unwrap();
        assert_eq!(status.downloaded_bytes, encoder.bytes + 10);
        assert_eq!(status.bytes, encoder.bytes + model.bytes);
    }

    #[tokio::test]
    async fn a_server_that_ignores_ranges_sends_the_file_again_from_the_start() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), &PAYLOAD[..10]).unwrap();
        let (url, asked) = file_server(Server::NoRanges);
        ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(url))
            .await
            .unwrap();
        assert_eq!(fs::read(dir.path().join(BASE.file)).unwrap(), PAYLOAD);
        assert_eq!(*asked.lock().unwrap(), ["bytes=10-"]);
    }

    #[tokio::test]
    async fn a_damaged_part_is_dropped_and_the_file_downloaded_once_more() {
        let dir = tempfile::tempdir().unwrap();
        // Wrong bytes: the rest makes a file whose hash fails.
        fs::write(dir.path().join(BASE.partial()), b"0123456789").unwrap();
        let (url, asked) = file_server(Server::Ranges);
        ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(url))
            .await
            .unwrap();
        assert_eq!(files(dir.path()), [BASE.file]);
        assert_eq!(fs::read(dir.path().join(BASE.file)).unwrap(), PAYLOAD);
        assert_eq!(*asked.lock().unwrap(), ["bytes=10-", ""]);

        // Longer than the file: dropped before anything is asked.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), vec![0u8; 100]).unwrap();
        let (url, asked) = file_server(Server::Ranges);
        ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(url))
            .await
            .unwrap();
        assert_eq!(fs::read(dir.path().join(BASE.file)).unwrap(), PAYLOAD);
        assert_eq!(*asked.lock().unwrap(), [""]);
    }

    #[tokio::test]
    async fn a_whole_part_is_installed_without_asking_again() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), PAYLOAD).unwrap();
        let (url, asked) = file_server(Server::Ranges);
        ModelManager::new(dir.path().to_owned(), &WHISPER)
            .download_from(BASE, &transfer(url))
            .await
            .unwrap();
        assert_eq!(files(dir.path()), [BASE.file]);
        assert!(asked.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_full_drive_is_reported_before_anything_is_downloaded() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), &PAYLOAD[..10]).unwrap();
        let (url, asked) = file_server(Server::Ranges);
        let manager = ModelManager::new(dir.path().to_owned(), &WHISPER);
        let error = manager
            .download_all(&[(BASE, transfer(url.clone()))], |_| Ok(1000))
            .await
            .unwrap_err();
        // What is still to come, the part counted, and the reserve.
        let needed = (PAYLOAD.len() - 10) as u64 + RESERVE;
        assert!(
            matches!(error, OfflineError::NoSpace { needed: n, available: 1000 } if n == needed),
            "{error:?}"
        );
        assert_eq!(error.code(), "offline.no_space");
        assert!(asked.lock().unwrap().is_empty());
        assert_eq!(files(dir.path()), [BASE.partial()]);
        // With room, it continues.
        manager
            .download_all(&[(BASE, transfer(url))], |_| Ok(u64::MAX))
            .await
            .unwrap();
        assert_eq!(fs::read(dir.path().join(BASE.file)).unwrap(), PAYLOAD);
    }

    #[test]
    fn parts_of_models_no_longer_listed_are_deleted_and_others_kept() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(BASE.partial()), b"kept").unwrap();
        fs::write(dir.path().join(VAD.partial()), b"kept").unwrap();
        fs::write(dir.path().join("ggml-tiny.bin.part"), b"gone").unwrap();
        fs::write(dir.path().join("notes.txt"), b"not ours").unwrap();
        remove_stale_partials(dir.path(), &WHISPER);
        let mut expected = vec![BASE.partial(), VAD.partial(), "notes.txt".to_owned()];
        expected.sort();
        assert_eq!(files(dir.path()), expected);
    }

    #[test]
    fn a_content_range_is_read() {
        assert_eq!(
            content_range(Some("bytes 100-999/1000")),
            (Some(100), Some(1000))
        );
        assert_eq!(content_range(Some("bytes 100-999/*")), (Some(100), None));
        assert_eq!(content_range(None), (None, None));
        assert_eq!(content_range(Some("items 1-2/3")), (None, None));
    }
}
