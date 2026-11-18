# 2026-11-18 · Gemini batches through a file, results set aside kept, names far apart, a job to resume

## Context

After [17 November's round](2026-11-17-page-xml-review-sample-claude-batches.md), four more
things the roadmap and the open issues listed needed neither a live run nor a Mac, and their
files did not overlap: Gemini's batches above 20 MB (economy mode's loose ends), what was left
of *What to check* ([#85](https://github.com/fmadore/Kalamu/issues/85)), the names the project
dictionary's proposed merges never compared
([#48](https://github.com/fmadore/Kalamu/issues/48)), and the design points kept from issue 1.
Each was written by an agent in a worktree of its own, reviewed, and gathered on one branch
where every check was run again. A Mac can now be rented, so the two test passes were written
again as well.

## Decision

### Gemini batches through a file

- **Inline where it fits, a file above.** The inline body's size is counted as it would be
  written, without keeping it. Under 20,000,000 bytes the batch goes as before, byte for byte;
  above, its requests are written once as JSON Lines (`{"key", "request"}`), uploaded through
  the Files API's two steps, and the batch made from the file's name. Gemini's own limit rises
  from 13 MiB to what a file allows, so the engine's 48 MiB is what bounds a batch.
- **Two methods on `Provider`, not a provider's name in the engine.** `upload_batch` (by
  default nothing) and `submit_uploaded_batch`. The engine records the uploaded file's id in
  the batch's record (`uploaded`, read as empty from older records, so no migration) before
  the batch is made. A batch refused, stopped while waiting for the rate, or found still being
  sent at the next launch has its upload deleted; a batch tried again after a rate limit is
  not uploaded twice.
- **Deleted by the names the batch gives.** Once ended, the batch names its input file and the
  file Google wrote the answers to; both are deleted through the mechanism of
  [15 November](2026-11-15-records-regions-batch-files.md), with the recorded upload, each
  once. Only a name of the form `files/<id>` is ever deleted, and a 404 counts as deleted.
- **The key goes to the API's own address only**, never to the upload address Google returns.
- **Built from the documentation alone**, read on 18 November 2026 (the batch guide, the Files
  guide and both references on `aistudio.google.com/docs`): inline under 20 MB, an input file
  up to 2 GB, uploaded files deleted by Google after 48 hours. The documentation does not say
  how long the answers' file is kept, nor whether the API may delete it: if it refuses, Kalamu
  tries five times and records the file as left.
- **Left**: the batch itself, which Google keeps and which holds an inline batch's pages and
  answers, is not deleted, as Claude's now is.
- `gemini.rs` passed 500 lines (553) with this: the batch code is in `gemini/batch.rs`
  already, and the file is left whole until the next checkpoint.

### What to check: results set aside, kept

- **Stored per scope, like the sample.** A third table, `review_set_aside`, by its own
  migration; rows go with their job or project. `review_queue` and the manifest's export no
  longer take the list from the interface: what is stored is the only source, so the sample,
  the counts and the manifest cannot disagree with the screen.
- **The sample is drawn without them**, as before, and since they are now kept, a seed, a
  scope and the list set aside give the same pages after a restart.
- **The manifest names them**: a `set_aside` list in the JSON, always written. Version 1
  stays, the field being an addition; the CSV does not change.
- **A seed typed by hand**, six digits at most, refused above that in `projects` as well as on
  the screen; an empty box draws a new one. The limits of size and seed reach the interface
  from Rust.
- **Invalid evidence anchors were not built**, and cannot be yet: an answer holds a value and
  a reason, no quotation and no place in the text, and no prompt asks for one. They wait for
  answers that quote their evidence ([#47](https://github.com/fmadore/Kalamu/issues/47)).

### The dictionary: names far apart in the alphabet

- **Keys, not a dependency.** Each form gives a few cheap keys, written here: the whole name in
  any order, the name as one word or several, a word with the initials of the others (a full
  name meets only a form written with initials), and each word alone; all through a sound key
  of about sixty lines tolerant of transliteration into Latin script, with some thirty
  particles and forty titles left out. A phonetic crate would bring English's rules to names
  that are not English.
- **A key is a reason to look, never a merge.** Rows that share one are sent again together, in
  groups the model reads as it reads the others; the same reader takes their answers, and a
  proposal reached twice is offered once.
- **Bounded.** A key more than ten rows hold makes no group; a group an alphabetical request
  already holds whole is not sent; the strongest keys go first and the extra rows stop at the
  field's own count, so the request costs at most twice the alphabetical groups. Rows kept
  apart are never sent together.
- **Nothing changes for a field that fits one request**: its text is the same byte for byte,
  so stored answers are still found.
- On a generated field of 414 rows in five alphabetical requests, three extra groups of 221
  rows brought together each of the 97 variants the alphabet had set apart.
- **Left**: names that differ by their vowels (Muhammad, Mohamed), shortened forenames, German
  `ch` read as `kh`, and other scripts. What a model makes of these groups, and whether ten is
  the right cap, only a live run shows.

### The interface

- **A job to resume in the sidebar's foot.** Paused and interrupted jobs only, never a
  workflow step waiting for a check; the one stopped last, the others behind a button, so the
  foot keeps one job's height. Resume asks the engine as the job's page does. The focus goes to
  the next job to resume, then the job in the list, never to nothing; a failure is said under
  the row in a live region, the toast staying the window's one alert. The sidebar now scrolls
  by itself, which a short window needed once the foot grew.
- **The error colour.** Its text passes everywhere (5.6:1 at least). The edge of a button that
  cancels or deletes did not (1.3 to 2.0:1) and now does (3.2 to 3.8:1 in both themes). Red and
  the hand blue are too close to tell by colour (1.06 to 1.32:1), so the three are told apart
  by form, as they already were: a failure is words, a correction an underline or a chip, a
  difference a fill. `DESIGN.md` records it. Left: the progress bar's red segment beside the
  ink.
- **Typesetting.** Colons and semicolons go through the translation files, so French sets its
  space before them; the dash of an empty value is one component, with a name a screen reader
  reads. The apparatus's `]` was gone since the redesign.
- **The collation table was not built**: the job's page counts by file and set-up and knows no
  piece's state. It needs a command that gives, for each piece, each set-up's state.

### The test passes

[windows-test-pass.md](../windows-test-pass.md) and [mac-test-day.md](../mac-test-day.md) are
written again for what `main` holds, each step numbered, each with a brief for Claude to take
control of the computer and run it. Claude installs Kalamu from File Explorer, never from its
terminal, which sees a private copy of the user's folders; keys, passwords, system settings
and Narrator's voice stay the maintainer's.

## Consequences

- An economy job with Gemini is no longer cut into batches of 13 MiB; nothing it uploads is
  meant to stay at Google. The privacy policy says so, with a new effective date, and says
  what a project now keeps of the results set aside.
- A large field's request for proposed merges costs up to twice what it did, and says so
  before it is sent.
- Nothing here has been tried in the packaged app, on a Mac, or in a live run. The Gemini file
  route above all rests on documentation: step F4 of the Windows pass is its first real run.
- `rfd` moves to 0.18.0, which the start-up message alone uses.
