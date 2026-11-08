# 2026-11-08 · Architecture checkpoint after version 1.5.0

## Context

Version 1.5.0 shipped to the Microsoft Store with the offline OCR vision models, run by
llama.cpp's own program on the processor or the graphics processor. A release is the end of a
milestone, and several files had passed ~500 lines since the [4 November
checkpoint](2026-11-04-checkpoint.md). Pull request #136 (offline analyses with Gemma 4) was open
at the same time and rewrites `crates/offline` (`host.rs`, `llm.rs`, `vision.rs`, `llama.rs`,
`cpu.rs`), moves the analysis schema helpers into `core/src/structured.rs`, and touches
`core/src/lib.rs`, `media/src/lib.rs`, the app's `setup.rs` and offline and settings commands,
the translation files, `state.svelte.ts` and the mocks. This checkpoint kept out of those files
so the two branches would not conflict. #136 merged while it ran, with #137 (Whisper on the
graphics processor through a Vulkan helper, which split `whisper.rs` into `whisper/`); the
branch was brought up to date and the findings those files held were taken up where still worth
it. What is left is listed below.

Two read-only reviews ran in parallel, one of the Rust workspace and one of the interface. Their
findings were checked against the code before being acted on or recorded here.

## What holds

- **Dependencies point inwards.** No new internal dependency since 4 November; `dictionary`,
  `projects` and `search` reach `store` and `export` only in their tests; the engine names no
  provider outside its tests.
- **The rules.** No `unwrap()` or `expect()` outside tests and start-up, no `anyhow` below the
  app, `cfg(target_os)` only in `platform/`, keys redacted in `Debug` and exposed only into
  requests.
- **Every trait earns its place.** `ProjectStore`, `DictionaryStore` and `SearchIndex` have one
  implementation each but keep dependencies pointing inwards; the others have a fake for tests or
  several implementations.

## Done in this checkpoint

All behaviour-preserving, except the one bug noted.

- **Duplication folded (Rust).**
  - Howard Hinnant's date algorithm, written twice (ALTO's `dateTime`, the pipeline record's
    date), is `Timestamp::utc_date` and `utc_date_time` in core.
  - The results' and the units' CSV are written through `kalamu_export::table_csv`, like
    descriptions, profiles and the dictionary; the golden files are unchanged byte for byte.
  - OpenAI's and Mistral's batch modules read their output files through one
    `batch::file_lines`.
  - The engine's fourteen poison-tolerant locks go through one `control::lock`.
  - The app's settings and granted folders are written through one `settings::write_json`.
- **Off the window's thread.** Exporting a project's descriptions, which reads every document,
  was the last synchronous export command; it runs through `off_thread`. `Projects` and
  `Dictionary` are `Clone` (they hold only `Arc`s) so the work can own them.
- **The offline engines' shared host.** Whisper, Tesseract and the model manager use
  `host::lock`, added by #136, instead of their own copies; Whisper's and Tesseract's idle
  janitors run through one `host::sweep_idle` with one `IDLE_UNLOAD`, each engine keeping its
  sweep (Whisper waits for the turn and empties its slot; Tesseract drops engines left unused).
- **Files split.** `catalog/src/lib.rs` 770 → 421 lines (its tests in `tests.rs`).
  `whisper/mod.rs`, 1,092 lines after #137, gives its samples, windows cut at pauses and loudest
  stretch to `whisper/audio.rs` with their tests (974 lines left, about 680 of them code).
- **Dead code removed.** `Catalog::languages_for`, `BatchState::is_open`,
  `AudioSource::is_video` (and the field only it read), and the unused `time` workspace
  dependency.
- **Interface.**
  - One `clock()` and one `sounding()` for the audio and video players (`pieces.ts`).
  - One `baseName()` and `pathParts()` (`paths.ts`) for seven copies of "the last part of a
    path".
  - One `pickJsonSave(name)` for four identical save pickers, and one source picker under
    `pickFiles` and `pickDocuments`.
  - `mayStart()`: consent, then the granted folders, the same steps before a job and a workflow
    run.
  - The review's zoom modifier and its written form asked of `platform.ts` (`zoomsWithWheel`,
    `modifier`), with tests, rather than tested in `SourcePane`.
  - One `.ellipsis` rule in `app.css` instead of eight scoped copies.
  - The dictionary export's default file name goes through the translation files
    (`dictionary.fileName`), and three strings nothing read (`app.tagline`,
    `pipeline.analysisSource`, `review.aiText`) are gone.
  - **Bug:** closing New job or a project before the drop listener was registered left it
    registered, so a later drop reached a screen that was gone. `listenWhileMounted()`
    removes it, as project search already did by hand, and project search now uses it too.

## Accepted

- **`projects/views.rs` keeps its own job loader.** It loads each job's pieces and corrections
  as well; built on `TiedResults` it would clone every job to save about eight lines.
- **Whisper keeps its own processor check.** `cpu::ensure`, from #136, writes "which {engine}
  need" for the plural engines; Whisper's sentence ("offline transcription needs") would change.
- **`ProjectError::Export` stays a string.** An `impl From<ExportError>` would remove one
  wrapper in the app, while `described.rs` uses the same constructor for serde errors.
- **`PipelineDiagram.svelte` (935 lines)** stays one illustration: the script is 78 lines, the
  rest is the drawing and its CSS.
- **Files near 500 lines that are one screen or one concern** stay: `core/units.rs`,
  `core/analysis.rs`, `prompts/defaults.rs` (preset text), `prompts/record.rs`,
  `engine/compare.rs`, `offline/tesseract.rs`,
  `SourcePane`, `ProjectSearch`, `ContentsReview`, `AllJobs`.

## Left for later

### What the merged files hold

1. **FFmpeg's and llama.cpp's helpers.** `quiet()` (no console window) and the lookup of a
   bundled program (environment variable, the app's folders, then `PATH` in debug builds) are
   written in `media/src/ffmpeg.rs` and `offline/src/llama.rs`: export both from `media`, which
   `offline` may depend on.
2. **`cfg!(windows)` below the app**: `llama.rs`'s `locate_vulkan` and `locate_opencl` test the
   system; drop the tests (those programs are bundled only on Windows and the file check fails
   elsewhere), or have the app say which builds exist.
3. **The four tests that write a JSON file for the interface** (`core/src/lib.rs`,
   `engine/src/rules.rs`, `search/src/fold.rs`, `search/src/terms.rs`) repeat "write only if
   changed": one helper behind a test feature of core.
4. **`timed_lines`** (reading `[hh:mm:ss]` markers into segments) is a shared reading of stored
   results that the engine reaches into `export` for; it belongs in core.
5. **Sizes**: `offline/src/vision.rs` (836 lines: `placeholder`, `grounded` and
   `without_runaway` into `vision/answer.rs`), `offline/src/llm.rs` (845) and `llama.rs` (560),
   each reviewed with the offline host in mind.
6. **The file picker's extensions** (`tauri.ts` `EXTENSIONS`) restate `InputKind::from_path`
   and have drifted: `.aiff`/`.aif`, `.wmv`/`.m4v` and `.xlsx` are read by the engine but hidden
   by the picker. Write each kind's extensions into `coreLists.json`, with the export formats
   that need a layout (`newJob.ts` `LAYOUT_FORMATS`) and the tasks that write text
   (`ProjectSearch`'s `GENERATED_TASKS`). A fix with a visible effect, so a change of its own.
7. **`tauri.ts` (551 lines)**: the pickers and native questions into `dialogs.ts`, with
   `state.svelte.ts`'s imports.
8. **`mock.ts` (769 lines)**: settings, presets and pipelines into `mockPresets.ts`, the job
   and review commands into a `jobCommand`, and `mockView` into `mockViews.ts`, as profiles and
   projects already are.
9. **Reader sort order** is written several ways (`jobList.ts`, `projects.ts`, `activity.ts`,
   `SetupList`, the collators of `glossary.ts`, `languages.ts` and `OfflineModels`): one
   `readerOrder` in `i18n/index.ts`, keeping each variant's options.

### The rest

- **Interface splits that keep behaviour**: `NewJob.svelte` (828: the set-aside notice, the
  reading options and the estimate into their own files), `newJob.ts` (527: model choice into
  `newJobModels.ts`, `warningText` and `duration` into `fileNotes.ts`), `ReviewView.svelte`
  (661: the open readings into `readings.svelte.ts`, the text splice of `useReading` into a
  tested `withReading()`), `ProjectWorkspace.svelte` (its sections and edit form), and
  `SetupList.svelte` (a `SetupCard` per draft).
- **Shared interface code not yet used everywhere**: the `Action` wrapper in `StepBanner`,
  `UseTextMenu`, `WorkflowEditor`, `WorkflowList`, `ProjectList`, `RunStart`, `SpeakerList`,
  `ReviewQueue`, and as a toast in `AllJobs`, `JobMenu`, `LanguagePicker`, `SetupList` and
  `SettingsView`; one `ContextMenu` for `JobMenu` and `ItemMenu`; one combobox for the two
  language pickers; `profiles.ts`'s `moved` in `FieldsEditor`.
- From the 4 November checkpoint, still open: Whisper's voice detection on accented data folders
  on Windows, the folders out of reach shown at launch, Tesseract's drawing resolution in
  provenance and its engines pooled.

## Consequences

- AGENTS.md is unchanged: its crate table lists what a crate may depend on, and `offline` will
  use `media` for item 1.
- The roadmap's first item points to this checkpoint and what it left.
