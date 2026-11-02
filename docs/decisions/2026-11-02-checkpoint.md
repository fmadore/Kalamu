# 2026-11-02 · Architecture checkpoint after version 1.0.0

## Context

Version 1.0.0 went to the Microsoft Store, and since the [29 October
checkpoint](2026-10-29-checkpoint.md) about 29,000 lines were added: project workspaces and their
crate, the project dictionary and its crate with merges proposed by a model, workflows,
reviewed-text snapshots, thinking levels, each model's output limit and several pages per call.
AGENTS.md asks for a review at a release, when a crate is added and when files pass ~500 lines;
all three applied. Three read-only reviews ran in parallel, as before (Rust structure, the paths
that call a provider, the front end), and their main findings were checked against the code
before being recorded here.

## What holds

- **Dependencies point inwards.** `store` is only a dev-dependency of `engine`, `projects` and
  `dictionary`; `offline` uses `catalog` in tests only; `dictionary` never reaches the engine,
  the app builds its proposal job.
- **Every new path that calls a model is an ordinary engine job**: a workflow's steps, *Analyse
  this text* and *Use this text in a new job*, and the dictionary's proposed merges. Each is
  cached, resumed, costed and recorded in provenance as any job.
- **Frozen text is safe**: its hash is the source's in every cache key, files are hashed again
  before a step reads them, and a later correction marks what read the text out of date.
- **The rules hold**: no `unwrap()` or `expect()` outside tests, `anyhow` only in the app, no
  hard-coded interface text, no new path for a key into a log, the store or provenance.
- **The 29 October refactor held**: the reviews share `ReviewFrame`; New job, the job page and
  the sidebar stayed split; the task rules come from Rust; bindings and command names agree.

## Fixed with this checkpoint

1. **Work paid twice or not counted** (engine). How many pages a model reads in one call was
   worked out again from the live catalogue at every run, so a catalogue refreshed between two
   runs regrouped the pages, changed their keys and sent them again; it is now settled when the
   job is created (`Setup::pages_cap`). A sample no longer sets the group size, so the whole run
   finds its pages in the cache. A grouped answer from which no page could be kept is recorded
   as a charge, in the job's cost and provenance's `refused`.
2. **Workflows.** Steps made at launch are tied to their project (the app subscribed after they
   were announced); a step's job and its progress are written together, so a crash never leaves
   a job the run does not know; a step that could not start is tried again when its input is
   read; cancelling takes the run's lock, so no step starts after.
3. **The proposal job** is named in the interface's language, writes only its provenance, into a
   folder of its own rather than the snapshots folder, and is deleted when its request cannot be
   recorded.
4. **Rules and sums from Rust.** The engine leaves out a searchable PDF that no model of the job
   can write; cost estimates moved from the app into `Engine::estimate`, which also sums them,
   and the catalogue's into `catalog/src/cost.rs`; `TaskRules` says where line breaks, samples
   and pages per call apply and `ModelRef` which model is chosen by default; "waiting for you" is
   `StepState::needs_you`, sent with each step.
5. **Duplication.** One lookup of corrected answers in core (`Correction::of`,
   `JobStore::finished_result`, `JobStore::answers`) instead of four; one `SetupRequest`, in core,
   instead of it and `PipelineSetup`; pipelines and workflows on one `Folder` of saved
   definitions; error ids given by each error type. The store's migrations and results, the
   dictionary's requests and prompts' rendering are modules of their own. In the interface,
   shared table, filter and selection styles, `Missing.svelte` for a value not known, and a
   development mock that fails a command it cannot answer instead of returning nothing.

## Accepted

- **Pipelines stay in `prompts`.** The 29 October checkpoint wanted them in a crate of their own
  when next touched. `prompts` now holds the definitions a researcher saves and shares (presets,
  pipelines, workflows), which is one job, and importing a pipeline or a workflow brings its
  presets in through the prompt library, so a separate crate would depend on `prompts` anyway.
- **A new run starts from the models' text.** Corrections belong to the job they were made in,
  so running a reviewed workflow again reads the uncorrected text and pays for the steps after
  the review again. The specification says so; keeping the last run's corrections is on the
  roadmap.
- **Jobs made by 1.0.0 resumed by this version** send the thinking level now suggested, under
  their old keys: the level was not part of a set-up then. Each result's provenance records
  what its request carried.
- **A lagging event channel** can still drop the event that ties a workflow step to its project.
  A pass at launch would tie them, but `link_job` would also undo a source the researcher
  untied, so it is left until linking happens where the step's job is made.
- **The front end still sums** a table of contents' pieces and an Activity's jobs it filtered,
  both over views Rust already built, and narrows what the workflow editor offers before the
  engine checks it again.
- **`PipelineDiagram`** (1,002 lines) stays one drawing, and `ReviewView` (782) is next to split
  if the review grows again.

## Consequences

- The release after 1.0.0 carries these fixes; feature work resumes after it with #50.
- Resume tests now cover pages read together across a catalogue change and a sample, a grouped
  answer that keeps no page, a step that could not start, and a step job recorded with its step.
