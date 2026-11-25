//! The originals a project file carried, written into the data folder when it was opened
//! (`originals/<project>`), removed once the project is deleted (#274); and what a crash or an
//! earlier version left, swept at launch: working copies of older files, and the folders of
//! projects that no longer exist.
//!
//! Only files under the project's own folder are touched, never one the researcher keeps
//! elsewhere; and a file there that a document of another project names (a document added from
//! one of the project's jobs) is kept. The jobs made in the project stay, as deleting a project
//! always left them: their results and corrections are in the store, and a review of the file
//! itself shows it as missing, which the confirmation before deleting says.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use kalamu_core::project_file::ArchivePlaces;
use kalamu_core::{FilesRemoved, ProjectFileError, ProjectId, StoreError};
use rusqlite::OptionalExtension;

use super::check::WORKING_PREFIX;
use crate::{SqliteStore, db};

fn store_err(e: rusqlite::Error) -> ProjectFileError {
    ProjectFileError::Store(db(e))
}

/// How many files deleting `project` would remove from its folder.
pub(super) fn count(
    store: &SqliteStore,
    project: ProjectId,
    places: &ArchivePlaces,
) -> Result<u32, ProjectFileError> {
    let root = places.originals.join(project.to_string());
    if !root.is_dir() {
        return Ok(0);
    }
    let named = named_by_documents(store, Some(project))?;
    Ok(files_under(&root)
        .iter()
        .filter(|f| !named.contains(*f))
        .count() as u32)
}

/// Removes the project's folder of originals, but for what another project's documents name.
pub(super) fn remove(
    store: &SqliteStore,
    project: ProjectId,
    places: &ArchivePlaces,
) -> Result<FilesRemoved, ProjectFileError> {
    if exists(store, project)? {
        return Err(ProjectFileError::Write(format!(
            "the project {project} still exists, so its original files are kept"
        )));
    }
    let root = places.originals.join(project.to_string());
    if !root.is_dir() {
        return Ok(FilesRemoved::default());
    }
    let named = named_by_documents(store, None)?;
    Ok(clear(&root, &named))
}

/// Empties the working folder of the copies older files were brought up to date on, and
/// removes the folders of originals whose project no longer exists. Run at launch, before any
/// file is opened or checked, so nothing it removes is in use.
pub(super) fn tidy(
    store: &SqliteStore,
    places: &ArchivePlaces,
) -> Result<FilesRemoved, ProjectFileError> {
    let mut done = FilesRemoved::default();
    for entry in std::fs::read_dir(&places.working)
        .into_iter()
        .flatten()
        .flatten()
    {
        let is_copy = entry
            .file_name()
            .to_str()
            .is_some_and(|n| n.starts_with(WORKING_PREFIX));
        let is_file = entry.file_type().is_ok_and(|t| t.is_file());
        if is_copy && is_file {
            match std::fs::remove_file(entry.path()) {
                Ok(()) => done.removed += 1,
                Err(_) => done.failed += 1,
            }
        }
    }

    let mut named: Option<HashSet<PathBuf>> = None;
    for entry in std::fs::read_dir(&places.originals)
        .into_iter()
        .flatten()
        .flatten()
    {
        // Only a folder named by a project's id, as opening names them; anything else is not
        // Kalamu's to remove.
        let Some(project) = entry
            .file_name()
            .to_str()
            .and_then(|n| n.parse::<ProjectId>().ok())
        else {
            continue;
        };
        if !entry.file_type().is_ok_and(|t| t.is_dir()) || exists(store, project)? {
            continue;
        }
        if named.is_none() {
            named = Some(named_by_documents(store, None)?);
        }
        let Some(keep) = &named else { continue };
        let cleared = clear(&entry.path(), keep);
        done.removed += cleared.removed;
        done.kept += cleared.kept;
        done.failed += cleared.failed;
    }
    Ok(done)
}

fn exists(store: &SqliteStore, project: ProjectId) -> Result<bool, ProjectFileError> {
    store
        .conn()
        .query_row(
            "SELECT 1 FROM projects WHERE id = ?1",
            [project.to_string()],
            |_| Ok(()),
        )
        .optional()
        .map(|found| found.is_some())
        .map_err(store_err)
}

/// Every file a document names (its own, or its pages'), of every project but `except`.
fn named_by_documents(
    store: &SqliteStore,
    except: Option<ProjectId>,
) -> Result<HashSet<PathBuf>, ProjectFileError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare("SELECT source FROM documents WHERE project_id IS NOT ?1")
        .map_err(store_err)?;
    let sources = stmt
        .query_map([except.map(|p| p.to_string())], |row| {
            row.get::<_, String>(0)
        })
        .map_err(store_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(store_err)?;
    let mut named = HashSet::new();
    for source in sources {
        let source: serde_json::Value = serde_json::from_str(&source)
            .map_err(|e| ProjectFileError::Store(StoreError::Corrupt(e.to_string())))?;
        let pages = source["pages"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        named.extend(
            std::iter::once(&source["path"])
                .chain(pages.iter().map(|p| &p["path"]))
                .filter_map(|p| p.as_str().map(PathBuf::from)),
        );
    }
    Ok(named)
}

/// Every file under `root`, its folders walked without following a link out of it.
fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).into_iter().flatten().flatten() {
            match entry.file_type() {
                Ok(t) if t.is_dir() => folders.push(entry.path()),
                Ok(_) => files.push(entry.path()),
                Err(_) => {}
            }
        }
    }
    files
}

/// Removes every file under `root` but those `keep` names, then the folders left empty,
/// `root` included.
fn clear(root: &Path, keep: &HashSet<PathBuf>) -> FilesRemoved {
    let mut done = FilesRemoved::default();
    for file in files_under(root) {
        if keep.contains(&file) {
            done.kept += 1;
            continue;
        }
        match std::fs::remove_file(&file) {
            Ok(()) => done.removed += 1,
            Err(_) => done.failed += 1,
        }
    }
    let mut folders = vec![root.to_path_buf()];
    let mut i = 0;
    while i < folders.len() {
        for entry in std::fs::read_dir(&folders[i])
            .into_iter()
            .flatten()
            .flatten()
        {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                folders.push(entry.path());
            }
        }
        i += 1;
    }
    // The deepest first: a folder is empty only once those inside it are gone.
    for folder in folders.iter().rev() {
        let _ = std::fs::remove_dir(folder);
    }
    done
}
