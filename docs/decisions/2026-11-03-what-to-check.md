# 2026-11-03 · What to check across a project

## Context

[#85](https://github.com/fmadore/Kalamu/issues/85): after processing a collection, a researcher
had to open every job in turn to find the pages and answers that need them. Kalamu already knew
most of the reasons, each inside one job's review: pieces that failed or were not read, answers
cut off at the model's limit, set-ups of a comparison that read a page differently, analyses
read from a text corrected since (#47), and pages not yet reviewed. The first delivery gathers
those signals into one list per project, collection or selection, adds no model call, and
opens each item in its review. It changed four crates (`core`, `projects`, `engine`, `app`),
so this record is also the architecture checkpoint AGENTS.md asks for.

None of the three reference projects has a review queue across jobs; iwac-ai-pipelines and
zmo-ai-pipelines review outputs file by file. The issue points at Docima's human-review queue
for the workflow idea and warns against its numerical auto-accept thresholds; Kalamu takes the
list and leaves the thresholds out.

## Decision

- **Split by what each crate knows.** `projects` chooses the results (`Projects::queue_plan`):
  the documents in scope, and for each document and task the researcher's preferred result or,
  where none is preferred, every result they have not set aside. `engine` says what each needs
  (`Engine::review_queue`), since only it knows the stored pieces, the word diff the comparison
  review highlights and whether a frozen text has changed. The two meet through plain values in
  `core` (`ReviewScope`, `ReviewTarget`, `ReviewQueue`, `Signal`), and the app's one command,
  `review_queue`, passes the first to the second. No crate gained a dependency on another.
- **One item per piece of a result.** A document read by two set-ups gives two items for the
  same page, each named by its set-up, because each is reviewed on its own (the review's sigla
  key marks each set-up's page). Documents are counted once: the coverage counts documents in
  scope and documents read, then results and pieces, so alternative outputs never pass for more
  documents. Without a preferred result the queue lists which results it reads, with a box to
  set each aside; a preferred result is never set aside.
- **Reasons, kept apart.** `failed` and `not_read` are processing problems, listed first and
  never resolved by a review. An answer is listed when it is `truncated`, when it `disagrees`
  with another set-up of its job (texts by the words the review highlights, with their count;
  analyses and tables of contents by the list, choice and entry fields whose values differ once
  capitals and spaces are set aside, as the analysis review's sigla match them; prose fields
  are not compared, since two models never word a summary alike), when it is `stale` (it read a
  frozen text that changed, or whose job was deleted, or the pages of a document unit (#50)
  since given other pages or deleted, as `Engine::derived_states` already says), and while it is `not_reviewed`. A table
  of contents is reviewed as its combined entries, so its pages share that review.
- **Only the researcher resolves an answer.** Marking a piece reviewed resolves its item, which
  stays listed (as reviewed) for as long as its reasons hold; nothing else does. Agreement
  between models only means one reason is absent; no confidence or score is computed. A changed
  input stays open after a review, since the review was of an answer to the earlier text. A new
  answer is a new revision (its fingerprint and date are shown), and corrections belong to the
  job and set-up they were made in, so a new run never inherits a review.
- **The list does not hide the corpus.** The counts are over every result read, whatever the
  filters show. By default the list shows flagged items still open; *Not reviewed* shows every
  unreviewed page, and *Include reviewed* the resolved ones.
- **In the interface**, *To check* is a pane of the project beside the dictionary, with its own
  choice of documents (the project, a collection, or the table's selection). An item opens its
  review at its page, with a banner naming the item and its reasons and stepping to the
  previous or next one, in that review or another, or back to the list as it was set.

## Checkpoint

- **Dependencies still point inwards.** `projects` depends on `core` only; the engine gained a
  module (`attention.rs`, about 440 lines with its unit tests), not a dependency; the app wires.
- **No new path to a provider.** The queue reads the store and the frozen texts; the engine's
  tests check that building it calls no model.
- **Reuse rather than copies.** Disagreement uses the comparison review's `diff`; freshness uses
  `Engine::derived_states`; corrections use `Correction::of`. The analysis review's own sigla
  matching (`src/lib/analysis.ts`) is the same rule in TypeScript and stays there for display.
- **Files near the limit**: `engine/src/compare.rs` (471 lines) and `engine/src/tests.rs` were
  not grown; the new tests are a module of their own.

## Consequences

- Tested in Rust for each reason, for review resolving an item, for a changed and a deleted
  input, for analyses that agree, and for the result policy (preferred, set aside, collection,
  selection, a document of another project refused). The interface's filters and wording are
  tested in TypeScript.
- **Later increments of #85**: items kept aside (deferred) with the researcher's note; a
  reproducible random sample of unflagged pages; a review manifest export with source and
  result identities, reasons and decisions; invalid evidence anchors once analyses quote their
  text; and optional field rules (required values, vocabulary, dates) recorded with their
  version. Results read from document units (#50) are already in the list as any result,
  their changed pages included; units not yet read are not listed.
- The review still leads with a comparison's first set-up; an item about the second opens the
  same review, where its own page is marked reviewed from the sigla key.
