# 2026-11-02 · Merges proposed by a model in a project's dictionary

## Context

The [first increment of the dictionary](2026-11-01-project-dictionary.md) left the researcher
to find every variant by hand, with only look-alike spellings pointed out. #48 asks for a
model to propose merges: the inventory sent in bounded groups with overlap and explicit
coverage, references to existing rows checked, an estimate first, nothing applied without the
researcher, and the pairs kept apart never offered.

iwac-ai-pipelines' authority curation (`AI_authority_curation`) has a model review each
candidate pair its fuzzy matching found, with the documents that name both; every verdict is
checked (evidence only from the documents supplied), an invalid one is never applied, and a
curator approves each row of the plan. Its rules (a spelling variant is not proof, a title or a
surname alone is not enough) are in Kalamu's prompt. Kalamu differs in asking for groups among
a sorted list rather than verdicts on pairs found beforehand: the first increment already
points out what folding finds, and the model is there for what folding misses.
zmo-ai-pipelines and Live Translation reconcile nothing.

## Decisions

- **An analysis job, not a new task.** The request is an ordinary analysis of text the app
  writes itself, answered in an entries field, as a table of contents' second step already is.
  It goes through the engine, so caching, resuming, economy mode, cost and provenance are those
  of any job, and it is listed in Jobs. A new `Task` would have touched every crate and the
  interface for something New job never offers. The texts are kept by their content beside the
  frozen text (`Engine::text_sources`), so asking again about unchanged rows costs nothing.
- **A prompt of its own, editable** (`dictionary.merges`). Prompts lists it under Dictionary,
  and the request links to it, so the researcher can read what is sent and change what counts
  as one person or term, then reset it; it is frozen into each job and its provenance like any
  prompt. Its answer's fields are fixed, since the dictionary reads them, and New job and the
  workflow steps never offer it, since it reads the dictionary's rows, not a document.
- **Rows by reference, kept by their forms.** Each row is `R1`, `R2` in its text, and the
  request keeps each row as the forms it held. The answer is read against them when shown, as
  the rows stand then: a later merge or split does not break a proposal. A reference to no row
  is counted and dropped; a group of one row, or of rows already one entry, is no proposal;
  rows kept apart are never offered together, whatever the model says.
- **One request for the whole field, as a rule.** The rows go together up to about 100,000
  tokens of input (a token counted as three characters, which errs on the long side), two to
  three thousand rows, which every model offered for analyses reads with room for the answer;
  a model compares best what it sees at once. A larger field is cut into requests of that size,
  in alphabetical order with capitals and accents set aside, each sharing a tenth of its rows
  with the next; names far apart in the alphabet (`Ouagadougou`, `Wagadugu`) are then not
  compared, and the interface says so before sending. The catalogue has no context window per
  model yet; with one, the budget could follow the model chosen.
- **Context, not documents.** Each row carries its other spellings and the titles of up to
  three documents naming it, so a model can tell two people apart by where they appear; the
  documents' text is not sent. The privacy policy says so.
- **Acceptance is the ordinary merge.** The interface calls `dictionary_merge` with the rows
  ticked and the label as edited, so a proposal follows every rule of a merge by hand (kept
  apart refuses it, the oldest entry keeps its id) and is undone the same way. Turning a
  proposal down is kept by the forms it would join, survives a restart and can be taken back.
  Certainty is the model's word (certain, likely, possible), shown as such and never as a
  number.

## Checkpoint

- **Dependencies still point inwards.** `dictionary` depends on `core` only and reads the
  proposal job's answers through `JobStore`; it never reaches the engine. The app wires the two:
  the dictionary prepares the texts, the engine runs them, the dictionary keeps the request.
  `prompts` gains a module of its own for the dictionary's prompt; `core` holds the answer's schema,
  as it holds a table of contents'.
- **Resuming, caching, progress, provenance** cover the new path, since it is a job.
- **Size.** `dictionary/src/proposals.rs` is just under 500 lines; reading the answers is the
  first thing to move out if it grows.

## Consequences

- The dictionary gains *Ask a model to propose merges*, the requests made with their state,
  and the proposals of the field shown.
- The app sends something new when asked: the privacy policy, effective 2 November 2026,
  lists it, and the store keeps the requests and the proposals turned down.
