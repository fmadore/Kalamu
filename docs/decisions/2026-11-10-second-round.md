# 2026-11-10 · The review's second round

## Context

The [review of 9 November](2026-11-09-review.md) fixed what lost work or money and left the rest
in issues #143 to #149. This round took up what could be built and tested where it ran (Linux:
the crates below the offline engines, and the interface in Chromium on its sample data). Four
branches were written in parallel, each in a worktree of its own (large jobs; media and the
engine's sources; keyboard and accessibility; the interface), and merged one after another, the
checks run again after each merge. `offline` and the shell were left alone because they cannot
be compiled there; what they would take up is listed under *Left*.

## Decisions

- **FFmpeg is watched for silence, not for time.** A decode of several hours is legitimate, so no
  run is cut by the clock. Every run reports its progress (`-progress pipe:2`, twice a second
  while it decodes) and is ended after two minutes with nothing on either pipe: a file on a
  share that stopped answering, a disk gone. ffprobe writes only at its end, so for it the two
  minutes are a limit on the whole run.
- **A pause during a recording's analysis plans nothing.** `AudioSource::try_pieces` returns
  `Stopped` instead of falling back to cuts of a fixed length, which once recorded would have
  been kept for good. The job ends paused with no piece recorded, and plans again when resumed.
- **A source's failure is kept as failed pieces.** A source never planned gets one failed piece
  per set-up carrying the reason, replaced by the real plan when a retry succeeds; a source that
  fails part-way keeps what was read and marks the rest. No new table and no exported type
  changed. A recording's stored plan is made of its segments only, so the placeholder is never
  taken for one.
- **TIFF pages up to 1 GiB decoded are read** (the `tiff` crate's default refuses anything over
  256 MiB, less than a 16-bit colour A3 page at 600 dpi). Palette pages, which the crate
  refuses, are read as indices from a copy whose photometric entry is patched to grey, then
  looked up in the colour map. CMYK is converted without a colour profile.
- **Exports read one source's rows.** `JobStore` gained `source_pieces`, `source_corrections`,
  `source_charges` and `source_speaker_names`, and `ProjectStore` `links_of`; the existing keys
  serve them. The files written are byte for byte what they were (the golden files are
  untouched).
- **Costs come from an index named in the query.** `results_costs (cache_key, cost_usd,
  truncated)` covers the job list and what a job spent; SQLite's planner prefers the table's
  own key, so the two queries say `INDEXED BY`, and a test checks their plans.
- **The result cache can be pruned, by the engine only for now.** `prune_results` deletes the
  results no piece refers to (a piece's `cache_key` is the only reference there is, and a piece
  records its key before its answer is stored) and gives back the space. An answer pruned is
  paid for again if asked for again, so it will be offered in Settings with that said, when no
  job runs.
- **Controls have an edge of their own.** `--control-line` (#8a8478 in light, #7a756c in dark;
  3.1 to 3.9 to 1 on every ground they sit on) for inputs, selects and bordered buttons; `--line`
  and `--line-strong` stay as quiet as they were for dividers, tables and sigla. The joined
  choice strips keep the quiet line: their radios say what they are.
- **A failure is not a confirmation.** A failure's message stays until dismissed and is read out
  at once (`role="alert"`); a confirmation fades and may carry one action (*Undo*). Both live
  regions are always mounted, since WebKit often says nothing of a region inserted with its
  text.
- **A text's differences are one tab stop**, with ← and → between them; opening one from the
  keyboard takes the focus into its readings and closing gives it back. Readings opened by
  pointing never take the focus. An analysis's names work the same way, their menu on Enter.
- **Page Up and Page Down turn pages except where they scroll**: in a field or an editor, and on
  a pane that scrolls and has the focus.
- **Back is a history**, the last thirty screens, behind the same guard as every other way of
  leaving a draft. A change within one place (a project's pane, a review's page) replaces the
  entry. In a review Alt ← and → still turn pages, so on Windows Back there is the button or the
  mouse; on a Mac, ⌘ [. A project now opens on the pane it was left on, which replaces the 9
  November rule that it opens on its documents.
- **The dark colours are written once**; the page always carries its theme, taken from the
  system until the settings are read.
- **A correction's files are written later.** Saving a correction or speakers' names stores
  it and marks the source's files under that set-up out of date (`outdated_files`, one row per
  job, set-up and source, deleted with its job) in the same transaction, and writes nothing.
  The engine writes them once no save has come for that job for three seconds, from one task
  of its own; before a job's files are listed, opened, exported or read for a summary; when the
  app exits, whichever way it was quit (for at most twenty seconds, so a share that stopped
  answering does not hold the exit); and at the next launch, off the window's thread, so a
  crash leaves nothing old for good. A mark is cleared only once every format is written, and
  only at the generation read before the files were: a save during the write keeps its mark.
  A write that fails (the scan moved, the folder gone, a folder the Mac App Store's build has
  no grant for yet) keeps its mark, is logged and reported as the job's `SourceFailed`, and is
  tried again at the next of these, never in a loop. The files are what the immediate write
  made, byte for byte; a job's first files at the end of its run are still written at once.

## Done

- **Media and engine**: `kalamu_media::program::run` (spawn, both pipes drained, a stop looked
  for every 100 ms, killed and reaped on a stop or a silence), used by every FFmpeg and ffprobe
  run; `MediaError::Stopped`; the stop signal passed from the job to opening, planning and
  cutting a recording. 1-, 2- and 4-bit grey (CCITT G4 included), palette and CMYK TIFF pages.
  Source failures kept. Tests with a program that is the test itself started again, so nothing
  differs between systems, and with real FFmpeg where there is one.
- **Large jobs**: the scoped reads above in exporting, combining a table of contents, freezing a
  text, scoring and items; the covering index; pruning. On a job of 2,000 one-page PDFs under
  two set-ups (`crates/engine/src/tests/large.rs`, ignored tests run on demand): every file of
  the job written in 3.6 s where it took 55.3 s; one source exported in 3.5 ms (16.3); the job
  list in 6.6 ms (27.8); a correction saved in 3.5 ms (19.7).
- **Keyboard and accessibility**: the decisions above; a zoomed scan moved with the arrow keys;
  scrolling panes reached by Tab where nothing inside them is; a ⋯ button on each job of the
  sidebar; menus that give the focus back; the result of testing a key read out; faint text no
  longer carrying anything that is read.
- **Interface**: Back; the Jobs page's filters and sort, a project's pane, its table's sort and
  tag and each scan's rotation remembered; Ctrl F on Jobs, Documents, Search, Dictionary and
  Glossary; a Keyboard section in the Glossary, written for each system; a dashed frame over the
  work area while files are dragged over a screen that takes them; *Undo* after removing a name
  from an analysis or documents from a collection; words on the marks of a project's Results
  column; the answer's fingerprint gone from *To check*.
- **Corrections written later**: `JobStore::outdated_files` and `clear_outdated`, the mark
  set by `put_correction` and `put_speaker_names`; `Engine::write_outdated` and
  `finish_writing`; the shell writes at exit and at launch, and lists a job's files off the
  window's thread since listing may now write. On the 300-page sample, a save takes 0.04 to
  0.06 ms where it took 84 ms on the same machine (121 ms where it was first timed), and the
  files are written once afterwards in 80 to 120 ms. The review's toasts say the files follow
  in a few seconds.
- **CI** (#145): the screen tests have a Chromium project, run on the Linux lane; the app's own
  binary is linked on Windows x64 and ARM64 as on macOS; `audit.yml` runs `cargo audit` and
  `npm audit` every week.

## Left

- **The shell** (#148, and from #143): nothing of it was touched. It gains a *Clear unused
  results* on `Engine::prune_results`.
- **The offline engines** (#144): `llama.rs::run_program` on `media`'s runner with a silence
  limit of its own; the graphics processor given up too easily; downloads; the FFmpeg build's
  decoders.
- **Also from #143**: a job's whole record is read for each export and each earlier source's
  name compared, about 1 ms and 2 ms on the 2,000-source job; the interface's side (the running
  job's counts, tables that draw every row, thumbnails).
- **From #146 and #149**: a recording's transcript shown twice; the Results marks from the
  keyboard; the looped commands.
- #147 (a model's abilities in the catalogue) and the rest of #145.

## To check

- Everything in WebKit and on Windows; with VoiceOver and Narrator, that the mounted regions are
  read.
- The shell's build with `MediaError::Stopped` and the new trait methods: no signature it calls
  changed, but CI is the first to compile it.
- The writes at exit and at launch on Windows and on a Mac, the Mac App Store's build among
  them: the shell's part (`RunEvent::Exit`, the launch thread, the job's files listed off the
  window's thread) is first compiled by CI.
- The sparse 1 GB recording the engine's pause test writes, on NTFS, where FFmpeg is on the
  runner's path (the test is skipped where it is not).
- A real drag of files over the window: the frame was drawn from a switch in the sample data.
- The controls' darker edge, by eye, on a real screen in both themes: one token to change.

## Consequences

- AGENTS.md is unchanged. The engine tests' large job uses `projects` as a development
  dependency, as `dictionary`, `projects` and `search` already use `store` in theirs.
- DESIGN.md records the controls' edge, the failure message, the keyboard rules, Back, what is
  remembered, the drop target and the toast's action.
- The privacy policy is unchanged: nothing new is sent, and what is newly remembered (filters,
  panes, rotations) stays in the webview's storage on the computer with the sidebar's fold. The
  job database's new marks say only which files are to be written again.
