# 2026-10-31 · Analysing corrected text from frozen snapshots

## Context

[#47](https://github.com/fmadore/Kalamu/issues/47): a researcher who has corrected an
extraction or a transcript wants to analyse that text, and to know later exactly what the
analysis read. Until now *Analyse the results* opened New job on the `.txt` files the job had
written: the text as last exported, read from disk with nothing to say where it came from, and
silently different if the files were exported again after a correction. Nothing in the
reference projects freezes an input (iwac-ai-pipelines chains steps through files on disk), so
the design is Kalamu's own.

## Decisions

- **A snapshot is a file, named by its hash.** The corrected view of one source under one
  set-up (what the text export writes: corrections over the model's output, speakers renamed)
  is written once to `snapshots/<sha256>.txt` in the app's data folder and never rewritten; the
  store keeps a `Snapshot` record with the result it came from, when, and where each piece sits.
  The analysis then reads an ordinary text source. That keeps the engine as it was: caching,
  resuming, the changed-file check, page previews and the review need no snapshot case, and no
  path bypasses the cache. A text source carries `derived_from` (snapshot, job, set-up, source,
  original), so whatever reads a source knows where it came from without the store.
- **Fingerprints follow the text.** The source's SHA-256 is the frozen text's, and every cache
  key holds it: corrected text is analysed again, unchanged text is not, and a frozen copy is
  shared by identical texts.
- **Stale, not rewritten.** A correction after the snapshot leaves the snapshot and the
  analysis's answers as they were; the engine compares the current corrected view's hash with
  the source's and the job page says the analysis is out of date, offering to analyse the
  corrected text as a new job. A deleted upstream job leaves the snapshot standing ("gone").
  Recomputing only what is affected, across several steps, is #49's.
- **Partial coverage is explicit.** Pieces that failed are left out of the text; pieces failed
  or not yet read are listed in the snapshot and in `provenance.json` (`missing`), with how many
  were reviewed and corrected. "Processed" is never written as "reviewed".
- **Offsets are characters.** A part's span and an anchor count Unicode scalar values, so they
  mean the same in Rust, in TypeScript and in Arabic. `locate` finds every place a quotation
  stands, each inside one piece; `check` refuses an empty, shifted, overlong or cross-piece span.
  Page markers sit outside every span. Showing a model's quotations as evidence (#38, #51) will
  go through `check`.
- **Results go beside the original**, not into the data folder: with *beside the sources*, a
  derived source writes next to `derived_from.original_path`; text frozen from a YouTube
  transcript needs a chosen folder, as the video did.
- **Snapshots outlive jobs.** The table has no foreign key to `jobs`: an analysis keeps knowing
  what it read after the job it read from is deleted. Deleting snapshot files is left for a
  retention setting; they are small text files.
- **Projects.** A derived analysis joins the document of the result it read, not a new one.
- **The interface asks the engine** whether a task yields text (`TaskRules::yields_text`),
  rather than keeping a copy of the rule. Tables of contents keep *Analyse the results* over
  their written files until their entries become document units (#50).

## Consequences

- The privacy policy lists the `snapshots` folder.
- Left in #47: frozen structured results (an analysis's corrected answers, a table of
  contents' entries), selecting a snapshot of only the reviewed pieces, and a retention policy
  for snapshot files. #38's evidence-linked quotations and #49's multi-step workflows build on
  `Snapshot`, `locate` and `check`.
