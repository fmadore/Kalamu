# 2026-11-21 · Screening: long documents, the answer in the rows, the question asked again

## Context

The first increment of research-question screening
([20 November](2026-11-20-research-question-screening.md), [#51](https://github.com/fmadore/Kalamu/issues/51))
left three things a researcher with hundreds of documents meets at once: one document too long
for the model refused the whole job; the answer, its reason and its passages could be read only
by opening each document's review; and a question asked of a project's documents could not be
asked again of the documents added since. The long interviews read in parts
([20 November](2026-11-20-interview-windows-guide-themes.md)) had meanwhile built what the first
needs.

## Decision

### A long document read in parts, as a long interview is

The windows of whole turns were already generic: every analysis with a cited field on a model
with a catalogue limit is given `Setup.windows` and the step that brings the parts together, and
a document read from pages is frozen with its paragraphs as numbered turns. A long document was
in fact already read in parts; what was wrong was the wording, one reading of the answers, and
the one refusal left.

- **Words for a document.** `cited::REQUEST` told the model of turns, times, speakers and "the
  words said". A text whose turns have no speaker and no time (a page's paragraphs, a text
  file's) is now sent `REQUEST_PASSAGES` and, for one part, *this part holds passages 12 to 20*
  (`cited::request_for`, `cited::part_request`, in `core/analysis/cited/request.rs`). A
  transcript is sent exactly what it always was. Neither request enters a cache key (the key
  holds the numbered text, `cited_turns:<sha256>`), so the wording **changes no key**, of an
  interview or of a document.
- **The step's instructions fit both.** `Preset::consolidation_snapshot`'s instructions say the
  parts are made of whole numbered turns, *a speaker's turn in a transcript, a passage in a
  document*, and that each quotation comes with its turn, its speaker and time *or its page*;
  the step's input gives a quotation's `page` when it has one (left out otherwise, so an
  interview's input is unchanged). These instructions are frozen in the step's prompt, which
  enters only the step's own key: a long interview analysed from now on brings its parts
  together under a new step key, and its parts' keys are unchanged. No released build had made
  such a step.
- **Relevance brought together coherently, by the prompt.** The step's instructions say a choice
  is answered for the whole text as the prompt's own instructions say for a text read in parts;
  the screening prompt says it: *Relevant* if any part read is, else *Partly relevant* if any
  part is, else *Unsure* if any part is, *Not relevant* only if every part read is. Changing the
  prompt changes the keys of screenings made with it from now on, as any change of a prompt does.
  Rejected: enforcing the order in code. The engine never names a prompt, and a choice in
  general (a tone, say) has no order; a rule written for a choice's values would be the
  screening's, hidden in the engine.
- **The answers of a document read in parts are the step's.** `projects::answered` read the
  first finished piece of a document, which for a long one was its first part: the filter and the
  answers table would have shown one part's relevance. It now skips the parts
  (`PieceRef::is_window`), as the reviews, exports, search and dictionary already did.
- **Pages survive.** The step can only refer to quotations by number, and Kalamu puts each back
  as the part kept it, its page included; the review names a part not read by the pages it
  covers (*Pages 4 to 7 could not be read*), from `CitedSource.pages`, each turn's page; the files
  name it `part 3 (pages 4–7)`, or `part 3 (passages 48–71)` for a text file.
- **A document one passage of which the model cannot read is left out, not the job refused.**
  The estimate lists every such text (`EstimateRow.too_long` is now a list, each with its
  `source`), and prices the job without them, as a job that leaves them out costs. The screening
  dialog names them with the documents left out for having no text, *Left out, too long for
  Gemma 4 E4B even read in parts: …*, and creates the job over the others. The engine still
  refuses a job holding such a text (`job.text_too_long`), so New job and the interview dialog,
  which send what the researcher chose, are unchanged. Rejected: failing that document's piece
  at run time, which would have shown a researcher a failed job for a choice the dialog could
  make before Start.
