# 2026-10-25 · Graphite and highlighter

## Context

The researcher found the Critical Edition interface dull and generic: a navy shell, Garamond for
every title, an apparatus criticus that asked users to read edition conventions (sigla, lemma,
"om.", "add.") before they could check a word. The brief was an interface that looks like a
professional desktop tool. A design canvas with five artboards (two reviews, New job, an
interrupted job, the system sheet) was agreed before any code changed.

## Decisions

- **One visual rule per colour.** Grey surfaces carry the structure; black is the only button
  colour, one primary action per screen with its shortcut; the highlighter yellow marks only
  what needs a look (models disagree, pages left to review); blue marks the researcher's own
  corrections. Tokens in `src/app.css`, light and dark.
- **Three faces with one job each.** Instrument Sans for the interface, Source Serif 4 for the
  text being checked only, IBM Plex Mono for what a machine produced (file names, models, times,
  costs, shortcuts). All three are bundled through Fontsource (OFL), so the app works offline;
  EB Garamond and Source Sans 3 are removed.
- **The sidebar lists the jobs.** Running, unfinished and finished jobs are always in view, with
  their progress; the separate Jobs list and the foldable menu are gone. New job is the black
  button at the top (Ctrl N); Prompts and Settings sit at the foot.
- **Differences are read in the text, not below it.** The apparatus list is replaced by
  highlighted words that open their readings under the line. Any reading, or one the researcher
  types, is saved as a correction through the same `save_correction` command, so the model's
  text is still never overwritten and nothing changed in the engine or the store. `apparatus.ts`
  still computes the entries; only its presentation changed.
- **The scan sits on a dark lightbox in both themes**, so the page is the brightest thing on
  screen, with a floating zoom control. A strip of pages under the review shows which are
  reviewed. Ctrl Enter marks a page reviewed; F and Shift F step through the differences.
- **The review's room is the researcher's to divide.** The boundaries between source, text and
  compared text are dragged (or moved with the arrow keys), and the sidebar folds to a rail
  (Ctrl B); both are remembered in the webview's local storage, a convenience that the review
  does without when storage is unavailable, not a setting.
- **Readings show on pointing, over the text.** The readings card moved from under the line to a
  popover by the words, shown after 300ms of pointing and pinned by a click or F, so checking a
  difference no longer pushes the text down. Each reading is one row to click.
- **Compare puts another set-up's whole text beside the text.** It reads the same word diffs as
  the highlights (`witnessParts` in `apparatus.ts`), so a difference is one place in both texts.
  Words a witness adds, which have no lemma, now show as a mark in the text too.
- **New job numbers its steps with a CSS counter**, so a step a task does not need leaves no
  gap, and keeps the estimate and Start in a sticky summary beside the form (Ctrl Enter starts).

## Consequences

- Every visible string still goes through `en.ts`; strings for the apparatus and the old jobs
  list were removed, new ones added for the shell, the readings and the job page.
- DESIGN.md and the `.impeccable` sidecar were regenerated from the shipped code.
- The app icon, still the navy page and quill, is replaced by the sidebar's mark (graphite tile,
  two lines, a highlighter bar), drawn in `src-tauri/icons/source.svg` and generated at every
  size with `npx tauri icon`. It closes the icon question left open in the roadmap.
- The review and analysis review now share more markup (header, lightbox, models list); the
  roadmap's planned extraction of their common frame is easier and still to do.
