//! What the researcher decided in a project's review queue, beside the projects' tables: the
//! random sample asked for in each scope (its seed and size), and the items kept aside with
//! their notes. Deleting the project deletes them; deleting a job deletes what was kept aside
//! of its results.

use kalamu_core::{
    JobId, KeptAside, ProjectId, SampleSpec, SetupId, SourceId, StoreError, Timestamp,
};
use rusqlite::{OptionalExtension, params};

use super::{SqliteStore, corrupt, db};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE review_samples (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    scope TEXT NOT NULL,
    seed INTEGER NOT NULL,
    size INTEGER NOT NULL,
    drawn_at INTEGER NOT NULL,
    PRIMARY KEY (project_id, scope)
);
CREATE TABLE review_kept (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    revision TEXT,
    note TEXT NOT NULL,
    kept_at INTEGER NOT NULL,
    PRIMARY KEY (project_id, job_id, setup_id, source_id, idx)
);
CREATE INDEX review_kept_by_job ON review_kept(job_id);
"#;

/// The result as a whole, which has no piece index, in the `idx` column.
const WHOLE: i64 = -1;

fn idx(index: Option<u32>) -> i64 {
    index.map_or(WHOLE, i64::from)
}

fn parsed<T: std::str::FromStr>(s: &str) -> Result<T, StoreError>
where
    T::Err: std::fmt::Display,
{
    s.parse::<T>().map_err(corrupt)
}

pub(crate) fn set_sample(
    store: &SqliteStore,
    project: ProjectId,
    scope: &str,
    sample: Option<&SampleSpec>,
) -> Result<(), StoreError> {
    let conn = store.conn();
    match sample {
        Some(s) => conn.execute(
            "INSERT INTO review_samples (project_id, scope, seed, size, drawn_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (project_id, scope) DO UPDATE SET seed = ?3, size = ?4, drawn_at = ?5",
            params![project.to_string(), scope, s.seed, s.size, s.drawn_at.0],
        ),
        None => conn.execute(
            "DELETE FROM review_samples WHERE project_id = ?1 AND scope = ?2",
            params![project.to_string(), scope],
        ),
    }
    .map_err(db)?;
    Ok(())
}

pub(crate) fn sample(
    store: &SqliteStore,
    project: ProjectId,
    scope: &str,
) -> Result<Option<SampleSpec>, StoreError> {
    store
        .conn()
        .query_row(
            "SELECT seed, size, drawn_at FROM review_samples WHERE project_id = ?1 AND scope = ?2",
            params![project.to_string(), scope],
            |row| {
                Ok(SampleSpec {
                    seed: row.get(0)?,
                    size: row.get(1)?,
                    drawn_at: Timestamp(row.get(2)?),
                })
            },
        )
        .optional()
        .map_err(db)
}

pub(crate) fn put_kept(
    store: &SqliteStore,
    project: ProjectId,
    kept: &KeptAside,
) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "INSERT INTO review_kept
             (project_id, job_id, setup_id, source_id, idx, revision, note, kept_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (project_id, job_id, setup_id, source_id, idx)
             DO UPDATE SET revision = ?6, note = ?7, kept_at = ?8",
            params![
                project.to_string(),
                kept.job_id.to_string(),
                kept.setup_id.to_string(),
                kept.source_id.to_string(),
                idx(kept.index),
                kept.revision,
                kept.note,
                kept.kept_at.0
            ],
        )
        .map_err(db)?;
    Ok(())
}

pub(crate) fn remove_kept(
    store: &SqliteStore,
    project: ProjectId,
    key: (JobId, SetupId, SourceId, Option<u32>),
) -> Result<(), StoreError> {
    let (job, setup, source, index) = key;
    store
        .conn()
        .execute(
            "DELETE FROM review_kept WHERE project_id = ?1 AND job_id = ?2 AND setup_id = ?3
             AND source_id = ?4 AND idx = ?5",
            params![
                project.to_string(),
                job.to_string(),
                setup.to_string(),
                source.to_string(),
                idx(index)
            ],
        )
        .map_err(db)?;
    Ok(())
}

pub(crate) fn kept(store: &SqliteStore, project: ProjectId) -> Result<Vec<KeptAside>, StoreError> {
    let conn = store.conn();
    let mut stmt = conn
        .prepare(
            "SELECT job_id, setup_id, source_id, idx, revision, note, kept_at FROM review_kept
             WHERE project_id = ?1 ORDER BY kept_at, rowid",
        )
        .map_err(db)?;
    let rows = stmt
        .query_map([project.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(db)?;
    rows.map(|r| {
        let (job, setup, source, index, revision, note, at) = r.map_err(db)?;
        Ok(KeptAside {
            job_id: parsed(&job)?,
            setup_id: parsed(&setup)?,
            source_id: parsed(&source)?,
            index: if index == WHOLE {
                None
            } else {
                Some(u32::try_from(index).map_err(corrupt)?)
            },
            revision,
            note,
            kept_at: Timestamp(at),
        })
    })
    .collect()
}
