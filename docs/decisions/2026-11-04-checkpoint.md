# 2026-11-04 · Architecture checkpoint after version 1.4.0

## Context

Since the [3 November checkpoint](2026-11-03-checkpoint.md), versions 1.3.0 and 1.4.0 shipped:
offline text extraction with Tesseract and ALTO XML, the wide layout and a smoother review, the
Jobs page, a notarised disk image for macOS, and a sandboxed build for the Mac App Store. That
checkpoint fixed the bugs it found and left its refactor for later; feature work went on and the
files it said to split grew. A release and a new distribution channel are the end of a
milestone, so the refactor came first and a checkpoint followed, on one branch.

The refactor was done in two halves at once (the Rust side here, the interface's in a worktree,
merged after), and the checkpoint ran as two read-only reviews in parallel: the Rust structure,
and the paths that call a provider or write results (Tesseract, ALTO and the searchable PDF, the
Jobs page, the Mac App Store's grants). Their findings were checked against the code before
being acted on or recorded here.

## The 3 November refactor, done

- **One helper for the results a document has** (`core::TiedResults`): every set-up of every
  job tied to a project's documents, which one the researcher preferred, each job read once, a
  job deleted since skipped and any other failure returned. Export profiles, the review queue,
  search (twice) and the dictionary use it instead of five copies. `analysis::latest_labels` is
  the one "the latest analysis names the field".
- **One corrected text and one speakers' names index**: `JobStore::text`,
  `speakers::text_as_left`, and `SpeakerIndex` through `JobStore::speakers`, in the review, the
  comparison, the export document, search and profiles.
- **Tables through the export crate**: descriptions, profiles and the dictionary return a
  `core::Table`, and `kalamu_export::table_csv` writes it with the byte-order mark; `projects`
  and `dictionary` lose their `csv` dependency. Their JSON stays with them: each is a document of
  its own, written by serde.
- **Rust's rules sent, not restated**: `src/lib/coreLists.json` (description parts, text tasks,
  queue reasons) and `src/lib/matchCases.json` (what a query finds, which the review's search is
  tested against), written by `cargo test` like `searchRules.json` and checked by CI with the
  bindings. `QueueCounts.reasons` counts every reason, so the queue no longer counts again.
- **The interface's shared code**: one action wrapper (busy, error, reload) for twelve
  components, one selection for four, one dialog for three (focus moved in, kept in and given
  back), one place to rename and delete a job; mocks as wide as their commands (search options,
  the queue's selection, items' revisions, half pages).
- **Files split**: `engine/src/lib.rs` 522 → 333 lines (`error.rs`, `control.rs`),
  `store/src/lib.rs` 600 → 255 (`jobs.rs`, `batches.rs`), `src-tauri/src/lib.rs` 540 → 258
  (`setup.rs`), `ReviewView.svelte` 979 → 661 (`reviewSearch.svelte.ts`, `caret.ts`,
  `compare.svelte.ts`, `ComparePane`, `CompareBar`, `ScoreFoot`), `Sidebar.svelte` 847 → 432
  (`SidebarJobs`, `SidebarProjects`, `jobDrag.svelte.ts`), `PipelineDiagram.svelte` 1,086 → 935
  (`pipeline.ts`), `NewJob.svelte` 713 → 677 (`newJob.ts`).
- **Tests that were missing**: an item's job interrupted and resumed from the cache, half a
  spread read as an item, Gemini 3.5 Transcribe costed from its tokens through a whole job, and
  a job database failure said by search rather than counted as nothing read.

## What holds

- **Dependencies point inwards.** Every library crate depends on `core` alone, except `engine`
  (catalog, core, export, media, prompts); `store` is a dev-dependency only; the app wires.
- **No side channel to a model.** Tesseract is called only through `Engine::call`, each piece
  recorded before and after, its answer under its cache key; realign runs only on export.
- **The rules.** No `anyhow` in the crates; `unwrap`/`expect` only in tests; `cfg(target_os)`
  only in `platform/`; no provider named in the engine; no key in a URL, an error or a log.
  The Mac App Store build is a build variant (`app_store`, `tauri.appstore.conf.json`), not a
  system, and its bookmark code lives in `platform/macos.rs`.
- **Grants are opened before anything resumes**, so a resumed job or workflow reaches its
  folders; a write that fails is reported through `SourceFailed`.

## Found and fixed

1. **A page the engine read as empty and the researcher typed** had no text in ALTO or the
   searchable PDF, though the `.txt` had it. It is now one block over the page.
2. **Realign presented guesses as measured.** A skipped column typed in was squeezed into the
   box of the word before it, each word a sliver, and ALTO wrote it as measured. With more than
   three words in one box, the page is placed as an estimated block instead.
3. **A granted folder out of reach at launch was dropped for good** in the sandboxed build (an
   unplugged disk), so the next launch could not reach it and nothing asked again. It is kept.
4. **Photos with a 180° or mirrored EXIF orientation reached Tesseract unturned**: only quarter
   turns were re-encoded, and Tesseract ignores EXIF. Any orientation now sends an upright copy.
5. **Writing results froze the window**: saving a correction or speakers' names, exporting a
   source, comparing, error rates, the review's pieces, freezing text and walking dropped
   folders were synchronous commands, waiting on the lock a running job holds while it writes.
   They run off the window's thread (`commands::off_thread`).
6. **A file dropped with its folder was listed twice** (`dedup` only removes neighbours).
7. **CI checked only the bindings** among the files the tests write for the interface; it now
   checks the four JSON files too.
8. **`retry_failed` reset a running job's failed pieces** before `start` refused it; it now
   refuses first.
9. **The privacy policy** did not list `grants.json`, the container the sandboxed app's data
   lives in, that moving a Mac app to the Bin leaves its data, or that logs hold file paths. It
   does now, with a new effective date.
10. **Tesseract's cache identity**: its results are cached under `tesseract-5`, which holds
    neither the engine's version nor the language data's revision. A test now pins the three
    together, so a Dependabot bump or a new `tessdata_best` commit cannot reuse cached pages
    without a decision about the model's id.

Also tidied: `Correction::of` in the export document, `EXE_SUFFIX` rather than a Windows test
for FFmpeg's name, and two engine functions that only forwarded removed.

## Accepted

- **Deleting a running job is not refused by the engine.** The Jobs page and the job menu do
  not offer it, and the delete command is also how a job collecting its batches (held in the
  engine's running map) is cancelled and deleted. Pieces in flight finish into the cache; their
  row is refused by the foreign key and an error logged. Refusing it would break that path.
- **Pausing Tesseract waits for the page in progress** (up to one per engine): `recognize`
  cannot be interrupted. Nothing is half written; the comment on `Control` now says so.

## Left for later

- **Accented data folders on Windows**: `models.rs` turns Whisper's voice detection off for a
  non-ASCII path, while `build.rs` says the UTF-8 code page manifest makes such paths readable.
  Try a data folder under `C:\Users\Frédérick` and remove the check if it holds.
- **The grants' interface**: show the folders out of reach at launch, and grant again the
  folders of jobs made by the disk image (already in the Mac App Store's next steps).
- **Provenance of Tesseract's drawing**: PDF pages drawn at 300 dpi, or less for a page over
  7,000 pixels, are not recorded; ALTO states 300 dpi in every case.
- **Tesseract's engines**: idle ones hold their language data for three minutes (a delete is
  refused as in use meanwhile), each new one hashes the whole file in memory, and two jobs in
  different languages rebuild engines on every page. A bounded pool, release on delete, and a
  streamed hash.
- **Duplication still to fold**: `projects/views.rs`'s job loader onto `TiedResults`; the
  results' and units' CSV through `table_csv`; Hinnant's date algorithm in `alto.rs` and
  `prompts/record.rs` into a `Timestamp` method in core; the offline engines' `lock` and idle
  janitor; write-then-rename in the app's settings and grants; the four tests that write a JSON
  file for the interface; `catalog/src/lib.rs`'s tests into `tests.rs`;
  `export_descriptions` off the window's thread.

## Consequences

- AGENTS.md's crate table says `export` also writes the tables other crates make, and `core`
  holds the shared reading of a project's results.
- The roadmap's first priority is done; the items left above are listed under it.
