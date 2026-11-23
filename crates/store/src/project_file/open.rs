//! Opening a project file: checked (a Kalamu project file, whole, of a version this Kalamu
//! knows), its carried files written into this computer's places, then every row put in the
//! store in one transaction, identifiers new and paths pointed at the files written. Anything
//! that fails rolls the rows back and deletes the originals written, so the store is as it was.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use kalamu_core::file_names::FolderNames;
use kalamu_core::project_file::{self as pf, ArchivePlaces, Manifest};
use kalamu_core::{
    JobId, JobStatus, ProjectFileError, ProjectFileOpened, ProjectFileProgress, ProjectId,
    Timestamp,
};
use rusqlite::types::Value;
use rusqlite::{Connection, ErrorCode, OpenFlags};

use super::carried::{self, Carried};
use super::remap::Remap;
use super::{IDENTIFIED, RESULTS, ROLE_SNAPSHOT, TABLES, read_err, rows, schema};
use crate::SqliteStore;

/// A copy of the file in the temporary folder, brought up to this schema, deleted when done.
struct Upgraded(PathBuf);

impl Drop for Upgraded {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// What SQLite's refusal says of the file.
fn refused(e: rusqlite::Error) -> ProjectFileError {
    match e.sqlite_error_code() {
        Some(ErrorCode::NotADatabase) => ProjectFileError::NotProjectFile,
        Some(ErrorCode::DatabaseCorrupt) => ProjectFileError::Damaged(e.to_string()),
        _ if e.to_string().contains("no such table") => ProjectFileError::NotProjectFile,
        _ => read_err(e),
    }
}

/// The file opened for reading, its manifest, and the upgraded copy it may be read from.
fn check(path: &Path) -> Result<(Connection, Manifest, Option<Upgraded>), ProjectFileError> {
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
        return Ok((file, manifest, None));
    }
    // Saved by an older Kalamu: its rows are brought up to date on a copy, never in the file.
    drop(file);
    let copy = std::env::temp_dir().join(format!("kalamu-open-{}.kalamu", JobId::new()));
    std::fs::copy(path, &copy).map_err(read_err)?;
    let upgraded = Upgraded(copy);
    let file = Connection::open(&upgraded.0).map_err(read_err)?;
    crate::migrate(&file).map_err(ProjectFileError::Store)?;
    Ok((file, manifest, Some(upgraded)))
}

pub(super) fn open(
    store: &SqliteStore,
    path: &Path,
    places: &ArchivePlaces,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<ProjectFileOpened, ProjectFileError> {
    let (file, manifest, _upgraded) = check(path)?;
    let mut remap = Remap::default();
    for table in IDENTIFIED {
        let mut stmt = file
            .prepare(&format!("SELECT id FROM \"{table}\""))
            .map_err(refused)?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(refused)?;
        for id in ids {
            remap
                .ids
                .insert(id.map_err(refused)?, JobId::new().to_string());
        }
    }
    let old_project: String = file
        .query_row("SELECT id FROM projects", [], |row| row.get(0))
        .map_err(|_| ProjectFileError::Damaged("it holds no project".into()))?;
    let project_id: ProjectId = remap
        .ids
        .get(&old_project)
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| ProjectFileError::Damaged("it holds no project".into()))?;

    let mut written: Vec<PathBuf> = Vec::new();
    let opened = (|| {
        let originals = write_carried(
            &file,
            places,
            project_id,
            &mut remap,
            &mut written,
            progress,
        )?;
        name_page_documents(&file, &mut remap)?;
        let name = put_rows(store, &file, &remap, &old_project, &manifest)?;
        let count = |table: &str| -> Result<u32, ProjectFileError> {
            file.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|n| n.max(0) as u32)
            .map_err(refused)
        };
        Ok(ProjectFileOpened {
            project_id,
            name,
            documents: count("documents")?,
            // The jobs that read its documents, as its card counts them.
            jobs: count("(SELECT DISTINCT job_id FROM document_links)")?,
            originals,
            with_originals: manifest.with_originals,
            saved_at: manifest.saved_at,
        })
    })();
    if opened.is_err() {
        for path in &written {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_dir_all(places.originals.join(project_id.to_string()));
    }
    opened
}

/// Writes the files the project file carries: frozen texts into the snapshots folder, named by
/// their content and never rewritten; originals into a folder of this project, a folder for
/// each folder they shared, each under its own name, numbered when an earlier original of its
/// folder took that name once cleaned (`draft?.pdf` and `draft_.pdf`, or names apart only by
/// case: `draft_ (2).pdf`). Records the new paths; returns how many originals were written.
fn write_carried(
    file: &Connection,
    places: &ArchivePlaces,
    project: ProjectId,
    remap: &mut Remap,
    written: &mut Vec<PathBuf>,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<u32, ProjectFileError> {
    let carried = carried::carried(file)?;
    let total = carried.iter().map(|c| c.size).sum();
    let mut done = 0;
    progress(ProjectFileProgress { done, total });
    let root = places.originals.join(project.to_string());
    let mut folders: HashMap<String, PathBuf> = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    let mut names: HashMap<PathBuf, FolderNames> = HashMap::new();
    let mut originals = 0;
    for c in &carried {
        let to = if c.role == ROLE_SNAPSHOT {
            places
                .snapshots
                .join(pf::portable_name(pf::last_part(&c.path), &c.sha256))
        } else {
            let folder = folder_for(&root, &c.path, &mut folders, &mut taken);
            let name = names
                .entry(folder.clone())
                .or_default()
                .give_file(&pf::portable_name(
                    pf::last_part(&c.path),
                    &fallback_name(c),
                ));
            folder.join(name)
        };
        if c.role == ROLE_SNAPSHOT && to.is_file() {
            // The same text frozen here already: one file, never rewritten.
            done += c.size;
        } else {
            carried::extract(file, c, &to, &mut done, total, progress)?;
            if c.role != ROLE_SNAPSHOT {
                written.push(to.clone());
                originals += 1;
            }
        }
        remap.paths.insert(c.path.clone(), to.display().to_string());
    }
    Ok(originals)
}

fn fallback_name(c: &Carried) -> String {
    let name = pf::portable_name(&c.file_name, "");
    if name.is_empty() {
        c.sha256.clone()
    } else {
        name
    }
}

/// The folder of this computer for the files of one folder of the other: named after it, a
/// number added when two share a name.
fn folder_for(
    root: &Path,
    path: &str,
    folders: &mut HashMap<String, PathBuf>,
    taken: &mut HashSet<String>,
) -> PathBuf {
    let old = pf::folder_part(path).to_string();
    if let Some(folder) = folders.get(&old) {
        return folder.clone();
    }
    let base = pf::portable_name(pf::last_part(&old), "Files");
    let mut name = base.clone();
    let mut n = 2;
    while !taken.insert(name.to_lowercase()) {
        name = format!("{base} {n}");
        n += 1;
    }
    let folder = root.join(name);
    folders.insert(old, folder.clone());
    folder
}

/// A document of images names itself in its first page's folder: once its pages are written
/// here, its own path follows them, so its results are written beside them.
fn name_page_documents(file: &Connection, remap: &mut Remap) -> Result<(), ProjectFileError> {
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for sql in ["SELECT source FROM documents", "SELECT data FROM jobs"] {
        let mut stmt = file.prepare(sql).map_err(refused)?;
        let texts = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(refused)?;
        for text in texts {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text.map_err(refused)?)
            else {
                continue;
            };
            page_documents(&value, remap, &mut found);
        }
    }
    remap.paths.extend(found);
    Ok(())
}

fn page_documents(value: &serde_json::Value, remap: &Remap, found: &mut BTreeMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let (Some(serde_json::Value::String(path)), Some(serde_json::Value::Array(pages))) =
                (map.get("path"), map.get("pages"))
                && let Some(first) = pages
                    .first()
                    .and_then(|p| p.get("path"))
                    .and_then(|p| p.as_str())
                && let Some(new_first) = remap.paths.get(first)
                && !remap.paths.contains_key(path)
            {
                let folder = Path::new(new_first)
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_default();
                let name = pf::portable_name(pf::last_part(path), "Document");
                found.insert(path.clone(), folder.join(name).display().to_string());
            }
            for v in map.values() {
                page_documents(v, remap, found);
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                page_documents(v, remap, found);
            }
        }
        _ => {}
    }
}

/// Every row of the file into the store, in one transaction. The project keeps its name unless
/// one of this computer's projects has it; it then takes the date the file was saved, then a
/// number. Returns the project's name here.
fn put_rows(
    store: &SqliteStore,
    file: &Connection,
    remap: &Remap,
    old_project: &str,
    manifest: &Manifest,
) -> Result<String, ProjectFileError> {
    let mut conn = store.conn();
    let names: HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM projects")
            .map_err(|e| ProjectFileError::Store(crate::db(e)))?;
        stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| ProjectFileError::Store(crate::db(e)))?
            .collect::<Result<_, _>>()
            .map_err(|e| ProjectFileError::Store(crate::db(e)))?
    };
    let name: String = file
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            [old_project],
            |row| row.get(0),
        )
        .map_err(refused)?;
    let name = free_name(&name, manifest.saved_at, &names);
    let settled = settled_statuses(file, remap)?;
    let now = Timestamp::now().0;
    let tx = conn
        .transaction()
        .map_err(|e| ProjectFileError::Store(crate::db(e)))?;
    for table in TABLES {
        let results = *table == RESULTS;
        rows::copy(file, &tx, table, "", &[], results, |columns, row| {
            if results {
                return;
            }
            let identified = IDENTIFIED.contains(table);
            for (column, value) in columns.iter().zip(row.iter_mut()) {
                if let Value::Text(text) = value {
                    let new = if identified && column == "id" {
                        remap.cell(text)
                    } else {
                        remap.text(text)
                    };
                    if let Some(new) = new {
                        *text = new;
                    }
                }
            }
            if *table == "jobs"
                && let (Some(id), Some(at)) =
                    (rows::column(columns, "id"), rows::column(columns, "status"))
                && let Value::Text(id) = &row[id]
                && let Some(status) = settled.get(id)
            {
                row[at] = Value::Text((*status).to_string());
            }
            if *table == "projects" {
                for (column, value) in [
                    ("name", Value::Text(name.clone())),
                    ("archived", Value::Integer(0)),
                    ("updated_at", Value::Integer(now)),
                ] {
                    if let Some(at) = rows::column(columns, column) {
                        row[at] = value;
                    }
                }
            }
        })
        .map_err(put_failed)?;
    }
    tx.commit()
        .map_err(|e| ProjectFileError::Store(crate::db(e)))?;
    Ok(name)
}

/// The status of each job a file saved by 1.10.0-rc1 holds as waiting to start although its
/// pieces were read, keyed by the job's new identity. That version saved the status a job was
/// created with rather than the one it reached, so a finished job came back unfinished, with
/// Resume. A job waiting to start has no pieces yet: one that has them was started, and goes as
/// finished when every piece was read or failed, as interrupted otherwise.
fn settled_statuses(
    file: &Connection,
    remap: &Remap,
) -> Result<HashMap<String, &'static str>, ProjectFileError> {
    let mut stmt = file
        .prepare(
            "SELECT j.id, COUNT(*), COALESCE(SUM(p.status IN ('done', 'cached')), 0),
                    COALESCE(SUM(p.status = 'failed'), 0)
             FROM jobs j JOIN pieces p ON p.job_id = j.id
             WHERE j.status = 'pending' GROUP BY j.id",
        )
        .map_err(refused)?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(refused)?;
    let mut out = HashMap::new();
    for row in rows {
        let (old, pieces, read, failed) = row.map_err(refused)?;
        let status = if read + failed < pieces {
            JobStatus::Interrupted
        } else if failed > 0 {
            JobStatus::CompletedWithErrors
        } else {
            JobStatus::Completed
        };
        if let Some(new) = remap.ids.get(&old) {
            out.insert(new.clone(), status.as_str());
        }
    }
    Ok(out)
}

/// A row the store refuses (one that names what the file does not hold) means a damaged file;
/// anything else is the store's own trouble.
fn put_failed(e: rusqlite::Error) -> ProjectFileError {
    match e.sqlite_error_code() {
        Some(ErrorCode::ConstraintViolation) => ProjectFileError::Damaged(e.to_string()),
        _ => ProjectFileError::Store(crate::db(e)),
    }
}

/// The project's name here: its own, else with the day it was saved, else with a number.
fn free_name(name: &str, saved_at: Timestamp, taken: &HashSet<String>) -> String {
    let free = |n: &String| !taken.iter().any(|t| t.to_lowercase() == n.to_lowercase());
    let own = name.to_string();
    if free(&own) {
        return own;
    }
    let dated = format!("{name} ({})", saved_at.utc_date());
    if free(&dated) {
        return dated;
    }
    (2..)
        .map(|n| format!("{dated} {n}"))
        .find(free)
        .unwrap_or(dated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_in_use_takes_the_date_then_a_number() {
        let saved = Timestamp(1_795_219_200_000); // 2026-11-21
        let mut taken = HashSet::new();
        assert_eq!(free_name("Letters", saved, &taken), "Letters");
        taken.insert("letters".to_string());
        assert_eq!(free_name("Letters", saved, &taken), "Letters (2026-11-21)");
        taken.insert("Letters (2026-11-21)".to_string());
        assert_eq!(
            free_name("Letters", saved, &taken),
            "Letters (2026-11-21) 2"
        );
    }
}
