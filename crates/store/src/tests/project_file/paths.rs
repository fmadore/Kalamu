//! A file saved on Windows opened on a Mac, and the reverse: the paths it holds are written as
//! the other system writes them, here made so by rewriting a file saved on this computer.

use super::*;

/// Rewrites, everywhere the file holds it, the letter's path as `to`: in the JSON documents
/// (escaped as JSON writes it) and in the list of carried files.
fn written_elsewhere(path: &Path, from: &str, to: &str) -> Result<(), StoreError> {
    let json = |s: &str| serde_json::to_string(s).map(|q| q[1..q.len() - 1].to_string());
    let (from_json, to_json) = (json(from).map_err(corrupt)?, json(to).map_err(corrupt)?);
    let file = rusqlite::Connection::open(path).map_err(db)?;
    for (table, column) in [
        ("documents", "source"),
        ("jobs", "data"),
        ("snapshots", "data"),
    ] {
        file.execute(
            &format!("UPDATE {table} SET {column} = REPLACE({column}, ?1, ?2)"),
            [&from_json, &to_json],
        )
        .map_err(db)?;
    }
    file.execute(
        "UPDATE kalamu_files SET path = ?2 WHERE path = ?1",
        [from, to],
    )
    .map_err(db)?;
    Ok(())
}

fn opened_from(written: &str, with_originals: bool) -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(with_originals)?;
    written_elsewhere(&path, &f.letter.display().to_string(), written)?;
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    let doc = &there.documents(opened.project_id)?[0];
    let job = there.job(there.links(opened.project_id)?[0].job_id)?;
    assert_eq!(doc.source.file_name, "lettre 1.pdf");
    if with_originals {
        // Written here under its own name, in a folder named after the one it was in.
        let expected = f
            .places("there")
            .originals
            .join(opened.project_id.to_string())
            .join("Archives")
            .join("lettre 1.pdf");
        assert_eq!(PathBuf::from(&doc.source.path), expected);
        assert_eq!(job.sources[0].path, doc.source.path);
        assert_eq!(
            std::fs::read(&expected).map_err(corrupt)?,
            b"%PDF-1.4 a letter of 1915"
        );
        // The frozen text's record names the original where it now is.
        let derived = there.job(there.links(opened.project_id)?[1].job_id)?;
        let from = derived.sources[0]
            .derived_from
            .clone()
            .ok_or(StoreError::NotFound)?;
        assert_eq!(from.original_path, doc.source.path);
    } else {
        // Kept as written there: a path of the other system, which simply is not found here.
        assert_eq!(doc.source.path, written);
        assert_eq!(job.sources[0].path, written);
        assert!(!Path::new(&doc.source.path).exists());
    }
    Ok(())
}

#[test]
fn a_file_saved_on_windows_opens_with_posix_paths() -> Result<(), ProjectFileError> {
    opened_from(r"C:\Users\Ama\Archives\lettre 1.pdf", true)?;
    opened_from(r"C:\Users\Ama\Archives\lettre 1.pdf", false)
}

#[test]
fn a_file_saved_on_a_mac_opens_with_windows_paths() -> Result<(), ProjectFileError> {
    opened_from("/Users/ama/Archives/lettre 1.pdf", true)?;
    opened_from("/Users/ama/Archives/lettre 1.pdf", false)
}

/// The letter and a second document of the same folder saved as written at `first` and
/// `second` on the other computer, whose names are one once cleaned for this one, then opened:
/// each document must point to a file of its own bytes (#238). Returns the two files' names.
fn two_originals_named(first: &str, second: &str) -> Result<(String, String), ProjectFileError> {
    let f = fixture()?;
    let draft = f.dir.path().join("Archives").join("lettre 2.pdf");
    let source = file_source(&draft, b"%PDF-1.4 a draft of 1916")?;
    f.store
        .put_document(&document(f.project, "Draft of 1916", source))?;
    let path = f.saved(true)?;
    written_elsewhere(&path, &f.letter.display().to_string(), first)?;
    written_elsewhere(&path, &draft.display().to_string(), second)?;

    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    assert_eq!(opened.originals, 2);
    let documents = there.documents(opened.project_id)?;
    let path_of = |title: &str| -> Result<PathBuf, ProjectFileError> {
        documents
            .iter()
            .find(|d| d.title == title)
            .map(|d| PathBuf::from(&d.source.path))
            .ok_or(ProjectFileError::Store(StoreError::NotFound))
    };
    let (letter, draft) = (path_of("Letter of 1915")?, path_of("Draft of 1916")?);
    let folder = f
        .places("there")
        .originals
        .join(opened.project_id.to_string())
        .join("Archives");
    assert_eq!(letter.parent(), Some(folder.as_path()));
    assert_eq!(draft.parent(), Some(folder.as_path()));
    assert_eq!(
        std::fs::read(&letter).map_err(corrupt)?,
        b"%PDF-1.4 a letter of 1915"
    );
    assert_eq!(
        std::fs::read(&draft).map_err(corrupt)?,
        b"%PDF-1.4 a draft of 1916"
    );
    // The job that read the letter reads it where it was written.
    let job = there.job(there.links(opened.project_id)?[0].job_id)?;
    assert_eq!(PathBuf::from(&job.sources[0].path), letter);
    let name = |p: &Path| {
        p.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    Ok((name(&letter), name(&draft)))
}

#[test]
fn two_originals_whose_names_meet_once_cleaned_are_both_kept() -> Result<(), ProjectFileError> {
    // A name Windows refuses, cleaned to the other's.
    assert_eq!(
        two_originals_named(
            "/Users/ama/Archives/draft?.pdf",
            "/Users/ama/Archives/draft_.pdf"
        )?,
        ("draft_.pdf".into(), "draft_ (2).pdf".into())
    );
    // Names apart only by case, as a case-sensitive volume allows.
    assert_eq!(
        two_originals_named(
            "/Users/ama/Archives/Draft.pdf",
            "/Users/ama/Archives/draft.pdf"
        )?,
        ("Draft.pdf".into(), "draft (2).pdf".into())
    );
    // Names apart only by a trailing space or dot, which neither system keeps.
    assert_eq!(
        two_originals_named(
            "/Users/ama/Archives/draft.pdf ",
            "/Users/ama/Archives/draft.pdf."
        )?,
        ("draft.pdf".into(), "draft (2).pdf".into())
    );
    Ok(())
}
