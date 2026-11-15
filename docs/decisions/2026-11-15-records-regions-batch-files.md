# 2026-11-15 · Records, regions over the scan, a batch's files deleted, and five leftovers

## Context

With the two checkpoints' refactors merged, the roadmap's *Next* held four things that could be
built side by side, their files hardly overlapping: what was "still to write" from the reviews
of 9 and 14 November, economy mode's files left in the researcher's storage at OpenAI and
Mistral (item 4), the first step of source-linked review
([#84](https://github.com/fmadore/Kalamu/issues/84), item 5) and the first increment of records
([#171](https://github.com/fmadore/Kalamu/issues/171), item 6). Each was written in a worktree
of its own and gathered on one branch.

## Decision

### Records: an analysis whose only field is a repeating group

- **Not a task of its own.** A records analysis is an analysis whose fields are one repeating
  group ("one row per entry"), which `FieldKind::Entries` already described for tables of
  contents. What makes it records is its fields (`core::records::is_records`), never a name.
- **The group stands alone.** An analysis holds at most one group and nothing beside it
  (`validate_analysis`): a page-level field (a heading, a year above its entries) is refused
  for now, and is where values inherited from a heading will be settled, with
  [#54](https://github.com/fmadore/Kalamu/issues/54). A group holds no group. Its columns are
  text, one choice, several choices or a list.
- **Page by page.** Such a job cuts PDFs, images and documents of ordered images into pages as
  an extraction does (`plan::reads_by_page`, from the fields frozen into the job); a text file
  is still read whole. Caching, resuming, economy mode and the marking of failed and cut-off
  pages are the page pieces' own, unchanged. A comparison that mixes a records analysis with
  one read once per document is refused when the job is made (`job.mixed_reading`): their
  results would not line up.
- **The correction layer.** A page's corrected rows are its corrected answers
  (`Correction.fields`), as every analysis's are; the model's stay as written. Each change is
  one `RowEdit` (a cell, a row added, a row removed) made in Rust (`core::records::edit`,
  `Engine::edit_rows`), which reads a cell as its column does. A cell edited leaves the page
  reviewed or not as it was.
- **Exports.** `.csv`: `file`, `page`, `position`, the columns as named (a `(reason)` column
  after those that ask for one), `corrected`, `reviewed`, `not_read`; a page not read is one
  row saying so, never no rows. `.json`: the usual envelope with a top-level `rows` list.
- **Two ready-made examples**, a directory and a register: literal readings, ditto marks kept
  as written, nothing carried down from a heading, an entry cut by the page's end given as the
  part on that page.
- ***What to check*** compares two set-ups' rows by their first column, as it compares a table
  of contents' entries.

### Regions over the scan

- **One geometry.** The review draws what the engines already stored (`TextRegion`): no second
  model. `TextRegion::estimated_lines` moved from the ALTO exporter to `core`, and
  `onto_whole_page` from the searchable PDF, so the review, ALTO and the searchable PDF place
  text the same way.
- **Coordinates.** A box is in fractions of the piece as the review shows it, which is the
  piece as the engines read it: a PDF page with its own rotation applied, one half of a split
  spread alone, an image turned upright by its EXIF orientation. They do not depend on the
  resolution a page was drawn at, nor on the zoom; the reader's quarter turns are laid over
  them.
- **A place in the text only where it is certain.** When the text shown is the regions' words
  in order (Tesseract, uncorrected), each region's place follows from its position. Otherwise
  (Markdown around the words, a correction) a region has a place only where its words stand
  exactly once in the model's text and once in the text shown; a repeated phrase or corrected
  words have none, and the review shows the region's own text and says so.
- **Approximate looks approximate.** Lines and words estimated within a block placed whole
  (Mistral OCR, LightOnOCR-3) are dashed; an engine's own are solid.
- **Granularity by zoom**: blocks, lines from 175%, words from 350%; beyond 600 regions only
  those near the view are drawn (a 4,800-word page: 858 ms to switch to words before, 162 ms
  after).
- `core` gains `unicode-normalization` (0.1.26, MIT or Apache-2.0), to compare accents typed
  either way.

### What a batch leaves at the provider

- **Through the trait.** `Provider::batch_files` names the files an ended batch left, read
  from the batch by its recorded id, never from a listing of the account's storage;
  `delete_batch_file` deletes one, a file already gone counting as deleted. The defaults name
  none.
- **Only after the answers are stored.** The batch's record is marked for cleaning in the same
  write that marks it collected or given up on, and the ids are recorded before any is
  deleted: a crash in between is finished at the next collection or launch. A batch still
  sending or waiting is never cleaned.
- **A deleted job.** Its batches still to clean are kept in a table of their own
  (`batch_leftovers`, migration 20) until the deleting ends.
- **Failures** never touch a job: logged, tried again at later collections and launches, five
  times, then left and recorded as left.
- **OpenAI**: the uploaded requests, the answers and the errors (30 days otherwise); an upload
  no batch was made from is deleted at once. **Mistral**: the files the job names (kept with
  no stated end otherwise). **Claude** and **Gemini** leave no file; their batches are not
  deleted, though Claude's API allows it.

### Five leftovers

- **Why an answer stopped.** `CutReason` (limit, filter, other) travels beside `truncated`;
  stored results and cache keys are unchanged, a result without one reading as the limit. The
  review, the queue and the error say which; `provenance.json` lists `cut_by_filter`. Claude's
  `model_context_window_exceeded` and Mistral's `model_length` and `error` are now kept and
  marked as cut off.
- **A launch that fails before setup.** The window is made by Kalamu's setup
  (`"create": false`) rather than by Tauri before it, so a webview that cannot be made is told
  in the start-up message, naming the logs folder and what to do on each system; a failure of
  Tauri's own start goes through the same message.
- **Escape** from a language picker's list leaves it closed; a name's menu takes **Home and
  End**; an **empty reading** taken into the text takes one of its spaces with it.

## Consequences

- To try by hand: the launch on Windows and a Mac now that setup makes the window, and with
  WebView2 missing (`WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` set to a folder that does not exist);
  the regions on real Tesseract and Mistral OCR pages, on a PDF whose CropBox differs from its
  MediaBox, with a trackpad and with a screen reader; a records job on a real directory and
  register, and in economy mode, which has no test of its own.
- A live run must show whether Mistral names its inline requests' file among the job's files;
  if it does not, Kalamu cannot find it without listing the storage, which it will not do.
- A failure of Tauri's start that is not the window's is still told as the window's.
- A records job's text formats (`.txt`, `.md`, `.docx`) show only each page's first column,
  joined with semicolons: a table there, or those formats not offered, is left to do.
- Left of #171: entries that run over a page, a region per row, rows across a project, error
  rates by row, a cut-off page read again in halves. Left of #84: stable region identities,
  regions drawn by the researcher, a region read again, the analysis and contents reviews.
- Building in one shared `target` folder from several worktrees mixes their crates (the build
  folders' names do not depend on the worktree): each worktree builds in its own, deleted when
  it is done.
