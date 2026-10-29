# 2026-10-29 · Results that reach the researcher

## Context

The [checkpoint of the same day](2026-10-29-checkpoint.md) found two ways finished work did not
reach the researcher's folder:

- A document's files were written only once every one of its pieces had succeeded. A page that
  fails every time (a scan with no text layer sent for correction, a page a provider's copyright
  filter refuses) kept every other page of the document from disk; the only way out was to save
  a correction, which writes the files again.
- Files were named after the source alone, so two sources that share a name in one folder,
  `scan.pdf` and `scan.tif` side by side, or two `scan.pdf` from different folders sent to one,
  wrote over each other's files and `provenance.json`, the last one winning.

## Decisions

- **A document is written once its pieces are settled**, each done or failed after its retries,
  with at least one done. A failed piece keeps its place: `ExportPart::failed` holds the reason,
  the text formats and the CSV show `[Not read: reason]` (under the page's own marker), JSON
  gives `failed` with an empty text, a searchable PDF leaves the page without a text layer, and
  `provenance.json` lists it under `failed`. Retrying the failed pieces writes the files again,
  whole. An analysis is one piece, so a failed one still writes nothing; a table of contents
  keeps the old rule, since without its combined entries it has nothing to write.
- **Sources of the same name are numbered.** Where two sources of a job would get the same name
  in the same folder, the later ones in the job's order are named as Windows names copies,
  `scan (2)`, `scan (3)`. The comparison ignores case, as Windows does. The first keeps its name,
  so jobs without a clash write exactly as before.

## Consequences

- A job that ends "completed with errors" now has its files on disk, with the gaps marked.
- The marker is English, like the page markers and labels the text formats already carry.
- Tested in the engine (a page that keeps failing is marked in the text and listed in
  provenance, then written whole after a retry; two `Scan.pdf` from different folders sent to
  one become `Scan.txt` and `Scan (2).txt`) and in the exporters' golden files, where the letter
  gains a third page that was not read.
