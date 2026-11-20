//! Files refused, each for its own reason, and an open stopped half-way that leaves the store as
//! it was.

use super::*;
use crate::project_file::tables;

/// Changes a file behind Kalamu's back, its keys unchecked.
fn edit(path: &Path, sql: &str) -> Result<(), StoreError> {
    rusqlite::Connection::open(path)
        .map_err(db)?
        .execute_batch(&format!("PRAGMA foreign_keys = OFF; {sql}"))
        .map_err(db)
}

/// How many rows each table the file carries holds in the store.
fn counts(store: &SqliteStore) -> Result<Vec<(String, i64)>, StoreError> {
    let conn = store.conn();
    tables()
        .iter()
        .map(|t| {
            conn.query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |row| row.get(0))
                .map(|n| (t.to_string(), n))
                .map_err(db)
        })
        .collect()
}

#[test]
fn a_file_that_is_not_a_project_or_is_damaged_is_refused() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let there = SqliteStore::in_memory()?;
    let other = f.dir.path().join("notes.kalamu");
    std::fs::write(
        &other,
        "Not a database at all, just some notes.".repeat(200),
    )
    .map_err(corrupt)?;
    assert!(matches!(
        opened(&f, &there, &other, "there"),
        Err(ProjectFileError::NotProjectFile)
    ));
    // A SQLite database, but not a project file: the job database itself.
    let jobs = f.dir.path().join("jobs.sqlite");
    drop(SqliteStore::open(&jobs)?);
    assert!(matches!(
        opened(&f, &there, &jobs, "there"),
        Err(ProjectFileError::NotProjectFile)
    ));

    // Cut short, as a copy stopped half-way would leave it.
    let path = f.saved(true)?;
    let bytes = std::fs::read(&path).map_err(corrupt)?;
    let cut = f.dir.path().join("cut.kalamu");
    std::fs::write(&cut, &bytes[..bytes.len() / 2]).map_err(corrupt)?;
    assert!(matches!(
        opened(&f, &there, &cut, "there"),
        Err(ProjectFileError::Damaged(_))
    ));
    // A carried file whose bytes changed.
    edit(&path, "UPDATE kalamu_file_chunks SET data = x'00'")?;
    assert!(matches!(
        opened(&f, &there, &path, "there"),
        Err(ProjectFileError::Damaged(_))
    ));
    assert!(there.projects()?.is_empty());
    Ok(())
}

#[test]
fn a_file_from_a_newer_kalamu_is_refused() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let there = SqliteStore::in_memory()?;
    let path = f.saved(false)?;
    edit(&path, "UPDATE kalamu_project_file SET version = 99")?;
    assert!(matches!(
        opened(&f, &there, &path, "there"),
        Err(ProjectFileError::Newer(_))
    ));
    let path = f.saved(false)?;
    edit(&path, "PRAGMA user_version = 9999")?;
    assert!(matches!(
        opened(&f, &there, &path, "there"),
        Err(ProjectFileError::Newer(_))
    ));
    assert!(there.projects()?.is_empty());
    Ok(())
}

#[test]
fn an_open_stopped_half_way_leaves_the_store_as_it_was() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let before = counts(&f.store)?;
    let path = f.saved(true)?;
    // A late row the store refuses: a pair kept apart naming an entry the file does not hold.
    edit(
        &path,
        "INSERT INTO dictionary_apart (project_id, a, b)
         SELECT id, 'no-such-entry', 'nor-this-one' FROM projects",
    )?;
    let refused = opened(&f, &f.store, &path, "here");
    assert!(
        matches!(refused, Err(ProjectFileError::Damaged(_))),
        "{refused:?}"
    );
    assert_eq!(counts(&f.store)?, before, "nothing half-imported");
    // The original written before the rows were refused is gone again.
    let originals = f.places("here").originals;
    let left = std::fs::read_dir(&originals)
        .map(|entries| entries.count())
        .unwrap_or_default();
    assert_eq!(left, 0, "no original left behind");
    Ok(())
}
