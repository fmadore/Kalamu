# 2026-11-23 · Gemini's file slicing is not adopted

## Context

Google's document processing guide (aistudio.google.com/docs/document-processing, *Customize
document processing*) describes slicing a file on Google's side. A file uploaded through the
Files API is named in a `document` block with a `processing` object (`"type": "static"`):

- for PDFs and Office files, `page_options` with `start_page` and `end_page` (1-indexed,
  inclusive) and `content_layers` (`page_image`, `page_text` or both), the block's own
  `resolution` (`low`, `medium`, `high`) beside it;
- for spreadsheets, `spreadsheet_options` with a `sheet_index`, `visible_cells` (an A1 range) and
  `content_layers` (`data`, `formulas`, `visual`).

Only the slice is billed. The guide gives it for Gemini 3 and 4+, through the Interactions API
(`/interactions`, with an `Api-Revision` header), and every example names an uploaded file by
its `uri`; whether `generateContent`, or a file sent inline, takes `processing` is not said. The
section was read from a summary of the page, so its field names are to be checked against the
page before any use.

Kalamu already slices, on the researcher's computer and for every provider: `media` cuts a PDF
into one-page pieces with lopdf, and `providers/gemini.rs` sends each page inline to
`generateContent`, billed alone, its media resolution set for the request, which carries one
piece.

## Decision

Not adopted. What it would bring, Kalamu has; what it would cost goes against the architecture:

- **Pages are already billed alone**, and resolution is already set per piece.
- **Resuming.** Google deletes an uploaded file after 48 hours, and a job can stay paused longer.
  Every resume would check the upload and send the file again, bookkeeping for one provider only.
- **One way of splitting.** The engine splits every job the same way, whatever the provider, and
  results are cached by piece. A split made by Google for Gemini alone is the per-provider branch
  AGENTS.md rules out.
- **What leaves the computer.** The whole file would be uploaded even when the researcher chose
  a few pages, a change the privacy policy would have to state.
- **Another API.** Slicing belongs, as far as the guide shows, to the Interactions API, not to
  `generateContent`: adopting it would move the provider, batch mode included.

## Consequences

- No change to the code, the specification or the privacy policy.
- **Born-digital PDFs**: what `page_text` saves (the embedded text sent instead of the page's
  image) is better had locally, for every provider, by reading the text layer in `media` and
  sending plain text.
- **Neighbouring pages as context** (an article continued overleaf in a detailed table of
  contents) can be had by sending a small group of pages cut locally; slicing is not needed.
- **Revisit when Office files become an input.** Gemini renders DOCX and PPTX as page images and
  slices spreadsheets by sheet and range itself, which would spare the computer a converter. If
  that is taken up, it is a capability in the catalogue that the engine reads, not a branch on
  Gemini's name, and the questions above (48-hour uploads, what is sent) are answered then.
