//! The originals a project file carried, removed with the project that copied them in, and
//! never a file kept elsewhere or named by another project (#274); what a crash left (a
//! project's folder, a working copy of an older file) swept at launch.

use super::*;

/// Every file under `root`.
fn files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The fixture's file, with its originals, opened in a store of its own.
fn opened_with_originals(f: &Fixture) -> Result<(SqliteStore, ProjectId), ProjectFileError> {
    let path = f.saved(true)?;
    let there = SqliteStore::in_memory()?;
    let opened = opened(f, &there, &path, "there")?;
    assert_eq!(opened.originals, 1);
    Ok((there, opened.project_id))
}

#[test]
fn deleting_a_project_removes_the_originals_it_copied_in_and_nothing_else()
-> Result<(), ProjectFileError> {
    let f = fixture()?;
    let (there, p) = opened_with_originals(&f)?;
    let places = f.places("there");
    let folder = places.originals.join(p.to_string());
    assert_eq!(files(&folder).len(), 1);
    let snapshots = files(&places.snapshots);
    assert!(!snapshots.is_empty());
    // Files beside the project's folder: Kalamu's to keep, or not Kalamu's at all.
    let beside = places.originals.join("notes.txt");
    std::fs::write(&beside, b"mine").map_err(corrupt)?;
    let jobs = there.links(p)?;

    assert_eq!(there.copied_originals(p, &places)?, 1);
    assert!(
        there.remove_copied_originals(p, &places).is_err(),
        "never while the project exists"
    );
    assert!(folder.is_dir());

    there.delete_project(p)?;
    let removed = there.remove_copied_originals(p, &places)?;
    assert_eq!(
        removed,
        FilesRemoved {
            removed: 1,
            kept: 0,
            failed: 0
        }
    );
    assert!(!folder.exists(), "the project's folder is gone");
    assert!(f.letter.is_file(), "the researcher's own file is kept");
    assert!(beside.is_file());
    assert_eq!(files(&places.snapshots), snapshots, "frozen texts are kept");
    // The jobs stay, as deleting a project always left them; their file is now missing.
    for link in &jobs {
        there.job(link.job_id)?;
    }
    let reader = there.job(jobs[0].job_id)?;
    assert!(
        reader.sources[0]
            .path
            .starts_with(&folder.display().to_string())
    );
    assert!(!Path::new(&reader.sources[0].path).exists());
    // Nothing more to remove.
    assert_eq!(
        there.remove_copied_originals(p, &places)?,
        FilesRemoved::default()
    );
    Ok(())
}

#[test]
fn an_original_another_project_names_is_kept() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let (there, p) = opened_with_originals(&f)?;
    let places = f.places("there");
    let copied = there.documents(p)?[0].source.clone();
    let other = project("Letters, second reading");
    there.put_project(&other)?;
    there.put_document(&document(other.id, "Letter of 1915", copied.clone()))?;

    assert_eq!(there.copied_originals(p, &places)?, 0);
    there.delete_project(p)?;
    let removed = there.remove_copied_originals(p, &places)?;
    assert_eq!((removed.removed, removed.kept), (0, 1));
    assert!(Path::new(&copied.path).is_file());
    // Nor does the sweep at launch take it.
    assert_eq!(there.tidy_project_files(&places)?.removed, 0);
    assert!(Path::new(&copied.path).is_file());
    Ok(())
}

#[test]
fn launch_sweeps_what_a_crash_or_an_earlier_version_left() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let (there, p) = opened_with_originals(&f)?;
    let places = f.places("there");
    // The same file opened again: a project that lives, whose folder stays.
    let path = f.dir.path().join("Mission letters.kalamu");
    let living = opened(&f, &there, &path, "there")?.project_id;
    // Deleted by a version before this fix, or a crash before its files were removed.
    there.delete_project(p)?;
    let orphan = places.originals.join(p.to_string());
    let older = places.originals.join(ProjectId::new().to_string());
    std::fs::create_dir_all(older.join("Scans")).map_err(corrupt)?;
    std::fs::write(older.join("Scans").join("page 1.jpg"), b"jpg").map_err(corrupt)?;
    // A folder not named by a project's id is not Kalamu's to remove.
    let other = places.originals.join("Mine");
    std::fs::create_dir_all(&other).map_err(corrupt)?;
    std::fs::write(other.join("keep.txt"), b"mine").map_err(corrupt)?;
    // A working copy a crash left, and a file of another name beside it.
    std::fs::create_dir_all(&places.working).map_err(corrupt)?;
    let copy = places.working.join("kalamu-open-left.kalamu");
    std::fs::write(&copy, b"copy").map_err(corrupt)?;
    let notes = places.working.join("notes.txt");
    std::fs::write(&notes, b"kept").map_err(corrupt)?;

    let swept = there.tidy_project_files(&places)?;
    assert_eq!(
        swept,
        FilesRemoved {
            removed: 3,
            kept: 0,
            failed: 0
        }
    );
    assert!(!orphan.exists());
    assert!(!older.exists());
    assert!(!copy.exists());
    assert!(notes.is_file());
    assert!(other.join("keep.txt").is_file());
    assert_eq!(files(&places.originals.join(living.to_string())).len(), 1);
    assert!(f.letter.is_file());
    // Folders that are not there yet are no failure.
    let empty = SqliteStore::in_memory()?;
    let nowhere = super::places(f.dir.path(), "nowhere");
    assert_eq!(empty.tidy_project_files(&nowhere)?, FilesRemoved::default());
    Ok(())
}

#[test]
fn an_older_file_is_brought_up_to_date_in_the_working_folder() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(true)?;
    // Made as a Kalamu one migration older would have made it.
    let file = rusqlite::Connection::open(&path).map_err(db)?;
    let schema: i64 = file
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db)?;
    file.execute_batch(&format!(
        "DROP INDEX region_readings_by_source; DROP TABLE region_readings;
         PRAGMA user_version = {};",
        schema - 1
    ))
    .map_err(db)?;
    drop(file);

    let places = f.places("there");
    let seen = std::cell::Cell::new(false);
    let watch = |_| {
        let copies = files(&places.working);
        if copies.iter().any(|c| {
            c.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("kalamu-open-"))
        }) {
            seen.set(true);
        }
    };
    let check = f.store.check_project_file(&path, &places, &watch)?;
    assert_eq!(check.verdict, kalamu_core::ProjectFileVerdict::Whole);
    assert!(seen.take(), "the copy is made in the working folder");
    assert!(files(&places.working).is_empty(), "and deleted when done");

    let there = SqliteStore::in_memory()?;
    there.open_project_file(&path, &places, &watch)?;
    assert!(seen.take());
    assert!(files(&places.working).is_empty());
    Ok(())
}
