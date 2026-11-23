//! Where the files an export writes beside the one the researcher chose go: the Markdown beside
//! an exported pipeline's or workflow's record, the workbook and CSV beside a project's table.
//! The researcher chose one file's name, and the save dialog asked before replacing it; nobody
//! asked about the files beside it, so one that an export of Kalamu did not write is never
//! replaced.

use std::path::{Path, PathBuf};

/// How many numbered names are tried before giving up.
const TRIES: u32 = 999;

/// The files to write beside `chosen`, one for each of `extensions`, all under the same name:
/// `name.ext`, unless one of them is there and `ours` does not recognise it as written by an
/// export; then the first of `name (2).ext`, `name (3).ext`… where each is free or
/// recognised. `None` when no name is. A folder, a link or a file that cannot be read is never
/// recognised.
pub fn beside(
    chosen: &Path,
    extensions: &[&str],
    ours: impl Fn(&Path) -> bool,
) -> Option<Vec<PathBuf>> {
    let stem = chosen.file_stem()?.to_string_lossy().into_owned();
    let folder = chosen.parent().unwrap_or_else(|| Path::new(""));
    (1..=TRIES)
        .map(|n| {
            let name = match n {
                1 => stem.clone(),
                n => format!("{stem} ({n})"),
            };
            extensions
                .iter()
                .map(|ext| folder.join(format!("{name}.{ext}")))
                .collect::<Vec<_>>()
        })
        .find(|paths| paths.iter().all(|path| may_replace(path, &ours)))
}

/// The Markdown file to write beside `json`: `name.md`, unless a file of that name is there
/// that an export did not write; then the first of `name (2).md`, `name (3).md`… that is free
/// or was written by an export. `None` when none of them is.
pub fn markdown_beside(json: &Path) -> Option<PathBuf> {
    beside(json, &["md"], |path| {
        std::fs::read_to_string(path).is_ok_and(|text| written_by_export(&text))
    })?
    .pop()
}

/// Whether `path` is free, or a file `ours` recognises.
fn may_replace(path: &Path, ours: &impl Fn(&Path) -> bool) -> bool {
    match std::fs::symlink_metadata(path) {
        Err(_) => true,
        Ok(meta) => meta.is_file() && ours(path),
    }
}

/// Whether a Markdown text is the record of a pipeline or a workflow as Kalamu exports it: a
/// title, then the sentence that says what it is (`PipelineFile::to_markdown`,
/// `WorkflowFile::to_markdown`).
pub(crate) fn written_by_export(text: &str) -> bool {
    let mut lines = text.lines();
    lines.next().is_some_and(|title| title.starts_with("# "))
        && lines.next().is_some_and(str::is_empty)
        && lines.next().is_some_and(|line| {
            line.starts_with("A Kalamu pipeline, exported on ")
                || line.starts_with("A Kalamu workflow (version ")
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const EXPORTED: &str = "# Letters\n\nA Kalamu pipeline, exported on 2026-11-10 from Kalamu \
                            1.5.0. The JSON file exported with it can be imported.\n";

    #[test]
    fn the_markdown_goes_beside_the_json_when_nothing_is_there() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("Letters.json");
        assert_eq!(markdown_beside(&json), Some(dir.path().join("Letters.md")));
    }

    #[test]
    fn an_earlier_export_is_replaced_and_anything_else_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let json = dir.path().join("Letters.json");
        std::fs::write(dir.path().join("Letters.md"), EXPORTED).unwrap();
        assert_eq!(markdown_beside(&json), Some(dir.path().join("Letters.md")));

        let workflow = "# Letters\n\nA Kalamu workflow (version 3), exported on 2026-11-10.\n";
        std::fs::write(dir.path().join("Letters.md"), workflow).unwrap();
        assert_eq!(markdown_beside(&json), Some(dir.path().join("Letters.md")));

        // The researcher's own notes of the same name.
        std::fs::write(dir.path().join("Letters.md"), "# Letters\n\nMy notes.\n").unwrap();
        assert_eq!(
            markdown_beside(&json),
            Some(dir.path().join("Letters (2).md"))
        );
        std::fs::write(dir.path().join("Letters (2).md"), "Other notes").unwrap();
        assert_eq!(
            markdown_beside(&json),
            Some(dir.path().join("Letters (3).md"))
        );
        // The numbered Markdown of an earlier export is replaced rather than piled up.
        std::fs::write(dir.path().join("Letters (2).md"), EXPORTED).unwrap();
        assert_eq!(
            markdown_beside(&json),
            Some(dir.path().join("Letters (2).md"))
        );
    }

    #[test]
    fn a_folder_of_that_name_is_not_a_place_to_write() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("Letters.md")).unwrap();
        assert_eq!(
            markdown_beside(&dir.path().join("Letters.json")),
            Some(dir.path().join("Letters (2).md"))
        );
    }

    #[test]
    fn only_kalamus_own_records_are_recognised() {
        assert!(written_by_export(EXPORTED));
        assert!(!written_by_export(""));
        assert!(!written_by_export(
            "A Kalamu pipeline, exported on 2026-11-10"
        ));
        assert!(!written_by_export(
            "# Notes\n\nSome text\nA Kalamu pipeline, exported on 2026-11-10"
        ));
    }

    #[test]
    fn the_files_beside_keep_one_name_and_skip_any_not_recognised() {
        let dir = tempfile::tempdir().unwrap();
        let chosen = dir.path().join("inventory.json");
        let ours = |p: &Path| std::fs::read(p).is_ok_and(|b| b == b"ours");
        assert_eq!(
            beside(&chosen, &["xlsx", "csv"], ours),
            Some(vec![
                dir.path().join("inventory.xlsx"),
                dir.path().join("inventory.csv")
            ])
        );
        // Only the CSV is the researcher's: both go under the next name, together.
        std::fs::write(dir.path().join("inventory.xlsx"), "ours").unwrap();
        std::fs::write(dir.path().join("inventory.csv"), "theirs").unwrap();
        assert_eq!(
            beside(&chosen, &["xlsx", "csv"], ours),
            Some(vec![
                dir.path().join("inventory (2).xlsx"),
                dir.path().join("inventory (2).csv")
            ])
        );
        std::fs::write(dir.path().join("inventory.csv"), "ours").unwrap();
        assert_eq!(
            beside(&chosen, &["xlsx", "csv"], ours),
            Some(vec![
                dir.path().join("inventory.xlsx"),
                dir.path().join("inventory.csv")
            ])
        );
    }
}
