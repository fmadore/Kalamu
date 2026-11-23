//! A frozen text is what a job's results were read from (#239): a save without it, or with it
//! changed, fails and leaves a file saved earlier as it was; a file saved by 1.10.0 without one
//! opens and says which texts are missing and which jobs read them.

use super::*;

/// The frozen text the fixture's analysis read.
fn frozen(f: &Fixture) -> PathBuf {
    PathBuf::from(&f.derived.sources[0].path)
}

/// Saves the fixture again over a file saved before it, which must come out as it was, with no
/// part left beside it; returns the refusal.
fn refused_over_good_file(
    f: &Fixture,
    before: &[u8],
    path: &Path,
) -> Result<ProjectFileError, ProjectFileError> {
    let refused = match f.saved(true) {
        Ok(_) => panic!("the save should have failed"),
        Err(e) => e,
    };
    assert_eq!(
        std::fs::read(path).map_err(corrupt)?,
        before,
        "the file saved earlier is untouched"
    );
    assert!(!f.dir.path().join("Mission letters.kalamu.part").exists());
    Ok(refused)
}

#[test]
fn a_save_with_a_frozen_text_missing_fails_and_keeps_the_earlier_file()
-> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(true)?;
    let before = std::fs::read(&path).map_err(corrupt)?;
    std::fs::remove_file(frozen(&f)).map_err(corrupt)?;

    let refused = refused_over_good_file(&f, &before, &path)?;
    let ProjectFileError::FrozenTextMissing(named) = &refused else {
        panic!("not refused as missing: {refused:?}");
    };
    assert_eq!(refused.code(), "project.frozen_text_missing");
    assert!(named.contains("Letter of 1915"), "{named}");
    assert!(named.contains(&f.derived.name), "{named}");
    Ok(())
}

#[test]
fn a_save_with_a_frozen_text_changed_fails_and_keeps_the_earlier_file()
-> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(false)?;
    let before = std::fs::read(&path).map_err(corrupt)?;
    std::fs::write(frozen(&f), "Mon cher Père, changed behind Kalamu's back").map_err(corrupt)?;

    let refused = refused_over_good_file(&f, &before, &path)?;
    let ProjectFileError::FrozenTextChanged(named) = &refused else {
        panic!("not refused as changed: {refused:?}");
    };
    assert_eq!(refused.code(), "project.frozen_text_changed");
    assert!(named.contains("Letter of 1915"), "{named}");
    Ok(())
}

#[test]
fn a_file_saved_without_a_frozen_text_opens_and_says_what_is_missing()
-> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(false)?;
    // What 1.10.0 wrote when the text had changed: listed, not carried, its bytes gone.
    rusqlite::Connection::open(&path)
        .map_err(db)?
        .execute_batch(
            "DELETE FROM kalamu_file_chunks WHERE file_id IN
                 (SELECT id FROM kalamu_files WHERE role = 'snapshot');
             UPDATE kalamu_files SET carried = 0 WHERE role = 'snapshot';",
        )
        .map_err(db)?;

    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    assert_eq!((opened.documents, opened.jobs), (1, 2), "opened whole");
    assert_eq!(
        opened.missing_texts,
        vec![kalamu_core::MissingText {
            documents: vec!["Letter of 1915".into()],
            jobs: vec![f.derived.name.clone()],
            file_name: "lettre 1.txt".into(),
        }]
    );

    // A complete file says nothing is missing.
    let complete = f.saved(false)?;
    assert!(opened_missing(&f, &complete)?.is_empty());
    Ok(())
}

fn opened_missing(
    f: &Fixture,
    path: &Path,
) -> Result<Vec<kalamu_core::MissingText>, ProjectFileError> {
    let there = SqliteStore::in_memory()?;
    Ok(opened(f, &there, path, "elsewhere")?.missing_texts)
}
