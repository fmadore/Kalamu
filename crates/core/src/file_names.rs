//! The names Kalamu gives files in one folder, each kept for one file (#238): a job's results,
//! one name per source, and the originals a project file carries, written when it is opened.
//!
//! A name already given is numbered, `scan (2)`, as Windows names copies, and the numbered name
//! is itself kept, so a file whose own name is `scan (2)` does not take it again: it becomes
//! `scan (2) (2)`. Windows and macOS ignore case, so names are compared without it. The names
//! follow the order they are asked for alone, never what is on disk.

use std::collections::HashSet;

/// The names given so far in one folder, compared without case.
#[derive(Debug, Clone, Default)]
pub struct FolderNames {
    taken: HashSet<String>,
}

impl FolderNames {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gives `stem` followed by `extension` (written with its dot, or empty), or when that name
    /// is taken the first of `stem (2)`, `stem (3)`… that is free, and keeps it. Returns the name
    /// given, extension included.
    pub fn give(&mut self, stem: &str, extension: &str) -> String {
        let mut name = format!("{stem}{extension}");
        let mut n = 2u64;
        while !self.taken.insert(name.to_lowercase()) {
            name = format!("{stem} ({n}){extension}");
            n += 1;
        }
        name
    }

    /// Gives a file's name, numbered before its extension when taken: `draft_.pdf`, then
    /// `draft_ (2).pdf`. The extension is what follows the last dot, unless that dot starts the
    /// name.
    pub fn give_file(&mut self, name: &str) -> String {
        match name.rfind('.') {
            Some(dot) if dot > 0 => self.give(&name[..dot], &name[dot..]),
            _ => self.give(name, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_numbered_name_is_kept_too() {
        let mut names = FolderNames::new();
        assert_eq!(names.give("scan", ""), "scan");
        assert_eq!(names.give("Scan", ""), "Scan (2)");
        // Its own name is the one just numbered: numbered again, never shared.
        assert_eq!(names.give("scan (2)", ""), "scan (2) (2)");
        assert_eq!(names.give("SCAN", ""), "SCAN (3)");
    }

    #[test]
    fn names_that_never_meet_are_numbered_as_before() {
        // Without a name that is another's number, the k-th of a name is `name (k)`.
        let mut names = FolderNames::new();
        let given: Vec<String> = ["Scan", "scan", "lettre", "SCAN"]
            .iter()
            .map(|s| names.give(s, ""))
            .collect();
        assert_eq!(given, ["Scan", "scan (2)", "lettre", "SCAN (3)"]);
    }

    #[test]
    fn a_file_is_numbered_before_its_extension() {
        let mut names = FolderNames::new();
        assert_eq!(names.give_file("draft_.pdf"), "draft_.pdf");
        assert_eq!(names.give_file("Draft_.PDF"), "Draft_ (2).PDF");
        assert_eq!(names.give_file("draft_ (2).pdf"), "draft_ (2) (2).pdf");
        assert_eq!(names.give_file("archive.tar.gz"), "archive.tar.gz");
        assert_eq!(names.give_file("archive.tar.gz"), "archive.tar (2).gz");
        assert_eq!(names.give_file(".notes"), ".notes");
        assert_eq!(names.give_file(".notes"), ".notes (2)");
        assert_eq!(names.give_file("README"), "README");
        assert_eq!(names.give_file("readme"), "readme (2)");
    }
}
