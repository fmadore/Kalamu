//! What a run reads from disk: the page image and a text's files, each run's in a folder of its
//! own in the scratch folder, deleted afterwards, and swept at start-up when a crash left them.

use std::path::{Path, PathBuf};

use kalamu_core::ProviderError;

/// The names of the folders a run's files are written in, inside the scratch folder: only
/// folders with these names are swept from it.
const SCRATCH_PREFIXES: [&str; 2] = ["kalamu-text-", "kalamu-page-"];

/// A folder of its own for one run's files, in `root` (the app's scratch folder, or the
/// system's temporary folder when there is none), deleted when dropped.
fn scratch_dir(root: &Path, prefix: &str) -> std::io::Result<tempfile::TempDir> {
    let builder = {
        let mut builder = tempfile::Builder::new();
        builder.prefix(prefix);
        builder
    };
    if root.as_os_str().is_empty() {
        builder.tempdir()
    } else {
        std::fs::create_dir_all(root)?;
        builder.tempdir_in(root)
    }
}

/// Deletes what runs left in the scratch folder when the app was killed or crashed before it
/// could delete their files: the page images and a text's instructions and contents, the
/// researcher's material. Only the folders runs make are touched, so a wrong root deletes
/// nothing else. Called once, before any run.
pub(crate) fn sweep_scratch(root: &Path) {
    if root.as_os_str().is_empty() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut swept = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let ours = SCRATCH_PREFIXES
            .iter()
            .any(|prefix| name.to_string_lossy().starts_with(prefix));
        if ours
            && entry.file_type().is_ok_and(|t| t.is_dir())
            && std::fs::remove_dir_all(entry.path()).is_ok()
        {
            swept += 1;
        }
    }
    if swept > 0 {
        tracing::info!(swept, "files of interrupted offline runs deleted");
    }
}

/// Files written where the program can read them (a text's instructions, the text, the answer's
/// schema), deleted afterwards.
pub struct Scratch {
    dir: tempfile::TempDir,
}

impl Scratch {
    /// A folder for one request's files in `root`, the app's scratch folder.
    pub fn new(root: &Path) -> Result<Self, ProviderError> {
        let dir = scratch_dir(root, SCRATCH_PREFIXES[0])
            .map_err(|e| ProviderError::Other(format!("Could not write the request: {e}")))?;
        Ok(Self { dir })
    }

    /// Writes `bytes` as `name` and returns its path.
    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, ProviderError> {
        let path = self.dir.path().join(name);
        std::fs::write(&path, bytes)
            .map_err(|e| ProviderError::Other(format!("Could not write the request: {e}")))?;
        Ok(path)
    }
}

/// A page written where the program can read it, deleted afterwards.
pub struct PageFile {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl PageFile {
    /// Writes the page in a folder of its own in `root`, the app's scratch folder.
    pub fn write(root: &Path, bytes: &[u8], extension: &str) -> Result<Self, ProviderError> {
        let dir = scratch_dir(root, SCRATCH_PREFIXES[1])
            .map_err(|e| ProviderError::Other(format!("Could not write the page image: {e}")))?;
        let path = dir.path().join(format!("page.{extension}"));
        std::fs::write(&path, bytes)
            .map_err(|e| ProviderError::Other(format!("Could not write the page image: {e}")))?;
        Ok(Self { _dir: dir, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// The page as the model should see it: no larger than `max_side` on its longest side, as PNG,
/// or the original bytes when they are small enough already.
pub fn page_image(
    bytes: &[u8],
    mime: &str,
    max_side: u32,
) -> Result<(Vec<u8>, &'static str), ProviderError> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| ProviderError::InvalidRequest(format!("The page image is unreadable: {e}")))?;
    if image.width().max(image.height()) <= max_side {
        let extension = if mime == "image/png" { "png" } else { "jpg" };
        return Ok((bytes.to_vec(), extension));
    }
    let resized = image.resize(max_side, max_side, image::imageops::FilterType::Lanczos3);
    let mut png = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| ProviderError::Other(format!("Could not prepare the page image: {e}")))?;
    Ok((png.into_inner(), "png"))
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn run_files_go_in_the_scratch_folder_and_what_a_crash_left_is_swept() {
        let root = tempfile::tempdir().unwrap();
        let scratch = root.path().join("offline-scratch");
        let page = PageFile::write(&scratch, b"png", "png").unwrap();
        assert!(page.path().starts_with(&scratch));
        let request = Scratch::new(&scratch).unwrap();
        let text = request.write("request.txt", b"a text").unwrap();
        assert!(text.starts_with(&scratch));
        // As a kill leaves them: never dropped.
        std::mem::forget(page);
        std::mem::forget(request);
        // Something else in the folder is not the sweep's to delete.
        std::fs::write(scratch.join("notes.txt"), b"kept").unwrap();
        std::fs::create_dir(scratch.join("kept")).unwrap();
        assert_eq!(std::fs::read_dir(&scratch).unwrap().count(), 4);
        sweep_scratch(&scratch);
        let mut left: Vec<String> = std::fs::read_dir(&scratch)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, ["kept", "notes.txt"]);
        // No root: the system's temporary folder, never swept.
        sweep_scratch(Path::new(""));
        let page = PageFile::write(Path::new(""), b"png", "png").unwrap();
        assert!(page.path().starts_with(std::env::temp_dir()));
    }
}
