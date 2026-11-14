# 2026-11-14 · What the 8 November checkpoint left, taken up

## Context

The [checkpoint of 8 November](2026-11-08-checkpoint.md) listed nine things under *What the
merged files hold* and left them for later. After 1.7.0 they were the second item of the
roadmap's *Next*, and the only one that needs no Mac. They were done in one round, in three
groups whose files do not overlap: the offline engines and `media`; `core`, `engine`, `export`,
`search` and the pickers; the development mock and the sort order.

## Decision

Numbers are the checkpoint's.

1. **One helper for the programs shipped beside the app.** `media/src/bundled.rs` holds `quiet()`
   (no console window on Windows) and `folders()` (the environment variable and `PATH` in
   development builds only, the app's folders between them). FFmpeg, llama.cpp and the Whisper
   helper use it. Whisper's own lookup of its helper stays: it reads the app's folders only,
   with no variable and no `PATH`, and folding it would change that.
2. **`llama.rs` no longer asks which system it runs on.** `locate_vulkan` and `locate_opencl`
   look for their files and nothing else: only the Windows packages ship the Vulkan build, and
   only Windows ARM64 the OpenCL one, so elsewhere they are not found. A release build behaves
   as before on every system. In a development build, one of these programs on `PATH` is now
   found on a system that does not ship it; it still runs only if it lists a device.
3. **One "write only if changed" helper** for the four tests that write a JSON file for the
   interface: `core`'s `test_support::write_interface_json`, behind a `test-support` feature
   that `engine` and `search` turn on in their development dependencies only.
4. **`timed_lines` is in `core`** (`timed.rs`), with `segments_from_text`: a shared reading of
   stored results. `export` and the engine's review use it; the engine still depends on
   `export` for the exporters.
5. **Sizes**, each a move with its tests: `vision.rs` 835 → 671 lines (`vision/answer.rs`),
   `llm.rs` 871 → 722 (`llm/command.rs`), `llama.rs` 672 → 506 (`llama/files.rs`).
6. **The file pickers' extensions come from Rust.** `InputKind::EXTENSIONS` is the one table:
   `from_path` reads it, `media`'s text source reads its text row, and `coreLists.json` carries
   it to the interface with each task's kinds (`Task::source_kinds`, which already existed), the
   tasks in New job's order and the export formats that need a layout. `EXTENSIONS` and `PAGES`
   in `tauri.ts`, `LAYOUT_FORMATS` in `newJob.ts` and `GENERATED_TASKS` in the project search are
   derived from it. Dropping files already asked Rust, so picking and dropping now agree.
7. **`tauri.ts`** 611 → 424 lines: the pickers and native questions are in `dialogs.ts`.
8. **`mock.ts`** 944 → 354 lines: `mockPresets.ts`, `mockJobCommand.ts` and `mockViews.ts`.
9. **One `readerOrder`** in `i18n/index.ts`, a collator for the interface language, each list
   stating its own options (`numeric`, `sensitivity`). `dictionary.ts` held a tenth copy.

## What changes for the researcher

- **Correcting and translating a text offer `.csv`** in their pickers. The engine already read
  it for these tasks, and a dropped `.csv` was already accepted.
- **A dropped `.xlsx` is refused when it is dropped.** The checkpoint said the engine read
  `.xlsx` and the picker hid it. It did not: `from_path` called it text, the text source refused
  it, and the job failed when it ran. It is now out of the table.
- **Lists sorted with the webview's own language** (the job list, a project's documents and
  tags, the activity filters, a model's languages) are sorted in the interface language.

## Consequences

- AGENTS.md is unchanged: no new internal dependency.
- Still open from the checkpoint's *The rest*: the screens to split (`NewJob.svelte`,
  `newJob.ts`, `ReviewView.svelte`, `ProjectWorkspace.svelte`, `SetupList.svelte`), the shared
  `Action`, `ContextMenu` and combobox not yet used everywhere, and from 4 November Whisper's
  voice detection on accented data folders on Windows, the folders out of reach shown at launch,
  and Tesseract's drawing resolution in provenance and its engines pooled.
- Seen and left: `offline/src/models.rs` tests the system for a path that must be ASCII on
  Windows, a limit of the C library rather than a choice of build.
- Checked on Windows ARM64. `bundled.rs`'s side for other systems and the Whisper helper's call
  to it are first compiled for macOS by CI.
