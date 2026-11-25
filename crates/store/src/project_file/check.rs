//! Checking a project file without opening it (#246): whether this Kalamu can read it (a project
//! file, whole, of a format and schema it knows), whether each file it carries matches its
//! fingerprint, and what opening it would not bring back as it was: frozen texts not carried,
//! originals left out, names this system would change. The file is only read; one saved by an
//! older Kalamu is brought up to date on a copy in the app's working folder (#274), deleted when
//! done, as opening does. Nothing is written to the store, no project is created, no provider called.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kalamu_core::project_file::{self as pf, Manifest};
use kalamu_core::{
    JobId, ProjectFileCheck, ProjectFileError, ProjectFileProgress, ProjectFileVerdict,
    RenamedOriginal,
};
use rusqlite::{Connection, ErrorCode, OpenFlags};

use super::carried;
use super::open::Placement;
use super::{ROLE_ORIGINAL, ROLE_SNAPSHOT, read_err, schema};

/// The start of a working copy's name: what the sweep at launch removes from the working folder.
pub(super) const WORKING_PREFIX: &str = "kalamu-open-";

/// A copy of the file in the app's working folder, brought up to this schema, deleted when done;
/// one a crash left is removed at the next launch (`copied::tidy`).
pub(super) struct Upgraded(PathBuf);

impl Drop for Upgraded {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// What SQLite's refusal says of the file.
pub(super) fn refused(e: rusqlite::Error) -> ProjectFileError {
    match e.sqlite_error_code() {
        Some(ErrorCode::NotADatabase) => ProjectFileError::NotProjectFile,
        Some(ErrorCode::DatabaseCorrupt) => ProjectFileError::Damaged(e.to_string()),
        _ if e.to_string().contains("no such table") => ProjectFileError::NotProjectFile,
        _ => read_err(e),
    }
}

/// The upgraded copy the file may be read from, the file opened for reading, and its manifest.
/// The copy comes first so that a caller binding the three in this order closes the connection
/// before the copy is deleted: Windows deletes no file that is still open, and left the copy
/// behind when the connection was bound first. Refused when it is not a project file, is damaged beyond reading, or is newer than this
/// Kalamu.
pub(super) fn readable(
    path: &Path,
    working: &Path,
) -> Result<(Option<Upgraded>, Connection, Manifest), ProjectFileError> {
    if !path.is_file() {
        return Err(read_err(format!("{} is not a file", path.display())));
    }
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let file = Connection::open_with_flags(path, flags).map_err(refused)?;
    let (format, version, manifest): (String, i64, String) = file
        .query_row(
            "SELECT format, version, manifest FROM kalamu_project_file",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(refused)?;
    if format != pf::FORMAT {
        return Err(ProjectFileError::NotProjectFile);
    }
    if version > i64::from(pf::VERSION) {
        return Err(ProjectFileError::Newer(format!(
            "file format {version}; this version reads {}",
            pf::VERSION
        )));
    }
    let manifest: Manifest =
        serde_json::from_str(&manifest).map_err(|e| ProjectFileError::Damaged(e.to_string()))?;
    let checked: String = file
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(refused)?;
    if checked != "ok" {
        return Err(ProjectFileError::Damaged(checked));
    }
    let applied: i64 = file
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(refused)?;
    let known = i64::from(schema());
    if applied > known {
        return Err(ProjectFileError::Newer(format!(
            "schema {applied}; this version knows {known}"
        )));
    }
    if applied == known {
        return Ok((None, file, manifest));
    }
    // Saved by an older Kalamu: its rows are brought up to date on a copy, never in the file.
    drop(file);
    std::fs::create_dir_all(working).map_err(read_err)?;
    let copy = working.join(format!("{WORKING_PREFIX}{}.kalamu", JobId::new()));
    std::fs::copy(path, &copy).map_err(read_err)?;
    let upgraded = Upgraded(copy);
    let file = Connection::open(&upgraded.0).map_err(read_err)?;
    crate::migrate(&file).map_err(ProjectFileError::Store)?;
    Ok((Some(upgraded), file, manifest))
}

pub(super) fn check(
    path: &Path,
    working: &Path,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<ProjectFileCheck, ProjectFileError> {
    // The copy first: what is bound first is dropped last, after the connection that holds it.
    let (_upgraded, file, manifest) = readable(path, working)?;
    let documents = documents(&file)?;
    let titles = |path: &str| -> Vec<String> {
        documents
            .iter()
            .filter(|(_, paths)| paths.iter().any(|p| p == path))
            .map(|(title, _)| title.clone())
            .collect()
    };

    // Every carried file read through and its fingerprint checked, the originals given the
    // names opening would give them, in the same order.
    let carried = carried::carried(&file)?;
    let total = carried.iter().map(|c| c.size).sum();
    let mut done = 0;
    progress(ProjectFileProgress { done, total });
    let (mut texts, mut originals) = (0, 0);
    let mut damaged = Vec::new();
    let mut renamed = Vec::new();
    let mut placement = Placement::default();
    let root = Path::new("project");
    for c in &carried {
        let whole = carried::read_through(&file, c, |_| Ok(()), &mut done, total, progress)?;
        if c.role == ROLE_SNAPSHOT {
            if whole {
                texts += 1;
            } else {
                let read = carried::readers(&file, vec![(c.path.clone(), c.file_name.clone())])?;
                damaged.push(
                    read.into_iter()
                        .map(|t| t.documents.join(", "))
                        .find(|d| !d.is_empty())
                        .unwrap_or_else(|| c.file_name.clone()),
                );
            }
            continue;
        }
        let documents = titles(&c.path);
        if whole {
            originals += 1;
        } else {
            damaged.push(if documents.is_empty() {
                c.file_name.clone()
            } else {
                documents.join(", ")
            });
        }
        let to = placement.place(root, c);
        let to = to
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let from = pf::last_part(&c.path);
        if to != from {
            renamed.push(RenamedOriginal {
                documents,
                from: from.to_string(),
                to,
            });
        }
    }

    // The originals listed and not carried: left out by choice, or not found when saving.
    let mut stmt = file
        .prepare("SELECT path FROM kalamu_files WHERE role = ?1 AND carried = 0 ORDER BY id")
        .map_err(refused)?;
    let left: BTreeSet<String> = stmt
        .query_map([ROLE_ORIGINAL], |row| row.get(0))
        .map_err(refused)?
        .collect::<Result<_, _>>()
        .map_err(refused)?;
    let left_count = left.len() as u32;
    let to_find = documents
        .iter()
        .filter(|(_, paths)| paths.iter().any(|p| left.contains(p)))
        .map(|(title, _)| title.clone())
        .collect();

    let missing_texts = carried::missing(&file)?;
    let verdict = if !damaged.is_empty() {
        ProjectFileVerdict::Damaged
    } else if !missing_texts.is_empty() {
        ProjectFileVerdict::Incomplete
    } else {
        ProjectFileVerdict::Whole
    };
    let count = |sql: &str| -> Result<u32, ProjectFileError> {
        file.query_row(sql, [], |row| row.get::<_, i64>(0))
            .map(|n| n.max(0) as u32)
            .map_err(refused)
    };
    Ok(ProjectFileCheck {
        verdict,
        name: manifest.project_name,
        saved_at: manifest.saved_at,
        app_version: manifest.app_version,
        documents: count("SELECT COUNT(*) FROM documents")?,
        // The jobs that read its documents, as its card counts them.
        jobs: count("SELECT COUNT(DISTINCT job_id) FROM document_links")?,
        texts,
        missing_texts,
        damaged,
        with_originals: manifest.with_originals,
        originals,
        originals_not_chosen: if manifest.with_originals {
            0
        } else {
            left_count
        },
        originals_left: if manifest.with_originals {
            left_count
        } else {
            0
        },
        to_find,
        renamed,
    })
}

/// Each document's title and the files it names (its own, or its pages'), in the order the
/// project lists them; read loosely, only the paths matter.
fn documents(file: &Connection) -> Result<Vec<(String, Vec<String>)>, ProjectFileError> {
    let mut stmt = file
        .prepare("SELECT title, source FROM documents ORDER BY created_at, rowid")
        .map_err(refused)?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(refused)?
        .collect::<Result<_, _>>()
        .map_err(refused)?;
    Ok(rows
        .into_iter()
        .map(|(title, source)| {
            let source: serde_json::Value = serde_json::from_str(&source).unwrap_or_default();
            let pages = source["pages"].as_array().map(Vec::as_slice).unwrap_or(&[]);
            let paths = std::iter::once(&source["path"])
                .chain(pages.iter().map(|p| &p["path"]))
                .filter_map(|p| p.as_str().map(str::to_string))
                .collect();
            (title, paths)
        })
        .collect())
}
