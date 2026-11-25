//! Opening an index file of its own (the word search's, the vectors'), and replacing it only
//! when SQLite says it cannot be read: not a database, or damaged. Any other failure (a full
//! disk, a folder it may not write in, a file held by another program) is returned and the
//! file left where it is, so a passing fault never discards what the index holds (#273).

use std::ffi::OsString;
use std::io;
use std::path::Path;

use kalamu_core::StoreError;
use rusqlite::ErrorCode;

use super::db;

/// What becomes of an index file SQLite cannot read before a new one is made in its place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unreadable {
    /// Deleted, for an index made again for free.
    Delete,
    /// Renamed `<name>.unreadable` (its log `<name>.unreadable-wal`), replacing one set aside
    /// before, for an index whose making was billed.
    SetAside,
}

/// The suffix of a file set aside.
pub(crate) const ASIDE: &str = ".unreadable";

/// Opens the file at `path` with `open`; when SQLite says it is not a database or is damaged,
/// deletes it or sets it aside as `unreadable` says, with its log, and opens it anew.
pub(crate) fn open_index<T>(
    path: &Path,
    unreadable: Unreadable,
    open: impl Fn(&Path) -> rusqlite::Result<T>,
) -> Result<T, StoreError> {
    match open(path) {
        Ok(index) => Ok(index),
        Err(e) if cannot_be_read(&e) => {
            replace(path, unreadable).map_err(|io| {
                StoreError::Database(format!(
                    "{} cannot be read ({e}) and could not be replaced: {io}",
                    path.display()
                ))
            })?;
            open(path).map_err(db)
        }
        Err(e) => Err(db(e)),
    }
}

/// Whether SQLite's error says the file is no database it can read, as the project file's
/// `refused` tells them apart.
fn cannot_be_read(e: &rusqlite::Error) -> bool {
    matches!(
        e.sqlite_error_code(),
        Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt)
    )
}

/// `path` with `suffix` added to its name.
fn with_suffix(path: &Path, suffix: &str) -> OsString {
    let mut file = path.as_os_str().to_owned();
    file.push(suffix);
    file
}

/// Removes a file, a missing one being no failure.
fn remove(file: &OsString) -> io::Result<()> {
    match std::fs::remove_file(file) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// The file and its log out of the way; its shared-memory file, rebuilt by SQLite, deleted.
fn replace(path: &Path, unreadable: Unreadable) -> io::Result<()> {
    for log in ["", "-wal"] {
        let file = with_suffix(path, log);
        match unreadable {
            Unreadable::Delete => remove(&file)?,
            Unreadable::SetAside => {
                // What was set aside before goes, its log too, so no old log pairs with a
                // new file.
                let aside = with_suffix(path, &format!("{ASIDE}{log}"));
                remove(&aside)?;
                if Path::new(&file).exists() {
                    std::fs::rename(&file, &aside)?;
                }
            }
        }
    }
    remove(&with_suffix(path, "-shm"))
}
