# 2026-11-21 · Estimates count the model's thinking, newspaper pages and each document

## Context

The Windows test pass of 1.10.0-rc1 billed far more than New job had estimated on printed
newspapers and on screening:

| Job | Estimated | Billed |
| --- | --- | --- |
| 36 newspaper pages, Gemini 4 Hydrogen (*Newspaper or magazine*) | about $0.35 | $1.21 |
| A 4-page issue, Hydrogen and GPT-6.2 Sol | $0.092 | $0.31 |
| A 3-page sample of the 36 | $0.030 | $0.13 |
| A table of contents of a 4-page issue | $0.029 | $0.058 |
| Screening 10 corrected texts (nine of about 4,800 words), Hydrogen | $0.018 | $0.20 |
| A handwritten page of 1755, Claude Sonnet 5.5 | $0.014 | $0.021 |

Three things in `catalog::cost` explained it. Every page was priced at 900 tokens of output,
whatever it held, with nothing for what a model thinks before writing, which providers bill as
output even at the lowest level Kalamu sends: a newspaper page was billed 4,000 to 5,500 tokens
of output, about 2,000 of them its text. An analysis was priced as one answer for the whole
set-up, its instructions sent once, however many documents it read, though each document is a
call of its own. And a text was priced as pages of 2,500 characters at 1,100 tokens each,
nearly twice what it is sent as.

## Decision

- **Thinking is counted**, at the reasoning level the set-up sends (the suggestion when none is
  chosen): an allowance measured at `low`, half of it at `minimal`, none at `none` or for a
  model that does not reason, two to five times it from `medium` to `max`.
- **A dense prompt's pages are priced as dense pages** (`Preset::is_dense`, the newspaper's):
  2,000 tokens of text and 2,500 of thinking at `low`, where an ordinary page is 900 and 600.
- **An analysis is priced document by document**: each document's text at four characters a
  token (its pages as pictures at 1,100), its instructions, and 700 tokens of answer with 800 of
  thinking at `low`. A table of contents' page is 500 tokens of entries with an ordinary page's
  thinking; a records page 400 of rows with the same, as before in all.
- The figures lean to the high side: each case above is now estimated between 0.85 and 1.3
  times what it was billed (`catalog` and `engine` tests pin them). Audio is unchanged: it was
  estimated as billed.

## Consequences

- New job, the screening dialog, a sample's *Run the rest* and every estimate that goes through
  `Engine::estimate` show the new figures; nothing in the interface holds a figure of its own.
- The catalogue needs no new field: a model's reasoning levels and the prompt's density are
  enough. A model whose thinking runs far from the allowance (a heavy thinker at its lowest
  level) can be given a field of its own when measured.
- The first screening estimate of the test pass ($0.018) cannot be reproduced from the engine's
  formula with the texts described, which gives about $0.18 before this change, as the dialog
  showed when opened again; it was a tenth of that, as if the texts first priced were a tenth as
  long. Left to watch in the next test pass.

Followed by the [entry of 22 November](2026-11-22-estimates-gemini-thinking-text-layer.md),
from the rc2 test pass: each model's thinking on a page, small print known from the text
layer, an interview's longer answer and a region priced as its share of the page.
