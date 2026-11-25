//! A project file checked without opening it (#246): a whole file passes; a carried file
//! altered, a frozen text missing, a newer format are each reported, by name; originals left
//! out and names this system would change are said; and nothing is written or created.

use kalamu_core::{ProjectFileCheck, ProjectFileVerdict, RenamedOriginal};

use super::paths::written_elsewhere;
use super::*;

/// Every file under `root`, with its size: what a check must leave as it was.
fn listing(root: &Path) -> Vec<(PathBuf, u64)> {
    let mut out = Vec::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
            } else {
                let size = entry.metadata().map(|m| m.len()).unwrap_or_default();
                out.push((path, size));
            }
        }
    }
    out.sort();
    out
}

fn checked(f: &Fixture, path: &Path) -> Result<ProjectFileCheck, ProjectFileError> {
    f.store.check_project_file(path, &f.places("here"), &|_| {})
}

/// Changes a file behind Kalamu's back.
fn edit(path: &Path, sql: &str) -> Result<(), StoreError> {
    rusqlite::Connection::open(path)
        .map_err(db)?
        .execute_batch(sql)
        .map_err(db)
}

#[test]
fn a_whole_file_passes_and_nothing_is_written_or_created() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(true)?;
    let before = listing(f.dir.path());
    let projects = f.store.projects()?.len();

    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Whole);
    assert_eq!(check.name, "Mission letters");
    assert_eq!(check.app_version, "1.9.0");
    assert_eq!((check.documents, check.jobs), (1, 2));
    assert_eq!((check.texts, check.originals), (1, 1));
    assert!(check.with_originals);
    assert_eq!((check.originals_not_chosen, check.originals_left), (0, 0));
    assert!(check.damaged.is_empty());
    assert!(check.missing_texts.is_empty());
    assert!(check.to_find.is_empty());
    assert!(check.renamed.is_empty());

    assert_eq!(
        listing(f.dir.path()),
        before,
        "nothing written beside the file"
    );
    assert_eq!(f.store.projects()?.len(), projects, "no project created");
    Ok(())
}

#[test]
fn a_save_checks_its_file_before_it_takes_its_name() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.dir.path().join("Mission letters.kalamu");
    let progress = std::cell::RefCell::new(Vec::new());
    let saved = f
        .store
        .save_project_file(f.project, &path, true, &f.places("here"), &|p| {
            progress.borrow_mut().push(p)
        })?;
    assert_eq!(saved.check.verdict, ProjectFileVerdict::Whole);
    // The check read the part, under another name, and found what the file now holds.
    assert_eq!(saved.check, checked(&f, &path)?);
    assert!(!f.dir.path().join("Mission letters.kalamu.part").exists());
    // One bar: the writing fills its first half, the check its second, to the end.
    let progress = progress.into_inner();
    let last = progress
        .last()
        .copied()
        .unwrap_or(kalamu_core::ProjectFileProgress { done: 0, total: 0 });
    assert!(last.total > 0 && last.done == last.total, "{last:?}");
    assert!(progress.windows(2).all(|w| w[0].done <= w[1].done));
    Ok(())
}

#[test]
fn originals_left_out_name_the_documents_to_find_again() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    // Saved without them, by choice.
    let path = f.saved(false)?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Whole);
    assert!(!check.with_originals);
    assert_eq!(check.originals, 0);
    assert_eq!((check.originals_not_chosen, check.originals_left), (1, 0));
    assert_eq!(check.to_find, ["Letter of 1915"]);

    // Chosen, but no longer on this computer when saved.
    std::fs::remove_file(&f.letter).map_err(corrupt)?;
    let path = f.saved(true)?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Whole);
    assert_eq!((check.originals_not_chosen, check.originals_left), (0, 1));
    assert_eq!(check.to_find, ["Letter of 1915"]);
    Ok(())
}

#[test]
fn a_carried_file_altered_is_reported_by_name() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(true)?;
    edit(
        &path,
        "UPDATE kalamu_file_chunks SET data = x'00' WHERE file_id IN
             (SELECT id FROM kalamu_files WHERE role = 'original')",
    )?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Damaged);
    assert_eq!(check.damaged, ["Letter of 1915"]);
    assert_eq!((check.texts, check.originals), (1, 0));

    // A frozen text altered is named by the document it was read for.
    let path = f.saved(true)?;
    edit(
        &path,
        "UPDATE kalamu_file_chunks SET data = x'00' WHERE file_id IN
             (SELECT id FROM kalamu_files WHERE role = 'snapshot')",
    )?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Damaged);
    assert_eq!(check.damaged, ["Letter of 1915"]);
    assert_eq!((check.texts, check.originals), (0, 1));

    // A chunk gone is as damaged as one changed.
    let path = f.saved(true)?;
    edit(
        &path,
        "DELETE FROM kalamu_file_chunks WHERE file_id IN
             (SELECT id FROM kalamu_files WHERE role = 'original')",
    )?;
    assert_eq!(checked(&f, &path)?.verdict, ProjectFileVerdict::Damaged);
    Ok(())
}

#[test]
fn a_frozen_text_missing_is_reported_with_who_read_it() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let path = f.saved(false)?;
    // What 1.10.0 wrote when the text had changed: listed, not carried, its bytes gone.
    edit(
        &path,
        "DELETE FROM kalamu_file_chunks WHERE file_id IN
             (SELECT id FROM kalamu_files WHERE role = 'snapshot');
         UPDATE kalamu_files SET carried = 0 WHERE role = 'snapshot';",
    )?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Incomplete);
    assert_eq!(check.texts, 0);
    assert_eq!(
        check.missing_texts,
        vec![kalamu_core::MissingText {
            documents: vec!["Letter of 1915".into()],
            jobs: vec![f.derived.name.clone()],
            file_name: "lettre 1.txt".into(),
        }]
    );
    Ok(())
}

#[test]
fn a_file_this_version_cannot_read_is_refused_as_opening_refuses_it() -> Result<(), ProjectFileError>
{
    let f = fixture()?;
    let path = f.saved(false)?;
    edit(&path, "UPDATE kalamu_project_file SET version = 99")?;
    let newer = checked(&f, &path);
    assert!(
        matches!(newer, Err(ProjectFileError::Newer(_))),
        "{newer:?}"
    );

    let path = f.saved(false)?;
    edit(&path, "PRAGMA user_version = 9999")?;
    assert!(matches!(
        checked(&f, &path),
        Err(ProjectFileError::Newer(_))
    ));

    let notes = f.dir.path().join("notes.kalamu");
    std::fs::write(&notes, "Not a database, just notes.".repeat(200)).map_err(corrupt)?;
    assert!(matches!(
        checked(&f, &notes),
        Err(ProjectFileError::NotProjectFile)
    ));

    let path = f.saved(true)?;
    let bytes = std::fs::read(&path).map_err(corrupt)?;
    let cut = f.dir.path().join("cut.kalamu");
    std::fs::write(&cut, &bytes[..bytes.len() / 2]).map_err(corrupt)?;
    assert!(matches!(
        checked(&f, &cut),
        Err(ProjectFileError::Damaged(_))
    ));
    Ok(())
}

#[test]
fn names_this_system_would_change_are_said() -> Result<(), ProjectFileError> {
    let f = fixture()?;
    let draft = f.dir.path().join("Archives").join("lettre 2.pdf");
    let source = file_source(&draft, b"%PDF-1.4 a draft of 1916")?;
    f.store
        .put_document(&document(f.project, "Draft of 1916", source))?;
    let path = f.saved(true)?;
    // Saved on a Mac: a name Windows refuses, cleaned to the other's (#238).
    written_elsewhere(
        &path,
        &f.letter.display().to_string(),
        "/Users/ama/Archives/draft?.pdf",
    )?;
    written_elsewhere(
        &path,
        &draft.display().to_string(),
        "/Users/ama/Archives/draft_.pdf",
    )?;
    let check = checked(&f, &path)?;
    assert_eq!(check.verdict, ProjectFileVerdict::Whole);
    assert_eq!(check.originals, 2);
    assert_eq!(
        check.renamed,
        vec![
            RenamedOriginal {
                documents: vec!["Letter of 1915".into()],
                from: "draft?.pdf".into(),
                to: "draft_.pdf".into(),
            },
            RenamedOriginal {
                documents: vec!["Draft of 1916".into()],
                from: "draft_.pdf".into(),
                to: "draft_ (2).pdf".into(),
            },
        ]
    );

    // Opening gives them exactly those names.
    let there = SqliteStore::in_memory()?;
    let opened = opened(&f, &there, &path, "there")?;
    let mut names: Vec<String> = there
        .documents(opened.project_id)?
        .iter()
        .filter_map(|d| {
            Path::new(&d.source.path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .collect();
    names.sort();
    assert_eq!(names, ["draft_ (2).pdf", "draft_.pdf"]);
    Ok(())
}
