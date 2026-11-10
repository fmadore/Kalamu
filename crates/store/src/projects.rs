//! [`ProjectStore`] for the same database as the jobs: projects, collections, documents, their
//! memberships, the links from documents to the jobs that read them, and preferred results.
//!
//! Deleting a job removes its links and preferences (foreign keys), never a document; deleting
//! a project or a document removes what organises it, never a job or a result.

use kalamu_core::{
    Collection, CollectionId, Document, DocumentId, DocumentLink, JobId, PreferredResult, Project,
    ProjectId, ProjectStore, SetupId, SourceId, StoreError, Task, Timestamp,
};
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    archived INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX documents_by_project ON documents(project_id);
CREATE TABLE collection_documents (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    PRIMARY KEY (collection_id, document_id)
);
CREATE TABLE document_links (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL,
    linked_at INTEGER NOT NULL,
    PRIMARY KEY (document_id, job_id, source_id)
);
CREATE TABLE preferred_results (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    task TEXT NOT NULL,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    PRIMARY KEY (document_id, task)
);
"#;

/// The researcher's descriptions of sources: each document's own, and the parts a project's
/// documents share.
pub(crate) const DESCRIPTIONS: &str = r#"
ALTER TABLE projects ADD COLUMN source_defaults TEXT NOT NULL DEFAULT '{}';
ALTER TABLE documents ADD COLUMN source_description TEXT NOT NULL DEFAULT '{}';
"#;

/// The links `filter` keeps (`l` is `document_links`), oldest first.
fn links_where(
    conn: &Connection,
    filter: &str,
    args: &[String],
) -> Result<Vec<DocumentLink>, StoreError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT l.document_id, l.job_id, l.source_id, l.linked_at FROM document_links l
             {filter} ORDER BY l.linked_at, l.rowid"
        ))
        .map_err(db)?;
    let rows = stmt
        .query_map(params_from_iter(args), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(db)?;
    rows.map(|r| {
        let (d, j, s, at) = r.map_err(db)?;
        Ok(DocumentLink {
            document_id: id(&d)?,
            job_id: id(&j)?,
            source_id: id(&s)?,
            linked_at: Timestamp(at),
        })
    })
    .collect()
}

fn id<T: std::str::FromStr>(s: &str) -> Result<T, StoreError>
where
    T::Err: std::fmt::Display,
{
    s.parse::<T>().map_err(corrupt)
}

fn task(s: &str) -> Result<Task, StoreError> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(corrupt)
}

/// `NotFound` when a statement changed no row.
fn found(changed: usize) -> Result<(), StoreError> {
    if changed == 0 {
        Err(StoreError::NotFound)
    } else {
        Ok(())
    }
}

type ProjectRow = (String, String, String, bool, String, i64, i64);

const PROJECT_COLUMNS: &str =
    "id, name, description, archived, source_defaults, created_at, updated_at";

fn project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn project(
    (pid, name, description, archived, defaults, created, updated): ProjectRow,
) -> Result<Project, StoreError> {
    Ok(Project {
        id: id(&pid)?,
        name,
        description,
        archived,
        source_defaults: serde_json::from_str(&defaults).map_err(corrupt)?,
        created_at: Timestamp(created),
        updated_at: Timestamp(updated),
    })
}

type DocumentRow = (String, String, String, String, i64, String, String);

const DOCUMENT_COLUMNS: &str =
    "id, project_id, title, source, created_at, tags, source_description";

fn document_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn document(
    (did, pid, title, source, created, tags, description): DocumentRow,
) -> Result<Document, StoreError> {
    Ok(Document {
        id: id(&did)?,
        project_id: id(&pid)?,
        title,
        source: serde_json::from_str(&source).map_err(corrupt)?,
        created_at: Timestamp(created),
        tags: serde_json::from_str(&tags).map_err(corrupt)?,
        source_description: serde_json::from_str(&description).map_err(corrupt)?,
    })
}

impl ProjectStore for SqliteStore {
    fn put_project(&self, project: &Project) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO projects
                    (id, name, description, archived, created_at, updated_at, source_defaults)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET name = ?2, description = ?3, archived = ?4,
                    updated_at = ?6, source_defaults = ?7",
                params![
                    project.id.to_string(),
                    project.name,
                    project.description,
                    project.archived,
                    project.created_at.0,
                    project.updated_at.0,
                    to_json(&project.source_defaults)?,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn projects(&self) -> Result<Vec<Project>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {PROJECT_COLUMNS} FROM projects ORDER BY updated_at DESC"
            ))
            .map_err(db)?;
        let rows = stmt.query_map([], project_row).map_err(db)?;
        rows.map(|r| project(r.map_err(db)?)).collect()
    }

    fn project(&self, pid: ProjectId) -> Result<Project, StoreError> {
        let row = self
            .conn()
            .query_row(
                &format!("SELECT {PROJECT_COLUMNS} FROM projects WHERE id = ?1"),
                [pid.to_string()],
                project_row,
            )
            .optional()
            .map_err(db)?;
        project(row.ok_or(StoreError::NotFound)?)
    }

    fn delete_project(&self, pid: ProjectId) -> Result<(), StoreError> {
        let changed = self
            .conn()
            .execute("DELETE FROM projects WHERE id = ?1", [pid.to_string()])
            .map_err(db)?;
        found(changed)
    }

    fn put_collection(&self, collection: &Collection) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO collections (id, project_id, name, created_at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET name = ?3",
                params![
                    collection.id.to_string(),
                    collection.project_id.to_string(),
                    collection.name,
                    collection.created_at.0,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn collections(&self, pid: ProjectId) -> Result<Vec<Collection>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, created_at FROM collections WHERE project_id = ?1
                 ORDER BY name COLLATE NOCASE, created_at",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([pid.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(db)?;
        rows.map(|r| {
            let (cid, name, created) = r.map_err(db)?;
            Ok(Collection {
                id: id(&cid)?,
                project_id: pid,
                name,
                created_at: Timestamp(created),
            })
        })
        .collect()
    }

    fn delete_collection(&self, cid: CollectionId) -> Result<(), StoreError> {
        let changed = self
            .conn()
            .execute("DELETE FROM collections WHERE id = ?1", [cid.to_string()])
            .map_err(db)?;
        found(changed)
    }

    fn put_document(&self, document: &Document) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO documents
                    (id, project_id, title, source, created_at, tags, source_description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET title = ?3, source = ?4, tags = ?6,
                    source_description = ?7",
                params![
                    document.id.to_string(),
                    document.project_id.to_string(),
                    document.title,
                    to_json(&document.source)?,
                    document.created_at.0,
                    to_json(&document.tags)?,
                    to_json(&document.source_description)?,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn documents(&self, pid: ProjectId) -> Result<Vec<Document>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {DOCUMENT_COLUMNS} FROM documents
                 WHERE project_id = ?1 ORDER BY created_at, rowid"
            ))
            .map_err(db)?;
        let rows = stmt
            .query_map([pid.to_string()], document_row)
            .map_err(db)?;
        rows.map(|r| document(r.map_err(db)?)).collect()
    }

    fn document(&self, did: DocumentId) -> Result<Document, StoreError> {
        let row = self
            .conn()
            .query_row(
                &format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE id = ?1"),
                [did.to_string()],
                document_row,
            )
            .optional()
            .map_err(db)?;
        document(row.ok_or(StoreError::NotFound)?)
    }

    fn delete_document(&self, did: DocumentId) -> Result<(), StoreError> {
        let changed = self
            .conn()
            .execute("DELETE FROM documents WHERE id = ?1", [did.to_string()])
            .map_err(db)?;
        found(changed)
    }

    fn add_member(&self, cid: CollectionId, did: DocumentId) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO collection_documents (collection_id, document_id)
                 VALUES (?1, ?2)",
                params![cid.to_string(), did.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn remove_member(&self, cid: CollectionId, did: DocumentId) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "DELETE FROM collection_documents WHERE collection_id = ?1 AND document_id = ?2",
                params![cid.to_string(), did.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn members(&self, pid: ProjectId) -> Result<Vec<(CollectionId, DocumentId)>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT m.collection_id, m.document_id FROM collection_documents m
                 JOIN collections c ON c.id = m.collection_id WHERE c.project_id = ?1",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([pid.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db)?;
        rows.map(|r| {
            let (c, d) = r.map_err(db)?;
            Ok((id(&c)?, id(&d)?))
        })
        .collect()
    }

    fn put_link(&self, link: &DocumentLink) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO document_links (document_id, job_id, source_id, linked_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    link.document_id.to_string(),
                    link.job_id.to_string(),
                    link.source_id.to_string(),
                    link.linked_at.0,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn remove_link(&self, did: DocumentId, job: JobId, source: SourceId) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "DELETE FROM document_links
                 WHERE document_id = ?1 AND job_id = ?2 AND source_id = ?3",
                params![did.to_string(), job.to_string(), source.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn links(&self, pid: ProjectId) -> Result<Vec<DocumentLink>, StoreError> {
        links_where(
            &self.conn(),
            "JOIN documents d ON d.id = l.document_id WHERE d.project_id = ?1",
            &[pid.to_string()],
        )
    }

    fn links_of(&self, job: JobId, source: SourceId) -> Result<Vec<DocumentLink>, StoreError> {
        // Found through `document_links_by_job`.
        links_where(
            &self.conn(),
            "WHERE l.job_id = ?1 AND l.source_id = ?2",
            &[job.to_string(), source.to_string()],
        )
    }

    fn set_preferred(
        &self,
        did: DocumentId,
        task: Task,
        choice: Option<(JobId, SetupId)>,
    ) -> Result<(), StoreError> {
        let conn = self.conn();
        match choice {
            Some((job, setup)) => conn.execute(
                "INSERT INTO preferred_results (document_id, task, job_id, setup_id)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(document_id, task) DO UPDATE SET job_id = ?3, setup_id = ?4",
                params![
                    did.to_string(),
                    task.as_str(),
                    job.to_string(),
                    setup.to_string()
                ],
            ),
            None => conn.execute(
                "DELETE FROM preferred_results WHERE document_id = ?1 AND task = ?2",
                params![did.to_string(), task.as_str()],
            ),
        }
        .map_err(db)?;
        Ok(())
    }

    fn preferred(&self, pid: ProjectId) -> Result<Vec<PreferredResult>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT p.document_id, p.task, p.job_id, p.setup_id FROM preferred_results p
                 JOIN documents d ON d.id = p.document_id WHERE d.project_id = ?1",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([pid.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(db)?;
        rows.map(|r| {
            let (d, t, j, s) = r.map_err(db)?;
            Ok(PreferredResult {
                document_id: id(&d)?,
                task: task(&t)?,
                job_id: id(&j)?,
                setup_id: id(&s)?,
            })
        })
        .collect()
    }
}
