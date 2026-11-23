# 2026-11-20 · Research-question screening, first increment

## Context

A researcher with 400 scanned pages or 150 articles already read into text asks which of them
speak of their question, why, and with which exact passage
([#51](https://github.com/fmadore/Kalamu/issues/51)). Until now they would have written an
analysis prompt of their own, run it in New job and read the results document by document. The
interview analysis ([19 November](2026-11-19-interview-analysis-design.md)) built what this
needs: the field whose entries cite the text (`FieldKind::Cited`), the engine's check that each
passage is found word for word in the frozen text, and the researcher's question carried as a
set-up option (`SetupOptions.question`) into the request and the cache key.

Three questions were open: whether `Cited` assumed a transcript's turns; how the documents are
filtered by the answer; and where the researcher's own decision is kept, apart from the model's.

## Decision

### A default prompt, nothing else in the engine

`analysis.screening`, *Does it speak of my question?*, beside the other analyses: a choice
(*Relevant*, *Partly relevant*, *Not relevant*, *Unsure*), a short *Reason*, and *Passages*, a
cited field with no fields of its own (up to three). The instructions say that *Unsure* is an
honest answer to prefer to a guess, that a document with nothing on the question is *Not
relevant* with no passage, never one invented to fill the list, and what to answer when no
question was given. The question is the set-up's `question`, as the interview's; another
question is another analysis, the same one costs nothing twice. Nothing in Rust or TypeScript
names the preset but the dialog that starts it and the tests: the engine runs it as any
analysis with a cited field (resumable, cached, costed, refused when a source has no text or is
too long for the model).

### `Cited` on page text: the page copied into each passage

`Cited` did not assume a transcript: a text read from pages is frozen with its turns read as
paragraphs (`snapshot::turns`), each inside one part, so the model already received numbered
passages and the check already found them. What was missing was where a passage stands for a
researcher: a transcript's quotation carries its speaker and time, a page's carried nothing but
offsets. `Evidence` gains `page: Option<PieceRef>`, copied in `cite::place` from the part the
passage lies in when that part is a page, as the speaker and the time are copied from the turn
and never taken from the model. It is written only when known (older answers and transcripts
read unchanged), said in the memo and the text files (`page 3`), on the review's cards (*Page
3*), and in a new last column, `page`, of the quotations table. An entry that is quotations
alone is named in an analysis's own CSV and in a profile's answer column by its quotations,
between quotation marks, where it was empty before (`CitedEntry::name`). The request the engine
adds for a cited field still speaks of turns; the screening prompt calls them passages, and the
two agree on the number.

Rejected: a second kind of field for documents, or a `place` enum of turn or page in the core.
The anchor already says where; the page is one more thing copied from the text, not a new kind.

### The documents' texts: chosen by `projects`, frozen by the engine

`Projects::document_texts` says, for each chosen document, what it is read in: the preferred,
or else the only, result of the first text task that has one (a correction, then an extraction
or a transcription; never a translation, whose passages would not be the document's words); a
text file nothing has read, its own words; and otherwise why not: nothing read yet, or several
readings and none preferred, which the app never chooses between (the export profiles' rule).
The app's `document_sources` command asks the engine to freeze each chosen result
(`snapshot_sources`, as *Use this text* does) and returns the sources with the titles left out.
`projects` only chooses and calls no model; the job is linked to the documents through the
frozen sources' `derived_from`, as an interview's is.

### The filter: any analysis's choice, by document

`ProjectView.choices` lists, for each analysis tied to the project that asked a choice field,
the answer it gave each document, the researcher's correction counting over the model's, with
the question it was asked with. The table of documents gains one select beside the tag filter,
shown only when there is such an answer: a group per choice, named by its question (or by its
field and job), each answer in the interface language (`localiseFields`) with how many
documents gave it. It is generic: a tone analysis's choices filter the table too. That is the
one visible change for a researcher who never screens, and only once an analysis with a choice
is in the project.

Rejected: extending the project search's filters (it searches text and lists matches, not
documents by an answer); a filter on screening alone (it would name a preset).

### The researcher's decision: two tags

Keep and Set aside are shown in the selection bar only while the table is filtered by an
answer, and each is one click on the documents chosen: the tag *Kept* (or *Set aside*), in the
interface's language, given, the other taken off. Tags are already the researcher's layer on a
document (never written by a model, filtered on, renamed, exported), so the model's answer is
never touched and no table, migration, command or store trait is added. The cost is that the two
tags are words like any other: renamed, removed or typed by hand, and named in the language the
interface had when they were given. (Amended on
[23 November](2026-11-23-screening-decision-any-language.md): they are read in every language
the interface ships, and a document carrying both is a conflict to settle.)

Rejected: a decision table beside the corrections (a new store, trait and export column for what
a tag already does); a correction of the choice itself (agreeing that a document is *Partly
relevant* and setting it aside are different acts); the review queue's set-aside (it sets aside
results from a list of what to check, not documents from a question).

### The table: one analysis, one row per document

*Export this table*, beside the filter while an answer is chosen, writes that analysis through
the existing table writers (`table_xlsx`, `table_csv`), as a workbook with a CSV beside it:
`document`, `file`, each field under the label the job froze (a choice's reason beside it when
it asks for one; passages as quotations with where each stands, one a line), then `tags`. Guarded
cells, since a model wrote them. It is `Projects::answers_table`, any analysis's, through the app's
`export_answers` command; no new exporter. A job's own exports are unchanged: each document's
analysis CSV and quotations table are still written beside it.

### The dialog

*Which of these speak of my question?* in the selection bar opens a dialog modelled on *Analyse
this interview*: one box, how many documents will be read and which are left out and why, the
model and the estimated cost, Start disabled until a question is typed; another model and the
language of the answers under *More choices*. Start asks the providers' consent as New job does,
creates the job, ties it to the project and opens it. The interview dialog's code is not shared,
since another increment is changing it; the few lines both need are written twice.

## Consequences

- Privacy: the text sent is a document's text and the question typed, to the provider chosen,
  after the same consent as any job. A document's text could already be sent to an analysis; a
  sentence typed by the researcher was named only for the interview, so `docs/privacy.md`
  should say that the question asked of a project's documents goes with their text.
- The quotations table of every cited analysis has one more column, `page`, empty for a
  transcript; the interview's golden files changed by that column only.
- `DocumentTable.svelte` passed 500 lines: its selection bar moved to
  `DocumentSelectionBar.svelte`, which also holds the dialog and the decision.
- A whole project's documents are frozen as the dialog opens: for 400 documents, 400 snapshot
  files, named by their content, so a second screening of the same texts writes none.

## What is left

- A document too long for the model refuses the whole job (the engine's rule for cited
  analyses); leaving that document out, or reading it in windows, waits for the interview's
  second slice.
- The question kept with a prompt for the next screenings (the interview's *Keep this*), and the
  question asked again over new documents of the project.
- The answer, the reason and the passages shown in the table's rows, without opening each
  document's review; Keep and Set aside from the review itself.
- The filter remembered on the computer with the tag and the sort.
- A translation or an analysis's corrected answers as the text read; a project's documents of
  images read as one document (they are left out with no text when nothing read them).
- New job offers the screening prompt with no box for the question: run there without one
  kept in Prompts, every answer is *Unsure*, as the prompt says.
