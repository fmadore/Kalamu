# 2026-10-29 · The first table of contents: its review, and the language of its answers

## Context

The first live table of contents, Claude Sonnet 5.5 reading and combining a 12-page issue of
*Islam Info* (14 October 2005), with 0.4.0, showed four problems:

- **Six entries for twelve pages**, all from pages 5 and 11. The other pages added nothing.
  Every page cut for Claude kept its scan and its OCR layer, and the 3,804 output tokens for the
  13 calls show that the readings of the other pages were nearly empty: the first step, not the
  combining one, came back short. The pages' answers are only in the job database, which could
  not be read for this record (the Claude app's shells see a stale copy of `%LOCALAPPDATA%`), so
  the cause is still open. A suspect: every Claude call is sent at `effort: "low"`.
- **The review did not follow the page.** The entries on the page shown were marked, but the
  list never moved to them, and the strip of pages with their thumbnails, which the text review
  has, was missing.
- **Summaries in two languages**: page 5's in English, page 11's in French. The presets told the
  model to write "in the language(s) of the source, detected automatically", in English
  instructions, and Sonnet followed it on some pages only. New job offered the source-language
  choice for a table of contents, while its presets used `{languages}` both for the source
  ("The issue is in …") and for the answers.
- **Settings scrolled the whole window into blank space.** The screen-reader text inside links
  (`.sr-only`, absolutely positioned) escaped `main`, which was not positioned, and stretched
  the page to the full height of the settings.

## Decisions

- **The strip of pages is a component of its own** (`PageStrip.svelte`), used by the text review
  and the table of contents' review: a first piece of the review refactor the roadmap plans.
  Turning the page in the table of contents' review scrolls its first entry to the top of the
  list; choosing an entry does not move the list.
- **A table of contents has an answer language, as an analysis has**: one language or the
  document's own, under "Language of the answers" in New job. `RenderContext::answers` says when
  `{languages}` is the answers' language; then, left to the document, it reads "the language
  the source is written in (the one most of it is in, when it mixes several), even though these
  instructions are in English", and a template without the variable gets "Write your answers in
  …" rather than "The source is in …". The page presets no longer state the source's language;
  the combining presets write every entry in that one language, titles and names as printed.
  The pipeline record names it "Language of the answers".
- **`main` is positioned**, so whatever a screen places absolutely scrolls with it.

## Consequences

- Prompts change for new jobs only; existing jobs keep their frozen prompts and cached results.
- The cause of the empty pages stays open in the roadmap: reading the pages' stored answers, and
  a run with Claude at a higher effort for reading pages, come first.
