//! What is kept beside the AI output, which is never changed: calls billed without a result to
//! keep, the researcher's corrections and the names given to speakers.

use kalamu_core::{
    Charge, Correction, JobId, SetupId, SourceId, SpeakerNames, StoreError, Timestamp,
};
use rusqlite::{params, params_from_iter};

use super::{SqliteStore, corrupt, db, scoped, to_json};

pub(crate) fn add_charge(store: &SqliteStore, charge: &Charge) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "INSERT INTO charges
                (job_id, setup_id, source_id, idx, usage, cost_usd, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                charge.job_id.to_string(),
                charge.setup_id.to_string(),
                charge.source_id.to_string(),
                charge.index,
                to_json(&charge.usage)?,
                charge.cost_usd,
                charge.created_at.0,
            ],
        )
        .map_err(db)?;
    Ok(())
}

/// A job's charges, or with `scope` one source's under one set-up, in the order they were
/// billed.
pub(crate) fn charges(
    store: &SqliteStore,
    job: JobId,
    scope: Option<(SetupId, SourceId)>,
) -> Result<Vec<Charge>, StoreError> {
    let (sql, args) = scoped(
        "SELECT setup_id, source_id, idx, usage, cost_usd, created_at
         FROM charges WHERE job_id = ?1",
        "ORDER BY rowid",
        "ORDER BY rowid",
        job,
        scope,
    );
    let conn = store.conn();
    let mut stmt = conn.prepare(&sql).map_err(db)?;
    let rows = stmt
        .query_map(params_from_iter(&args), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(db)?;
    let mut charges = Vec::new();
    for row in rows {
        let (setup, source, index, usage, cost_usd, created_at) = row.map_err(db)?;
        charges.push(Charge {
            job_id: job,
            setup_id: setup.parse::<SetupId>().map_err(corrupt)?,
            source_id: source.parse::<SourceId>().map_err(corrupt)?,
            index,
            usage: serde_json::from_str(&usage).map_err(corrupt)?,
            cost_usd,
            created_at: Timestamp(created_at),
        });
    }
    Ok(charges)
}

pub(crate) fn put_correction(
    store: &SqliteStore,
    correction: &Correction,
) -> Result<(), StoreError> {
    store
        .conn()
        .execute(
            "INSERT INTO corrections
                (job_id, setup_id, source_id, idx, text, reviewed, updated_at, fields)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (job_id, setup_id, source_id, idx) DO UPDATE SET
                text = excluded.text, reviewed = excluded.reviewed,
                updated_at = excluded.updated_at, fields = excluded.fields",
            params![
                correction.job_id.to_string(),
                correction.setup_id.to_string(),
                correction.source_id.to_string(),
                correction.index,
                correction.text,
                correction.reviewed,
                correction.updated_at.0,
                correction.fields.as_ref().map(to_json).transpose()?,
            ],
        )
        .map_err(db)?;
    Ok(())
}

/// A job's corrections, or with `scope` one source's under one set-up.
pub(crate) fn corrections(
    store: &SqliteStore,
    job: JobId,
    scope: Option<(SetupId, SourceId)>,
) -> Result<Vec<Correction>, StoreError> {
    let (sql, args) = scoped(
        "SELECT setup_id, source_id, idx, text, reviewed, updated_at, fields
         FROM corrections WHERE job_id = ?1",
        "",
        "",
        job,
        scope,
    );
    let conn = store.conn();
    let mut stmt = conn.prepare(&sql).map_err(db)?;
    let rows = stmt
        .query_map(params_from_iter(&args), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, bool>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<String>>(6)?,
            ))
        })
        .map_err(db)?;
    let mut out = Vec::new();
    for row in rows {
        let (setup, source, index, text, reviewed, updated, fields) = row.map_err(db)?;
        out.push(Correction {
            job_id: job,
            setup_id: setup.parse().map_err(corrupt)?,
            source_id: source.parse().map_err(corrupt)?,
            index,
            text,
            fields: fields
                .map(|f| serde_json::from_str(&f))
                .transpose()
                .map_err(corrupt)?,
            reviewed,
            updated_at: Timestamp(updated),
        });
    }
    Ok(out)
}

pub(crate) fn put_speaker_names(
    store: &SqliteStore,
    job: JobId,
    names: &SpeakerNames,
) -> Result<(), StoreError> {
    let mut conn = store.conn();
    let tx = conn.transaction().map_err(db)?;
    let key = [
        job.to_string(),
        names.setup_id.to_string(),
        names.source_id.to_string(),
    ];
    tx.execute(
        "DELETE FROM speaker_names WHERE job_id = ?1 AND setup_id = ?2 AND source_id = ?3",
        params![key[0], key[1], key[2]],
    )
    .map_err(db)?;
    for (label, name) in &names.names {
        if name.trim().is_empty() {
            continue;
        }
        tx.execute(
            "INSERT INTO speaker_names (job_id, setup_id, source_id, label, name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![key[0], key[1], key[2], label, name.trim()],
        )
        .map_err(db)?;
    }
    tx.commit().map_err(db)
}

/// The names given to speakers in a job, or with `scope` in one recording under one set-up.
pub(crate) fn speaker_names(
    store: &SqliteStore,
    job: JobId,
    scope: Option<(SetupId, SourceId)>,
) -> Result<Vec<SpeakerNames>, StoreError> {
    let (sql, args) = scoped(
        "SELECT setup_id, source_id, label, name FROM speaker_names WHERE job_id = ?1",
        "ORDER BY setup_id, source_id",
        "",
        job,
        scope,
    );
    let conn = store.conn();
    let mut stmt = conn.prepare(&sql).map_err(db)?;
    let rows = stmt
        .query_map(params_from_iter(&args), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(db)?;
    let mut out: Vec<SpeakerNames> = Vec::new();
    for row in rows {
        let (setup, source, label, name) = row.map_err(db)?;
        let setup_id: SetupId = setup.parse().map_err(corrupt)?;
        let source_id: SourceId = source.parse().map_err(corrupt)?;
        match out.last_mut() {
            Some(last) if last.setup_id == setup_id && last.source_id == source_id => {
                last.names.insert(label, name);
            }
            _ => out.push(SpeakerNames {
                setup_id,
                source_id,
                names: [(label, name)].into_iter().collect(),
            }),
        }
    }
    Ok(out)
}
