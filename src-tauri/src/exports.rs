//! The files Kalamu's table exports wrote, so that exporting again to the same name replaces
//! them and nothing else.
//!
//! A table export writes a workbook and a CSV beside the file the researcher chose, files the
//! save dialog never asked about. A CSV has no place for a mark saying who wrote it, and a
//! workbook opened and saved again keeps whatever mark it had, so the files are not recognised
//! by what they hold: Kalamu remembers each file it wrote, by path and by its bytes' SHA-256,
//! in `exports.json` in its data folder. A file there is Kalamu's own only while its bytes are
//! the ones written; once the researcher changes it, it is theirs. Losing the list loses
//! nothing but recognition: the next export writes beside the files under a numbered name.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kalamu_core::fingerprint::sha256_hex;
use serde::{Deserialize, Serialize};

/// How many files are remembered; the oldest are forgotten first.
const KEPT: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Written {
    path: PathBuf,
    sha256: String,
}

pub struct ExportLedger {
    file: PathBuf,
    /// Oldest first.
    written: Mutex<Vec<Written>>,
}

impl ExportLedger {
    /// The files remembered in `file`; none when it is missing or cannot be read.
    pub fn open(file: PathBuf) -> Self {
        let written = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self {
            file,
            written: Mutex::new(written),
        }
    }

    /// Whether the file at `path` is one an export wrote, its bytes unchanged since.
    pub fn wrote(&self, path: &Path) -> bool {
        let Some(sha256) = self
            .lock()
            .iter()
            .rev()
            .find(|w| w.path == path)
            .map(|w| w.sha256.clone())
        else {
            return false;
        };
        std::fs::read(path).is_ok_and(|bytes| sha256_hex(&bytes) == sha256)
    }

    /// Remembers that an export wrote `bytes` at `path`. A list that cannot be saved is said in
    /// the log only: the export itself succeeded.
    pub fn record(&self, path: &Path, bytes: &[u8]) {
        let mut written = self.lock();
        written.retain(|w| w.path != path);
        written.push(Written {
            path: path.to_path_buf(),
            sha256: sha256_hex(bytes),
        });
        let over = written.len().saturating_sub(KEPT);
        written.drain(..over);
        if let Err(e) = crate::settings::write_json(&self.file, &*written) {
            tracing::warn!(error = %e, "could not save the list of exported files");
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Written>> {
        self.written.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_kalamus_while_its_bytes_are_the_ones_written() {
        let dir = tempfile::tempdir().unwrap();
        let list = dir.path().join("exports.json");
        let csv = dir.path().join("table.csv");
        let ledger = ExportLedger::open(list.clone());
        assert!(!ledger.wrote(&csv));

        std::fs::write(&csv, "a,b\n").unwrap();
        assert!(!ledger.wrote(&csv), "not written by an export");
        ledger.record(&csv, b"a,b\n");
        assert!(ledger.wrote(&csv));
        // Remembered at the next launch.
        assert!(ExportLedger::open(list.clone()).wrote(&csv));

        std::fs::write(&csv, "a,b,my notes\n").unwrap();
        assert!(!ledger.wrote(&csv), "changed by the researcher");
        std::fs::remove_file(&csv).unwrap();
        assert!(!ledger.wrote(&csv));
    }

    #[test]
    fn the_oldest_files_are_forgotten_first() {
        let dir = tempfile::tempdir().unwrap();
        let ledger = ExportLedger::open(dir.path().join("exports.json"));
        let first = dir.path().join("first.csv");
        std::fs::write(&first, "x").unwrap();
        ledger.record(&first, b"x");
        for n in 0..KEPT {
            ledger.record(&dir.path().join(format!("{n}.csv")), b"y");
        }
        assert!(!ledger.wrote(&first));
        assert_eq!(ledger.lock().len(), KEPT);
    }
}
