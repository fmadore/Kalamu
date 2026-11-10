# 2026-11-10 · The review's fourth round

## Context

The [review of 9 November](2026-11-09-review.md) left its findings in issues #143 to #149. The
[second round](2026-11-10-second-round.md), the [shell](2026-11-10-shell.md), the
[offline engines](2026-11-10-offline-robustness.md) and
[a model's abilities](2026-11-10-model-abilities.md) took up most of them, all written on Linux,
where the shell and the offline engines do not build. This round took up what those left, and
ran on Windows (ARM64), where the whole workspace builds: six branches written in parallel, each
in a worktree of its own, merged one after another into one branch. After the last merge the
workspace's crates were built again from nothing and every check run on the result (see *What
was run*).

## Decisions

### The tray, the taskbar and the notification follow the engine (#148)

They were worked out in the webview from its job list and pushed back to the shell, and a
webview's timers are slowed exactly when its window is hidden in the notification area or the
menu bar. `src-tauri/src/activity.rs` now subscribes to the engine's events before jobs are
resumed, on a task of its own: a burst of progress is gathered for half a second, then the job
list is read once and the tray's status line, *Pause all*, the Quit label, the tooltip and the
taskbar's or Dock's progress are written; a finished job writes them at once and sends the
notification.

- **A job counts as running** when its stored status says so or the engine is running it, the
  rule the engine's own summaries use.
- **The interface sends only the words**, once at start-up and when the language changes
  (`set_shell_labels`), as templates whose placeholders (`{jobs}`, `{percent}`, `{status}`,
  `{job}`, `{state}`) the shell fills in one pass, so a job named `{state}` is not filled in
  twice. `set_tray_state` and `set_taskbar_progress` are gone. A notification due before the
  words arrive waits for them.
- **No notification while the window is in front, or while the app quits**: the second is new,
  and spares one "Paused" notification per job when Quit pauses them.
- **A workflow step that is merely paused reads "Paused"**; before, any waiting step read "some
  pages were not read".
- The taskbar's progress is written again when the window comes back from the tray.
- Nothing was added to `platform/`: Tauri's progress call and the notification plugin already
  cover both systems.

### The programs the app starts end with it (#144)

FFmpeg and ffprobe, llama.cpp and the Whisper helper are started through
`kalamu_media::program::spawn` and reported once reaped. What is done with each is a `Keeper`
(two functions) the shell gives once at start-up, so nothing below the shell asks which system
it runs on.

- **Windows: each program is put in one Job Object** with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`,
  made with the first of them. The app holds its only handle, which Windows closes however the
  process ends: a second Quit, a crash, Task Manager.
- **The app's own process is not put in the job**, though every child would then inherit it and
  nothing below the shell would change: the opener starts the browser, File Explorer and Notepad
  from inside the app's process (`ShellExecuteExW`), which cannot ask to break away from a job,
  so they would end with the app; and WebView2's processes are better left to WebView2.
- **macOS: the programs are kept by process id** and ended on Tauri's exit event (Quit, a second
  Quit, the window closed while idle, logging out); an id is forgotten once its program is
  reaped. Not covered: a crash, Force Quit or `kill`, after which a program runs until it next
  writes to the app's closed pipes. macOS has nothing like a Job Object; covering it would take
  a watchdog process, left out.
- `windows-sys` 0.61.2 and `libc` 0.2.191 join the shell's dependencies, each on its own system;
  both were already in the tree. The shell's `unsafe` is confined to these calls.

### A stopped download says what it kept (#144)

`OfflineModel.downloaded_bytes` now also carries, outside a download, what the `.part` files
hold. Settings shows *Stopped at N%*, that what was downloaded is kept and the download will
continue from there, *Continue download*, and *Discard*, which removes the parts.

### FFmpeg's decoders (#144)

Both builds' lists in `ffmpeg.yml` gain `pcm_s24be`, `pcm_s32be`, `pcm_s8`, `pcm_f32be`,
`wmavoice` and `adpcm_ima_qt`, and each build's check decodes a 24-bit and a 32-bit AIFF written
by hand (the build has no AIFF muxer; ffprobe names a codec it cannot decode, so only decoding
proves it). The pinned build is unchanged: the workflow is to be run and the new archives
pinned in `scripts/ffmpeg.json`.

### The release pipeline (#145)

- **No signing secret and no write token beside build code.** `release.yml`'s token is
  `contents: read`. The builds compile unsigned in read-only jobs; `notarise` is the only job
  with the Apple secrets and compiles nothing (`npm ci --ignore-scripts`, then `tauri bundle` on
  the build handed over); only `prepare`, `upload` and `publish` can write, and they run `gh`,
  `zip` and `jq`. `tauri-action` is no longer used.
- **Assets are attached once, at the end**, with `SHA256SUMS` and a build attestation
  (`actions/attest-build-provenance`, in a job with only `id-token` and `attestations`).
  [checking-downloads.md](../checking-downloads.md) says how to check both, and is added to each
  release's notes.
- **One Windows build per architecture** makes the MSIX and the installer
  (`.github/actions/windows-packages`), where two jobs each compiled from nothing. The MSIX is
  made first, since the NSIS bundler marks the executable.
- **A packaging dry run a pull request can fail** (`packaging.yml`): the same action in a debug
  build, unsigned, nothing uploaded, x64 on pull requests that touch the packaging and both
  architectures by hand.
- **Every action is pinned by commit**, GitHub's own included, with its version in a comment;
  the tag is read from `env`, never written into a script.
- **Interaction tests** (`e2e/interactions.spec.ts`, in Chromium and WebKit): a correction saved
  and found again, a dialog that keeps the focus until Escape, two shortcuts, a project search,
  New job kept across Settings; and every screen opened in French. They found one bug: *Correct*
  left the focus on the page when the reading place was unknown, the default; it now goes to the
  editor.
- `playwright.config.ts` reads `KALAMU_E2E_PORT`, so two checkouts no longer answer each other's
  tests.

### Long tables are drawn a page at a time (#143)

- **Paging, not a virtual list.** Jobs, a project's documents, the dictionary and a job's files
  draw 200 rows and *Show 200 more* (`paging.svelte.ts`, `MoreRows.svelte`, after the review
  queue's pattern). Rows vary in height, and paging leaves the Tab order and Ctrl F as the
  browser makes them. Filtering, sorting and *Select all* act on every row, drawn or not; a
  screen reader is told the whole count (`aria-rowcount`).
- **A running job's page reads counts, not pieces**: `job_progress` gives, for the files drawn,
  how many pieces are done and failed and the first reasons, counted in the engine
  (`progress.rs`). A command rather than an event, since it stays right under paging.
- **Thumbnails are drawn by the app**: `piece_thumbnail` gives a small PNG of a PDF page (hayro)
  or a small JPEG of an image, where each thumbnail fetched a whole page and started a pdf.js
  worker. The previews share one worker.
- **The Results marks are a button** whose name is the whole sentence, shown under the marks
  when the keyboard reaches it (#146).

On the sample data with 3,000 rows, in Chromium, on a machine busy with the other branches'
builds (the times are rough, the counts exact): all jobs, 94,157 elements drawn before and 6,601
after; a project's documents, 123,580 and 8,269; the dictionary opened in 39 s and now 0.6 s;
a job's page with 3,001 files never settled within five minutes and is now drawn in 2 to 8 s.
On the 2,000-source job of `large.rs`, the page's read went from 1.15 MB of pieces to 58 KB of
counts.

### A recording's transcript is shown once (#149)

The right pane stays the transcript: the one text that is read, corrected and exported. The
player's pane no longer repeats its words: under the player it lists the turns (time, speaker,
opening words on one line), each a place to listen from. The passage being heard is found in the
text by its words (`heard.ts`: a line's first words, or its last when a correction changed the
start) and given a faint wash, under the search's marks; while the recording plays the text
keeps it in view, and waits four seconds after the reader scrolls. Nothing is marked while
editing. Found by words rather than by position because a correction is free text and the
model's times no longer match it. No stored data and no export changed.

### One command for a rename, a link or a deletion of several (#149)

Renaming or removing a name across set-ups (`rename_listed`, with `save_corrections` for
*Undo*), linking several jobs to a project (`link_jobs`) and deleting several jobs
(`delete_jobs`) were looped from the interface and stopped half-way on a failure. Each is now
one command and one transaction: `JobStore::put_corrections` and `delete_jobs`,
`ProjectStore::attach_documents`. The renaming itself moved from the interface to
`core::list_items`.

### Economy batches capped by the engine, their body written once (#143)

- **A batch holds at most 48 MiB of prepared input** (`EngineConfig::max_batch_bytes`), or the
  provider's own declared limit when that is smaller; a piece larger than that goes alone, as
  before. Each batch is an ordinary record, so resume, cost and provenance are unchanged.
- **The body is written once** (`providers::batch::Body`): each item straight into one buffer of
  the size estimated, handed to the HTTP client as it is. Before, each provider held the items,
  a copy of them and the serialised text at once. Measured with a counting allocator on 32 MiB
  of pages: 213 MiB above the input before and 46 after for Anthropic, Gemini and Mistral; 132
  and 45 for OpenAI's file. Not streamed: the request borrows the engine's inputs.
- **An upload has five minutes plus the time its size takes at 1 Mbit/s**, where it had the
  fifteen minutes of any request.

### CSV cells a spreadsheet would compute (roadmap)

A cell that begins with `=`, `+`, `-`, `@`, a tab or a carriage return is written with an
apostrophe before it, unless the whole cell is a number (`-12`, `+3.5`, `-1,5`). An apostrophe
rather than a tab: it is seen, so a researcher who copies the cell knows what to take off, and a
tab is itself among the characters to guard against. The table says whether it is guarded
(`Table.cells`), and the crate that makes it decides:

| Table | Cells | Why |
| --- | --- | --- |
| A job's CSV, every task | guarded | Text a model read off a document, for a spreadsheet; the JSON keeps it exactly |
| A table of contents' units | guarded | The same; its `.json` twin is the list for programs |
| A project's descriptions | guarded | For a spreadsheet, beside an exact JSON; never read back |
| The dictionary's forms | guarded | Forms from answers; the entries' JSON is the exact form |
| An export profile's table | as written | Made for another tool's import, which would take the apostrophe as part of the value |

No golden file changed: none had such a cell. The cost is real for transcriptions: a line of
dialogue written `- Oui` now reads `'- Oui` in the CSV (and only there).

## What was run

On the merged branch, on Windows ARM64, after `cargo clean` of every workspace crate (the six
worktrees had shared one target folder, and Cargo had given some of them another's outputs, so
the branches' own Rust results were not relied on):

- `cargo fmt --all --check`; `cargo clippy --workspace --exclude whisper-vulkan --all-targets
  --locked -- -D warnings`; `cargo test --workspace --exclude whisper-vulkan --locked`: all
  passed. `whisper-vulkan` needs the Vulkan SDK, which this computer lacks; nothing in it
  changed.
- `npm run lint`, `npm run check`, `npm test` (218 tests), `prettier --check .`: passed.
- Playwright in Chromium and WebKit: 77 passed, 4 skipped (the Mac-only ones) and one failed in
  the first full run, the correction's round trip in WebKit, waiting five seconds for the page
  while every worker started at once; alone it passed three times in three. CI retries once.

## Left

- **#143**: `job_progress` still reads every piece of the job in the store before counting (a
  `GROUP BY` there is the next step); a way to reach the failed files of a job of thousands,
  which now may sit behind *Show more*; `job_pieces`, which the interface no longer calls.
- **#144**: the new FFmpeg build run, published and pinned; on a Mac, the programs after a crash
  or a Force Quit.
- **#145**: nothing to write; everything under *To check*.
- Possible settings, not built: the batch cap; guarding cells per export profile.

## To check

- **The release workflow, which has never run.** In this order: the packaging dry run on the
  pull request, then by hand for both architectures; after merging, a validation tag
  (`v1.6.0-rc1`): seven assets and `SHA256SUMS` on the draft, the attestation in the run's
  summary, `gh attestation verify` and `sha256sum -c` on a download, the disk image through
  Gatekeeper. What only that proves: `tauri bundle` signing and notarising a build from another
  job, the NSIS bundler taking the build as it is and naming the installer as before, the
  attestation's permissions. A tag cut before this change cannot be built again by the new
  workflow.
- **macOS**, first compiled by CI: `platform/macos.rs` (`keep_children`, `end_children`, the
  exit event) and the `libc` dependency; the Dock's progress and the menu-bar tray through
  `activity.rs`.
- **In the app itself**, which was not opened: the tray, the taskbar and the notification with
  the window hidden for a whole job; whether Windows does lose a hidden window's taskbar
  progress; FFmpeg or llama.cpp gone after the app is ended from Task Manager, in the Microsoft
  Store's package too (a job inside the package's job); `job_progress` and `piece_thumbnail`
  over real IPC; the one pdf.js worker across many pages; a real recording followed in the text,
  Gemini's timed text among them; Settings with a download stopped part-way.
- **A real spreadsheet**: how Excel, LibreOffice and Numbers show the apostrophe.
- **A live economy batch**, split by the cap, on a slow connection.
- With VoiceOver and Narrator: the tables' row counts and the Results marks.

## Consequences

- AGENTS.md is unchanged: no crate gained an internal dependency, and
  `crates/core/tests/architecture.rs` passes. `projects` uses `csv` in its tests.
- [DESIGN.md](../../DESIGN.md) records paging, the Results marks, the job page's counts, the
  thumbnails, and a recording's one transcript with its *heard* wash.
- [formats.md](../formats.md) says how risky CSV cells are written; the specification's passage
  on recordings follows the review.
- [development.md](../development.md) and [packaging-msix.md](../packaging-msix.md) describe the
  release's new jobs and the dry run.
- The privacy policy is unchanged: nothing new is sent, to anyone, and nothing new is kept
  (download parts were already kept).
