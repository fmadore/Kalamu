//! [`DictionaryStore`] for the same database: a project's dictionary entries and the pairs the
//! researcher kept apart, the proposal jobs asked of a model and the proposals turned down.
//! Deleting the project removes them; nothing here reaches a result.

use kalamu_core::{
    DictionaryStore, Entry, EntryId, KeptApart, ProjectId, ProposalRequest, StoreError, Timestamp,
};
use rusqlite::params;

use super::{SqliteStore, corrupt, db, to_json};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE dictionary_entries (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    label TEXT NOT NULL,
    forms TEXT NOT NULL,
    reviewed INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX dictionary_entries_by_project ON dictionary_entries(project_id);
CREATE TABLE dictionary_apart (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    a TEXT NOT NULL REFERENCES dictionary_entries(id) ON DELETE CASCADE,
    b TEXT NOT NULL REFERENCES dictionary_entries(id) ON DELETE CASCADE,
    PRIMARY KEY (a, b)
);
"#;

/// Proposal requests and the proposals turned down, added after the first dictionary tables.
/// A request outlives its job, which is deleted apart; its answers are then simply gone.
pub(crate) const PROPOSALS: &str = r#"
CREATE TABLE dictionary_requests (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX dictionary_requests_by_project ON dictionary_requests(project_id);
CREATE TABLE dictionary_dismissed (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    PRIMARY KEY (project_id, namespace, key)
);
"#;

fn parse<T: std::str::FromStr>(s: &str) -> Result<T, StoreError>
where
    T::Err: std::fmt::Display,
{
    s.parse::<T>().map_err(corrupt)
}

impl DictionaryStore for SqliteStore {
    fn put_entry(&self, entry: &Entry) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO dictionary_entries
                    (id, project_id, namespace, label, forms, reviewed, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET label = ?4, forms = ?5, reviewed = ?6,
                    updated_at = ?8",
                params![
                    entry.id.to_string(),
                    entry.project_id.to_string(),
                    entry.namespace,
                    entry.label,
                    to_json(&entry.forms)?,
                    entry.reviewed,
                    entry.created_at.0,
                    entry.updated_at.0,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn entries(&self, project: ProjectId) -> Result<Vec<Entry>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, namespace, label, forms, reviewed, created_at, updated_at
                 FROM dictionary_entries WHERE project_id = ?1 ORDER BY created_at, rowid",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([project.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, bool>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(db)?;
        let mut entries = Vec::new();
        for row in rows {
            let (id, project_id, namespace, label, forms, reviewed, created, updated) =
                row.map_err(db)?;
            entries.push(Entry {
                id: parse(&id)?,
                project_id: parse(&project_id)?,
                namespace,
                label,
                forms: serde_json::from_str(&forms).map_err(corrupt)?,
                reviewed,
                created_at: Timestamp(created),
                updated_at: Timestamp(updated),
            });
        }
        Ok(entries)
    }

    fn delete_entry(&self, id: EntryId) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "DELETE FROM dictionary_entries WHERE id = ?1",
                [id.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn put_apart(&self, project: ProjectId, pair: &KeptApart) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO dictionary_apart (project_id, a, b) VALUES (?1, ?2, ?3)",
                params![project.to_string(), pair.a.to_string(), pair.b.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn remove_apart(&self, pair: &KeptApart) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "DELETE FROM dictionary_apart WHERE a = ?1 AND b = ?2",
                params![pair.a.to_string(), pair.b.to_string()],
            )
            .map_err(db)?;
        Ok(())
    }

    fn apart(&self, project: ProjectId) -> Result<Vec<KeptApart>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT a, b FROM dictionary_apart WHERE project_id = ?1 ORDER BY rowid")
            .map_err(db)?;
        let rows = stmt
            .query_map([project.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db)?;
        let mut pairs = Vec::new();
        for row in rows {
            let (a, b) = row.map_err(db)?;
            pairs.push(KeptApart {
                a: parse(&a)?,
                b: parse(&b)?,
            });
        }
        Ok(pairs)
    }

    fn put_request(&self, request: &ProposalRequest) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT INTO dictionary_requests (id, project_id, created_at, data)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET data = ?4",
                params![
                    request.id,
                    request.project_id.to_string(),
                    request.created_at.0,
                    to_json(request)?,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn requests(&self, project: ProjectId) -> Result<Vec<ProposalRequest>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT data FROM dictionary_requests WHERE project_id = ?1
                 ORDER BY created_at, rowid",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([project.to_string()], |row| row.get::<_, String>(0))
            .map_err(db)?;
        let mut requests = Vec::new();
        for row in rows {
            requests.push(serde_json::from_str(&row.map_err(db)?).map_err(corrupt)?);
        }
        Ok(requests)
    }

    fn delete_request(&self, id: &str) -> Result<(), StoreError> {
        self.conn()
            .execute("DELETE FROM dictionary_requests WHERE id = ?1", [id])
            .map_err(db)?;
        Ok(())
    }

    fn put_dismissed(
        &self,
        project: ProjectId,
        namespace: &str,
        key: &str,
    ) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "INSERT OR IGNORE INTO dictionary_dismissed (project_id, namespace, key)
                 VALUES (?1, ?2, ?3)",
                params![project.to_string(), namespace, key],
            )
            .map_err(db)?;
        Ok(())
    }

    fn remove_dismissed(
        &self,
        project: ProjectId,
        namespace: &str,
        key: &str,
    ) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "DELETE FROM dictionary_dismissed
                 WHERE project_id = ?1 AND namespace = ?2 AND key = ?3",
                params![project.to_string(), namespace, key],
            )
            .map_err(db)?;
        Ok(())
    }

    fn dismissed(&self, project: ProjectId) -> Result<Vec<(String, String)>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT namespace, key FROM dictionary_dismissed WHERE project_id = ?1
                 ORDER BY rowid",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([project.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db)?;
        rows.map(|r| r.map_err(db)).collect()
    }
}
