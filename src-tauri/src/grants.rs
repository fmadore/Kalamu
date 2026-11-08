//! Files and folders the researcher handed Kalamu, kept for later launches where the system
//! requires it.
//!
//! In the Mac App Store's build the App Sandbox lets the app reach, outside its own container,
//! only what the researcher picked in an open or save panel or dropped on the window, and only
//! until the app quits. To resume a job, write its results or open a project's documents after
//! a restart, the app keeps a security-scoped bookmark of each such path and opens them all
//! again at launch, before the engine looks at any job. Elsewhere (Windows, the notarised disk
//! image) nothing is kept and nothing is asked: every path is reachable as it is.
//!
//! Paths stay plain strings everywhere else: the engine, the store and the projects never learn
//! of bookmarks. The bookmarks themselves come from `platform`; this module only decides what
//! to keep and what is missing, so it is tested on any system.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// What the system offers: a bookmark of a path the app can reach now, and the path a bookmark
/// leads to at a later launch, with access opened, and whether it should be made again.
#[derive(Clone, Copy)]
pub struct Access {
    /// Whether the app runs in a sandbox at all. Without one, nothing is kept.
    pub sandboxed: bool,
    pub bookmark: fn(&Path) -> Option<Vec<u8>>,
    pub open: fn(&[u8]) -> Option<(PathBuf, bool)>,
}

pub struct Grants {
    file: PathBuf,
    access: Access,
    /// Each path kept, with its bookmark in base64.
    kept: Mutex<BTreeMap<PathBuf, String>>,
}

impl Grants {
    /// The grants kept in `file`, not yet opened: see [`Grants::restore`].
    pub fn open(file: PathBuf, access: Access) -> Self {
        let kept = if access.sandboxed {
            std::fs::read_to_string(&file)
                .ok()
                .and_then(|s| serde_json::from_str::<BTreeMap<PathBuf, String>>(&s).ok())
                .unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        Self {
            file,
            access,
            kept: Mutex::new(kept),
        }
    }

    /// Opens every kept path again, as the app starts. A bookmark that leads nowhere today is
    /// kept as it is, unopened: the disk may only be unplugged or the share not connected, and
    /// dropping it would leave the next launch unable to reach the folder with nothing asking
    /// for it again. One the system calls stale, or whose item moved, is made again from where
    /// it is now. Returns how many paths are open.
    pub fn restore(&self) -> usize {
        if !self.access.sandboxed {
            return 0;
        }
        let mut kept = self.lock();
        let before = kept.clone();
        let mut restored = BTreeMap::new();
        let mut open = 0;
        for (path, encoded) in before.iter() {
            let Some((now, stale)) = STANDARD
                .decode(encoded)
                .ok()
                .and_then(|data| (self.access.open)(&data))
            else {
                tracing::info!(path = %path.display(), "a folder granted earlier is out of reach");
                restored.insert(path.clone(), encoded.clone());
                continue;
            };
            let encoded = if stale || now != *path {
                (self.access.bookmark)(&now).map_or_else(|| encoded.clone(), |b| STANDARD.encode(b))
            } else {
                encoded.clone()
            };
            restored.insert(now, encoded);
            open += 1;
        }
        if restored != before {
            *kept = restored;
            self.save(&kept);
        }
        open
    }

    /// Keeps these paths, which the researcher has just handed the app, for later launches. A
    /// path inside a folder already kept needs nothing more; a folder kept replaces the paths
    /// inside it.
    pub fn keep<P: AsRef<Path>>(&self, paths: impl IntoIterator<Item = P>) {
        if !self.access.sandboxed {
            return;
        }
        let mut kept = self.lock();
        let mut changed = false;
        for path in paths {
            let path = path.as_ref();
            if path.as_os_str().is_empty() || covered(&kept, path) {
                continue;
            }
            let Some(bookmark) = (self.access.bookmark)(path) else {
                tracing::warn!(path = %path.display(), "no bookmark for a path handed to the app");
                continue;
            };
            kept.retain(|k, _| !k.starts_with(path));
            kept.insert(path.to_path_buf(), STANDARD.encode(bookmark));
            changed = true;
        }
        if changed {
            self.save(&kept);
        }
    }

    /// The folders results would be written to beside these paths, which the app may not write
    /// to yet: picking a file grants the file, not its folder. Empty outside a sandbox.
    pub fn folders_to_ask<P: AsRef<Path>>(
        &self,
        paths: impl IntoIterator<Item = P>,
    ) -> Vec<PathBuf> {
        if !self.access.sandboxed {
            return Vec::new();
        }
        let kept = self.lock();
        let folders: BTreeSet<PathBuf> = paths
            .into_iter()
            .filter_map(|p| p.as_ref().parent().map(Path::to_path_buf))
            .filter(|folder| !folder.as_os_str().is_empty() && !covered(&kept, folder))
            .collect();
        folders.into_iter().collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, String>> {
        self.kept.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Written whole through a temporary file, as the settings are, so a crash never leaves
    /// half a list.
    fn save(&self, kept: &BTreeMap<PathBuf, String>) {
        if let Err(e) = crate::settings::write_json(&self.file, kept) {
            tracing::warn!(error = %e, "granted folders not saved");
        }
    }
}

/// Whether `path` is kept, or lies inside a folder that is.
fn covered(kept: &BTreeMap<PathBuf, String>, path: &Path) -> bool {
    path.ancestors().any(|a| kept.contains_key(a))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Access, Grants};

    /// A bookmark is the path itself; one under `gone` no longer resolves, one under `moved`
    /// now leads to `elsewhere`, and one under `stale` asks to be made again.
    fn bookmark(path: &Path) -> Option<Vec<u8>> {
        Some(path.to_string_lossy().into_owned().into_bytes())
    }
    fn open(data: &[u8]) -> Option<(PathBuf, bool)> {
        let path = PathBuf::from(String::from_utf8(data.to_vec()).ok()?);
        if path.starts_with("/gone") {
            return None;
        }
        if let Ok(rest) = path.strip_prefix("/moved") {
            return Some((Path::new("/elsewhere").join(rest), false));
        }
        let stale = path.starts_with("/stale");
        Some((path, stale))
    }
    const SANDBOX: Access = Access {
        sandboxed: true,
        bookmark,
        open,
    };

    fn grants(dir: &tempfile::TempDir, access: Access) -> Grants {
        Grants::open(dir.path().join("grants.json"), access)
    }

    #[test]
    fn nothing_is_kept_or_asked_outside_a_sandbox() {
        let dir = tempfile::tempdir().unwrap();
        let g = grants(
            &dir,
            Access {
                sandboxed: false,
                ..SANDBOX
            },
        );
        g.keep(["/archive/letter.pdf"]);
        assert!(g.folders_to_ask(["/archive/letter.pdf"]).is_empty());
        assert_eq!(g.restore(), 0);
        assert!(!dir.path().join("grants.json").exists());
    }

    #[test]
    fn a_picked_file_still_needs_its_folder_for_results_beside_it() {
        let dir = tempfile::tempdir().unwrap();
        let g = grants(&dir, SANDBOX);
        g.keep(["/archive/letter.pdf", "/archive/map.jpg"]);
        assert_eq!(
            g.folders_to_ask(["/archive/letter.pdf", "/archive/map.jpg", "/other/a.wav"]),
            [PathBuf::from("/archive"), PathBuf::from("/other")]
        );
        g.keep(["/archive"]);
        assert_eq!(
            g.folders_to_ask(["/archive/letter.pdf", "/archive/sub/b.pdf"]),
            Vec::<PathBuf>::new()
        );
    }

    #[test]
    fn a_folder_replaces_the_paths_inside_it() {
        let dir = tempfile::tempdir().unwrap();
        let g = grants(&dir, SANDBOX);
        g.keep(["/archive/a.pdf", "/archive/sub/b.pdf", "/other/c.pdf"]);
        g.keep(["/archive"]);
        g.keep(["/archive/sub/d.pdf"]);
        let kept: Vec<PathBuf> = g.lock().keys().cloned().collect();
        assert_eq!(
            kept,
            [PathBuf::from("/archive"), PathBuf::from("/other/c.pdf")]
        );
    }

    #[test]
    fn grants_outlive_a_restart_and_follow_their_folders() {
        let dir = tempfile::tempdir().unwrap();
        grants(&dir, SANDBOX).keep(["/archive", "/gone/old", "/moved/box", "/stale/x.pdf"]);

        let g = grants(&dir, SANDBOX);
        assert_eq!(g.restore(), 3, "the folder out of reach is not opened");
        let kept: Vec<PathBuf> = g.lock().keys().cloned().collect();
        assert_eq!(
            kept,
            [
                PathBuf::from("/archive"),
                PathBuf::from("/elsewhere/box"),
                // An unplugged disk: kept for the launch when it is back.
                PathBuf::from("/gone/old"),
                PathBuf::from("/stale/x.pdf")
            ]
        );
        // What was restored is saved, so the next launch starts from it.
        let again = grants(&dir, SANDBOX);
        assert_eq!(again.lock().len(), 4);
    }
}
