# 2026-11-17 · PAGE XML, a sample drawn at random in What to check, Claude's batches deleted

## Context

After the [search, agreement and subtitles round](2026-11-17-search-agreement-subtitles.md),
three more things the roadmap listed as open needed neither a live run nor a Mac, and their
files did not overlap: PAGE XML beside ALTO, what was left of *What to check*
([#85](https://github.com/fmadore/Kalamu/issues/85)), and the deleting of Claude's batches,
with the one test [#168](https://github.com/fmadore/Kalamu/issues/168) still asked for. Each
was written by an agent in a worktree of its own, reviewed, and gathered on one branch where
every check was run again.

## Decision

### PAGE XML, and an exporter that writes several files

- **One file per page**, as the schema has it (a `PcGts` holds one `Page`) and as Transkribus,
  eScriptorium and Aletheia read it: `<name>.p0001.page.xml` and on, beside `<name>.alto.xml`.
  The extension is `page.xml`, never `.xml` alone, so the two never collide.
- **The `Exporter` trait gained `export_files`**, which by default returns the one file
  `export` writes, so no other exporter changed. Adding this format was therefore not local,
  which AGENTS.md asks to be weighed: the trait had assumed one file per document, and a
  format with one per page is a real case, not a fault of this format. The engine writes every
  file an exporter returns and lists a per-page format's files from the folder, in page order.
  The next per-page format is local again.
- **What ALTO and PAGE XML share is in one module** (`export/src/layout.rs`): which pages are
  written, their size in pixels, halves put back on the whole page, a block's lines, and XML
  escaping. ALTO's golden files did not change.
- **Dates from the results, not the clock.** `ExportPart` carries when a piece was read and
  when its correction was last saved, so `Created` and `LastChange` are those and writing the
  files again gives the same bytes.
- **No estimated words.** Where a model places only whole blocks, PAGE XML writes the lines
  ALTO estimates, marked as estimated, and no words. ALTO writes estimated words, as it did.
- **A PDF page names `file.pdf#page=N`** as its image, since no image file exists. Whether the
  tools that import PAGE XML accept that is not yet known.
- Nothing says whether a region is the model's or the researcher's: the stored regions do not
  say, and that waits for the second step of source-linked review
  ([#84](https://github.com/fmadore/Kalamu/issues/84)).
- The seven golden files were checked by hand against the official schema, with Python's lxml;
  no validator was added to the repository.

### What to check: a sample, items kept aside, a manifest

- **A sample of the pages nothing flags.** What the list flags was chosen for what is wrong
  with it, so an error rate found there says nothing of the corpus. A page is a candidate when
  every result read for it is read, current and flagged for nothing but not being reviewed.
- **Ranked by a hash, not shuffled.** Each candidate is ranked by the SHA-256 of
  `<seed>:<identity>` and the sample is the lowest: the same on every system and whatever the
  order of the rows; a larger sample holds the smaller one; a page flagged later leaves it and
  the next in rank takes its place, the others staying. No new dependency.
- **The seed is six digits at most**, short enough to write down, kept with the project for
  each scope (the project, a collection, a selection). Drawing again is an explicit act.
- **Kept aside is tied to the answer.** An item is kept with a note for the answer it then
  holds; a new answer never inherits it, and the item is listed again, marked. Keeping aside
  resolves nothing: only a review does.
- **Kept aside is not set aside.** The results a researcher sets aside from the list are still
  a choice of the session, not stored; the sample is drawn under them. Storing them is left.
- **The manifest is built in `projects`** as a table and written by the app through `export`'s
  one CSV writer, so `projects` still depends on `core` alone.
- One migration adds two tables, `review_samples` and `review_kept`; deleting a job or a
  project removes their rows.

### Claude's batches

- **The batch itself is what is deleted.** A Claude batch leaves no file, but Anthropic keeps
  it, with its requests and answers, for 29 days. `batch_files` names the batch by its own id
  once its `processing_status` is `ended`, and `delete_batch_file` sends
  `DELETE /v1/messages/batches/{id}`; a 404 counts as deleted. No new mechanism: the marking,
  the recording before deleting, the five tries and the leftovers of deleted jobs are those of
  [15 November](2026-11-15-records-regions-batch-files.md).
- **Only an ended batch.** Anthropic deletes none that is running or being cancelled, so such
  a batch names nothing and is asked about again at the next collection or launch.
- **Not those of earlier versions.** A Claude batch collected by 1.8.1 or earlier already has
  its record closed with nothing to delete, and stays at Anthropic until its 29 days end.
- The privacy policy says so, with a new effective date.

### The image description's test

One analysis job with the ready-made *Image description and keywords* now runs over a JPEG, a
PNG, a TIFF and a PDF against recorded answers, and the preset is refused on a model that does
not read images. The PDF has no text layer rather than embedded photographs: the engine sends
any PDF whole.

## Consequences

- A text extraction with Tesseract, Mistral OCR or LightOnOCR-3 can go on to Transkribus,
  eScriptorium or Aletheia. Importing the files there is still to try.
- A researcher can state how the unflagged pages read, from a sample anyone can draw again.
- After economy mode, nothing Kalamu sent stays at Anthropic longer than the job needs.
- Nothing here has been tried in the packaged app, nor on a Mac beyond what CI builds and
  tests. No live run was made: not of a Claude batch deleted, nor of the image description.
- The Store listings do not yet name PAGE XML; they change with the release that ships it.
