//! What of the store a project file holds: the project, its documents, and the jobs that read
//! them, each kept to the sources tied to this project's documents, so nothing of another
//! project goes with it. The jobs that asked a model for the dictionary's proposals go whole:
//! they read only rows Kalamu wrote.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use kalamu_core::{Document, Job, JobStatus, ProjectFileError, ProjectId, SourceFile};
use rusqlite::{Connection, OptionalExtension, params};

use super::{ROLE_ORIGINAL, ROLE_SNAPSHOT};
use crate::{corrupt, db};

/// One file a project's sources name: a page of a document of images, or a source's own file.
#[derive(Debug, Clone)]
pub(super) struct Listed {
    pub role: &'static str,
    pub path: String,
    pub file_name: String,
    pub sha256: String,
    pub size: u64,
}

pub(super) struct Selection {
    pub name: String,
    pub documents: Vec<Document>,
    /// The jobs, each with only the sources the file carries, ready to store.
    pub jobs: Vec<Job>,
    /// How many of them read the project's documents: the jobs the project's card counts. The
    /// dictionary's proposal jobs go in the file too, but read no document.
    pub read_jobs: u32,
    /// `[job, source]` pairs, as JSON, for the tables kept per source.
    pub pairs: String,
    /// The jobs' ids and the snapshots their sources read, as JSON lists.
    pub job_ids: String,
    pub snapshot_ids: String,
    /// The titles of the project's documents each `(job, source)` pair read.
    pub titles: BTreeMap<(String, String), BTreeSet<String>>,
}

fn store_err(e: kalamu_core::StoreError) -> ProjectFileError {
    ProjectFileError::Store(e)
}

pub(super) fn select(conn: &Connection, project: ProjectId) -> Result<Selection, ProjectFileError> {
    let name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            [project.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| store_err(db(e)))?
        .ok_or(ProjectFileError::Store(kalamu_core::StoreError::NotFound))?;
    let documents = documents(conn, project).map_err(store_err)?;

    // The sources each job read that are this project's documents, and those documents' titles.
    let mut kept: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut titles: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT l.job_id, l.source_id, d.title FROM document_links l
             JOIN documents d ON d.id = l.document_id WHERE d.project_id = ?1",
        )
        .map_err(|e| store_err(db(e)))?;
    let rows = stmt
        .query_map([project.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| store_err(db(e)))?;
    for row in rows {
        let (job, source, title) = row.map_err(|e| store_err(db(e)))?;
        kept.entry(job.clone()).or_default().insert(source.clone());
        titles.entry((job, source)).or_default().insert(title);
    }
    // The dictionary's proposal jobs, whole.
    let mut whole: BTreeSet<String> = BTreeSet::new();
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT json_extract(data, '$.job_id') FROM dictionary_requests
             WHERE project_id = ?1",
        )
        .map_err(|e| store_err(db(e)))?;
    let rows = stmt
        .query_map([project.to_string()], |row| row.get::<_, Option<String>>(0))
        .map_err(|e| store_err(db(e)))?;
    for row in rows {
        if let Some(job) = row.map_err(|e| store_err(db(e)))? {
            whole.insert(job);
        }
    }

    let mut jobs = Vec::new();
    let ids: BTreeSet<&String> = kept.keys().chain(whole.iter()).collect();
    for id in ids {
        let row: Option<(String, String)> = conn
            .query_row("SELECT data, status FROM jobs WHERE id = ?1", [id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .optional()
            .map_err(|e| store_err(db(e)))?;
        // A proposal's job deleted since leaves its request without answers: nothing to carry.
        let Some((data, status)) = row else { continue };
        let mut job: Job = serde_json::from_str(&data).map_err(|e| store_err(corrupt(e)))?;
        // The job's status is its column's: the copy in `data` is the one it was created with.
        job.status = crate::jobs::parse_status(&status).map_err(store_err)?;
        if !whole.contains(id) {
            let sources = kept.get(id).cloned().unwrap_or_default();
            job.sources.retain(|s| sources.contains(&s.id.to_string()));
        }
        if job.sources.is_empty() {
            continue;
        }
        // Nothing runs in a file: a job caught running goes as interrupted, to resume there.
        if job.status == JobStatus::Running {
            job.status = JobStatus::Interrupted;
        }
        jobs.push(job);
    }

    let read_jobs = jobs
        .iter()
        .filter(|j| kept.contains_key(&j.id.to_string()))
        .count() as u32;
    let pairs: Vec<[String; 2]> = jobs
        .iter()
        .flat_map(|j| {
            j.sources
                .iter()
                .map(move |s| [j.id.to_string(), s.id.to_string()])
        })
        .collect();
    let job_ids: Vec<String> = jobs.iter().map(|j| j.id.to_string()).collect();
    let snapshot_ids: BTreeSet<String> = jobs
        .iter()
        .flat_map(|j| j.sources.iter())
        .filter_map(|s| s.derived_from.as_ref().map(|d| d.snapshot_id.clone()))
        .collect();
    Ok(Selection {
        name,
        documents,
        jobs,
        read_jobs,
        pairs: json(&pairs),
        job_ids: json(&job_ids),
        snapshot_ids: json(&snapshot_ids),
        titles,
    })
}

impl Selection {
    /// Who read the file at `path`, to name in a refusal: the documents' titles, then the jobs'
    /// names, or the file's own name when nothing names it. The researcher's own words, so the
    /// same in every language.
    pub fn readers(&self, path: &str) -> String {
        let mut documents: BTreeSet<&str> = BTreeSet::new();
        let mut jobs: Vec<&str> = Vec::new();
        let mut file_name = "";
        for job in &self.jobs {
            for source in job.sources.iter().filter(|s| s.path == path) {
                file_name = &source.file_name;
                if !jobs.contains(&job.name.as_str()) {
                    jobs.push(&job.name);
                }
                let key = (job.id.to_string(), source.id.to_string());
                documents.extend(
                    self.titles
                        .get(&key)
                        .into_iter()
                        .flatten()
                        .map(String::as_str),
                );
            }
        }
        let mut named: Vec<&str> = documents.into_iter().collect();
        if named.is_empty() && !file_name.is_empty() {
            named.push(file_name);
        }
        let named = named.join(", ");
        match (named.is_empty(), jobs.is_empty()) {
            (_, true) => named,
            (true, false) => jobs.join(", "),
            (false, false) => format!("{named} — {}", jobs.join(", ")),
        }
    }
}

/// A list of strings as JSON, which cannot fail.
fn json<T: serde::Serialize>(list: &T) -> String {
    serde_json::to_string(list).unwrap_or_else(|_| "[]".to_string())
}

fn documents(
    conn: &Connection,
    project: ProjectId,
) -> Result<Vec<Document>, kalamu_core::StoreError> {
    let mut stmt = conn
        .prepare("SELECT id FROM documents WHERE project_id = ?1 ORDER BY created_at, rowid")
        .map_err(db)?;
    let ids = stmt
        .query_map(params![project.to_string()], |row| row.get::<_, String>(0))
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    let mut out = Vec::new();
    for id in ids {
        let (title, source, created, tags, description): (String, String, i64, String, String) =
            conn.query_row(
                "SELECT title, source, created_at, tags, source_description FROM documents
                 WHERE id = ?1",
                [&id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .map_err(db)?;
        out.push(Document {
            id: id.parse().map_err(corrupt)?,
            project_id: project,
            title,
            source: serde_json::from_str(&source).map_err(corrupt)?,
            created_at: kalamu_core::Timestamp(created),
            tags: serde_json::from_str(&tags).map_err(corrupt)?,
            source_description: serde_json::from_str(&description).map_err(corrupt)?,
        });
    }
    Ok(out)
}

/// Every file the project's documents and jobs name, each once: Kalamu's frozen texts (under
/// `snapshots`) apart from the researcher's originals. A YouTube video names none.
pub(super) fn files(selection: &Selection, snapshots: &Path) -> Vec<Listed> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    let sources = selection
        .documents
        .iter()
        .map(|d| &d.source)
        .chain(selection.jobs.iter().flat_map(|j| j.sources.iter()));
    for source in sources {
        for file in files_of(source) {
            let role = if Path::new(&file.path).starts_with(snapshots) {
                ROLE_SNAPSHOT
            } else {
                ROLE_ORIGINAL
            };
            if seen.insert(file.path.clone()) {
                out.push(Listed { role, ..file });
            }
        }
    }
    out
}

fn files_of(source: &SourceFile) -> Vec<Listed> {
    let listed = |path: &str, file_name: &str, sha256: &str, size: u64| Listed {
        role: ROLE_ORIGINAL,
        path: path.to_string(),
        file_name: file_name.to_string(),
        sha256: sha256.to_string(),
        size,
    };
    if !source.pages.is_empty() {
        return source
            .pages
            .iter()
            .map(|p| listed(&p.path, &p.file_name, &p.sha256, p.size_bytes))
            .collect();
    }
    if source.files().is_empty() {
        return Vec::new();
    }
    vec![listed(
        &source.path,
        &source.file_name,
        &source.sha256,
        source.size_bytes,
    )]
}
