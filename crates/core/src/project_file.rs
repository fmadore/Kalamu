//! A project saved as one file, to open on another computer (#213): what the store writes and
//! reads ([`ProjectArchive`]), what the interface is told before, during and after, and why a
//! file is refused. The file's layout is the store's; it is described in `docs/formats.md`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{ProjectId, StoreError, Timestamp};

/// The extension of a project file, without its dot.
pub const EXTENSION: &str = "kalamu";
/// What a project file's manifest says it is.
pub const FORMAT: &str = "kalamu-project";
/// The version of the file's own tables (the manifest and the files it carries); the job
/// database's tables have the schema's version beside it.
pub const VERSION: u32 = 1;

/// Where the files a project file carries live on this computer.
#[derive(Debug, Clone)]
pub struct ArchivePlaces {
    /// Kalamu's frozen texts, named by their content (`<sha256>.txt`): always carried.
    pub snapshots: PathBuf,
    /// Where the originals a file carries are written when it is opened, a folder per project
    /// named by its id: deleting the project removes its folder (#274).
    pub originals: PathBuf,
    /// Where a file saved by an older Kalamu is copied to be brought up to date, the copy
    /// deleted when done and whatever a crash left emptied at launch: a folder of the app's
    /// own, never the system's temporary folder (#274).
    pub working: PathBuf,
    /// This Kalamu's version, recorded in the manifest.
    pub app_version: String,
}

/// What saving a project would write, shown before the researcher chooses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectFilePlan {
    pub name: String,
    pub documents: u32,
    /// The jobs that read the project's documents, as the project's card counts them; the
    /// dictionary's proposal jobs, carried too, read none and are not counted.
    pub jobs: u32,
    /// About how many bytes the file takes without the originals.
    #[ts(type = "number")]
    pub results_bytes: u64,
    /// The originals found on this computer, and what they weigh.
    pub originals: u32,
    #[ts(type = "number")]
    pub originals_bytes: u64,
    /// The originals no longer where they were added: left out even when chosen.
    pub originals_missing: u32,
    /// The file's name offered, from the project's, with its extension.
    pub file_name: String,
}

/// How far a save or an open has got, in bytes of the files it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectFileProgress {
    #[ts(type = "number")]
    pub done: u64,
    #[ts(type = "number")]
    pub total: u64,
}

/// A project saved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectFileSaved {
    /// The file, as written.
    pub path: String,
    #[ts(type = "number")]
    pub bytes: u64,
    /// Originals carried, and those that could not be (not found, or changed since).
    pub originals: u32,
    pub originals_left: u32,
    /// The file written, checked as *Check a project file…* checks one before it took its
    /// name: a file that does not check fails the save, so this one is whole.
    pub check: ProjectFileCheck,
}

/// A project opened from a file: a new project beside the others.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectFileOpened {
    pub project_id: ProjectId,
    /// Its name here, which takes a date when a project of this computer has the file's.
    pub name: String,
    pub documents: u32,
    /// The jobs that read the project's documents, as the project's card counts them; the
    /// dictionary's proposal jobs, carried too, read none and are not counted.
    pub jobs: u32,
    /// Originals the file carried, now in Kalamu's data folder.
    pub originals: u32,
    /// Whether the file was saved with its originals.
    pub with_originals: bool,
    /// When the file was saved.
    pub saved_at: Timestamp,
    /// The frozen texts the file should have carried and did not: a file saved by 1.10.0 when
    /// a text had gone from the data folder, or changed, since a job read it (#239). The jobs
    /// that read them are restored, without the text behind their answers.
    pub missing_texts: Vec<MissingText>,
}

/// A frozen text a project file did not carry, and who read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MissingText {
    /// The documents the text was read for, by their titles; empty when none is linked.
    pub documents: Vec<String>,
    /// The jobs that read it, by their names.
    pub jobs: Vec<String>,
    /// The text's name as its source records it, for when no document names it.
    pub file_name: String,
}

/// What a check of a project file found, in one word: whether opening it would bring back
/// everything it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ProjectFileVerdict {
    /// Everything in the file can be opened as it was saved.
    Whole,
    /// It opens, but frozen texts its jobs read are not in it (a file saved by 1.10.0).
    Incomplete,
    /// Files it carries no longer match their fingerprints: opening it would be refused.
    Damaged,
}

/// A project file checked without opening it (#246): read, never written, nothing created and
/// no provider called. A file this version cannot read at all (not a project file, damaged
/// beyond reading, or newer) is refused with the same error as opening it instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProjectFileCheck {
    pub verdict: ProjectFileVerdict,
    /// The project's name, as the file saved it.
    pub name: String,
    pub saved_at: Timestamp,
    /// The version of Kalamu that saved it.
    pub app_version: String,
    pub documents: u32,
    /// The jobs that read the project's documents, as the project's card counts them.
    pub jobs: u32,
    /// The frozen texts carried whole.
    pub texts: u32,
    /// The frozen texts the jobs read that the file does not carry, and who read them.
    pub missing_texts: Vec<MissingText>,
    /// The files it carries whose bytes no longer match their fingerprints, by their names.
    pub damaged: Vec<String>,
    /// Whether the file was saved with its originals.
    pub with_originals: bool,
    /// Originals carried whole.
    pub originals: u32,
    /// Originals left out because the file was saved without them.
    pub originals_not_chosen: u32,
    /// Originals chosen that could not be carried (not found, or changed since read).
    pub originals_left: u32,
    /// The documents whose original files are not in the file, by their titles: on another
    /// computer they show as missing until their files are found again.
    pub to_find: Vec<String>,
    /// The originals that opening would write under another name on this computer: a name
    /// this system refuses, or two names that become one here (#238).
    pub renamed: Vec<RenamedOriginal>,
}

/// An original that opening a project file would write under another name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RenamedOriginal {
    /// The documents it belongs to, by their titles; empty when none is linked.
    pub documents: Vec<String>,
    /// Its name as it was saved, and the name it would take here.
    pub from: String,
    pub to: String,
}

/// The manifest at the head of a project file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    /// The job database's schema the rows follow (its number of migrations).
    pub schema: u32,
    pub app_version: String,
    pub saved_at: Timestamp,
    pub project_name: String,
    pub documents: u32,
    /// The jobs that read the project's documents, as the project's card counts them; the
    /// dictionary's proposal jobs, carried too, read none and are not counted.
    pub jobs: u32,
    /// Whether the originals were chosen; those not found are listed as not carried.
    pub with_originals: bool,
}

#[derive(Debug, Error)]
pub enum ProjectFileError {
    #[error("This is not a Kalamu project file")]
    NotProjectFile,
    #[error("The project file is damaged: {0}")]
    Damaged(String),
    /// A format or a schema this version does not know.
    #[error("The project file was made by a newer version of Kalamu ({0})")]
    Newer(String),
    #[error("The project file could not be written: {0}")]
    Write(String),
    #[error("The project file could not be read: {0}")]
    Read(String),
    /// A frozen text a job read is no longer in the data folder, so the project was not saved;
    /// the text's readers are named, its document then its jobs.
    #[error("A text a job read is missing from Kalamu's data folder ({0})")]
    FrozenTextMissing(String),
    /// A frozen text whose bytes no longer match what the job read, so the project was not
    /// saved.
    #[error("A text a job read has changed since it was read ({0})")]
    FrozenTextChanged(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl ProjectFileError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotProjectFile => "project.file_not_kalamu",
            Self::Damaged(_) => "project.file_damaged",
            Self::Newer(_) => "project.file_newer",
            Self::Write(_) => "project.file_write",
            Self::Read(_) => "project.file_read",
            Self::FrozenTextMissing(_) => "project.frozen_text_missing",
            Self::FrozenTextChanged(_) => "project.frozen_text_changed",
            Self::Store(StoreError::NotFound) => "project.not_found",
            Self::Store(_) => "job.database",
        }
    }
}

/// Saves a project to a file, checks one, and opens one as a new project. Implemented by the
/// job database, whose tables the file holds.
pub trait ProjectArchive: Send + Sync {
    /// What saving `project` would write, with and without its originals.
    fn plan_project_file(
        &self,
        project: ProjectId,
        places: &ArchivePlaces,
    ) -> Result<ProjectFilePlan, ProjectFileError>;

    /// Writes `project` to `path`, through a part renamed once complete.
    fn save_project_file(
        &self,
        project: ProjectId,
        path: &Path,
        with_originals: bool,
        places: &ArchivePlaces,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileSaved, ProjectFileError>;

    /// Opens the file at `path` as a new project, all or nothing.
    fn open_project_file(
        &self,
        path: &Path,
        places: &ArchivePlaces,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileOpened, ProjectFileError>;

    /// Checks the file at `path` without opening it: whether it can be read, whether each file
    /// it carries matches its fingerprint, and what opening it would not bring back as it was.
    /// Writes nothing outside `places.working` and changes nothing in the store.
    fn check_project_file(
        &self,
        path: &Path,
        places: &ArchivePlaces,
        progress: &dyn Fn(ProjectFileProgress),
    ) -> Result<ProjectFileCheck, ProjectFileError>;

    /// How many of the originals Kalamu copied in when `project` was opened from a file
    /// deleting it would remove: those in its folder under `places.originals` that no
    /// document of another project names.
    fn copied_originals(
        &self,
        project: ProjectId,
        places: &ArchivePlaces,
    ) -> Result<u32, ProjectFileError>;

    /// Removes the originals Kalamu copied in for `project`, once the project is deleted: only
    /// files in its own folder under `places.originals`, never one a document of another
    /// project names. Refused while the project exists.
    fn remove_copied_originals(
        &self,
        project: ProjectId,
        places: &ArchivePlaces,
    ) -> Result<FilesRemoved, ProjectFileError>;

    /// What a crash or an earlier version left, swept at launch before anything else runs:
    /// the working copies in `places.working`, and the folders under `places.originals` whose
    /// project no longer exists.
    fn tidy_project_files(&self, places: &ArchivePlaces) -> Result<FilesRemoved, ProjectFileError>;
}

/// What removing copied-in originals, or tidying at launch, did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FilesRemoved {
    /// Files removed.
    pub removed: u32,
    /// Files kept because a document of another project names them.
    pub kept: u32,
    /// Files that could not be removed: tried again at the next launch.
    pub failed: u32,
}

/// A path's folders and name, written with either system's separator: a path saved on
/// Windows read on a Mac, or the reverse, without asking the system to parse it.
pub fn path_parts(path: &str) -> Vec<&str> {
    path.split(['\\', '/']).filter(|p| !p.is_empty()).collect()
}

/// The last part of a path written on any system; empty for an empty path.
pub fn last_part(path: &str) -> &str {
    path_parts(path).pop().unwrap_or_default()
}

/// The folder of a path written on any system: everything before its last separator.
pub fn folder_part(path: &str) -> &str {
    path.rfind(['\\', '/']).map_or("", |at| &path[..at])
}

/// A file or folder name both Windows and macOS accept: what either refuses (`<>:"/\|?*`,
/// control characters) made `_`, a trailing dot or space taken off, and a name left empty, or
/// one Windows keeps for a device, given `fallback`.
pub fn portable_name(name: &str, fallback: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let cleaned = cleaned.trim_end_matches(['.', ' ']).trim_start();
    let stem = cleaned.split('.').next().unwrap_or_default().to_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem[3..].chars().all(|c| c.is_ascii_digit()));
    if cleaned.is_empty() || reserved {
        fallback.to_string()
    } else {
        cleaned.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_of_either_system_are_split_alike() {
        let windows = r"C:\Users\Ama\Archives\lettre 1.pdf";
        let mac = "/Users/ama/Archives/lettre 1.pdf";
        assert_eq!(last_part(windows), "lettre 1.pdf");
        assert_eq!(last_part(mac), "lettre 1.pdf");
        assert_eq!(folder_part(windows), r"C:\Users\Ama\Archives");
        assert_eq!(folder_part(mac), "/Users/ama/Archives");
        assert_eq!(
            path_parts(r"\\server\share\a.pdf"),
            ["server", "share", "a.pdf"]
        );
        assert_eq!(last_part(""), "");
        assert_eq!(folder_part("a.pdf"), "");
    }

    #[test]
    fn names_are_kept_to_what_both_systems_accept() {
        assert_eq!(portable_name("Lettres: 1915?", "x"), "Lettres_ 1915_");
        assert_eq!(portable_name("Rapport. ", "x"), "Rapport");
        assert_eq!(portable_name("con.txt", "file"), "file");
        assert_eq!(portable_name("COM1", "file"), "file");
        assert_eq!(portable_name("Comores.pdf", "file"), "Comores.pdf");
        assert_eq!(portable_name("...", "file"), "file");
        assert_eq!(portable_name("Église d’Ouidah", "x"), "Église d’Ouidah");
    }

    #[test]
    fn every_failure_has_its_own_sentence() {
        let codes = [
            ProjectFileError::NotProjectFile.code(),
            ProjectFileError::Damaged(String::new()).code(),
            ProjectFileError::Newer(String::new()).code(),
            ProjectFileError::Write(String::new()).code(),
            ProjectFileError::Read(String::new()).code(),
            ProjectFileError::FrozenTextMissing(String::new()).code(),
            ProjectFileError::FrozenTextChanged(String::new()).code(),
        ];
        let unique: std::collections::BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }
}
