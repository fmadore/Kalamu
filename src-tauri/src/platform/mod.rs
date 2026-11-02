//! What differs between the operating systems Kalamu ships for, in one place: each OS has its
//! own file with the same items, and the rest of the shell uses them without asking which OS
//! it runs on. The crates below the shell receive these facts as values (the player's formats
//! go to the engine through its config), so none of them matches on the OS.
//!
//! Adding an operating system is one more file here with the same items.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(any(windows, target_os = "macos")))]
compile_error!("Kalamu ships for Windows and macOS: add src-tauri/src/platform/<os>.rs first.");

use std::path::PathBuf;

/// What became of an earlier data folder at this start.
pub enum Moved {
    /// There was none, or the data folder already exists.
    Nothing,
    /// It was moved into the data folder.
    From(PathBuf),
    /// It could not be moved, so it stays where it is and is used there.
    Failed(PathBuf, std::io::Error),
}

/// The folder to keep data in: `folder`, into which the first of `earlier` that exists is
/// moved, once, while `folder` does not exist yet. Every stored path is relative to the data
/// folder, so it moves whole. If the move fails, the earlier folder is left as it was and used,
/// so nothing seems lost.
fn settle(folder: PathBuf, earlier: &[PathBuf]) -> (PathBuf, Moved) {
    if folder.exists() {
        return (folder, Moved::Nothing);
    }
    let Some(old) = earlier.iter().find(|p| p.is_dir()) else {
        return (folder, Moved::Nothing);
    };
    match std::fs::rename(old, &folder) {
        Ok(()) => (folder, Moved::From(old.clone())),
        Err(e) => (old.clone(), Moved::Failed(old.clone(), e)),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{Moved, settle};

    #[test]
    fn the_newest_earlier_data_folder_moves_once() {
        let root = tempfile::tempdir().unwrap();
        let kalamu = root.path().join("io.github.fmadore.kalamu");
        let scriptorium = root.path().join("io.github.fmadore.scriptorium");
        let data = root.path().join("Kalamu Data");
        std::fs::create_dir_all(kalamu.join("presets")).unwrap();
        std::fs::write(kalamu.join("jobs.sqlite"), b"jobs").unwrap();
        std::fs::create_dir_all(&scriptorium).unwrap();
        let earlier = [kalamu.clone(), scriptorium.clone()];

        let (used, moved) = settle(data.clone(), &earlier);
        assert_eq!(used, data);
        assert!(matches!(moved, Moved::From(ref old) if *old == kalamu));
        assert_eq!(std::fs::read(data.join("jobs.sqlite")).unwrap(), b"jobs");
        assert!(data.join("presets").is_dir());
        assert!(!kalamu.exists());

        // An earlier folder left behind never replaces the data folder.
        let (used, moved) = settle(data.clone(), &earlier);
        assert_eq!(used, data);
        assert!(matches!(moved, Moved::Nothing));
        assert!(scriptorium.exists());
    }

    #[test]
    fn a_folder_that_cannot_move_stays_in_use() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("io.github.fmadore.kalamu");
        std::fs::create_dir_all(&old).unwrap();
        // A parent that does not exist makes the move fail.
        let data = root.path().join("missing").join("Kalamu Data");

        let (used, moved) = settle(data.clone(), std::slice::from_ref(&old));
        assert_eq!(used, old);
        assert!(matches!(moved, Moved::Failed(..)));
        assert!(!data.exists());
    }
}
