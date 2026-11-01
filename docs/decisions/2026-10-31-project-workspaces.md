# 2026-10-31 · Project workspaces, first increment

## Context

[#57](https://github.com/fmadore/Kalamu/issues/57) asks for persistent projects: documents
grouped in collections, with every result read from each across jobs, the foundation for the
project dictionary (#48) and multi-step workflows (#49). Until now a researcher found their
material job by job. None of the three reference projects organises work above the run
(iwac-ai-pipelines and zmo-ai-pipelines process folders; Live Translation has sessions), so
nothing was ported: the rules below are Kalamu's own.

A new crate is an architecture checkpoint (AGENTS.md). Its findings are at the end.

## Decisions

- **A crate of its own, `projects`.** Organising is not job orchestration: it never calls a
  model, never schedules work and never changes a result, so it does not belong in `engine`.
  It depends on `core` only, reading jobs through `JobStore` and keeping its rows through a new
  `ProjectStore` trait, apart from `JobStore` so the engine cannot reach it. The app wires both
  traits to the same `SqliteStore`.
- **Projects live in the job database.** Their tables sit beside the jobs, so a job deleted
  unties itself from its documents by foreign key, and a project, collection or document
  deleted never cascades into a job or a result. The store's project code is a module of its
  own (`store/src/projects.rs`), as `lib.rs` had passed 600 lines.
- **A document is not a file.** It references its original where it lies (a `SourceFile`,
  hashed as New job hashes it), never copies or moves it. The same file at the same place with
  the same content is one document; equal bytes elsewhere are counted and offered as possibly
  the same item, never merged without the researcher (#57: identical bytes do not establish
  identity). Merging moves links, memberships and preferences and needs no call.
- **Explicit links tie documents to jobs**, by job and source: every set-up of that job over
  that source is a result of the document. A job started from the project's selection is
  linked by the source's id or its path and content, without asking, since its files were
  chosen in the project. A job run before is attached after a plan shows, for each file, the
  document proposed (same file), the documents with the same content, a new document, or
  leaving it out.
- **The selection is the job's own.** A job keeps the sources it was started with, so a
  collection changed later does not rewrite a past run or add work. The workflow runs of #49
  will freeze their selection the same way.
- **Preferred results are the researcher's.** One per document and task, checked to be a
  result of that document for that task; untying the job clears it; nothing chooses it
  automatically.
- **Views are read, not stored.** A project's table and a document's page are computed from
  the job store each time (pieces, corrections, results' costs), so they cannot drift from it.
- **Interface.** Projects sit under New job in the sidebar. A project shows its collections
  down the side, its documents in a table (found by title, file name or folder, sorted by
  title as a reader sorts, by date added or last read), and its activity; a selection can be
  grouped, removed or sent to New job. Files dropped on the window join the project shown.

## Checkpoint

- **Dependencies point inwards.** `projects` → `core`; `app` → `projects` to wire it; nothing
  else knows projects exist. The engine is unchanged.
- **One job each.** `projects` organises; `engine` runs; `store` persists both behind two
  traits.
- **Resuming, caching, progress, provenance** are untouched: no project path calls a
  provider, and a job started from a project goes through the engine as any job.
- **Size.** Every new file is under 500 lines; `store/src/lib.rs` (about 710) stays one file as
  the 29 October checkpoint accepted, with the project code beside it.
- **To watch**: `views.rs` reads each linked job's pieces and corrections on every call. For
  projects of thousands of documents it may need a summary query in the store.

## Consequences

- Left in #57: tags, the Activity view's filters and summed costs, document metadata, and
  workflow runs on a selection (#49). Result snapshots (#47) will link a downstream result to
  the document and revision it read.
- The privacy policy lists what projects keep, with a new effective date; the app sends nothing
  new.
