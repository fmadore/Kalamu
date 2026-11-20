# 2026-11-20 · Long interviews read in parts

## Context

A social-science interview runs from an hour and a quarter to two and a half hours. The first
slice of interview analysis ([19 November](2026-11-19-interview-analysis-design.md)) read a
transcript whole in one request, and refused before Start one too long for the model chosen: a
model on the computer stops near an hour, so the longest interviews could not be analysed on it
at all. The design left *windows of whole turns and a consolidation step* for the second slice,
on the pattern of the table of contents (`Setup.merge`, a `Whole` piece keyed by its parts), the
consolidation receiving findings whose quotations are already checked and grouping them by their
numbers.

## Decision

- **Parts are pieces.** `PieceRef::Turns { number, first, last }`, turns counted from 1 as the
  model is sent them. Each part is sent with its turns numbered as in the whole text (the same
  `numbered_line` the whole text is made of), and its quotations are looked for in the whole
  text by the same check as a text read at once: nothing in how a quotation is found changes.
  The step that brings the parts together is the `Whole` piece after them, keyed by the parts'
  keys and by the parts that could not be read (`CacheKey::combined`, of which a table of
  contents' `CacheKey::merge` is now one case, its keys unchanged).
- **How a set-up reads a long text is settled when the job is created**, from the catalogue's
  `max_input_tokens` at four characters a token: `Setup.windows` (`TurnWindows`: the most the
  model reads, the most one part sends, seven tenths of that, and two turns repeated) and
  `Setup.merge`, the step on the set-up's own model with its own instructions
  (`Preset::consolidation_snapshot`: what to do with the parts' findings, then the prompt's own
  instructions, then its fields with each field of cited entries answered as entries with the
  numbers of their quotations). Both are settled for every analysis whose fields cite the text
  on a model with a limit, as `pages_cap` is, so that a workflow's later step, whose text does
  not exist yet, reads a long one in parts too; whether a text is read in parts is decided when
  it is read. Neither enters a cache key: a transcript that fits is read whole, in one request,
  under the key it always had, and its `provenance.json` names no step after parts.
- **Parts as even as the text allows**: as many as the text needs at the share of the limit, the
  turns spread evenly over them, each part repeating up to two turns of the one before (never
  more than a third of a part, never the whole of it) so a passage across a cut is read whole in
  one of them. A turn longer than a part is a part of its own without repetition. **Only a text
  one turn of which the model cannot read alone is refused** (`job.text_too_long`, and the
  estimate's `too_long`, as before).
- **Quotations by number, never by words.** The step is sent each part's answers and every
  quotation the parts kept, numbered once (`Q1`, `Q2`…): the same quotation found by two parts
  that overlap is the same anchor, and one number. The step answers each entry with numbers;
  Kalamu puts back each number's evidence as the part kept it (`consolidate::resolve`), so the
  words, the turn, the speaker and the time are the transcript's, found by Kalamu, before and
  after the step. Words written in place of a number, or a number past the last, are left out
  and counted with the quotations not found; a quotation lying inside another of the same entry
  is kept once, the longer; an entry left with none is left out and counted. An answer that gave
  numbers and not one Kalamu knows is refused and asked again, as one whose quotations are all
  missing is. Referring by identity was chosen over checking the words again after the step: a
  check would accept a quotation the step moved to words of the transcript no part chose, or
  cut differently, and would refuse an honest regrouping for a changed apostrophe.
- **A part that failed does not stop the others.** Unlike a table of contents, which has nothing
  to write without every page, the step runs once every part is read or failed, with at least
  one read, and is told which turns were not read. Its stored record lists `turns_read` and
  `turns_not_read`; the files name the part where it stands (`[Not read: part 3 (turns 48–71,
  01:02:10 to 01:20:00): …]`); the review says it in a notice with *Read it again*, which puts
  the failed part back in the queue (`retry_failed`), and the step runs again under its new key.
- **One machine for both second steps.** `merge_pages` became `second_step` behind a small trait
  (`SecondStep`: its key's name, whether a failed part fails it, the sentence when nothing was
  read, and its input), implemented by `CombinePages` and by `BringTogether`; the answer check
  of the latter is an `AnswerCheck` like the quotations' and the timed lines', which can now
  add to the stored record what the request carried (`AnswerCheck::settings`). `process_piece`
  takes such an input with its check (`Input::Checked`).
- **The parts stand behind the whole.** Reviews, exports, a project's search and documents, the
  dictionary, the agreement table and the list of what to check read the step's answer and skip
  the parts (`PieceRef::is_window`); a change from the review reaches the step's piece whatever
  index it names (`windows::answered`), so a comparison of a set-up that read in parts with one
  that read whole changes both.
- **The cost before Start** prices every part (the characters sent, repeated turns included, the
  instructions with each, an analysis's answer each) and the step apart, as a table of contents'
  second step is (`EstimateRow.parts`, and a `merge` row). Never in an economy batch: no cited
  analysis is, and the step needs the parts' answers.

## Consequences

- An older build opening a job made by this one ignores `windows` and reads a long text whole,
  which its provider then refuses; no released build opens a newer database.
- A model's limit in the catalogue now decides how many requests a long interview costs, not
  whether it can be analysed: a wrong limit costs a part more or fewer, never a quotation.
- Exports of a long interview are those of a short one; the only new shape is a part not read.

## Left

- Research profiles: a project's question and themes reused over its interviews.
- Several interviews compared: themes across the interviews of a project, with the
  dictionary's patterns.
