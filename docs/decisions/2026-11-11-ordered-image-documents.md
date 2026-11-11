# 2026-11-11 · Images read as the pages of one document

## Context

Archives are photographed page by page: a letter of four sheets is four JPEGs, a register a
folder of hundreds. Kalamu read every image as a document of its own, so a letter came back as
four results, and a table of contents could not follow an article over two photographs. The
only way to read them as one was to make a PDF first, outside the app
([#52](https://github.com/fmadore/Kalamu/issues/52)). This is the first of the issue's two
increments; pairing existing transcriptions with their scans is the second.

The reference projects read a folder of images as separate files in plain `sorted()` order
(`page10` before `page2`) and never as one document, so there was nothing to port: the natural
order and the document are Kalamu's.

The design question was the cache. A result is keyed by the source's fingerprint and the piece
(`CacheKey::compute(task, source_sha256, piece, …)`), and a PDF's page is `PieceRef::Page {
number }`, its place in the file. A document of images keyed the same way, by a hash of its
pages and their places, would read every page again when one page moved.

## Decisions

- **A source that is an ordered list of files.** `SourceFile` gains `pages: Vec<PageFile>`
  (`kalamu_core::ordered`), empty for every other source. A `PageFile` is the original where it
  lies (path, name, SHA-256, size, modification time), how many pages it holds (`frames`, more
  than one for a multi-page TIFF), an optional label, and an `id` derived from its path and
  hash: the page's identity, never its position. The document's `sha256` is a hash of its pages'
  hashes in their order, `page_count` the sum of their frames, `file_name` the researcher's name
  (made a file name both Windows and macOS accept: `ordered::file_name_of`), and `path` that name
  in the first page's folder, which is not a file on disk but where the results go. Kind stays
  `image`, so every rule about what reads images (which tasks, which models, a page per call)
  holds without a case of its own.
- **The cache key: the page's own file, never its position.** `SourceFile::cache_identity(piece)`
  turns page *n* of a document into page *frame* of the file holding it, keyed by that file's
  hash; every other source gives back its own hash and the piece unchanged, so no key stored
  before changes (the test that pins an old key still passes). Moving a page therefore reads
  nothing again, and a photograph read alone in an earlier job is the same page in a document.
  Only order-dependent work is keyed by order: a table of contents' combining step keeps
  `CacheKey::merge(source.sha256, page_keys…)`, whose page keys come in the new order, so it is
  combined again after a move, while its pages are not read again. Pieces in the store stay
  numbered by position, as a PDF's are, so the review, corrections, exports and error rates need
  nothing new.
- **One test of "is this a document of images", in `core`.** `is_page_set` is asked by
  `cache_identity`, `files`, `file_of`, `names_itself` and `settle_pages`, all on `SourceFile`;
  the engine, the exporters and the shell ask those questions instead (which files does it read,
  which file is this page, is its name a file's) and never test the kind themselves. `media`
  opens such a source as `OpenSource::Pages(PageSet)`, the one place that reads its files.
- **`media` checks each page as it reads it.** A `PageSet` opens a file only when one of its
  pages is prepared, checks its size, and its hash unless the modification time is unchanged,
  and keeps the last file open for the next page (a TIFF's pages, a spread's halves). A file
  moved, deleted or replaced fails its page alone, with a reason naming the file
  (`MediaError::PageChanged`, or the read error with its path); `check_unchanged` passes such a
  source, since the whole-source check would fail every page for one. The other pages are read
  and the document is still written, the failed page marked `[Not read: …]`, as for a PDF page.
  Originals are only read.
- **The engine makes the document, and settles it again.** `Engine::image_document(name,
  sources)` takes images already inspected (nothing is hashed again), sorts them with
  `natural_cmp` (ASCII digits as numbers, capitals ignored, space, `_`, `-` and `.` alike, digits
  of other scripts as letters, ties broken by fewer leading zeros then as written), names an
  unnamed document after its folder, and gives `page_notes` (the number each name carries, its
  repeats and the numbers missing below it) and every page's checks, renumbered. New job only
  moves pages; `create_job` calls `settle_pages`, which works out the hash, count, place and
  identities again from the pages as ordered, so nothing the interface sends is trusted for
  them.
- **Only tasks that read pages.** `Task::reads_page_documents` (an extraction, a table of
  contents), given to the interface as `TaskRules::page_documents`; `check_job` refuses a
  document of images for any other task. An analysis would need several images in one request,
  which `PreparedInput` is not.
- **Stored with its job: no migration.** The job's sources are already kept as JSON in its own
  row (`jobs.data`), and a project's document keeps its source the same way
  (`documents.source`). The manifest is a field of the source, `skip_serializing_if` empty and
  `serde(default)`, so the schema and every row written before are unchanged: an older job reads
  with no pages, and no migration runs. A table of pages would have been a second copy of what
  the row holds, to keep in step; one can come later if pages are ever queried across jobs.
  A data folder written by this version and opened by an older one reads a document of images
  as an image whose file is missing, and fails it with that reason; nothing is lost.
- **Exports.** One result for the document, named after it, pages numbered in its order as a
  PDF's are (golden files `photographed.*`). `provenance.json` records `source.pages`: each file
  in the order read with the page it begins, its id, path, hash, size, frames and label. ALTO is
  written when the model gives positions, each page measured in its own image; a searchable PDF
  needs a PDF, as for a single image, and New job says so beside the order.
- **Review and projects.** `PieceView.original_file` names a page's file, and the masthead opens
  it (`open_original`, which opens only files of the job's sources). A project takes the
  document as one: its title is its name, and it is missing when any of its files is.

## Consequences

- The privacy policy is unchanged: the job database already keeps "the paths, sizes,
  modification times and fingerprints of its files", which is what the manifest holds, and
  `provenance.json` is written beside the results on the researcher's own computer as before.
- The specification, roadmap, formats, DESIGN.md, README and Store listing say what is built.
- Left for later in #52: typing a label for a page in New job (the manifest holds it); reordering,
  adding or removing pages of a job already made; an analysis of a document of images; adding
  one to a project before a job reads it; a document's items cut out and read on their own;
  several images sent in one call; and the second increment, transcriptions paired with scans.

## Architecture checkpoint

The change crossed five crates and the shell, as the issue expected, without a new internal
dependency: `core` holds the type, the order, the notes and the cache identity; `media` reads
the pages; `engine` makes and settles the document and keys its pages through `core`; `export`
measures ALTO pages per file; `projects` asks `core` which files a document has. `store` needed
no change beyond a test. The engine gained `image` as a test-only dependency, already in the
workspace. `core/src/ordered.rs` stays under 500 lines with its tests in a file of their own;
`media/src/lib.rs` was already past 500 (544 now) and keeps the page set in its own module;
`NewJob.svelte` grows by 50 lines to 975, the order itself in `PageOrder.svelte` and the logic
in `pageOrder.ts`. Resuming, caching, progress and provenance cover the new path: a job stopped
mid-document resumes with the pages not yet read, a page moved hits the cache, progress counts
pages as for a PDF, and provenance names every file.
