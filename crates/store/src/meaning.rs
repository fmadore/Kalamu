//! What the job database keeps of the search by meaning: the model each project uses, with the
//! researcher's consent to send its text and what its questions cost; and each preparation run
//! with its counts and cost. The vectors themselves are in a file of their own
//! ([`SqliteVectors`](crate::SqliteVectors)); these rows are the record that outlives it.
//! Deleting the project deletes them.

use kalamu_core::{
    EmbeddingLog, EmbeddingRun, EmbeddingRunState, EmbeddingSpace, ProjectId, ProviderId,
    SpaceChoice, StoreError, Timestamp,
};
use rusqlite::{OptionalExtension, Row, params};

use super::{SqliteStore, corrupt, db};

pub(crate) const MIGRATION: &str = r#"
CREATE TABLE meaning_spaces (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    dimensions INTEGER NOT NULL,
    chosen_at INTEGER NOT NULL,
    consented_at INTEGER,
    queries INTEGER NOT NULL DEFAULT 0,
    query_tokens INTEGER NOT NULL DEFAULT 0,
    query_cost_usd REAL,
    PRIMARY KEY (project_id, provider, model, dimensions)
);
CREATE TABLE meaning_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    dimensions INTEGER NOT NULL,
    state TEXT NOT NULL,
    chunks INTEGER NOT NULL,
    tokens INTEGER NOT NULL,
    cost_usd REAL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    error TEXT
);
CREATE INDEX meaning_runs_by_project ON meaning_runs(project_id);
"#;

fn state_name(state: EmbeddingRunState) -> &'static str {
    match state {
        EmbeddingRunState::Running => "running",
        EmbeddingRunState::Paused => "paused",
        EmbeddingRunState::Cancelled => "cancelled",
        EmbeddingRunState::Interrupted => "interrupted",
        EmbeddingRunState::Failed => "failed",
        EmbeddingRunState::Done => "done",
    }
}

fn parse_state(s: &str) -> Result<EmbeddingRunState, StoreError> {
    Ok(match s {
        "running" => EmbeddingRunState::Running,
        "paused" => EmbeddingRunState::Paused,
        "cancelled" => EmbeddingRunState::Cancelled,
        "interrupted" => EmbeddingRunState::Interrupted,
        "failed" => EmbeddingRunState::Failed,
        "done" => EmbeddingRunState::Done,
        other => return Err(StoreError::Corrupt(format!("run state {other}"))),
    })
}

fn choice(row: &Row<'_>) -> rusqlite::Result<(String, SpaceChoiceRow)> {
    Ok((
        row.get(0)?,
        SpaceChoiceRow {
            provider: row.get(1)?,
            model: row.get(2)?,
            dimensions: row.get(3)?,
            chosen_at: row.get(4)?,
            consented_at: row.get(5)?,
            queries: row.get(6)?,
            query_tokens: row.get(7)?,
            query_cost_usd: row.get(8)?,
        },
    ))
}

struct SpaceChoiceRow {
    provider: String,
    model: String,
    dimensions: u32,
    chosen_at: i64,
    consented_at: Option<i64>,
    queries: u32,
    query_tokens: i64,
    query_cost_usd: Option<f64>,
}

const CHOICE: &str = "SELECT project_id, provider, model, dimensions, chosen_at, consented_at,
     queries, query_tokens, query_cost_usd FROM meaning_spaces";

fn read_choice(project: &str, row: SpaceChoiceRow) -> Result<SpaceChoice, StoreError> {
    Ok(SpaceChoice {
        project_id: project.parse().map_err(corrupt)?,
        space: EmbeddingSpace {
            provider: ProviderId::new(row.provider),
            model: row.model,
            dimensions: row.dimensions,
        },
        chosen_at: Timestamp(row.chosen_at),
        consented_at: row.consented_at.map(Timestamp),
        queries: row.queries,
        query_tokens: u64::try_from(row.query_tokens).unwrap_or_default(),
        query_cost_usd: row.query_cost_usd,
    })
}

fn read_run(row: &Row<'_>) -> rusqlite::Result<RunRow> {
    Ok(RunRow {
        id: row.get(0)?,
        project: row.get(1)?,
        provider: row.get(2)?,
        model: row.get(3)?,
        dimensions: row.get(4)?,
        state: row.get(5)?,
        chunks: row.get(6)?,
        tokens: row.get(7)?,
        cost_usd: row.get(8)?,
        started_at: row.get(9)?,
        ended_at: row.get(10)?,
        error: row.get(11)?,
    })
}

struct RunRow {
    id: i64,
    project: String,
    provider: String,
    model: String,
    dimensions: u32,
    state: String,
    chunks: u32,
    tokens: i64,
    cost_usd: Option<f64>,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
}

impl RunRow {
    fn into_run(self) -> Result<EmbeddingRun, StoreError> {
        Ok(EmbeddingRun {
            id: self.id,
            project_id: self.project.parse().map_err(corrupt)?,
            space: EmbeddingSpace {
                provider: ProviderId::new(self.provider),
                model: self.model,
                dimensions: self.dimensions,
            },
            state: parse_state(&self.state)?,
            chunks: self.chunks,
            tokens: u64::try_from(self.tokens).unwrap_or_default(),
            cost_usd: self.cost_usd,
            started_at: Timestamp(self.started_at),
            ended_at: self.ended_at.map(Timestamp),
            error: self.error,
        })
    }
}

fn space_args(project: ProjectId, space: &EmbeddingSpace) -> [rusqlite::types::Value; 4] {
    [
        project.to_string().into(),
        space.provider.as_str().to_string().into(),
        space.model.clone().into(),
        i64::from(space.dimensions).into(),
    ]
}

impl EmbeddingLog for SqliteStore {
    fn current_space(&self, project: ProjectId) -> Result<Option<SpaceChoice>, StoreError> {
        let conn = self.conn();
        let row = conn
            .query_row(
                &format!("{CHOICE} WHERE project_id = ?1 ORDER BY chosen_at DESC, rowid DESC"),
                [project.to_string()],
                choice,
            )
            .optional()
            .map_err(db)?;
        row.map(|(p, r)| read_choice(&p, r)).transpose()
    }

    fn choose_space(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        at: Timestamp,
    ) -> Result<SpaceChoice, StoreError> {
        {
            let conn = self.conn();
            // Each choice later than the one before, so the newest is the current one even
            // within one millisecond.
            let latest: Option<i64> = conn
                .query_row(
                    "SELECT MAX(chosen_at) FROM meaning_spaces WHERE project_id = ?1",
                    [project.to_string()],
                    |row| row.get(0),
                )
                .map_err(db)?;
            let at = latest.map_or(at.0, |l| at.0.max(l + 1));
            let [p, provider, model, dimensions] = space_args(project, space);
            conn.execute(
                "INSERT INTO meaning_spaces (project_id, provider, model, dimensions, chosen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (project_id, provider, model, dimensions) DO UPDATE SET chosen_at = ?5",
                params![p, provider, model, dimensions, at],
            )
            .map_err(db)?;
        }
        self.current_space(project)?.ok_or(StoreError::NotFound)
    }

    fn consent(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        let changed = self
            .conn()
            .execute(
                "UPDATE meaning_spaces SET consented_at = ?5 WHERE project_id = ?1
                 AND provider = ?2 AND model = ?3 AND dimensions = ?4",
                rusqlite::params_from_iter(
                    space_args(project, space).into_iter().chain([at.0.into()]),
                ),
            )
            .map_err(db)?;
        if changed == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    fn add_query(
        &self,
        project: ProjectId,
        space: &EmbeddingSpace,
        tokens: u64,
        cost_usd: Option<f64>,
    ) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "UPDATE meaning_spaces SET queries = queries + 1,
                 query_tokens = query_tokens + ?5,
                 query_cost_usd = CASE WHEN ?6 IS NULL THEN query_cost_usd
                     ELSE COALESCE(query_cost_usd, 0) + ?6 END
                 WHERE project_id = ?1 AND provider = ?2 AND model = ?3 AND dimensions = ?4",
                rusqlite::params_from_iter(space_args(project, space).into_iter().chain([
                    i64::try_from(tokens).unwrap_or(i64::MAX).into(),
                    cost_usd.into(),
                ])),
            )
            .map_err(db)?;
        Ok(())
    }

    fn start_run(&self, run: &EmbeddingRun) -> Result<i64, StoreError> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO meaning_runs (project_id, provider, model, dimensions, state, chunks,
             tokens, cost_usd, started_at, ended_at, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                run.project_id.to_string(),
                run.space.provider.as_str(),
                run.space.model,
                run.space.dimensions,
                state_name(run.state),
                run.chunks,
                i64::try_from(run.tokens).unwrap_or(i64::MAX),
                run.cost_usd,
                run.started_at.0,
                run.ended_at.map(|t| t.0),
                run.error,
            ],
        )
        .map_err(db)?;
        Ok(conn.last_insert_rowid())
    }

    fn update_run(&self, run: &EmbeddingRun) -> Result<(), StoreError> {
        self.conn()
            .execute(
                "UPDATE meaning_runs SET state = ?2, chunks = ?3, tokens = ?4, cost_usd = ?5,
                 ended_at = ?6, error = ?7 WHERE id = ?1",
                params![
                    run.id,
                    state_name(run.state),
                    run.chunks,
                    i64::try_from(run.tokens).unwrap_or(i64::MAX),
                    run.cost_usd,
                    run.ended_at.map(|t| t.0),
                    run.error,
                ],
            )
            .map_err(db)?;
        Ok(())
    }

    fn embedding_runs(&self, project: ProjectId) -> Result<Vec<EmbeddingRun>, StoreError> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, provider, model, dimensions, state, chunks, tokens,
                 cost_usd, started_at, ended_at, error FROM meaning_runs
                 WHERE project_id = ?1 ORDER BY id DESC",
            )
            .map_err(db)?;
        let rows = stmt
            .query_map([project.to_string()], read_run)
            .map_err(db)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(db)?.into_run()?);
        }
        Ok(out)
    }

    fn mark_running_runs_interrupted(&self) -> Result<u32, StoreError> {
        let n = self
            .conn()
            .execute(
                "UPDATE meaning_runs SET state = 'interrupted' WHERE state = 'running'",
                [],
            )
            .map_err(db)?;
        Ok(n as u32)
    }
}
