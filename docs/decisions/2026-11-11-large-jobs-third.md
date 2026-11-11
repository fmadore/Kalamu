# 2026-11-11 · Large jobs, the third round

## Context

Issue [#143](https://github.com/fmadore/Kalamu/issues/143) asks that a folder job of several
thousand pages, and a data folder grown over months, stay as quick to run, export, review and
list as a small job. The [second round](2026-11-10-second-round.md) scoped the exports' reads,
indexed the costs, wrote corrections later and made the result cache prunable; the
[shell round](2026-11-10-shell.md) put *Clear unused results* in Settings; the
[fourth round](2026-11-10-fourth-round.md) paged the long tables, counted a running job's pieces
in the engine, drew thumbnails in Rust and capped economy batches. Its *Left* named three things:
the counts still read every piece of the job, a large job's failed files could sit behind *Show
200 more*, and `job_pieces` was no longer called. This round took those up, then checked every
point of the issue against the code.

## Decisions

- **The store counts a job's progress.** `JobStore::job_progress(job, sources)` gives what the
  job's page shows: the pieces held in batches and the files that failed, over the whole job, and
  each file asked for counted under each set-up (planned, done, failed, the first three different
  reasons in the pieces' order). It is three grouped queries (`crates/store/src/progress.rs`);
  the engine's `job_progress` only passes it on. `FileProgress` and `JobProgress` moved from
  `engine` to `core`, where the trait is, under the same names, so their bindings are unchanged
  but for the new field. The files asked for go as one JSON list (`json_each`), so a page of any
  length is one query and no parameter limit is reached.
- **One index, no column.** `pieces_progress (job_id, source_id, setup_id, status)`, added by a
  new migration (`CREATE INDEX IF NOT EXISTS`, so an existing data folder gains it at the next
  launch and loses nothing). It covers the counts, the pieces held and the failed files: a test
  checks each plan reads `COVERING INDEX pieces_progress`. Only the reasons are read from the
  rows, for the files asked for. An older Kalamu refuses a database a newer one has migrated, as
  it already does for every migration.
- **A large job's failed files are reached by a filter.** `JobProgress.failed_sources` lists every
  file with a failed piece, asked for or not, and the job's page offers *All files (N)* or *Files
  with failures (M)* beside its heading once there is one. The filter acts on every file, drawn
  or not, keeps the job's order and draws from the first page; a status line, always mounted and
  hidden from sight, tells a screen reader how many are listed. Chosen over "failed first",
  which would reorder the files under the researcher while a job runs, and over a count alone,
  which would not reach the files. The list grows only with failures; a job where thousands of
  files fail sends their ids with each reading, which is still far less than its pieces.
- **The page reads again only when the files drawn change.** Each reading brings a new list of
  failed files and so a new filtered list; the files drawn are compared as one string, so the
  same files drawn are not read again for it (otherwise filtering would read without end).
- **`job_pieces` is removed**: the command, its registration, its `allow-job-pieces` grant and
  `api.jobPieces`, and `Engine::pieces`, which only it called. Nothing in `src/` called it since
  the fourth round. `PieceState` keeps its binding: `core` still exports it and other types use
  it.

## The issue, point by point

| Point of #143 | State | Where |
| --- | --- | --- |
| Exports quadratic | Done but one part | `output.rs:26`, `document.rs:41,48,282`, `contents.rs:32` read one source's rows (second round). Left: `output_base` (`output.rs:289`) compares the source's name with every earlier source's, about 2 ms a source at 2,000 |
| Every export scans every project's links | Done | `ProjectStore::links_of` (`core/src/project.rs:305`, `store/src/projects.rs:455`), used by `SourceDocuments::documents` (`projects/src/described.rs:63`) |
| Each saved correction rewrites every output | Done | `save_correction` stores and marks (`engine/src/lib.rs`, `put_correction` then `write_later`); the files are written once saves rest for three seconds (second round) |
| `list_jobs` and `spent` read through payloads | Done | `INDEXED BY results_costs` in `LIST_JOBS` and `SPENT` (`store/src/jobs.rs:70,78`), the index in `migrations.rs:113`, its plan tested. `list_jobs` still parses each job's record for its models and sources: it grows with a job's sources, not with stored results |
| The result cache never pruned | Done | `prune_results` (`store/src/jobs.rs`), *Clear unused results* in Settings (`src-tauri/src/commands/settings.rs:53`, `SettingsView.svelte:110`) |
| A running job's page recounts every piece | Done | Fourth round: counts for the files drawn; this round: counted by the store from an index |
| Tables render every row | Done | `Paging` in `AllJobs`, `DocumentTable`, `DictionaryView`, `JobDetail` (fourth round) |
| A pdf.js worker per preview, a whole page per thumbnail | Done | One worker in `src/lib/pdf.ts`; `piece_thumbnail` (`src-tauri/src/commands/review.rs:88`) |
| Economy batches held several times over | Done | `max_batch_bytes` (`engine/src/batch/send.rs:97`), `providers::batch::Body`, `upload_timeout` (`providers/src/http.rs:39`) |

| Acceptance | State |
| --- | --- |
| A benchmark job, numbers before and after | Partly: `large.rs` is 2,000 one-page sources under two set-ups, not the 5,000 the issue names; its numbers are in the second and fourth rounds and below. Not ticked |
| Exporting one finished source reads only its rows | Done (second round), but for the name comparison above |
| A correction on page 200 of 300 does not rewrite the PDF each save | Done (second round; `long_document_correction` in `large.rs`) |
| The job list's time does not grow with the bytes of results | Done (second round): the covering index, its plan tested |
| Golden files byte for byte | Held: no exporter or golden file changed in this round |

## What was measured

`cargo test -p kalamu-engine --release large_job -- --ignored --nocapture`, once, on Windows
ARM64, the computer busy with two other agents' builds (rough times, exact sizes):

| Step | Time |
| --- | --- |
| The job's page: every piece read, as the engine's count did before this round | 8.76 ms (1,153,781 bytes of pieces as JSON) |
| The job's page: the store's counts of 200 files | 1.36 ms (58,040 bytes sent) |
| `list_jobs` | 13.68 ms |
| `spent` | 2.28 ms |
| One source exported | 8.37 ms (`export_document` alone 0.23 ms) |
| Every file of the job written (`export_finished` over 2,000 sources) | 62,314 ms |
| `save_correction` | 2.28 ms |
| `prune_results` after deleting the job | 315 ms, 4,000 results, 45,211,648 bytes |

The "before" for the page is the read the old count began with, timed in the same run; its
counting in memory came on top. The other rows are not comparable with the second round's
(Linux, 3.6 s for every file of the job): this was Windows, with the disk and processor shared,
and writing 4,000 files through `write_atomic` there is the bulk of the 62 s. They are recorded
as found, not as a regression or a gain of this round, which changed no export path.

## Left

- **`output_base`** compares each source's name with every earlier source's to number files of
  the same name, so writing a job's files is still quadratic in its sources, if cheaply. Fixing
  it means working out every source's name once per job and passing it down every write path
  (export, rewriting provenance, listing a job's files), a change across `output.rs` and its
  callers; not done here.
- **A benchmark of 5,000 sources**, as the issue asks: `large.rs`'s count is one constant.
- **#143's box for the benchmark** stays open until then.

## Consequences

- The job database gains one index (`pieces_progress`); no table or column changed. Writing a
  piece now updates that index as well.
- `JobProgress` gains `failed_sources`; the `job_progress` command keeps its arguments.
- The privacy policy is unchanged: nothing new is sent or kept.
- AGENTS.md is unchanged: no crate gained an internal dependency.
