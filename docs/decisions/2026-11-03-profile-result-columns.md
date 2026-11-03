# 2026-11-03 · Results' columns in export profiles

## Context

Export profiles wrote a project's documents in the researcher's own columns, from their
descriptions only ([decision](2026-11-03-export-profiles.md)). A researcher preparing a dataset
(an Omeka S import, a spreadsheet to code by hand) also needs what was read from each
document: its text as corrected, how far it was checked, an analysis's answers.
[#55](https://github.com/fmadore/Kalamu/issues/55) asks for review status and preferred results
in the export, and the profiles decision left them for later because they need the job store
beside the project store; `Projects` already holds both.

iwac-ai-pipelines writes each model output into an Omeka S property of the item (an
`bibo:content` text, entities as linked items) and has no notion of a reviewed result;
zmo-ai-pipelines writes one CSV per run. Neither chooses between several readings of one
document, which Kalamu's projects allow.

## Decisions

- **Four values read from a result.** `ProfileValue` gains `Status { task }` (`reviewed`,
  `partly_reviewed`, `not_reviewed`, counted over its pages or segments), `Model { task }`,
  `Text { task }` (the pieces' corrected text, speakers' names applied, apart by a blank line)
  and `Answer { key }` (an analysis's answer to a field, corrected; a list stays a list, entries
  give their first field). The values are stable codes, not translated words, so a spreadsheet
  can filter on them in any interface language.
- **Which result: the preferred one, or the only one.** With several results for a task and
  none preferred the cell is empty: the app never chooses between readings, as the review queue
  and the document's page already hold. One result needs no choice. A preferred result whose
  job was deleted gives nothing rather than another one.
- **Checked like the rest.** A text column needs a task that writes text, an answer column a
  key on one line (`ProfileError::BadValue`, shown beside the column). An answer's key is free,
  since a profile serves every project; the dialog offers the keys the project's analyses
  asked (`answer_fields`), with their labels.
- **Read from the store, once per export.** `projects::resulted` gathers the jobs tied to the
  documents shown, their pieces, corrections and speakers' names, only when a column needs
  them, and reads stored answers through the job store. Nothing is sent to a model; another
  profile, or the same one again, only writes again.

## Consequences

- `core` gains the four values and `BadValue`; `projects` `resulted.rs` (`AnswerField`,
  `answer_fields`); the app the `project_answer_fields` command; the export dialog a group of
  result values and an answer's field.
- No new data is kept and nothing new leaves the computer: the privacy policy is unchanged.
- Left for later: a dictionary's canonical names beside the answers' own forms (with #48),
  rows per item rather than per document (with #50), normalised and inferred values (with #54).
