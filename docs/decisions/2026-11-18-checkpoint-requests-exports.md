# 2026-11-18 · Checkpoint: one lifecycle for a request from the review, exports that read the researcher's regions

## Context

Two rounds earlier the same day added a passage corrected by a model, then a region read again,
the second written on purpose as a mirror of the first so it could ship safely
([decision](2026-11-18-drawn-regions-region-read-again.md)). That left the same lifecycle
twice in `core`, `store`, `engine`, the commands and the interface, with a third use in sight
(post-interview analysis, [#38](https://github.com/fmadore/Kalamu/issues/38)). It also left
ALTO, PAGE XML and the searchable PDF writing the model's own boxes, against the rule that
exporters read the corrected view. `AGENTS.md` asks for a checkpoint at such a point. Three
agents worked in worktrees on parts sharing no files; they were reviewed and gathered on one
branch, where every check was run again.

## Decision

### One lifecycle for a request made from the review

- **What is shared is the life of the request**, not the request: checked, recorded as
  asked, answered from the cache at no cost, otherwise called with a stop raced against it,
  recorded as it ended, a billed refusal charged, then accepted with its effect in one
  transaction, discarded or undone. `core::proposal` holds the status and its transitions,
  `store::proposals` what the two tables read the same way and the cancelling at start-up,
  `engine::asked` the rest.
- **What differs stays apart**: the two requests and their errors, the checks before asking,
  the insert statements (other columns, other effects), the provenance. Sharing them would
  have changed the interface's types or built SQL from parts.
- **No change that can be seen or stored**: no migration, no binding changed, no existing
  test edited. A test now fails if a table of requests is missing from a job's cost or from
  what pruning keeps, so a third kind cannot be forgotten there.
- **In the interface**, one state class (`modelAsk.svelte.ts`) for the model chosen, its cost,
  the consent, the wait and the answer, and three small components; each dialog keeps what it
  sends and where the answer goes. The two dialogs' screenshots are the same pixel for pixel
  before and after, at every stage, in both themes and both languages. The buttons under a
  proposal stay in each dialog: sharing ten lines would have cost five properties.
- **The text review** hands the passage correction, the following of the player and the pager
  to their own files (879 lines to 697). What is left shares the editing state and stays
  together. `AnalysisReview`, `SourcePane` and `ContentsReview` remain over 500 lines.

### Exports read the researcher's regions

- **One rule for whose box it is**, the review's and provenance's: the engine's
  `regions_exported` feeds both the files and `provenance.json`.
- **`TextRegion` keeps its shape.** An exported part carries, beside its regions already laid
  with the researcher's boxes, what says whose each is; empty, it is left out, and every
  earlier golden file is unchanged byte for byte.
- **Widened, never scaled.** A block moved alone keeps its measured lines and is widened to
  hold them: scaling would move measured words to guessed places.
- **Each format's own way**: ALTO's `PROCESSINGREFS` to a second processing step, made for
  this in 4.1; PAGE XML's `comments`, where an estimated position was already said, with a
  `MetadataItem` for the step. Checked by hand against both official schemas.
- **A box put back is one write**: a shown region names the adjustment that moved it
  (`adjusted_by`).

### Smaller things

- The page's class in the review is `page-sheet`, no longer the global `sheet` whose padding
  it took; the override that undid it is gone.
- The resume buttons' map in the sidebar is reactive, which ends a warning on every screen.

## Consequences

- A third request from the review costs its own input, answer and effect, not a fourth copy
  of the lifecycle.
- The review and the exports differ in one place: a block placed whole that the researcher
  moved has its estimated lines drawn within the new box in the files, within the model's in
  the review. Left to do.
- Left as it was: Escape while a model answers stops and closes the passage dialog, and only
  stops the region dialog.
- Not done: a cost estimate that follows a region's size. Providers count a picture's tokens
  differently (tiles, pixels, a flat rate, a whole page for OCR), so it needs a rule per
  price family in the catalogue.
- Not tried: the new files opened in Transkribus, eScriptorium or Aletheia; a real Mac.
