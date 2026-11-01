# 2026-11-01 · Several pages of a PDF in one call

## Context

Text extraction sent every page of a PDF on its own. The maintainer asked for the choice of
sending several pages, or the whole document, in one call: the model then sees an article that
runs over the next page, and a short file costs fewer calls. The risk is the output limit: an
answer cut off there loses the last pages, which is worse than losing the end of one page. Two
ways to handle a model that does not mark the pages were offered; the maintainer chose to read
such pages again one at a time, so that the review and error rates never break.
iwac-ai-pipelines and zmo-ai-pipelines send a page per call (or a whole file to Mistral OCR,
which answers page by page); nothing there splits one answer back into pages.

## Decisions

- **A setting of the job, for text extraction: one page at a time (the default), several
  pages, or the whole document.** It applies to a PDF read at once (not in a batch), whose
  spreads are not split, by a model that takes instructions and reads PDFs. Elsewhere pages go
  one by one, as New job says: in economy mode, and for Mistral OCR, which takes no prompt.
- **Pages stay pages.** Planning, the store, the review, corrections, exports and error rates
  are unchanged: only the sending groups consecutive pages still to read. The call carries a
  PDF of those pages and an instruction to begin each page with `--- Page N ---`, numbered as
  the document numbers it. The answer is split at those marks (`core::pages::split`), loosely
  (other dashes, a heading, capitals, marks numbered from 1) but never by guessing: marks out of
  order, repeated or out of range leave every page to be read alone.
- **Nothing is lost silently.** A page whose mark is missing is read again on its own. An
  answer stopped at the output limit keeps the pages before the last one found; the last one,
  which may be incomplete, is read again. A group the provider refuses or that keeps failing is
  tried page by page before any page is marked failed. A group too large for the model's
  request limit goes page by page.
- **Sized to the model's output.** A group holds no more pages than three fifths of the model's
  output limit can take, at 1,500 tokens a page, or 4,000 with the newspaper preset; the rest
  is room for thinking and denser pages. With Claude's 128,000 tokens that is 51 letter pages
  or 19 newspaper pages, with Gemini's 65,536 tokens 26 or 9. "Whole document" means as many as
  fit, in as many calls as needed.
- **Cost and keys.** The call's usage and cost are stored with the first page of the group, so
  each is counted once. A page read in a group has a key that names the group size, so an
  answer read in a group is never reused for a page read alone, or the reverse; the same job
  run again is read from the cache. Provenance lists the pages that went together (`pages`,
  `5-8`) among the request settings.
- **One retry loop.** `process_piece`'s calls and retries moved into `Engine::call`, which a
  group uses too, so a group is retried, charged and paused like a page.

## Consequences

- Economy mode does not group pages yet; batches would need a group as one batch item.
- How well each model marks pages is untested against the live APIs; a model that marks badly
  costs a second reading of the pages it leaves out, which provenance shows (pages without
  `pages` in their request).
- `engine/src/group.rs` is about 360 lines; `piece.rs` about 410.
