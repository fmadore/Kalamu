//! Saving a project to a file: its rows read under the store's lock into a new database beside
//! the file chosen (`<name>.kalamu.part`), the files it carries written after, then the part
//! checked as *Check a project file…* checks one (#246), and renamed only once it checks. A
//! failure, a frozen text missing or changed or a part that does not check among them, deletes
//! the part and leaves a file saved earlier under that name as it was.

use std::path::{Path, PathBuf};

use kalamu_core::project_file::{self as pf, ArchivePlaces};
use kalamu_core::{
    PieceStatus, ProjectFileCheck, ProjectFileError, ProjectFilePlan, ProjectFileProgress,
    ProjectFileSaved, ProjectFileVerdict, ProjectId,
};
use rusqlite::types::Value;
use rusqlite::{Connection, ToSql};

use super::selection::{self, Selection};
use super::{ROLE_ORIGINAL, TABLES, carried, check, create_schema, manifest, rows, write_err};
use crate::{SqliteStore, db};

/// The rows of each table a project's file holds, by the named arguments `:project` (its id),
/// `:pairs` (its jobs' `[job, source]` pairs), `:jobs` and `:snapshots` (JSON lists).
fn filter(table: &str) -> &'static str {
    match table {
        "projects" => "WHERE id = :project",
        "collections"
        | "documents"
        | "review_samples"
        | "dictionary_entries"
        | "dictionary_apart"
        | "dictionary_requests"
        | "dictionary_dismissed" => "WHERE project_id = :project",
        "collection_documents" => {
            "WHERE collection_id IN (SELECT id FROM collections WHERE project_id = :project)"
        }
        "jobs" => "WHERE id IN (SELECT value FROM json_each(:jobs))",
        "document_links" => {
            "WHERE document_id IN (SELECT id FROM documents WHERE project_id = :project)
             AND job_id IN (SELECT value FROM json_each(:jobs))"
        }
        "preferred_results" => {
            "WHERE document_id IN (SELECT id FROM documents WHERE project_id = :project)
             AND job_id IN (SELECT value FROM json_each(:jobs))"
        }
        "pieces" | "corrections" | "speaker_names" | "charges" | "passages" | "drawn_regions"
        | "region_readings" => {
            "WHERE (job_id, source_id) IN (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs))"
        }
        "results" => {
            "WHERE cache_key IN (SELECT cache_key FROM pieces WHERE cache_key IS NOT NULL AND
                 (job_id, source_id) IN (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs)))
             OR cache_key IN (SELECT cache_key FROM passages WHERE
                 (job_id, source_id) IN (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs)))
             OR cache_key IN (SELECT cache_key FROM region_readings WHERE
                 (job_id, source_id) IN (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs)))"
        }
        "units" => {
            "WHERE (job_id, json_extract(data, '$.source_id')) IN
                 (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs))"
        }
        "snapshots" => "WHERE id IN (SELECT value FROM json_each(:snapshots))",
        "review_kept" => {
            "WHERE project_id = :project AND (job_id, source_id) IN
                 (SELECT value ->> 0, value ->> 1 FROM json_each(:pairs))"
        }
        "review_set_aside" => {
            "WHERE project_id = :project AND job_id IN (SELECT value FROM json_each(:jobs))"
        }
        // Every table carried is named above; anything else holds nothing of the project.
        _ => "WHERE 0",
    }
}

/// The file's name offered: the project's, kept to what both systems accept.
fn file_name(name: &str) -> String {
    format!("{}.{}", pf::portable_name(name, "Project"), pf::EXTENSION)
}

pub(super) fn plan(
    store: &SqliteStore,
    project: ProjectId,
    places: &ArchivePlaces,
) -> Result<ProjectFilePlan, ProjectFileError> {
    let conn = store.conn();
    let selection = selection::select(&conn, project)?;
    let args = arguments(project, &selection);
    let args: Vec<(&str, &dyn ToSql)> = args.iter().map(|(n, v)| (*n, v as &dyn ToSql)).collect();
    let mut results_bytes: u64 = 64 << 10;
    for (table, columns) in [
        ("results", "output, request"),
        ("jobs", "data"),
        ("corrections", "text, fields"),
        ("snapshots", "data"),
        ("units", "data"),
    ] {
        let sum = columns
            .split(", ")
            .map(|c| format!("COALESCE(LENGTH(CAST({c} AS BLOB)), 0)"))
            .collect::<Vec<_>>()
            .join(" + ");
        let filter = filter(table);
        let bound: Vec<(&str, &dyn ToSql)> = args
            .iter()
            .filter(|(n, _)| filter.contains(n))
            .copied()
            .collect();
        let bytes: i64 = conn
            .query_row(
                &format!("SELECT COALESCE(SUM({sum}), 0) FROM {table} {filter}"),
                bound.as_slice(),
                |row| row.get(0),
            )
            .map_err(|e| ProjectFileError::Store(db(e)))?;
        results_bytes += bytes.max(0) as u64;
    }
    drop(conn);
    let (mut originals, mut originals_bytes, mut originals_missing) = (0, 0, 0);
    for file in selection::files(&selection, &places.snapshots) {
        let size = std::fs::metadata(&file.path).ok().map(|m| m.len());
        match (file.role == ROLE_ORIGINAL, size) {
            (true, Some(size)) => {
                originals += 1;
                originals_bytes += size;
            }
            (true, None) => originals_missing += 1,
            (false, Some(size)) => results_bytes += size,
            (false, None) => {}
        }
    }
    Ok(ProjectFilePlan {
        file_name: file_name(&selection.name),
        name: selection.name,
        documents: selection.documents.len() as u32,
        jobs: selection.read_jobs,
        results_bytes,
        originals,
        originals_bytes,
        originals_missing,
    })
}

fn arguments(project: ProjectId, selection: &Selection) -> [(&'static str, String); 4] {
    [
        (":project", project.to_string()),
        (":pairs", selection.pairs.clone()),
        (":jobs", selection.job_ids.clone()),
        (":snapshots", selection.snapshot_ids.clone()),
    ]
}

/// The part written beside `path` until the file is whole.
fn part_of(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

pub(super) fn save(
    store: &SqliteStore,
    project: ProjectId,
    path: &Path,
    with_originals: bool,
    places: &ArchivePlaces,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<ProjectFileSaved, ProjectFileError> {
    let part = part_of(path);
    match write_checked(store, project, &part, with_originals, places, progress) {
        Ok((originals, originals_left, check)) => {
            if let Err(e) = std::fs::rename(&part, path) {
                let _ = std::fs::remove_file(&part);
                return Err(write_err(e));
            }
            Ok(ProjectFileSaved {
                path: path.display().to_string(),
                bytes: std::fs::metadata(path).map(|m| m.len()).unwrap_or_default(),
                originals,
                originals_left,
                check,
            })
        }
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

/// Writes the part, then checks it as *Check a project file…* would: only a part that checks
/// may take the file's name. Writing fills the first half of the progress, the check the
/// second.
fn write_checked(
    store: &SqliteStore,
    project: ProjectId,
    part: &Path,
    with_originals: bool,
    places: &ArchivePlaces,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<(u32, u32, ProjectFileCheck), ProjectFileError> {
    let writing = |p: ProjectFileProgress| {
        progress(ProjectFileProgress {
            done: p.done,
            total: p.total * 2,
        })
    };
    let (originals, originals_left) =
        write(store, project, part, with_originals, places, &writing)?;
    let checking = |p: ProjectFileProgress| {
        progress(ProjectFileProgress {
            done: p.total + p.done,
            total: p.total * 2,
        })
    };
    let check = checked(check::check(part, &places.working, &checking))?;
    Ok((originals, originals_left, check))
}

/// The check of a part just written, as the save takes it: a part that cannot be read back,
/// or whose carried files no longer match their fingerprints, was not written as it should
/// have been, and is refused as a write that failed, naming those files.
fn checked(
    check: Result<ProjectFileCheck, ProjectFileError>,
) -> Result<ProjectFileCheck, ProjectFileError> {
    match check {
        Ok(check) if check.verdict == ProjectFileVerdict::Damaged => Err(write_err(format!(
            "the file written does not match what was read: {}",
            check.damaged.join("; ")
        ))),
        Ok(check) => Ok(check),
        Err(e) => Err(write_err(format!(
            "the file written cannot be read back: {e}"
        ))),
    }
}

/// Writes the whole file at `part`; returns how many originals it carries and how many it
/// could not.
fn write(
    store: &SqliteStore,
    project: ProjectId,
    part: &Path,
    with_originals: bool,
    places: &ArchivePlaces,
    progress: &dyn Fn(ProjectFileProgress),
) -> Result<(u32, u32), ProjectFileError> {
    if part.exists() {
        std::fs::remove_file(part).map_err(write_err)?;
    }
    let mut file = Connection::open(part).map_err(write_err)?;
    create_schema(&file)?;
    let selection = {
        let conn = store.conn();
        let selection = selection::select(&conn, project)?;
        copy_rows(&conn, &mut file, project, &selection)?;
        selection
    };
    let listed = selection::files(&selection, &places.snapshots);
    let written = carried::write_files(&file, &listed, with_originals, progress)?;
    // A frozen text is what the results were read from: a file without it is no backup, so the
    // save fails here, before the part replaces a file saved earlier.
    let named = |paths: &[String]| {
        paths
            .iter()
            .map(|p| selection.readers(p))
            .collect::<Vec<_>>()
            .join("; ")
    };
    if !written.frozen_missing.is_empty() {
        return Err(ProjectFileError::FrozenTextMissing(named(
            &written.frozen_missing,
        )));
    }
    if !written.frozen_changed.is_empty() {
        return Err(ProjectFileError::FrozenTextChanged(named(
            &written.frozen_changed,
        )));
    }
    let (originals, left) = (written.originals, written.originals_left);
    let manifest = manifest(
        &selection.name,
        selection.documents.len() as u32,
        selection.read_jobs,
        with_originals,
        places,
    );
    let manifest = serde_json::to_string(&manifest).map_err(write_err)?;
    file.execute(
        "INSERT INTO kalamu_project_file (format, version, manifest) VALUES (?1, ?2, ?3)",
        rusqlite::params![pf::FORMAT, pf::VERSION, manifest],
    )
    .map_err(write_err)?;
    file.close().map_err(|(_, e)| write_err(e))?;
    Ok((originals, left))
}

/// The project's rows, in one transaction of the file.
fn copy_rows(
    conn: &Connection,
    file: &mut Connection,
    project: ProjectId,
    selection: &Selection,
) -> Result<(), ProjectFileError> {
    let args = arguments(project, selection);
    let args: Vec<(&str, &dyn ToSql)> = args.iter().map(|(n, v)| (*n, v as &dyn ToSql)).collect();
    let jobs: std::collections::HashMap<String, (String, &'static str)> = selection
        .jobs
        .iter()
        .map(|j| {
            let data = serde_json::to_string(j).unwrap_or_default();
            (j.id.to_string(), (data, j.status.as_str()))
        })
        .collect();
    let tx = file.transaction().map_err(write_err)?;
    for table in TABLES {
        rows::copy(
            conn,
            &tx,
            table,
            filter(table),
            &args,
            false,
            |columns, row| {
                match *table {
                    // Each job kept to the sources the file carries.
                    "jobs" => trim_job(columns, row, &jobs),
                    // A piece caught running or held in a batch goes as not read yet: nothing
                    // runs in a file, and a batch belongs to the account and computer that sent it.
                    "pieces" => {
                        if let Some(at) = rows::column(columns, "status")
                            && let Value::Text(status) = &row[at]
                            && [PieceStatus::Running, PieceStatus::Submitted]
                                .iter()
                                .any(|s| s.as_str() == status)
                        {
                            row[at] = Value::Text(PieceStatus::Pending.as_str().to_string());
                        }
                    }
                    _ => {}
                }
            },
        )
        .map_err(write_err)?;
    }
    tx.commit().map_err(write_err)
}

fn trim_job(
    columns: &[String],
    row: &mut [Value],
    jobs: &std::collections::HashMap<String, (String, &'static str)>,
) {
    let id = rows::column(columns, "id").and_then(|at| match &row[at] {
        Value::Text(id) => Some(id.clone()),
        _ => None,
    });
    let Some((data, status)) = id.and_then(|id| jobs.get(&id)) else {
        return;
    };
    if let Some(at) = rows::column(columns, "data") {
        row[at] = Value::Text(data.clone());
    }
    if let Some(at) = rows::column(columns, "status") {
        row[at] = Value::Text((*status).to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(verdict: ProjectFileVerdict, damaged: &[&str]) -> ProjectFileCheck {
        ProjectFileCheck {
            verdict,
            name: "Mission letters".into(),
            saved_at: kalamu_core::Timestamp(1),
            app_version: "1.10.0".into(),
            documents: 1,
            jobs: 1,
            texts: 1,
            missing_texts: Vec::new(),
            damaged: damaged.iter().map(|d| d.to_string()).collect(),
            with_originals: true,
            originals: 1,
            originals_not_chosen: 0,
            originals_left: 0,
            to_find: Vec::new(),
            renamed: Vec::new(),
        }
    }

    #[test]
    fn a_part_that_does_not_check_is_a_write_that_failed() {
        let whole = report(ProjectFileVerdict::Whole, &[]);
        assert_eq!(checked(Ok(whole.clone())).ok(), Some(whole));

        let damaged = checked(Ok(report(
            ProjectFileVerdict::Damaged,
            &["Letter of 1915", "Map of Kayes"],
        )));
        let Err(e) = damaged else {
            panic!("a damaged part was kept");
        };
        assert_eq!(e.code(), "project.file_write");
        assert!(
            e.to_string().contains("Letter of 1915; Map of Kayes"),
            "{e}"
        );

        let unreadable = checked(Err(ProjectFileError::Damaged("malformed".into())));
        let Err(e) = unreadable else {
            panic!("an unreadable part was kept");
        };
        assert_eq!(e.code(), "project.file_write");
        assert!(e.to_string().contains("malformed"), "{e}");
    }
}
