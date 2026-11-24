# 2026-11-24 · A project file checked without opening it

## Context

A `.kalamu` file is made to be the copy a researcher relies on: before moving to a new
computer, or after a laptop breaks, it may be the only one. Until now the only way to know a
file was good was to open it, which adds a project and writes its originals into the data
folder ([#246](https://github.com/fmadore/Kalamu/issues/246)). The file already records what a
check needs: each carried file's role, fingerprint, size and whether it is carried
(`kalamu_files`), and opening already checks every fingerprint as it writes
([project file](2026-11-20-project-file.md)).

## Decision

- **`ProjectArchive::check_project_file`**, implemented by the store in
  `project_file/check.rs`, returns a `ProjectFileCheck`: a verdict, the manifest's name, date
  and version, the counts, the frozen texts carried whole and those missing (with their
  readers, as opening names them), the carried files whose bytes no longer match (by their
  documents' titles), the originals carried, left out by choice and left out because they could
  not be carried, the documents whose originals are not in the file, and the originals opening
  would write under another name.
- **The same reading as opening, never a second copy of it.** The checks that refuse a file
  (not a project file, SQLite's own check, a newer format or schema) moved from `open.rs` into
  `check.rs` as `readable`, which opening now calls. A carried file's bytes are read and hashed
  by one function, `carried::read_through`, which opening's `extract` writes through and the
  check reads through with nothing written. The names originals take moved into one
  `Placement`, given in the file's order by opening and by the check alike, so the names the
  check reports are those opening gives (a test opens the file and compares).
- **Three verdicts.** *Whole*: everything opens as saved. *Incomplete*: it opens, but frozen
  texts are missing (a file saved by 1.10.0). *Damaged*: a carried file no longer matches its
  fingerprint, so opening would be refused. Originals left out do not change the verdict: the
  file is whole for what it holds, and the documents to find again are listed below it.
- **A file this version cannot read at all is refused with the error opening gives**
  (`project.file_not_kalamu`, `project.file_damaged`, `project.file_newer`), shown in the
  check's dialog, rather than a fourth verdict: the sentences already exist in both languages
  and say what to do.
- **Nothing is written** but, for a file saved by an older Kalamu, the copy in the temporary
  folder that opening already brings up to date and deletes. Bytes are hashed chunk by chunk
  from the file, never extracted.
- **A save checks its part before the part takes the file's name.** The part
  (`<name>.kalamu.part`) is written, checked as above, and renamed only when it reads back and
  every carried file matches its fingerprint. A part that cannot be read back, or is damaged,
  is deleted and the save fails as a write that failed (`project.file_write`), the files that
  did not match named in the error's detail; a file saved earlier under that name stays as it
  was, as for a frozen text missing ([decision](2026-11-23-frozen-texts-always-carried.md)).
  Checking after the rename would have replaced a good earlier file before knowing the new
  one was good. A part with frozen texts missing cannot reach the check: the save already
  failed. So a save that succeeds is whole, says "saved and checked", and returns the check in
  `ProjectFileSaved::check`. Writing fills the first half of the progress and the check the
  second, so one bar covers both.
- **Only file names are reported as renamed.** A folder numbered because two folders share a
  name (`Archives 2`) changes no document's file name, and is left out.

## Consequences

- *Check a project file…* sits beside *Open a project file…* on the list of projects and opens
  a dialog: the file's path, progress, then the verdict and the details.
- Rows the store would refuse when opening (a row naming one the file does not hold) are not
  found by the check: only putting them in the store finds them, and opening still rolls back.
- The names are worked out by the rule both systems share (`portable_name`, `FolderNames`), so
  the same file reports the same renamed originals on Windows and on a Mac. Not yet tried on a
  real file carried between the two.
