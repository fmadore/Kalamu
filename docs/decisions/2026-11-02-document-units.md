# 2026-11-02 · Items of a table of contents, read on their own

## Context

[#50](https://github.com/fmadore/Kalamu/issues/50): a table of contents names the articles of
a newspaper issue, the letters of an archival file, the chapters of a report, but a researcher
could only analyse or extract the whole file, and the decision on snapshots
([2026-10-31](2026-10-31-reviewed-text-snapshots.md)) left *Analyse the results* over a table of
contents' written files until its entries became units. An entry's pages are free text written
by the combining model (`"1, 3-5"`), numbered as the file's pages, since the first step labels
each page by its piece; entries have no identity of their own, as a correction replaces the
whole list. iwac-ai-pipelines writes an issue's table of contents to Omeka (`AI_summary_issue`,
`03_update_omeka_toc.py`) but never reads an article on its own, so the design is Kalamu's.

## Decisions

- **Items are a layer of their own** (`DocumentUnit`, table `units`, migration 14): one source
  under one set-up of a table of contents, a stable id, a place in the list, a title, the
  entry's other fields, its pages as the job's pieces (`PieceRef::Page`, halves of a spread
  included), the entry it came from and a revision raised by every change. They go with their
  job; the table of contents and its corrections are never touched. "Items" in the interface,
  units in the code.
- **Made once, then the researcher's.** *Make items from the entries* reads the corrected
  entries and makes one item each; afterwards correcting the entries does not remake them,
  since the researcher's joins and splits would be lost. Pages the document does not have are
  left out and reported, an entry whose pages cannot be read gives an item without pages.
- **Pages are typed as the file numbers them**, in reading order: `1, 3-5, 8 (left)`; a page
  split in two is both halves unless one is named. `kalamu_core::units` reads and writes these
  lists, so the interface never parses them. The numbers printed on the pages are not known
  yet and are left for later.
- **Whole pages, said outright.** An item is made of whole pages; two items on one page each
  take all of it, and the list names the items a page also holds rather than pretend a cut
  isolates one. Finer boundaries (a column, a passage) are left for later in #50.
- **An item is read as a PDF of its pages, frozen like corrected text.** The selected items'
  pages are cut from the source in their order (`PdfSource::chosen_pages_pdf`, lopdf, no
  rendering), written once to `snapshots/<sha256>.pdf` and recorded as a `Snapshot` whose parts
  are the pages; the new job reads an ordinary PDF whose `derived_from` carries the item as it
  was (`FrozenUnit`). Caching, resuming, the review and the changed-file check need no case of
  their own, results are written beside the source under the item's name
  (`an-nasr-12 03 Fatwa.txt`, `an-nasr-12 03 Fatwa.analysis.csv`), and `provenance.json` names the item. An item given
  other pages marks the job out of date (`Freshness::Changed`), a deleted one `Gone`; a new
  title does not, as it only names the files. A page read in the whole issue and again in an
  item is two requests, since the item's PDF is a source of its own: the price of keeping the
  engine unchanged.
- **Export** writes each selected item's pages as a PDF into a chosen folder, with the list as
  `<source> units.csv` and `.json` (`kalamu_export::units`), never replacing a file there.
- **Only a PDF's items** are cut out for now: a multi-page image's items can be made and
  corrected, but not read or exported on their own, and the interface says so.

## Consequences

- The privacy policy lists the items' pages among what the `snapshots` folder keeps, with a
  new effective date; formats.md describes the list and `derived_from.unit`.
- A table of contents keeps *Analyse the results* over its written files, for an analysis of
  the whole issue's entries; items are how a single article is analysed.
- Left in #50: finer boundaries, printed page numbers, items of images, an item's text from an
  extraction of the whole issue (no second reading), items as project documents, importing an
  exported list, and combining a large issue's pages in bounded groups.

## Architecture checkpoint

Dependencies still point inwards: `core` holds the item, its page lists and its frozen form;
`store` the table; `media` the cutting of pages; `export` the list; `engine` making, changing,
freezing and exporting items; `app` eight thin commands. No crate gained an internal
dependency. `core/src/units.rs` passes 500 lines, half of them tests; `store/src/lib.rs` is at
554, its `JobStore` implementation forwarding to one module per table, which keeps it
readable. The interface's `UnitsPanel` keeps the table of contents' review under 500 lines.
