# 2026-11-23 · Every file name Kalamu gives is kept for one file

## Context

Two places give a file its name on disk by cleaning or numbering the original's, and neither
checked the name it ended with against the names already given
([#238](https://github.com/fmadore/Kalamu/issues/238)), so two sources could share a path and
the second replace the first:

- **A job's results** (`engine`, `OutputNames`). A source whose name was taken was numbered,
  `scan (2)`, but the numbered name was not itself kept. `scan.pdf`, `scan.tif` and
  `scan (2).pdf` in one folder gave `scan`, `scan (2)` and `scan (2)` again: the last two wrote
  the same exports and the same `scan (2).provenance.json`. Windows names copies exactly so.
- **The originals a project file carries**, written when it is opened (`store`,
  `project_file/open.rs`, `write_carried`). Folders were numbered when two shared a name; files
  were not. Two originals of one folder whose names are one once cleaned for this system
  (`draft?.pdf` and `draft_.pdf` from a Mac, or names apart only by case, a trailing space or
  a dot) were written to one path, and both documents then pointed to the second one's bytes.

## Decision

- **One rule, in `core`**: `file_names::FolderNames`, pure and without I/O, holds the names given
  in one folder, compared without case as Windows and macOS compare them. A name is given as it
  is when free; otherwise the first free of `name (2)`, `name (3)`… And the name given, numbered
  or not, is kept, so a later file whose own name is `scan (2)` becomes `scan (2) (2)`. A file's
  number goes before its extension (`draft_ (2).pdf`). Both `engine` and `store` already depend
  on `core`, so neither gains a dependency.
- **A job's names follow its order of sources alone**, worked out from the job as before and
  never from what is on disk, so a job resumed, exported again or listed names its files as the
  run that first wrote them. Where no source's own name is another's number, the rule gives
  exactly the names it gave before (the k-th source of a name is `name (k)`): jobs without such
  a clash keep their names. A source listed twice still takes a name at each place, as it did,
  so the names after it do not move.
- **On opening a project file**, an original whose cleaned name an earlier original of its
  folder took is numbered, `draft_ (2).pdf`, and its document, jobs and frozen texts point
  there; the project is never refused for it.

## Consequences

- **A job already written with such a clash** wrote two sources' files under one name: what is
  on disk under `scan (2).*` is the results of whichever of the two was written last. From now
  on the later source, the one whose own name is `scan (2)`, writes under `scan (2) (2).*`, and
  the numbered one keeps `scan (2).*`. Nothing is moved or deleted: the files already under
  `scan (2).*` stay until that source writes them again (after a correction, or when a failed page
  is read again), which replaces them with its own results; the later source's files appear
  under their new name only once it is written again, and until then the job's list of files
  leaves them out. A project's links and corrections are by source, not by file name, so
  nothing else changes.
- A project file opened before this fix with such names left one file for two documents; the
  file still holds both, so opening it again with this version restores each.
- Names that differ only by Unicode normalisation (macOS's decomposed `é` against a composed
  one) are still compared as different; no case has been reported.
- Tests: `core` (`file_names`: a numbered name kept, names that never meet numbered as before,
  the number before the extension); `engine` (`output_names_are_fixed`;
  `sources_whose_names_meet_once_numbered_each_write_their_own_files` and
  `a_job_with_names_that_meet_resumed_half_way_names_its_files_as_one_run_through`, which read
  each text, JSON and `provenance.json` back and check it is its own source's); `store`
  (`two_originals_whose_names_meet_once_cleaned_are_both_kept`, with names cleaned to one, apart
  by case, and apart by a trailing space or dot, each document's file read back).
