//! The database's schema: one migration per release that changed it, applied in order.

use super::{dictionary, projects, runs, snapshots};

/// Schema migrations, applied in order; `PRAGMA user_version` records how many have run.
pub(crate) const MIGRATIONS: &[&str] = &[
    r#"
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    task TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE TABLE pieces (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    piece TEXT NOT NULL,
    status TEXT NOT NULL,
    cache_key TEXT,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, idx)
);
CREATE TABLE results (
    cache_key TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    output TEXT NOT NULL,
    cost_usd REAL,
    duration_ms INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE TABLE corrections (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    text TEXT,
    reviewed INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, idx)
);
"#,
    r#"
CREATE TABLE speaker_names (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    label TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY (job_id, setup_id, source_id, label)
);
"#,
    r#"
ALTER TABLE corrections ADD COLUMN fields TEXT;
"#,
    r#"
CREATE TABLE batches (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    phase TEXT NOT NULL,
    submitted_at INTEGER NOT NULL,
    data TEXT NOT NULL
);
CREATE INDEX batches_by_job ON batches(job_id);
"#,
    r#"
ALTER TABLE results ADD COLUMN request TEXT;
"#,
    r#"
ALTER TABLE results ADD COLUMN truncated INTEGER NOT NULL DEFAULT 0;
"#,
    r#"
CREATE TABLE charges (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    setup_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    idx INTEGER NOT NULL,
    usage TEXT NOT NULL,
    cost_usd REAL,
    created_at INTEGER NOT NULL
);
CREATE INDEX charges_by_job ON charges(job_id);
"#,
    projects::MIGRATION,
    snapshots::MIGRATION,
    runs::MIGRATION,
    r#"
ALTER TABLE documents ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
"#,
    dictionary::MIGRATION,
    dictionary::PROPOSALS,
];
