# 2026-11-15 · The rest of the two checkpoints: screens split, shared pieces, Tesseract and Whisper

## Context

The [round of 14 November](2026-11-14-checkpoint-leftovers.md) took up the nine numbered things
the [8 November checkpoint](2026-11-08-checkpoint.md) left. Its *The rest* stayed open: the
larger screens to split, the shared interface pieces not yet used everywhere, and three things
from the [4 November checkpoint](2026-11-04-checkpoint.md) about Tesseract and Whisper. This
round took them up in four groups whose files do not overlap.

## Decision

### Screens split, nothing changed for the reader

- **New job.** `NewJob.svelte` 1,019 → 829 lines: the task step, the set-aside notice, the
  reading options and the pace are components of their own, and the estimate is followed from
  `newJobEstimate.svelte.ts`. `newJob.ts` 726 → 408: model choice in `newJobModels.ts`, a
  file's warnings and durations in `fileNotes.ts`. What is left in `NewJob.svelte` is the
  form's own state and what is done with it (adding files, a pipeline saved or applied, the
  start); cutting further would pass that state across components, a redesign and not a move.
- **The review.** `ReviewView.svelte` 799 → 695: the open readings are a `Readings` class in
  `readings.svelte.ts`, and a reading is taken into the text by `withReading()`, tested.
- **A project.** `ProjectWorkspace.svelte` 587 → 416, with `ProjectSections` and
  `ProjectEditForm`.
- **A job's set-ups.** `SetupList.svelte` 554 → 178, each set-up drawn by `SetupCard`.

### Shared pieces

- **`Action`** runs the commands of twelve more screens, each saying a failure where and how it
  did. Two stay by hand: a provider's key check in Settings, which keeps one status per
  provider, and the workflow editor, whose failure belongs to a step and is not cleared by the
  next command.
- **One `ContextMenu`** opens, moves and closes the job's and the name's menus. Their
  differences are kept as options: Home and End in the job's menu only, the name's menu a
  dialog while renaming.
- **One `FindBox`** is the search box and list of the two language pickers, each keeping which
  items the arrows reach, what Enter takes and where the focus goes after a choice.
- **`moved`** of `profiles.ts` orders an analysis's fields as it orders export columns.

### Tesseract

- **The resolution a PDF page was drawn at is recorded.** The drawn page states it (a `pHYs`
  chunk and a text chunk of Kalamu's own), Tesseract's result records `page_resolution`, and
  ALTO measures each page by it, naming the pages drawn below 300 dpi. A result that records
  none is measured at 300, as before. Tesseract is handed pixels, not the file, so it reads
  what it read: the same text, places and confidences with and without the chunks, checked with
  the real engine. No cache key moves.
- **Engines are pooled** by language set, as many as pages are read at once. An idle engine
  holds no claim on its language data, so deleting the data succeeds at once and releases its
  engines; one reading a page still refuses the delete. The data is checked with a streamed
  hash once a session, equal byte for byte to the hash of the whole file.

### Whisper's voice detection in an accented folder

whisper.cpp, built with MSVC or clang-cl, turns the path it is given into wide characters
itself, so the code page plays no part: tried on Windows from a test program, which has no UTF-8
manifest, the model opened from `kalamu-é-test` and from a folder named in Japanese. It did not
open from a folder named with an emoji: the conversion stops at the Basic Multilingual Plane.
The check now leaves the model out only there. The 14 November entry called this a limit of the
C library; it is this conversion's, and only for such characters. Only Tesseract needs the
manifest.

## Seen and left

- Taking an empty reading leaves both spaces around the words removed, and a lemma that was all
  punctuation keeps it after the reading taken (`Fin!.`). Pinned by `withReading`'s tests.
- In both language pickers, Escape from the list closes it and the focus returning to the box
  opens it again. Pinned by the new tests.
- The name's menu does not take Home and End, where the job's does.
- Telling Tesseract the page's true resolution might read a reduced page better. Not tried: it
  would change results whose cache keys stay the same.

## Consequences

- AGENTS.md is unchanged: no new internal dependency.
- Still open from 4 November: the folders out of reach shown at launch, which only the sandboxed
  Mac App Store build has and which needs a Mac.
- Checked on Windows ARM64. Still to try: a transcription from an accented data folder in the
  packaged app, and all of it on a Mac.
