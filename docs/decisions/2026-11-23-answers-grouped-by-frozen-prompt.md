# 2026-11-23 · Answers grouped by the prompt each run froze

## Context

The repository audit of 23 November found that a question asked again under an edited preset
was taken for the same question ([#241](https://github.com/fmadore/Kalamu/issues/241)). Two
analyses were "alike" when they shared the question, the preset's id, the provider and the
model. A preset edited between two runs keeps its id, so a run made before the edit and one made
after were one group in the filter of the table of documents, a document's newest answer
counting, and one table when exported, every row written under the fields of the run the table
was asked for. A row could not tell which wording produced it, although the choices offered or
the criteria might differ. #51 asks that exports carry the criteria's version: this was the gap.

The same audit found that the table read each job a project's links name once per link when it
was not an analysis ([#244](https://github.com/fmadore/Kalamu/issues/244)); that fix is in
the same pull request and needs no decision.

## Decision

**"Alike" also compares what each run froze.** `core` gives a frozen prompt a fingerprint,
`PromptSnapshot::fingerprint`: the SHA-256 of its instructions as sent (the answer languages are
filled into them, so a change of language changes them), the request sent with each document
(which holds the researcher's question and themes) and an analysis's fields (their keys, labels,
instructions, the choices offered, whether each asks for a reason). It is the part of a cache key
that is the prompt, hashed on its own, so the two never disagree on what a prompt is; the cache
key itself is unchanged, so no stored result loses its key. The preset's id and name, the
set-up's id and label are not in it. `projects::answered::alike` now also requires the same
fingerprint, so a question asked under an edited preset is a group of its own in the filter and
a table of its own when exported.

**Groups are told apart by when they were first asked.** `ChoiceAnswers` carries the group's
`prompt` fingerprint and `first_asked`, when its oldest analysis was made. When two groups of
two analyses would have the same title in the filter, each is followed by "asked since" and that
date and time (`answerGroups.ts`); the fingerprint itself is never shown in the interface, as it
means nothing to a researcher. *Read the new documents against this question* is offered for the
newest version of a question only (same question, provider and model, another prompt): asked
again now, it can only be asked with the preset as it stands.

**Asking again says when the prompt has changed.** The dialog asks the engine for the
fingerprint of the prompt it would freeze (`Engine::prompt_fingerprints`, command
`prompt_fingerprints`; nothing is created or sent) and, when it differs from the group's, says
before Start that the prompt has changed and the new answers will be kept apart. When nothing
changed, the new job froze the same prompt and joins its group as before.

**The table says what each row was read under.** Two columns before `tags`: `job_id`, the
analysis that gave the row's answer (as `provenance.json` and the review manifest name it), and
`prompt_fingerprint`, the first twelve characters of the fingerprint. Within one table it is the
same on every row, since runs under another prompt are another table; across tables it says
which version of the criteria each was read under. The headings stay fixed English words, like
`document`, `file` and `tags`.

## Consequences

- Nothing stored changes: the fingerprint is computed from the prompts the jobs already froze.
  Runs made before this change are regrouped when the project is next opened: those whose frozen
  prompts are the same stay one group; those made either side of an edit of the preset now stand
  apart, each with its own table.
- An update of Kalamu that rewords a ready-made prompt (the screening's included) also starts a
  new group for a question asked again after it, since what was asked changed. The dialog says
  so before anything is sent.
- The fingerprint does not include the reasoning level or the temperature: they change how a
  model answers, not what it is asked.
- Columns are added before `tags`, so a spreadsheet that read the table by position should read
  `tags` by name.
