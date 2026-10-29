# 2026-10-29 · Error rates after review, for any job

## Context

Comparison mode scored each set-up against the researcher's corrections, in a character error
rate shown at the foot of the review, ported from zmo-ai-pipelines' `zmo_evaluate.py`. A
researcher reviewing a single model's output, the common case, got no score at all. And the
score had two flaws: only *corrected* pages counted as references, so a page checked and found
right was left out and the rate overstated errors; and each page's rate was averaged, so a short
page with two errors weighed as much as a full one. The word error rate was computed and never
shown.

## Decisions

- **A reference is any page the researcher corrected or marked as reviewed.** A corrected page
  is scored against its correction; a page reviewed without changes against the model's own
  text, so it counts as no errors. In a comparison, one page's reference serves every set-up,
  and a correction comes before a text reviewed as it stands (then the first set-up's).
- **Rates are summed, not averaged**: edits over all the scored pages divided by all their
  reference characters (or words), as evaluation tools usually report them. White space is
  collapsed before counting characters, so a line break kept or joined is not an error.
- **Extracted text, transcriptions and corrections are scored**; analyses and tables of
  contents answer in fields, and their agreement is a separate item of the roadmap.
- **One pass for both views.** `Engine::job_scores` gives the rates for the job and each
  document, `Engine::compare` for one document; both read the stored results and corrections
  only (`scored_pieces`), without opening the source as the review does.
- **Said in plain words.** The job page gives each set-up's rates for the job among its facts
  and for each document under its progress bar; the review's foot for the document open
  ("1.4% of characters and 5.2% of words wrong (about 1 character in 71), on 12 reviewed
  pages"; "parts" for a recording). The "one in" figure is rounded as people say it, and the
  page says that only errors the researcher caught are counted, the reference being their
  correction of the model's own text.

## Consequences

- A comparison's scores change for pages that were reviewed without corrections, and wherever
  pages differ in length.
- Tested in the engine (pooling against hand-counted edits over two documents, the reference
  rule, a line break not counting) and in TypeScript (the sentences and rounding).
- Not yet in `provenance.json` or an export: a CSV of each page's rates for a methods section
  would be the next step if researchers ask for it.
