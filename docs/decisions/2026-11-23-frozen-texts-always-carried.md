# 2026-11-23 · A project file carries every frozen text, or is not written

## Context

A project file always carries Kalamu's frozen texts (the corrected text an analysis, a
translation or a correction read) and the researcher's originals only when chosen
([project file](2026-11-20-project-file.md)). Kalamu 1.10.0 treated a file it could not carry
the same way for both: an original left out was counted and said, a frozen text left out (gone
from the data folder, or changed since it was read) was listed with `carried = 0` and nothing
was said. The file was written, opened without complaint, and its jobs came back without the
text behind their answers ([#239](https://github.com/fmadore/Kalamu/issues/239)).

## Decision

- **A frozen text missing or changed fails the save.** `ProjectFileError::FrozenTextMissing`
  and `FrozenTextChanged` (`project.frozen_text_missing`, `project.frozen_text_changed`, worded
  in the translation files) carry the readers in the detail: the documents' titles, then the
  jobs' names, the researcher's own words. Every frozen text is checked before failing, so all
  are named at once. Originals left out stay a count, as before.
- **A good earlier file is never replaced by a failed save.** The save already wrote
  `<name>.kalamu.part` beside the file chosen and renamed it only once whole; the check sits
  before the manifest is written and before the rename, and a failure deletes the part.
- **An incomplete file from 1.10.0 still opens**, and `ProjectFileOpened::missing_texts` lists
  each frozen text listed but not carried, with the documents and jobs that read it, read from
  the file's own rows. The summary after opening shows it.
- **No manifest change.** `kalamu_files` already lists every frozen text with its `carried`
  flag, so an incomplete file tells itself apart: a flag in the manifest saying "all carried"
  would repeat what the rows say and could disagree with them. The format stays at version 1,
  and 1.10.0 files open unchanged.

## Consequences

- A researcher whose data folder lost a frozen text cannot save the project until the jobs that
  read it are deleted or the text put back; the message names them so they can choose.
- The review and the queue do not yet mark, beside such a job's results, that the text it read
  is missing; only the summary after opening says so. That belongs with
  [#85](https://github.com/fmadore/Kalamu/issues/85).
- The plan shown before saving does not yet warn that a frozen text is missing; the save says
  so when it stops.
