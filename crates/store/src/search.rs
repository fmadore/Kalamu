//! The search index: SQLite full-text search over the folded text of every passage, in a
//! database file of its own beside the job database.
//!
//! Nothing in it is authoritative. A file that cannot be opened, or whose layout is from
//! another version, is emptied and made again from the job store by the next search; its own
//! connection means indexing never holds the lock the review and the engine use.
//!
//! Matching uses FTS5's trigram tokenizer, so a phrase is found anywhere in the text as the
//! review's search finds it, inside words too: Arabic joins its article and some prepositions
//! to the word, and a name is found within `l'Union` or `بالمسجد`. The text arrives folded
//! (capitals, accents and vowel marks set aside, runs of spaces made one), so the tokenizer
//! folds nothing more.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use kalamu_core::{IndexSize, JobId, Passage, PassageKey, SearchIndex, StoreError};
use rusqlite::{Connection, OptionalExtension, params};

use super::{corrupt, db, to_json};

/// Raised whenever the layout below changes: an index of another version is made again.
const VERSION: i64 = 1;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS passages (
    id INTEGER PRIMARY KEY,
    job_id TEXT NOT NULL,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    stamp TEXT NOT NULL,
    data TEXT NOT NULL,
    UNIQUE (job_id, setup_id, source_id, idx)
);
CREATE VIRTUAL TABLE IF NOT EXISTS passage_text USING fts5(
    text,
    tokenize = 'trigram case_sensitive 1',
    content = '',
    contentless_delete = 1
);
"#;

pub struct SqliteIndex {
    conn: Mutex<Connection>,
}

impl SqliteIndex {
    /// Opens the index file, or makes it anew when it cannot be read: it holds nothing that
    /// the job store does not.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        match Self::open_file(path) {
            Ok(index) => Ok(index),
            Err(_) => {
                for suffix in ["", "-wal", "-shm"] {
                    let mut file = path.as_os_str().to_owned();
                    file.push(suffix);
                    let _ = std::fs::remove_file(&file);
                }
                Self::open_file(path)
            }
        }
    }

    fn open_file(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path).map_err(db)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(db)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(db)?;
        Self::init(conn)
    }

    /// A private index in memory, for tests.
    pub fn in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory().map_err(db)?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        // Reading the schema checks the file is a database at all.
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(db)?;
        if version != VERSION {
            drop_tables(&conn)?;
        }
        create_tables(&conn)?;
        conn.pragma_update(None, "user_version", VERSION)
            .map_err(db)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn drop_tables(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch("DROP TABLE IF EXISTS passage_text; DROP TABLE IF EXISTS passages;")
        .map_err(db)
}

fn create_tables(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(SCHEMA).map_err(db)
}

/// Removes one passage and its text, on `conn`, which may be a transaction's.
fn remove(conn: &Connection, key: &PassageKey) -> Result<(), StoreError> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT id FROM passages
             WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3 AND idx = ?4",
            params![
                key.job_id.to_string(),
                key.setup_id.to_string(),
                key.source_id.to_string(),
                key.index,
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    if let Some(id) = id {
        conn.execute("DELETE FROM passage_text WHERE rowid = ?1", [id])
            .map_err(db)?;
        conn.execute("DELETE FROM passages WHERE id = ?1", [id])
            .map_err(db)?;
    }
    Ok(())
}

fn parse_key(job: &str, setup: &str, source: &str, index: u32) -> Result<PassageKey, StoreError> {
    Ok(PassageKey {
        job_id: job.parse().map_err(corrupt)?,
        setup_id: setup.parse().map_err(corrupt)?,
        source_id: source.parse().map_err(corrupt)?,
        index,
    })
}

/// A literal phrase for FTS5: quoted, its own quotes doubled.
fn phrase(folded: &str) -> String {
    format!("\"{}\"", folded.replace('"', "\"\""))
}

/// Every term as a literal phrase, all of them required.
fn all_of(terms: &[String]) -> String {
    terms
        .iter()
        .map(|t| phrase(t))
        .collect::<Vec<_>>()
        .join(" AND ")
}

impl SearchIndex for SqliteIndex {
    fn stamps(&self, job: JobId) -> Result<Vec<(PassageKey, String)>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT setup_id, source_id, idx, stamp FROM passages WHERE job_id = ?1")
            .map_err(db)?;
        let job_text = job.to_string();
        let rows = stmt
            .query_map([&job_text], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            let (setup, source, index, stamp) = row.map_err(db)?;
            out.push((parse_key(&job_text, &setup, &source, index)?, stamp));
        }
        Ok(out)
    }

    fn put_passages(&self, passages: &[(Passage, String)]) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        for (passage, folded) in passages {
            remove(&tx, &passage.key)?;
            let key = &passage.key;
            tx.execute(
                "INSERT INTO passages (job_id, setup_id, source_id, idx, stamp, data)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    key.job_id.to_string(),
                    key.setup_id.to_string(),
                    key.source_id.to_string(),
                    key.index,
                    passage.stamp,
                    to_json(passage)?,
                ],
            )
            .map_err(db)?;
            let id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO passage_text (rowid, text) VALUES (?1, ?2)",
                params![id, folded],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)
    }

    fn remove_passages(&self, keys: &[PassageKey]) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        for key in keys {
            remove(&tx, key)?;
        }
        tx.commit().map_err(db)
    }

    fn indexed_jobs(&self) -> Result<Vec<JobId>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT DISTINCT job_id FROM passages")
            .map_err(db)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(db)?.parse().map_err(corrupt)?);
        }
        Ok(out)
    }

    fn forget_job(&self, job: JobId) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        tx.execute(
            "DELETE FROM passage_text WHERE rowid IN (SELECT id FROM passages WHERE job_id = ?1)",
            [job.to_string()],
        )
        .map_err(db)?;
        tx.execute("DELETE FROM passages WHERE job_id = ?1", [job.to_string()])
            .map_err(db)?;
        tx.commit().map_err(db)
    }

    fn find(&self, terms: &[String]) -> Result<Vec<Passage>, StoreError> {
        if terms.is_empty() || terms.iter().any(|t| t.chars().count() < 3) {
            return Ok(Vec::new());
        }
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT p.data FROM passage_text t JOIN passages p ON p.id = t.rowid
                 WHERE passage_text MATCH ?1",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([all_of(terms)], |row| row.get::<_, String>(0))
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(serde_json::from_str(&row.map_err(db)?).map_err(corrupt)?);
        }
        Ok(out)
    }

    fn clear(&self) -> Result<(), StoreError> {
        let conn = self.conn();
        drop_tables(&conn)?;
        create_tables(&conn)?;
        // Gives the space back to the system.
        conn.execute_batch("VACUUM;").map_err(db)
    }

    fn size(&self) -> Result<IndexSize, StoreError> {
        let conn = self.conn();
        let passages: i64 = conn
            .query_row("SELECT COUNT(*) FROM passages", [], |row| row.get(0))
            .map_err(db)?;
        let pages: i64 = conn
            .pragma_query_value(None, "page_count", |row| row.get(0))
            .map_err(db)?;
        let page_size: i64 = conn
            .pragma_query_value(None, "page_size", |row| row.get(0))
            .map_err(db)?;
        Ok(IndexSize {
            passages: u64::try_from(passages).unwrap_or_default(),
            bytes: u64::try_from(pages * page_size).unwrap_or_default(),
        })
    }
}
