# 2026-11-03 · Source descriptions in each result's provenance

## Context

[#55](https://github.com/fmadore/Kalamu/issues/55)'s first increments kept the researcher's
description of each document's source in the project and wrote it out from the project
([decision](2026-11-03-source-descriptions.md), [profiles](2026-11-03-export-profiles.md)).
The files a job writes beside its sources said nothing of it: a transcription's
`provenance.json` named the file and its hash, not the shelfmark or the repository, so a result
moved out of Kalamu lost what it was a reading of.

zmo-ai-pipelines' `provenance.json`, which Kalamu's follows, describes a source by its file
alone; iwac-ai-pipelines writes its results into the Omeka S item that holds the description.
Kalamu does neither: the description stays in the project, and each result's record names it.

## Decisions

- **`documents` in `provenance.json`.** Each project document the source is: the project's id
  and name, the document's id and title, and each part of its description as `{value, from}`,
  `from` being `document` or `project`, the shape of the descriptions export. A list, since one
  job's source can be a document of several projects; absent for a job tied to none. Optional
  and new, so the schema stays at version 1. The `.json` result, which holds the provenance,
  carries it too.
- **Only the files that record the method.** The text, Markdown, Word, subtitle and CSV files
  are unchanged: a `.txt` or `.md` is read again by later jobs and must stay the text alone, and
  a CSV's columns are the next increment of export profiles, results' columns beside the
  description's.
- **Read through a narrow trait.** `SourceDocuments` (in `core`) answers, for a job's source,
  the documents it is; `Projects` implements it, and the app gives it to the engine at startup
  (`set_source_documents`). The engine reads, never organises: the job store and the project
  store stay apart, and the engine still depends on no project code. Without it (the engine's
  tests, a lookup that fails) the files are written without `documents`, never left unwritten.
- **As it stands, written again when it changes.** The description is read each time a result's
  files are written. Changing a document's title or description, the project's defaults or
  name, attaching or detaching a job, merging or deleting documents, or deleting the project
  writes `provenance.json` and the `.json` result again (`rewrite_provenance`), in the
  background, for the results already on disk: nothing is read again, no other file is
  touched, and a running job is left to write its own when it finishes.

## Consequences

- `core` gains `DescribedBy` (moved from `projects`), `ProvenanceDocument`,
  `ProvenanceDescribed` and `SourceDocuments`; `Provenance` its `documents`; the engine
  `set_source_documents` and `rewrite_provenance`; `projects` `sources_of`.
- `docs/formats.md` documents `documents`; the privacy policy says `provenance.json` holds the
  project's name, the document's title and its description, under the effective date of
  3 November 2026, not yet released.
- Left for later in #55: normalised and inferred values beside the literal ones (with #54 and
  #48); results' columns (review status, a preferred result's text or fields) in a profile.
