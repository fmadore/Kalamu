# 2026-11-03 · Architecture checkpoint after version 1.2.0

## Context

Since the [2 November checkpoint](2026-11-02-checkpoint.md), versions 1.1.0 to 1.2.0 shipped.
About 9,000 lines of code were added, not counting samples: items read on their own (#50), what
to check (#85), project search and its crate (#86), source descriptions, export profiles and
result columns (#55), Gemini 3.5 Transcribe, explicit prompt caching, the French interface and
macOS in CI. AGENTS.md asks for a review at a release, when a crate is added and when files pass
~500 lines; all three applied. Three read-only reviews ran in parallel, as before (Rust
structure, the paths that call a provider, the front end). Their main findings were checked
against the code before being recorded here.

## What holds

- **Dependencies point inwards.** `store` is only a dev-dependency of `projects`, `dictionary`
  and `search`. `core` does no I/O. `projects`, `search` and `dictionary` read the job store and
  never write to it. The search index is a file of its own, versioned and made again when it
  cannot be opened.
- **No new side channel to a model.** Items read on their own become ordinary sources: their
  pages are frozen as a PDF named by its hash, written the same way each time, so cache keys and
  resuming hold. Gemini 3.5 Transcribe goes through `Engine::call` and is costed from its
  tokens. OpenAI's explicit caching leaves the cache key unchanged. The review queue and search
  read the store only, and search's background preparation stops between batches, never
  mid-write.
- **The rules hold.** No `unwrap()` or `expect()` outside tests, no `anyhow` outside the app,
  `cfg(target_os)` only in `platform/`, no match on a provider's name in the engine, nothing new
  logged that could hold a key. Nothing outside `platform.ts` tests the system. English and
  French have the same shape, placeholders and error sentences.
- **`attention.rs` and `units.rs` belong in the engine.** The queue needs the comparison and the
  snapshots' freshness; items become sources through snapshots. Export profiles belong in
  `prompts` with the other saved definitions.

## Found

### Bugs

1. **CSV for spreadsheets without a BOM.** The `export` crate's CSV files start with a UTF-8
   byte-order mark so Excel reads accents. The descriptions, profile and dictionary CSVs do not
   (`projects/src/described.rs`, `profiled.rs`, `dictionary/src/export.rs`), so Excel garbles
   the accents in exactly the files meant for spreadsheets.
2. **Renaming a tag can drop the table's filter.** `DocumentTable.svelte` filters on the name as
   typed, while `rename_tag` tidies spaces and takes over an existing tag's spelling. The mock
   keeps the typed name, which hides this.
3. **Project search runs at two letters.** The front end uses the review's two-letter rule,
   Rust's index three (`search/src/fold.rs` `MIN_QUERY`), so typing two letters shows an error.
4. **Search reports a database error as "not read".** `search/src/query.rs` turns every store
   error while loading a job into a skip, so coverage is misstated instead of the error shown.
5. **A blocked answer's tokens are not counted.** `ProviderError::billed` gives usage only for
   `InvalidAnswer`. Gemini (RECITATION, SAFETY, and now transcription) and OpenAI
   (`content_filter`) return `Blocked` before reading the usage the response reports, so the
   job's cost and provenance's `refused` can show nothing for a call that was billed.
6. **Item edits are several writes without a transaction.** `split_unit` and `join_units` leave
   two items in one place, or pages in two items, after a crash between writes. `save_unit` does
   not check that the stored item belongs to the result named.
7. **`rewrite_provenance` can overwrite a fresher file.** It checks once that the job is not
   running, then writes without a lock. A resume or a correction saved meanwhile can be
   overwritten by the older state.
8. **Item work freezes the window.** `unit_sources` and `export_units` are synchronous commands,
   so they run on the main thread, and every item opens and hashes the whole source PDF again.
9. **Misleading messages.** Joining an item to itself or to another result's item says "job not
   found"; splitting outside its pages says "no pages"; nothing chosen says "nothing read yet".
10. **The pipeline file misstates Gemini Transcribe's settings.** The file gives the defaults,
    not the set-up's speakers and word timings. Each result's provenance is right.
11. **Text outside the translation files.** The default descriptions file name (`…
    descriptions`), quotes and list commas around items' titles, a literal colon in the queue
    banner, a bare `…`, "Shift" in shortcuts (⇧ on a Mac, "Maj" in French), and the `Sources`
    filter of the file dialog.

### Duplication

12. **Which results a document has** is worked out five times: profiles, the queue, search
    twice, and the dictionary. Each handles store errors its own way, and "the latest analysis
    names the field" is written twice.
13. **Corrected text** ("the correction, else the model's text, speakers' names applied") is
    built again in `projects/resulted.rs`, `search/passages.rs` and `engine/document.rs`, though
    the last checkpoint centralised corrected answers. The speakers' names index is built five
    times.
14. **Tables written three ways.** CSV and ordered JSON for descriptions, profiles and the
    dictionary are written outside the `export` crate, with their own error helpers. This is why
    finding 1 exists, and it gave `projects` its `csv` dependency.
15. **In the interface**, the busy/error/reload wrapper is copied into 11 components; selection
    (toggle, select all, prune) into three; three dialogs share their backdrop CSS and Escape
    handling. The export dialog does not move focus inside although it says it is modal.
    Deleting and renaming a job are written twice.
16. **Rust's rules restated in TypeScript.** Tasks that write text are listed twice though
    `TaskRules` has them; description fields, result tasks and queue reasons copy Rust enums
    unchecked; the queue's counts per reason are summed again in the front end. The review and
    project search match text with two implementations that must agree on a match's number, with
    no shared fixture.
17. **Mocks narrower than their commands.** Search ignores its options, the queue its selection;
    items' join and split do not bump the revision, and a half page (`8 (left)`) is refused. The
    error-sentence test does not scan `units`, `profile`, `dictionary` or `projects`.

### Large files

- **Split:**
  - `store/src/lib.rs` (557): the jobs' and pieces' SQL into `jobs.rs`, batches into
    `batches.rs`.
  - `src-tauri/src/lib.rs` (519): state assembly, event forwarding and resume tasks into
    `setup.rs`.
  - `engine/src/lib.rs` (512): the errors into `error.rs`, pause, cancel and start into
    `control.rs`.
  - `ReviewView.svelte` (948, grown again as the last checkpoint foresaw): its search and the
    opening from project search into a `ReviewSearch` class, the caret helpers into `caret.ts`,
    the comparison into `ComparePane.svelte` and its state, the foot into `ScoreFoot.svelte`.
- **Move pure logic out, leave the rest:**
  - `PipelineDiagram.svelte` (its stages into `pipeline.ts`).
  - `NewJob.svelte` (the form-to-pipeline functions into `newJob.ts`).
  - `catalog/src/lib.rs` (its tests into `tests.rs`).
- **Leave:** `core/units.rs`, `core/analysis.rs`, `prompts/defaults.rs`, `prompts/record.rs`,
  `offline/whisper.rs`, each about 400 lines of code with tests beside it.

### Tests missing

- **Items:** an item's job paused and resumed; the same item frozen twice giving one file and a
  cache hit; a half page through the engine.
- **Gemini Transcribe:** its token cost reaching the job and provenance.
- **Blocked answers:** one that carried usage.
- **OpenAI batches:** the cached instructions in a batch line.

### Process

- **#99 never reached main.** It was opened against #98's branch and merged after #98, so its
  work was lost until #101 replayed it. A pull request targets `main`, or is retargeted to
  `main` before merging.

## Accepted

- **Two foldings.** Search's keeps offsets for matching inside words; the dictionary's drops
  punctuation to build name keys. Different jobs.
- **`SourceDocuments` and `SearchIndex` have one implementation each.** They keep dependencies
  pointing inwards.
- **`PipelineDiagram`'s drawing** stays one illustration.

## Fixed with this checkpoint

The bugs, in a pull request of their own before the refactor:

1. Every CSV starts with one `CSV_BOM` from `core::export`: descriptions, profiles and the
   dictionary, as the results' exports did.
2. `rename_tag` returns the name the documents now carry (`TagRenamed`), and the table filters
   on it; the mock follows Rust's rules.
3. Project search waits for a word as long as Rust searches for. The rule and worked cases are
   written by `cargo test` into `src/lib/searchRules.json`, which the interface reads and its
   tests check against, as `taskRules.json` is shared.
4. Search skips only a job deleted since it was tied; any other failure is reported as the job
   database's (`job.database`), not as the index's or as text not read.
5. `ProviderError::Blocked` carries the usage a response reported when a filter stopped it after
   the model wrote (Gemini's finish reasons, OpenAI's content filter and refusals, Claude's
   refusals); `billed` returns it, so the job's cost and provenance's `refused` count it. A
   request refused before the model wrote, such as Gemini's `promptFeedback`, is not billed.
6. A split, a join and made units are written in one transaction (`JobStore::replace_units`);
   saving a unit checks it belongs to the result named.
7. Writing a result's files and rewriting its provenance take one lock across reading and
   writing, so the last to run writes from what is stored then.
8. Cutting out and exporting items run off the window's thread, and each source of a batch is
   checked and opened once.
9. `UnitError` gains `NotJoinable`, `OtherResult`, `BadSplit` and `NoneChosen`, each with its
   sentence; the error-sentence test reads the unit, profile, dictionary and project ids too.
10. Gemini Transcribe's pipeline record lists no settings: speakers and word timings are the
    set-up's options, which the record states apart. Provenance still gives what was sent.
11. The descriptions file's name, item titles in quotes, the queue banner's colon, a bare `…`,
    Shift in shortcuts (⇧ on a Mac, "Maj" in French, `platform.shifted`) and the file picker's
    filter name go through the translation files.

Also changed: **Report a problem** leads with bugs and feedback, opens GitHub's choice of forms,
and keeps the form for offensive or harmful output that Store policy 11.16 asks for. A
misreading is corrected in the review, not reported, so the form no longer asks for invented
text.

Tests added: a refusal's tokens in a job's cost and provenance, and in Gemini's and Claude's
recorded answers; a prompt refused before the model wrote, not billed; unit errors and a join or
save across tables of contents; the same item frozen twice giving one file and every page from
the cache; OpenAI's batch lines caching their instructions explicitly; search's rule shared by
both sides.

## Left for the refactor

- Findings 12 to 16: one helper in core for the results a document has, one for corrected text
  and the speakers' names index, tables written through the `export` crate, the interface's
  shared wrapper, selection and dialog, Rust's rules sent rather than restated, and the queue's
  counts per reason from Rust.
- The mocks of finding 17, except the tag rename's.
- The file splits above.
- Tests still missing: an item's job paused and resumed, a half page through the engine, Gemini
  Transcribe's token cost through a whole job, and a store failure surfacing from search.
