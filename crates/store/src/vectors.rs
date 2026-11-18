//! The vector index: each chunk's vector, as little-endian 32-bit floats, in a database file of
//! its own beside the job database and the word search's index.
//!
//! Nothing in it is authoritative, but unlike the word search's index it is not free to make
//! again: a cloud model bills each chunk embedded. A file that cannot be opened, or whose layout
//! is from another version, is still emptied rather than trusted, and the researcher is shown
//! what preparing the text again would cost.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use kalamu_core::{
    ChunkStamp, ChunkVector, EmbeddingSpace, JobId, PassageKey, ProviderId, SpaceSize, StoreError,
    VectorIndex,
};
use rusqlite::{Connection, params};

use super::{corrupt, db};

/// Raised whenever the layout below changes: an index of another version is emptied.
const VERSION: i64 = 1;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS chunks (
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    dimensions INTEGER NOT NULL,
    job_id TEXT NOT NULL,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    chunk INTEGER NOT NULL,
    of INTEGER NOT NULL,
    stamp TEXT NOT NULL,
    content TEXT NOT NULL,
    start INTEGER NOT NULL,
    end INTEGER NOT NULL,
    vector BLOB NOT NULL,
    PRIMARY KEY (provider, model, dimensions, job_id, setup_id, source_id, idx, chunk)
);
CREATE INDEX IF NOT EXISTS chunks_by_job ON chunks(job_id);
"#;

pub struct SqliteVectors {
    conn: Mutex<Connection>,
    generation: AtomicU64,
}

/// The vector as stored: each number's four bytes, least significant first.
pub fn to_blob(vector: &[f32]) -> Vec<u8> {
    vector.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub fn from_blob(blob: &[u8]) -> Result<Vec<f32>, StoreError> {
    let (numbers, rest) = blob.as_chunks::<4>();
    if !rest.is_empty() {
        return Err(StoreError::Corrupt("a vector of broken length".into()));
    }
    Ok(numbers.iter().map(|b| f32::from_le_bytes(*b)).collect())
}

impl SqliteVectors {
    /// Opens the index file, or makes it anew when it cannot be read.
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
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(db)?;
        if version != VERSION {
            conn.execute_batch("DROP TABLE IF EXISTS chunks;")
                .map_err(db)?;
        }
        conn.execute_batch(SCHEMA).map_err(db)?;
        conn.pragma_update(None, "user_version", VERSION)
            .map_err(db)?;
        Ok(Self {
            conn: Mutex::new(conn),
            generation: AtomicU64::new(1),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn changed(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }
}

fn parse_key(job: &str, setup: &str, source: &str, index: u32) -> Result<PassageKey, StoreError> {
    Ok(PassageKey {
        job_id: job.parse().map_err(corrupt)?,
        setup_id: setup.parse().map_err(corrupt)?,
        source_id: source.parse().map_err(corrupt)?,
        index,
    })
}

/// `?1, ?2, …` for a list of jobs after `before` other arguments.
fn placeholders(before: usize, n: usize) -> String {
    (0..n)
        .map(|i| format!("?{}", before + i + 1))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The space's three columns, then the jobs, as one list of arguments.
fn space_and_jobs(space: &EmbeddingSpace, jobs: &[JobId]) -> Vec<rusqlite::types::Value> {
    let mut args: Vec<rusqlite::types::Value> = vec![
        space.provider.as_str().to_string().into(),
        space.model.clone().into(),
        i64::from(space.dimensions).into(),
    ];
    args.extend(jobs.iter().map(|j| j.to_string().into()));
    args
}

/// Jobs at a time in one query, well under SQLite's limit of arguments.
const JOBS_AT_ONCE: usize = 500;

impl VectorIndex for SqliteVectors {
    fn stamps(&self, space: &EmbeddingSpace, job: JobId) -> Result<Vec<ChunkStamp>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare_cached(
                "SELECT setup_id, source_id, idx, chunk, of, stamp, content FROM chunks
                 WHERE provider = ?1 AND model = ?2 AND dimensions = ?3 AND job_id = ?4",
            )
            .map_err(db)?;
        let job_text = job.to_string();
        let rows = stmt
            .query_map(
                params![
                    space.provider.as_str(),
                    space.model,
                    space.dimensions,
                    job_text
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, u32>(3)?,
                        row.get::<_, u32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            let (setup, source, index, chunk, of, stamp, content) = row.map_err(db)?;
            out.push(ChunkStamp {
                key: parse_key(&job_text, &setup, &source, index)?,
                chunk,
                of,
                stamp,
                content,
            });
        }
        Ok(out)
    }

    fn put(&self, space: &EmbeddingSpace, chunks: &[ChunkVector]) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        for c in chunks {
            let k = &c.key;
            let key = params![
                space.provider.as_str(),
                space.model,
                space.dimensions,
                k.job_id.to_string(),
                k.setup_id.to_string(),
                k.source_id.to_string(),
                k.index,
            ];
            // Chunks cut from another text of the passage are stale: gone. Those of the same
            // text take the newer stamp.
            tx.execute(
                "DELETE FROM chunks WHERE provider = ?1 AND model = ?2 AND dimensions = ?3
                 AND job_id = ?4 AND setup_id = ?5 AND source_id = ?6 AND idx = ?7
                 AND content <> ?8",
                rusqlite::params_from_iter(
                    key.iter()
                        .copied()
                        .chain([&c.content as &dyn rusqlite::ToSql]),
                ),
            )
            .map_err(db)?;
            tx.execute(
                "UPDATE chunks SET stamp = ?8 WHERE provider = ?1 AND model = ?2
                 AND dimensions = ?3 AND job_id = ?4 AND setup_id = ?5 AND source_id = ?6
                 AND idx = ?7",
                rusqlite::params_from_iter(
                    key.iter()
                        .copied()
                        .chain([&c.stamp as &dyn rusqlite::ToSql]),
                ),
            )
            .map_err(db)?;
            tx.execute(
                "INSERT OR REPLACE INTO chunks (provider, model, dimensions, job_id, setup_id,
                 source_id, idx, chunk, of, stamp, content, start, end, vector)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    space.provider.as_str(),
                    space.model,
                    space.dimensions,
                    k.job_id.to_string(),
                    k.setup_id.to_string(),
                    k.source_id.to_string(),
                    k.index,
                    c.chunk,
                    c.of,
                    c.stamp,
                    c.content,
                    c.start,
                    c.end,
                    to_blob(&c.vector),
                ],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.changed();
        Ok(())
    }

    fn restamp(
        &self,
        space: &EmbeddingSpace,
        key: &PassageKey,
        content: &str,
        stamp: &str,
    ) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "UPDATE chunks SET stamp = ?9 WHERE provider = ?1 AND model = ?2
                 AND dimensions = ?3 AND job_id = ?4 AND setup_id = ?5 AND source_id = ?6
                 AND idx = ?7 AND content = ?8",
                params![
                    space.provider.as_str(),
                    space.model,
                    space.dimensions,
                    key.job_id.to_string(),
                    key.setup_id.to_string(),
                    key.source_id.to_string(),
                    key.index,
                    content,
                    stamp,
                ],
            )
            .map_err(db)?;
        self.changed();
        Ok(())
    }

    fn remove_passages(&self, keys: &[PassageKey]) -> Result<(), StoreError> {
        let mut conn = self.conn();
        let tx = conn.transaction().map_err(db)?;
        for k in keys {
            tx.execute(
                "DELETE FROM chunks WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3
                 AND idx = ?4",
                params![
                    k.job_id.to_string(),
                    k.setup_id.to_string(),
                    k.source_id.to_string(),
                    k.index
                ],
            )
            .map_err(db)?;
        }
        tx.commit().map_err(db)?;
        self.changed();
        Ok(())
    }

    fn vectors(
        &self,
        space: &EmbeddingSpace,
        jobs: &[JobId],
    ) -> Result<Vec<ChunkVector>, StoreError> {
        let conn = self.conn();
        let mut out = Vec::new();
        for some in jobs.chunks(JOBS_AT_ONCE) {
            let sql = format!(
                "SELECT job_id, setup_id, source_id, idx, chunk, of, stamp, content, start, end,
                 vector FROM chunks
                 WHERE provider = ?1 AND model = ?2 AND dimensions = ?3 AND of > 0
                 AND job_id IN ({})",
                placeholders(3, some.len())
            );
            let mut stmt = conn.prepare(&sql).map_err(db)?;
            let mut rows = stmt
                .query(rusqlite::params_from_iter(space_and_jobs(space, some)))
                .map_err(db)?;
            while let Some(row) = rows.next().map_err(db)? {
                let job: String = row.get(0).map_err(db)?;
                let setup: String = row.get(1).map_err(db)?;
                let source: String = row.get(2).map_err(db)?;
                let blob = row.get_ref(10).map_err(db)?.as_blob().map_err(corrupt)?;
                out.push(ChunkVector {
                    key: parse_key(&job, &setup, &source, row.get(3).map_err(db)?)?,
                    chunk: row.get(4).map_err(db)?,
                    of: row.get(5).map_err(db)?,
                    stamp: row.get(6).map_err(db)?,
                    content: row.get(7).map_err(db)?,
                    start: row.get(8).map_err(db)?,
                    end: row.get(9).map_err(db)?,
                    vector: from_blob(blob)?,
                });
            }
        }
        Ok(out)
    }

    fn indexed_jobs(&self) -> Result<Vec<JobId>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT DISTINCT job_id FROM chunks")
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
        self.conn()
            .execute("DELETE FROM chunks WHERE job_id = ?1", [job.to_string()])
            .map_err(db)?;
        self.changed();
        Ok(())
    }

    fn spaces(&self, jobs: &[JobId]) -> Result<Vec<SpaceSize>, StoreError> {
        let conn = self.conn();
        let mut sizes: std::collections::BTreeMap<EmbeddingSpace, (u32, u64)> =
            std::collections::BTreeMap::new();
        for some in jobs.chunks(JOBS_AT_ONCE) {
            let sql = format!(
                "SELECT provider, model, dimensions, COUNT(*), SUM(LENGTH(vector))
                 FROM chunks WHERE of > 0 AND job_id IN ({})
                 GROUP BY provider, model, dimensions",
                placeholders(0, some.len())
            );
            let mut stmt = conn.prepare(&sql).map_err(db)?;
            let mut rows = stmt
                .query(rusqlite::params_from_iter(
                    some.iter().map(|j| j.to_string()),
                ))
                .map_err(db)?;
            while let Some(row) = rows.next().map_err(db)? {
                let space = EmbeddingSpace {
                    provider: ProviderId::new(row.get::<_, String>(0).map_err(db)?),
                    model: row.get(1).map_err(db)?,
                    dimensions: row.get(2).map_err(db)?,
                };
                let count: u32 = row.get(3).map_err(db)?;
                let bytes: i64 = row.get::<_, Option<i64>>(4).map_err(db)?.unwrap_or(0);
                let entry = sizes.entry(space).or_default();
                entry.0 += count;
                entry.1 += u64::try_from(bytes).unwrap_or_default();
            }
        }
        Ok(sizes
            .into_iter()
            .map(|(space, (chunks, bytes))| SpaceSize {
                space,
                chunks,
                bytes,
            })
            .collect())
    }

    fn remove_space(&self, space: &EmbeddingSpace, jobs: &[JobId]) -> Result<(), StoreError> {
        {
            let mut conn = self.conn();
            let tx = conn.transaction().map_err(db)?;
            for some in jobs.chunks(JOBS_AT_ONCE) {
                let sql = format!(
                    "DELETE FROM chunks WHERE provider = ?1 AND model = ?2 AND dimensions = ?3
                     AND job_id IN ({})",
                    placeholders(3, some.len())
                );
                tx.execute(
                    &sql,
                    rusqlite::params_from_iter(space_and_jobs(space, some)),
                )
                .map_err(db)?;
            }
            tx.commit().map_err(db)?;
        }
        self.changed();
        Ok(())
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
}
