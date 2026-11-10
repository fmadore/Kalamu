//! Where the Markdown written beside an exported pipeline's or workflow's record goes. The
//! researcher chose the JSON file's name, and the save dialog asked before replacing it; nobody
//! asked about the Markdown file beside it, so one that an export of Kalamu did not write is
//! never replaced.

use std::path::{Path, PathBuf};

/// How many numbered names are tried before giving up.
const TRIES: u32 = 999;

/// The Markdown file to write beside `json`: `name.md`, unless a file of that name is there
/// that an export did not write; then the first of `name (2).md`, `name (3).md`… that is free
/// or was written by an export. `None` when none of them is.
pub fn markdown_beside(json: &Path) -> Option<PathBuf> {
    let stem = json.file_stem()?.to_string_lossy().into_owned();
    let folder = json.parent().unwrap_or_else(|| Path::new(""));
    (1..=TRIES)
        .map(|n| match n {
            1 => folder.join(format!("{stem}.md")),
            n => folder.join(format!("{stem} ({n}).md")),
        })
        .find(|path| may_replace(path))
}

/// Whether `path` is free, or holds the Markdown of an earlier export. A file that cannot be
/// read is someone else's.
fn may_replace(path: &Path) -> bool {
    if std::fs::symlink_metadata(path).is_err() {
        return true;
    }
    std::fs::read_to_string(path).is_ok_and(|text| written_by_export(&text))
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
}
