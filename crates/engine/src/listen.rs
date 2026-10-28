//! Hearing a piece of a recording in the review view. The player reads the recording itself
//! when it can, so a piece starts at once; otherwise FFmpeg converts the piece, and the copy is
//! kept in a small cache so the next visit is instant.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use kalamu_core::{JobId, PieceRef, SourceId, StoreError};
use kalamu_media::{MediaError, OpenSource};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Engine, EngineError};

/// Room for converted pieces: about seventeen hours of speech as 32 kbit/s Opus. The pieces
/// heard longest ago go first.
pub const LISTENING_CACHE_BYTES: u64 = 256 * 1024 * 1024;

/// Where to hear one piece: a file on this PC and the span of it that is the piece.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PieceAudio {
    /// The recording itself, or a converted copy of the piece in the cache.
    pub path: String,
    /// Where the piece starts in that file.
    #[ts(type = "number")]
    pub start_ms: u64,
    /// Where it ends in that file; 0 to play to the end.
    #[ts(type = "number")]
    pub end_ms: u64,
    /// The recording itself: if the player cannot play it after all, ask again with `convert`.
    pub original: bool,
}

impl Engine {
    /// One piece of a recording to listen to: the recording itself when the player plays its
    /// format and `convert` is not asked for, else a converted copy (cached).
    pub fn piece_audio(
        &self,
        job_id: JobId,
        source_id: SourceId,
        index: u32,
        convert: bool,
    ) -> Result<PieceAudio, EngineError> {
        let job = self.store.job(job_id)?;
        let source = job
            .sources
            .iter()
            .find(|s| s.id == source_id)
            .ok_or(EngineError::Store(StoreError::NotFound))?;
        let piece = self
            .store
            .pieces(job_id)?
            .into_iter()
            .find(|p| p.source_id == source_id && p.index == index)
            .map(|p| p.piece)
            .ok_or(EngineError::Store(StoreError::NotFound))?;
        let PieceRef::Segment {
            start_ms, end_ms, ..
        } = piece
        else {
            return Err(MediaError::NoSuchPage(piece.label()).into());
        };
        if !convert && kalamu_media::plays_as_is(Path::new(&source.path)) {
            return Ok(PieceAudio {
                path: source.path.clone(),
                start_ms,
                end_ms,
                original: true,
            });
        }
        // The fingerprint of the recording and the span: the same piece of the same recording,
        // whichever job it belongs to.
        let cache = ListeningCache::new(&self.config.listening_cache, LISTENING_CACHE_BYTES);
        let stem = format!("{}-{start_ms}-{end_ms}", source.sha256);
        let path = match cache.get(&stem) {
            Some(path) => path,
            None => {
                let opened = OpenSource::open(source, self.config.ffmpeg.as_ref())?;
                let copy = opened.listening_copy(&piece)?;
                let ext = Path::new(&copy.file_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("ogg");
                cache
                    .put(&stem, ext, &copy.bytes)
                    .map_err(|e| EngineError::ListeningCopy(e.to_string()))?
            }
        };
        Ok(PieceAudio {
            path: path.display().to_string(),
            start_ms: 0,
            end_ms: 0,
            original: false,
        })
    }
}

/// A folder of converted pieces, kept under a size cap by removing those heard longest ago.
pub(crate) struct ListeningCache<'a> {
    dir: &'a Path,
    cap_bytes: u64,
}

/// The formats a listening copy is written in.
const EXTENSIONS: [&str; 2] = ["ogg", "wav"];

impl<'a> ListeningCache<'a> {
    pub(crate) fn new(dir: &'a Path, cap_bytes: u64) -> Self {
        Self { dir, cap_bytes }
    }

    /// The copy named `stem`, marked as just heard.
    pub(crate) fn get(&self, stem: &str) -> Option<PathBuf> {
        let path = EXTENSIONS
            .iter()
            .map(|ext| self.dir.join(format!("{stem}.{ext}")))
            .find(|p| p.is_file())?;
        // Best effort: a copy that cannot be touched is only removed a little early.
        if let Ok(file) = std::fs::File::options().write(true).open(&path) {
            let _ = file.set_modified(SystemTime::now());
        }
        Some(path)
    }

    /// Keeps `bytes` as `stem.ext`, then makes room. Written beside and renamed, so the player
    /// never reads half a file.
    pub(crate) fn put(&self, stem: &str, ext: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::fs::create_dir_all(self.dir)?;
        let path = self.dir.join(format!("{stem}.{ext}"));
        let part = self.dir.join(format!(
            "{stem}.{}-{}.part",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&part, bytes)?;
        if let Err(e) = std::fs::rename(&part, &path) {
            let _ = std::fs::remove_file(&part);
            // Another request for the same piece got there first, and the player may be
            // reading its copy.
            if !path.is_file() {
                return Err(e);
            }
        }
        self.prune(&path);
        Ok(path)
    }

    /// Removes the copies heard longest ago until the folder fits under the cap; never `keep`.
    fn prune(&self, keep: &Path) {
        let Ok(entries) = std::fs::read_dir(self.dir) else {
            return;
        };
        let mut files: Vec<(SystemTime, u64, PathBuf)> = entries
            .filter_map(Result::ok)
            .filter_map(|e| {
                let meta = e.metadata().ok().filter(|m| m.is_file())?;
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                Some((modified, meta.len(), e.path()))
            })
            .collect();
        let mut total: u64 = files.iter().map(|(_, len, _)| len).sum();
        files.sort();
        for (_, len, path) in files {
            if total <= self.cap_bytes {
                break;
            }
            // A copy the player has open cannot be removed on Windows; it goes next time.
            if path != keep && std::fs::remove_file(&path).is_ok() {
                total -= len;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn age(path: &Path, seconds: u64) {
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(seconds))
            .unwrap();
    }

    #[test]
    fn a_kept_copy_is_found_again_in_either_format() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ListeningCache::new(dir.path(), 1 << 20);
        assert_eq!(cache.get("abc-0-1000"), None);
        let ogg = cache.put("abc-0-1000", "ogg", b"OggS").unwrap();
        let wav = cache.put("abc-1000-2000", "wav", b"RIFF").unwrap();
        assert_eq!(cache.get("abc-0-1000"), Some(ogg));
        assert_eq!(cache.get("abc-1000-2000"), Some(wav));
        assert_eq!(
            cache.get("abc-0-2000"),
            None,
            "another span is another piece"
        );
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(names.len(), 2, "no part-written files are left behind");
    }

    #[test]
    fn the_copies_heard_longest_ago_go_first() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ListeningCache::new(dir.path(), 25);
        let old = cache.put("old", "ogg", &[0; 10]).unwrap();
        let heard = cache.put("heard", "ogg", &[0; 10]).unwrap();
        age(&old, 300);
        age(&heard, 600);
        // Hearing a copy again makes it the newest.
        cache.get("heard").unwrap();
        let new = cache.put("new", "ogg", &[0; 10]).unwrap();
        assert!(!old.exists(), "over the cap, the oldest is removed");
        assert!(heard.exists() && new.exists());
    }

    #[test]
    fn the_copy_just_made_stays_even_over_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ListeningCache::new(dir.path(), 5);
        let big = cache.put("big", "ogg", &[0; 10]).unwrap();
        assert!(big.exists());
    }
}
