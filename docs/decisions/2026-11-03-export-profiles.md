# 2026-11-03 · Export profiles for source descriptions

## Context

[#55](https://github.com/fmadore/Kalamu/issues/55)'s first increment wrote each project's
descriptions in Kalamu's own columns, every part beside where it came from
([decision](2026-11-03-source-descriptions.md)). A researcher importing them into another tool
(an Omeka S CSV import, a library's spreadsheet, a finding aid) has to rename, reorder and drop
columns by hand every time. #55 asks for saved profiles that choose, order and rename columns
and say how lists and missing values are written, that can be duplicated and reused, and whose
shared templates leave out a project's values.

iwac-ai-pipelines writes into Omeka S through its API and keeps no export mapping: each script
names the Dublin Core properties it fills. zmo-ai-pipelines writes fixed CSV columns. Neither
has a mapping the researcher edits, so there was nothing to port beyond the `dcterms:` terms
iwac-ai-pipelines uses, which the Dublin Core starter follows.

## Decisions

- **A profile is a saved definition, like a pipeline.** `ExportProfile` (in `core`, with its
  check) is a name, columns, a list separator, a missing value and whether line breaks become
  spaces. It is kept by `prompts` as one JSON file each in the data folder's `export-profiles`,
  through the same `Folder` as pipelines and workflows: saving under a name already used
  replaces it, a copy or an import takes a free name (`Omeka S (2)`). `prompts` becomes the crate
  of saved definitions, not only of prompts; a crate of its own for one small library was not
  worth it.
- **A column takes its value from a fixed vocabulary.** `ProfileValue`: the document's id, title,
  file name, path, its project's name, its collections and tags (lists), a part of the
  description as it stands, where that part came from, or a fixed text written in every row (an
  item set, a resource template). Free formulas are left out: the vocabulary is what the
  projects crate can answer from what is stored, so another profile never calls a model.
- **Checked in Rust, shown as it is edited.** A profile needs a name and a column; each heading
  is not empty, fits on one line and is not another column's whatever its capitals, which
  spreadsheets and importers ignore (accents do tell headings apart); the separator is not empty
  and fits on one line. The same value may be taken by two columns. `check_export_profile`
  returns the problem with the column at fault, so the dialog names it before saving, without
  the rules being written twice.
- **CSV as the profile says, JSON keeping types.** CSV joins lists with the profile's separator
  and writes its missing value; JSON keeps lists as arrays and writes `null`, with the headings
  as keys in the profile's order and the list of columns beside the rows
  (`kalamu-profiled-descriptions`, version 1). Line breaks become spaces in both when asked.
  Kalamu's own export stays the default, and *Every column* starts a profile that writes the
  same CSV, which a test checks.
- **A template leaves its fixed texts out unless asked.** Sharing a profile writes a
  `kalamu-export-profile` file with its columns; when it has fixed texts the researcher is asked
  whether to include them, since they are the one place a profile can hold a project's values.
  The profile's id stays on this computer.
- **In one dialog, from the project.** *Export descriptions* opens a dialog: Kalamu's columns or
  a saved profile, edited in place (headings, values, order, options), duplicated, shared,
  imported or deleted, then exported for the documents shown, as before. Profiles are not in
  Prompts: they are used where the export is.

## Consequences

- `core` gains `profile.rs`; `prompts` `ProfileLibrary` and `PromptError::Profile`; `projects`
  `export_profiled_csv` and `export_profiled_json`; the app eight commands and an optional
  profile on `export_descriptions`.
- The privacy policy lists the `export-profiles` folder, under the effective date of 3 November
  2026, not yet released.
- Left for later in #55: descriptions carried into each result's exports and `provenance.json`;
  normalised and inferred values beside the literal ones (with #54 and #48); results' columns
  (review status, a preferred result's text or fields) in a profile, which needs the job store
  beside the project store.
