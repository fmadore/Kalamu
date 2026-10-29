# 2026-10-29 · Detailed tables of contents, and correcting existing text

## Context

Two of iwac-ai-pipelines' pipelines serve researchers well beyond the Islam West Africa
Collection. `AI_summary_issue` indexes a magazine issue: a vision model reads each page for its
articles (title, authors, "continued on page X", a short summary), then a cheaper model combines
the pages into one entry per article with all its pages. `AI_ocr_correction` corrects the OCR
of newspaper text while keeping historical spelling. Social scientists meet both needs daily: a
whole newspaper issue, an archival file scanned as one PDF, a long report or thesis; and the
noisy text layers of Gallica, Europeana or Internet Archive downloads. Kalamu's analyses read
each file as one document, so "several documents in one file" could only be warned about.

## Decisions

- **Two new tasks**, `Task::Contents` ("Detailed table of contents") and
  `Task::TextCorrection` ("Correct existing text"), each with its presets, diagram, review and
  exports. New job now offers six choices, two by two: reading pages, working on texts, spoken
  sources.
- **Providers see analyses.** Both tasks are made of answers in fields, so they reach providers
  as analyses (`Task::request_task`): a page's entries, the combined entries, a corrected text.
  Providers only add the two tasks to their analysis match arms; the catalogue needs no new
  entries, since `ModelInfo::does` takes the analysis models that read the task's input (pages
  for a table of contents, text for a correction).
- **Entries are a field kind.** `FieldKind::Entries` (a list of objects, each with its own
  fields) and `FieldValue::Entries` let one answer hold a table of contents. Every provider's
  structured output takes nested objects; `AnalysisSchema::read_answer` checks them as it checks
  flat fields. The field editor does not offer the kind: Kalamu builds it.
- **A table of contents runs in two steps, on two models.** The set-up's own model reads each
  page (`contents::page_schema`: title, where it continues, the researcher's fields); the set-up's
  `MergeStep`, any provider's analysis model that reads text, combines the pages' JSON readings
  (`contents::merge_schema`: title, pages, the researcher's fields). As in iwac-ai-pipelines, a
  large model reads and a small one combines; unlike it, the two may be different providers'
  (Gemini Pro reads, Mistral combines). New job defaults to the reading provider's cheapest
  model, so one key is enough. The researcher chooses the entry's fields in Prompts, as for an
  analysis; title and pages are fixed so every entry can be found in the source.
- **The second step is a piece.** It is recorded as `PieceRef::Whole` after the pages, cached,
  retried, resumed and counted like any other. Its cache key (`CacheKey::merge`) is built from
  the pages' own keys, so reading a page again or choosing another merge model or prompt makes
  it new, and nothing else does. A failed page fails the step until the page is retried. In
  economy mode the pages go in batches and the step runs at once when they come back.
- **Three presets**, ported and widened: a newspaper or magazine issue (iwac's two prompts in
  English, without their Islamic-magazine specifics), an archival file item by item (date,
  sender, addressee, summary, names: an archivist's inventory), and a book, report or thesis
  (parts, chapters, sections).
- **A correction sends text.** A PDF's text layer (`PdfSource::page_text`) or a text file's
  pages (`--- Page N ---`, as Kalamu writes them) or passages of at most 12,000 characters cut
  at paragraph breaks (`PieceRef::Passage`). The corrected text comes back as one field, so
  structured output keeps it whole and an empty answer is retried. Two presets: a general one,
  and iwac's newspaper prompt verbatim. Line breaks are a choice, as for extraction.
- **The review compares a correction with the original.** `PieceView::source_text` carries the
  text before correction; the text review shows it as a witness ("O"), so every change is
  highlighted and the original reading can be taken back in one click. A table of contents has
  its own review: the scan beside the entries, the entries on the page shown marked, what the
  first step read on that page below, and the entries corrected, added or removed as a layer
  over the model's.
- **Outputs never replace their source**: `.contents.*` and `.corrected.*`, like
  `.analysis.*`. A table of contents exports one entry per heading (Markdown, Word), per
  numbered block (text) or per row (CSV), with its pages; JSON has an `entries` array.
  `provenance.json` records the merge step's provider, model, versions and prompt under `merge`.

## Consequences

- A text-only model can now read a PDF page from its text layer wherever the provider sends
  text only (`OpenSource::prepare` with `text/plain` alone), with a plain message when the page
  is a scan without one.
- Photographs of one archival file must first be put together into one PDF, as the tip says;
  treating a folder of images as one document is left for later.
- Text files are not yet offered for a table of contents (a long Word report, a transcript):
  they would need passages as the first step's pieces.
- The economy path for a table of contents is covered by the code path shared with the other
  tasks, but not yet by a test of its own; nothing has run against live APIs.

## Architecture checkpoint

Dependencies still point inwards: `core` holds the schemas and the merge input, `engine` the
second step (`engine/src/contents.rs`), `export` one module per rendering (`export/src/contents.rs`).
The new tasks touched every provider only in their analysis match arms, which a provider-facing
task enum would remove; with three provider kinds of call that stay stable, the or-patterns are
kept.
