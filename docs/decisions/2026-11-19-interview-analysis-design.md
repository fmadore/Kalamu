# 2026-11-19 · Design checkpoint before post-interview analysis

A checkpoint before a feature, as `AGENTS.md` asks: the specification, the roadmap and issues
[#38](https://github.com/fmadore/Kalamu/issues/38) and
[#47](https://github.com/fmadore/Kalamu/issues/47) read again against the code as it is. Nothing
here is built yet; this entry says what will be, in what order, and what was set aside.

## What the researcher gets

From a transcript they have corrected, one choice, **Analyse this interview**, beside the
existing *Use this text*. It asks one thing, *What is your research about?*, in a sentence or
two, and offers the default model. The answer is a page with a short summary, the themes of the
interview, its keywords and the passages worth quoting. Every theme and every quotation shows
the speaker's words with their name and the time; a click plays the recording there. Nothing is
shown as a quotation unless Kalamu found those exact words in the transcript.

Everything else is folded under one *More choices*: a list of the researcher's own themes to
look for, another model, the language of the answer. The default path has one box and one
button ([plain first](2026-11-18-drawn-regions-region-read-again.md)).

## What the code already has

- **The frozen transcript**: `Engine::snapshot_sources` writes the text as the researcher left
  it, named by its hash, read as an ordinary text source; `derived_states` says when the
  transcript changed since. The entry point (`UseTextMenu`) and the tie to the document
  (`derived_from`) exist.
- **Anchors**: `core::snapshot::{Anchor, locate, check}`, in characters, tested on repeated
  phrases and Arabic. **Nothing calls them**: the roadmap says quotations "can be anchored and
  checked", which is true of the functions and of no result.
- **Asking again**: an answer that breaks its fields is refused, charged and asked again
  within `max_attempts`. The check for a translation's numbered lines is a special argument of
  `Engine::call` (`lines`), not a hook.
- **Review as a layer**: `Correction.fields` and the analysis review's operations on a list
  (edit, rename, remove, add).
- **Play at a time**: the audio pane's `seek`, as the search's hits use it.

## What is missing

1. **No answer can hold evidence.** A field's answer is a value and a reason. A group of
   entries (`FieldKind::Entries`) must be a schema's only field, and makes the job read its
   source record by record: it cannot carry themes beside a summary.
2. **A snapshot does not know its turns.** `Snapshot.lines` holds the turns only when every
   piece has times, with their spacing tidied, and nothing says where a turn sits in the frozen
   text. The times written in the text are those of a piece of the recording, not of the
   recording.
3. **An analysis reads its text whole**: no windows, no consolidation.
4. **No export writes one row per quotation.**

Where issue #38 no longer matches the code: workflows now chain jobs (it says pipelines
cannot); speakers' names already hold for a whole recording; nested entries cannot be reused as
it suggests (point 1).

## Decisions

### An interview analysis is an analysis job

Not a new task, and not a third kind of request made from the review
([18 November](2026-11-18-checkpoint-requests-exports.md)): that lifecycle is for one call
about a piece being reviewed. An analysis job over a snapshot is already resumable, cached,
costed, in the provenance and tied to its document. A new task would touch every exporter's
`supports`, every model's tasks in the catalogue and the engine's rules, for nothing a field
kind does not give.

### One new kind of field: entries that cite the text

`FieldKind::Cited { fields, max }`: a list of entries, each with its own small fields (a
theme's name and description; a quotation's reason) **and one or more quotations**. Unlike
`Entries` it sits beside other fields and changes nothing in how the source is read.

- The model answers, for each quotation, the number of the turn and the words. Kalamu looks
  for the words in that turn, then in the turns around it, then in the whole text
  (`snapshot::locate`), and keeps an `Anchor`. **The speaker and the time are copied from the
  snapshot in Rust, never taken from the model.**
- Matching forgives what a model changes without meaning to (spacing, the kind of apostrophe
  or quotation mark, an ellipsis for three dots) and nothing else; the stored quotation is the
  transcript's own text between the anchor's ends, not the model's copy.
- Words found nowhere: the answer is refused and asked again once, like any broken answer. A
  quotation still not found is **left out**, and counted in the result ("2 quotations could not
  be found in the transcript and were left out"). An entry left with no quotation is kept only
  if its schema allows one without.
- The value is `FieldValue::Cited(Vec<CitedEntry { answers, evidence: Vec<Evidence> }>)`, with
  `Evidence { anchor, quote, turn, speaker, start_ms, end_ms }`. Older builds do not know the
  kind: `FieldKind` and `FieldValue` are checked for how an unknown variant reads before the
  first line is written, and the pipeline version moves if it must.

One kind serves the three findings (themes, keywords in context, quotations), and later the
screening of documents with verified passages (#51) and the dates and places proposed with
their evidence (#54), which wait on exactly this.

`Engine::call`'s `lines` argument becomes what it is one case of: **a check an answer must
pass**, given by the caller, with the reason sent back to the model. Numbered lines and cited
quotations are its two first implementations.

### Turns in the snapshot (what #38 needs of #47)

`Snapshot.turns: Vec<Turn { start, end, speaker, start_ms, end_ms }>`: each turn's place in the
frozen text, in characters, with its times on the recording's clock when it has them. Built
with the text in `snapshot::build`, so the two cannot disagree. A transcript without times
still has turns (its speaker lines, or its paragraphs). A turn's number is its place in the
list; with the snapshot's id that is the stable identity #47 asked for. `lines` stays for the
translation that reads it. A snapshot made before has no `turns`, and an interview analysis
always makes its own.

### The research profile is the job's question, at first

The sentence typed in the box becomes part of the job's user prompt, which the cache key
already covers: another question is another analysis, the same question costs nothing twice.
A list of the researcher's own themes travels the same way. **Saved profiles** (a project's
question and themes, reused over its interviews) are a later slice, in `prompts` beside
pipelines, not a new store.

### One request first; windows after

An hour of interview is about 55,000 characters, some 15,000 tokens: every model the catalogue
offers reads it in one request. The first slice analyses a transcript whole, up to a length the
catalogue's limits allow, and says plainly when one is too long. **Windows of whole turns and a
consolidation step** come second, on the pattern of the table of contents (`Setup.merge`, a
`Whole` piece keyed by its parts): the consolidation receives findings whose quotations are
already checked and may only group them by their numbers; a window that failed is named in the
result (which minutes were not analysed) rather than passed over.

### Review, and what is exported

- **Review**: the analysis review gains one component for a cited field: a card per entry with
  its quotations; a quotation opens the transcript at its turn and plays it. Editing, renaming
  and removing an entry or one of its quotations are the list operations that exist, kept in
  `Correction.fields`; the model's answer is never changed. Adding a quotation by selecting
  words in the transcript, checked by the same `check`, is in the second slice.
- **Out of date**: when the transcript is corrected after the analysis, the page says so and
  offers to analyse again; the findings keep the snapshot they were made from.
- **Exports**, each a module of its own: the Markdown memo (summary, themes with their
  quotations, speaker and time); JSON with the anchors and the snapshot's id and hash; and
  `…-quotations.csv`, one row per quotation, written by `table_csv`.

### Privacy

The transcript's text and the question go to the provider chosen, as for any analysis of a
text; with a model on this computer nothing leaves it. `docs/privacy.md` names the question
among what is sent, with a new date, when the first slice ships.

## Order of work

1. **Foundations** (no visible change): turns in snapshots; the answer check as a hook;
   `AnalysisReview.svelte` (592 lines), `NewJob.svelte` (874) and `prompts/defaults.rs` (697)
   brought under 500 lines before anything is added to them.
2. **First slice**: `Cited` in `core`, its schema and reading in `providers`, the check and the
   re-ask in `engine`; the preset `analysis.interview`; *Analyse this interview* and its one
   box; the cards with play; the three exports; golden files; a mock view; the documents.
3. **Second slice**: windows and consolidation with coverage; the researcher's own themes;
   saved profiles; a quotation added by hand.
4. **Later**: themes across the interviews of a project, with the dictionary's patterns.

## Set aside

- A new `Task`, and a third request lifecycle: see above.
- Fuzzy matching of quotations (a model's paraphrase accepted as a quotation): what is shown
  as someone's words must be their words.
- Roles for speakers (interviewer, interviewee) as data: the question can say who is who; a
  role per speaker waits for a need the names do not meet.
- Qualitative coding in the full sense (a codebook applied line by line, agreement between
  coders): Kalamu prepares and points to passages; the coding stays the researcher's, in the
  tool they use for it.

## Checkpoint findings beside this feature

- Dependencies still point inwards; nothing in this design adds an edge between crates.
- The roadmap's sentence on anchors is corrected to say they are not yet used.
- Nine Rust files and eight interface files are over 500 lines; the three this feature touches
  are split first (above), the others at the checkpoint that touches them.
