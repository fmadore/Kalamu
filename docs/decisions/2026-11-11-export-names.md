# 2026-11-11 · Export names, and what writing a large job's files costs

## Context

Issue [#143](https://github.com/fmadore/Kalamu/issues/143) asks that a folder job of several
thousand pages export as quickly, file for file, as a small one. The
[third round](2026-11-11-large-jobs-third.md) left two things: `output_base` compared each
source's output name with every earlier source's on every write, so writing a job's files grew
with the square of its sources; and the benchmark had 2,000 sources, not the issue's 5,000.

On a quiet Windows ARM64 machine, before this round, `large_job` (2,000 sources × 2 set-ups)
gave 27.3 s for `export_finished` over the job and 9.28 ms for exporting one source. At 9.28 ms a
source, 2,000 sources make 18.6 s, so about 8.7 s was unexplained. The first guess was the name
comparison. This round measured before changing anything.

## The split measured

`large.rs` now times each step of writing a source's files, for one source in ten. It repeats
`write_files` step by step in the test, with no timing in product code. It also times the names
alone over the whole job, a second `export_finished` over files that already exist, `outputs`,
and what one file costs on this disk. Each source writes **four files under each set-up, eight
in all**: `.txt` (2.3 KB), `.json` (8.5 KB), `.searchable.pdf` (22 KB) and `.provenance.json`
(5.5 KB). Each file is written to `name.ext.pending` and renamed. Nothing is flushed with
`sync_all`, no folder is listed, and nothing is probed on disk before writing. `create_dir_all`
ran once per write, for every set-up.

Per source (two set-ups), before the change, 5,000 sources, **busy machine**:

| Step | ms a source | Share |
| --- | --- | --- |
| Writing the four `.pending` files ×2 | 3.0 | 19 % |
| Renaming them into place ×2 | 6.8 | 44 % |
| Making the folder (`create_dir_all`) ×2 | 0.4 | 3 % |
| Names (`output_base`), at this size | 1.5 | 10 % |
| Reading (`export_document`) ×2 | 2.0 | 13 % |
| Rendering: searchable PDF / JSON / provenance / text | 1.21 / 0.09 / 0.03 / 0.01 | 8 % |
| Store: the source's pieces, the job's out-of-date marks | 0.46 | 3 % |
| All | 15.5 | |

At 2,000 sources the shares were the same but for the names: 0.75 ms a source, 5 %. Computed
alone over the whole job, the names took **1,326 ms at 2,000 sources and 7,706 ms at 5,000**:
5.8 times as long for 2.5 times the sources, the square as expected. On a quiet machine they
explain part of the 8.7 s gap above, not all of it. On a busy one, `export_finished` over the job
ran at twice the split's time per source before the change, but at less than it after; and
writing every file again took longer than writing them new. So the rest of the gap is mostly
the machine's load, not something in the code that grows with the job. Nothing else in the write
path depends on the number of sources.

One file on this disk (in `%TEMP%`, scanned by the antivirus), 500 times each, ms a file:

| | Busy, before | Busy, after |
| --- | --- | --- |
| 1 KB, new, written straight | 0.26–0.41 | 0.29–0.77 |
| 1 KB, new, temporary file renamed | 0.55–0.72 | 0.62–0.93 |
| 1 KB, replaced, temporary file renamed | 0.64–0.68 | 0.49–0.59 |
| 100 KB, new, written straight | 0.36–0.52 | 0.42 |
| 100 KB, new, temporary file renamed | 0.95–1.69 | 1.07–1.15 |
| `is_file` on a file that exists | 0.04–0.05 | 0.17–0.35 |

What a file costs barely depends on its size. It is the creating, closing and renaming:
NTFS metadata, and the antivirus, which scans every file as it is closed. That second cause is
inferred, not measured: Kalamu cannot turn the scanner off to compare. The second round
measured the same job's writes at 3.6 s on Linux.

## Decisions

- **Names are worked out once.** `OutputNames` (`engine/src/output.rs`) goes over the job's
  sources once, in order, and counts each folder and name as it goes, ignoring case. A source
  gets the number its earlier namesakes give it, exactly as the old pairwise comparison
  numbered it. A run makes it once and passes it to every `export_finished`, and so does
  collecting economy batches. `write_outdated_of` makes it once per job whose marks it writes.
  `outputs` and `text_outputs` make it once per listing. One-source paths (`export`,
  `write_outputs`, `rewrite_provenance`) go only as far as the source itself
  (`OutputNames::of_source`). The task's ending (`.analysis`, `.translated.<code>`…) and the
  set-up's label are added per write, as before.
- **The names are fixed by a test written before the change** (`tests/names.rs`). It passed on
  the old code and passes on the new. `output_names_are_fixed` covers sources written beside
  themselves and into one folder: `Scan.pdf`, `scan.pdf` from another folder, `Scan.tif` in the
  same folder, `SCAN.PDF`, a stem with dots, an item cut from a document (#50) written beside
  its original, and two videos whose titles differ only in case. It also covers a translated
  comparison (`Scan (3).pro.translated.zh-hans`) and each task's ending. It checks every name
  worked out for the whole job against the same source worked out alone.
  `a_job_resumed_half_way_names_its_files_as_one_run_through` runs a job of four colliding
  sources once; then runs another job of the same sources, deletes its files, marks its last two
  sources' pages in flight as a crash would, recovers and resumes it. Both jobs list the same
  twelve files. The names depend only on the job's stored order of sources, so a resumed job
  cannot name its files differently.
- **The folder is made when it is missing, not asked after for each file.** `write_atomic`
  makes the folder and tries once more when writing the `.pending` file finds it missing. This
  saves `create_dir_all` (two system calls, 0.2 ms) on every write. A folder deleted
  during a job is made again, as before.
- **Kept as it is, on purpose:**
  - **The temporary file and the rename.** They cost 44 % of the time, and they are what keeps
    a file either whole or absent after a crash.
  - **Writing each source as it finishes.** Outputs appear while the job runs.
  - **`provenance.json`** is per source and does not grow with the job (5.5 KB at either size).
    Nothing in the write path rewrites a job-wide file.
  - **A resumed run writes every finished source again.** Skipping files that hold the same
    bytes would not help, because `provenance.json` and the `.json` carry the time they were
    written; and a job's first run, the one that matters, has no files yet.
  - **The store's schema** is unchanged.

## Before and after

`cargo test -p kalamu-engine --release large_job -- --ignored --nocapture`, Windows ARM64.
**Every number below was taken on a busy machine** (three other agents building), so it is
rough. The quiet numbers from before this round are in *Context*.

| | 2,000 before | 2,000 after | 5,000 before | 5,000 after |
| --- | --- | --- | --- | --- |
| `export_finished` over the job | 59,883 ms | 30,197 ms | 158,272 ms | 54,886 ms |
| … a source | 29.9 ms | 15.1 ms | 31.7 ms | 11.0 ms |
| … of which the names, over the job | 1,326 ms | 1.7 ms | 7,706 ms | 7.2 ms |
| `export_finished` again, every file replaced | — | 32,749 ms | — | 70,868 ms |
| A source's files, split, all steps | 15.8 ms | 15.9 ms | 15.5 ms | 13.9 ms |
| Export of one source (`export`) | 10.18 ms | 10.04 ms | 14.71 ms | 13.91 ms |
| … of which reading the job's record | 3.28 ms | 3.46 ms | 8.80 ms | 7.78 ms |
| `export_document` of one source | 0.23 ms | 0.27 ms | 0.62 ms | 0.25 ms |
| `outputs` (the job's files listed) | — | 1,030 ms (16,000 files) | — | 2,137 ms (40,000 files) |
| `list_jobs` | 19.2 ms | 17.4 ms | 74.2 ms | 65.4 ms |
| `spent` | 3.41 ms | 10.32 ms | 14.43 ms | 19.19 ms |
| Job page, counts of 200 files | 2.85 ms | 2.31 ms | 2.92 ms | 3.03 ms |
| `save_correction` | 0.45 ms | 0.12 ms | 0.60 ms | 0.22 ms |

The names went from growing with the square of the sources to under 8 ms for 5,000. They are
no longer to be seen in the split (0.004 ms a source). After the change, the time a source takes
no longer grows with the job: 15 ms a source at 2,000 and 11 at 5,000, both within the noise of a
shared machine. The gap between before and after is larger than the names and the folder alone
explain (7.7 s + 2 s at 5,000), so most of it is the machine's load moving between runs. To be
rerun alone, on a quiet machine, from the repository's root in Git Bash:

```bash
cargo test -p kalamu-engine --release large_job -- --ignored --nocapture
KALAMU_LARGE_SOURCES=2000 cargo test -p kalamu-engine --release large_job -- --ignored --nocapture
```

## Left

- **The remaining cost is Windows' own.** About 85 % of a source's time is making and renaming
  eight files. A file costs about the same at 1 KB or 100 KB, and the rename alone costs as much
  as writing the file. The probe and the split are the evidence; the Linux figure is the
  comparison. Kalamu
  does not turn the antivirus off, nor write a folder of files into one container to escape it.
  The only lever left would be fewer files: fewer formats, chosen by the researcher.
- **`outputs` asks after every possible file one by one** (up to three names per format, so
  40,000 files make tens of thousands of `is_file` calls): about 2 s at 5,000 sources. Listing
  each folder once would be quicker, but would have to match names as each system does (Windows
  ignores case, macOS by default too, but not always). Not done.
- **`list_jobs` and reading one job's record grow with its sources** (65 ms and 7.8 ms at
  5,000): each job's record is parsed whole for its models and sources. This was already named
  in the third round.
- **`export_finished` reads a source's pieces twice** (once to see if it is settled, once in
  `export_document`), and reads the job's out-of-date marks on every write: 0.2 ms a write, 3 %.
  Not worth the change in signatures it would need.

## Consequences

- `output_base` is gone. `OutputNames::of(job)` / `of_source(job, source)` and `.base(job,
  source, label)` take its place. `export_finished` and `write_each_output` take the names as a
  parameter.
- No exporter, golden file, store table or binding changed. The names of the files a job
  writes are the same, byte for byte.
- The privacy policy is unchanged: nothing new is sent or kept.
- AGENTS.md is unchanged: no crate gained an internal dependency.
- The benchmark has 5,000 sources. `KALAMU_LARGE_SOURCES=2000` gives the earlier size.
