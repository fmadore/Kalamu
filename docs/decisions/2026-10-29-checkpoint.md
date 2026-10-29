# 2026-10-29 · Architecture checkpoint after error rates

## Context

`JobDetail.svelte` passed 500 lines with the error rates, and the 25 October checkpoint had
asked for the review's shared frame to be extracted "before the next feature that touches the
review", which three features since have touched. The whole architecture was reviewed against
AGENTS.md, the specification and the roadmap: the crates and their dependencies, duplication in
the providers, exporters and media steps, the oversized files, and whether resuming, caching,
progress and provenance still cover every path. Three read-only reviews were run in parallel
(front end, Rust structure, coverage of the job paths); their main findings were checked against
the code before being recorded here.

## What holds

- **Dependencies point inwards.** `core` depends on nothing internal; `engine` reaches
  `providers`, `offline` and `store` only through their traits; `offline` needs only `core`.
- **The rules hold in the code**: no `unwrap()` or `expect()` outside tests and the build script
  (clippy's lints enforce it), `anyhow` only in the app, no path found by which an API key could
  reach a log, the store or `provenance.json`.
- **Every provider call for a job goes through the engine**, recorded before and after; the
  table of contents' second step is a piece like any other (cached, retried, resumed, costed);
  AI output is never edited where it is stored. `test_api_key`, the catalogue refresh, Whisper
  downloads and YouTube's title lookup are not job calls and send no document.
- **The providers already share** sending, error mapping, retry classification and structured
  output (`http.rs`, `analysis.rs`, `batch.rs`); the exporters share their part helpers.

## Fixed with this checkpoint

- **Provenance claimed settings that were not sent.** The settings recorded with each result
  came from `request_settings(task, model)`, which knows neither the input nor whether the
  request is batched: every Gemini correction or analysis of a text recorded a media resolution
  it never sent, a YouTube video's low resolution went unrecorded, and Claude's batched results
  claimed `fallbacks`, which batches refuse. `Provider::sent_settings(request, batched)` now
  gives what a request carries, decided by the same helper that builds its body; the engine
  records it per piece, and per batch item rather than per batch. The provider tests now build
  requests from a page, a picture, a text and a video, and a batch, and check that a setting
  left out is absent as well as that one recorded is present.
- Two Gemini error messages carried runs of spaces from a lost line continuation.
- New job dropped photographed pages from an analysis, which the engine and the specification
  accept: the front end's copy of the rule (`readsKind`) had not followed the engine's.
- Unused: `kalamu_providers::find`, `serde` and `tracing` in `providers`, `serde` in `export`.

## To fix next, in this order

1. **Results that can be wrong without anyone knowing** (integrity):
   - An answer cut off at the model's output limit is stored as complete, though Gemini's code
     says the engine marks it: mark it, warn, and list it in provenance.
   - A source changed on disk after it was added is read without being hashed again, so new
     content is cached under the old fingerprint: check size and date, hash again if they
     changed, and refuse the source.
   - `put_result` replaces an answer stored under the same key (two identical set-ups, or a
     batch collected after a fast run): keep the first, so corrections stay over the text they
     corrected.
2. **Work lost or paid twice** (cost):
   - Quitting waits five seconds, then drops cloud calls still in flight, which are sent and
     billed again: wait for them, saying how many, with a way to quit at once.
   - Retry delays ignore pause and cancel, and an engine error returns without awaiting the
     calls already started.
   - At launch a job with a waiting batch is started in full, so a paused job also sends its
     remaining pages: collect only.
   - The cost shown covers the current run only, and billed answers that failed their fields
     are not counted.
3. **Results that do not reach the researcher**: a document's files are written only when every
   piece succeeded, so one page that always fails (no text layer, a copyright refusal) keeps
   the rest from disk; and two sources with the same name in one destination overwrite each
   other's files.
4. **Smaller provenance gaps**: whether Whisper used voice detection; speaker names; a table of
   contents' usage and cost per step; preprocessing entries written where nothing was done.

## Refactor

- **The review's frame**, copied in `ReviewView` (1,484 lines), `AnalysisReview` (778) and
  `ContentsReview` (682) with about 150–200 lines of identical CSS each, and drifting: only one
  has Ctrl+Enter, one stacks at 900 px instead of 1000, one has no splitter. One
  `ReviewFrame.svelte` (masthead, source pane with its preview and zoom, text pane, optional
  third pane, splitters, layout kept per computer, strip) taking snippets; `SiglaKey`,
  `CorrectionEditor` and `FieldInput` shared; `ReadingsPopover` and `MarkedText` out of
  `ReviewView`; the repeated loading and saving as a small TypeScript module. Text sources in
  `ReviewView` (a correction of a text file) wait for a preview that never comes; the frame's
  source slot fixes it. Page halves are shown as the raw `left` and `right`.
- **New job** (925): a joined choice strip for line breaks and pace, the language and output
  steps, and the start panel. **Job page** (549): facts, files and outputs. **Main page** (631):
  the sidebar. `PipelineDiagram` (1,002) stays one drawing: its animations reach across it.
- **Engine**: `run.rs` (644) into planning, one piece's call and the job loop, sharing the
  piece-saving and cost code `batch.rs` repeats; `batch.rs` into sending and collecting.
  `catalog` moves its cost estimates into a module of their own.
- **Rules kept in two places**: the front end copies the engine's rules on which files a task
  reads, which models can do a task, and which formats each task offers, and the engine does
  not check that a model does a task or that a source suits it. They should come from Rust.
- **Pipelines** have grown inside `prompts` (about 480 lines with their Markdown record), as the
  25 October checkpoint foresaw: a crate of their own when next touched.

## Accepted

- The searchable PDF exporter's copy of the page geometry stays, to keep `export` independent of
  `media`; the two copies have diverged on indirect page boxes, so `media` should follow
  `export`'s when next touched.
- Provider constructors, authentication headers and batch cancellation repeat a few lines per
  provider; a shared helper would hide less than it saves.
- `store` stays one file (its trait implementation reads best together); `prompts`' defaults
  and `core`'s analysis types are mostly data and tests.

## Consequences

- Feature work (search in the review) waits for the integrity and cost fixes and the review's
  frame, each a pull request of its own.
- The roadmap's *Next* lists them in this order.
