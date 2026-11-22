# 2026-11-22 · Estimates: each model's thinking on a page, small print from the text layer, interviews and regions

## Context

The [entry of 21 November](2026-11-21-estimates-count-thinking.md) brought the rc1 cases within
0.85 to 1.3 times what they were billed. The Windows test pass of 1.10.0-rc2 measured others
that still ran short, all at low thinking (the usage is in each job's `provenance.json`):

| Job | Estimated | Billed | Usage |
| --- | --- | --- | --- |
| Three set-ups on a 1-page newspaper PDF and a handwritten page of 1755, printed-document prompt | $0.12 | $0.27 | |
| · GPT-6.2 Sol | $0.038 | $0.062 | page 12,497 in, 2,495 out; 1755: 4,033 in, 404 out |
| · Gemini 4 Hydrogen | $0.029 | $0.063 | page 5,374 in, 5,127 out; 1755: 2,185 in, 1,783 out |
| · Gemini 4 Neon | $0.057 | $0.140 | page 5,374 in, 5,486 out; 1755: 2,185 in, 2,327 out |
| Interview analysis, Hydrogen, a transcript of about 250 words | $0.013 | $0.024 | 3,082 in, 2,521 out |
| A strip of two lines of the 1755 page read again, Claude Sonnet 5.5 | about $0.020 | not run | the whole page cost $0.021 |

Cases near or under, to keep so: a records analysis of a 4-page issue with Hydrogen ($0.040 for
$0.038), a translation of 207 words ($0.016 for $0.013), GPT-6.2 Sol's *People, places and
organisations* on texts of about 5,000 words (about $0.030 for $0.019 to $0.022), Mistral OCR
($0.004 a page).

Four causes:

- **Models think very differently on a page at their lowest level.** Gemini 4 thought about
  1,400 (Hydrogen) and 1,900 (Neon) tokens on the handwritten page, where every model was given
  600; GPT-6.2 Sol and Astra thought almost nothing (404 and 387 tokens out, the text included).
- **The newspaper page was read with the printed-document prompt**, so it was priced as an
  ordinary page. Its PDF has a text layer of about 1,940 words: the page itself says it is small
  print. Gemini also read that text layer beside the picture (5,374 tokens in, where a picture
  is 1,100), GPT-6.2 Sol a large picture in many tiles (12,497).
- **An interview's answer is long**: themes with a description and an origin, quotations worth
  keeping with a reason, each copied word for word, besides the summary and keywords. It was
  priced as any analysis's 700 tokens of answer.
- **A region was priced as a whole page**, though only its picture is sent.

## Decision

- **A model's own thinking on a page, where measured**: `ModelInfo::page_thinking`, tokens at
  `low`, replaces the allowance of 600 on an ordinary page and scales with the reasoning level
  as it does; a dense page keeps the larger of it and the dense allowance (2,500). Measured:
  Gemini 4 Hydrogen 1,400, Gemini 4 Neon 1,900, GPT-6.2 Sol and Astra 100. Not yet measured,
  and set from their siblings so they lean high: Gemini 4 Carbon 1,900 (as Neon), Gemini 4
  Argon 1,400 (as Hydrogen; its provider says it thinks with fewer tokens). Every other model
  keeps the allowance. It counts for reading a page (and a region), not for analyses, whose
  measures were near. The catalogue's date moves to 2026-11-22.
- **Small print known from the text layer**: when a PDF is added, the words of its first ten
  pages' text layer are counted (as the input checks sample them) and a page of more than 900
  is small print (`SourceFile::dense_pages`, scaled to the whole document). An estimate prices
  those pages as dense whatever the prompt, a sample or a spread's halves in proportion. A
  dense page is also read as 3,000 tokens (the text layer or the large picture), where an
  ordinary one is 1,100. A newspaper scanned as images has no text layer: only the newspaper
  prompt says it is dense.
- **An interview's longer answer**: each field whose entries have fields of their own and quote
  the text (an interview's themes and quotations) adds 500 tokens to an analysis's answer, in a
  document read whole, in parts and in the step that brings parts together. A screening's
  passages, quotations alone, add nothing, so screening is unchanged.
- **A region is priced as its share of the page** (`Workload::region`, its width by its height):
  its picture, text and thinking, at least a fifth of a page; the instructions whole. A model
  priced by the page (Mistral OCR) bills a whole page.

## Consequences

The rc2 cases, as `catalog`'s `estimates_meet_what_the_rc2_test_pass_was_billed` pins them from
the providers' usage (estimated over billed):

| Case | Estimated | Billed | Ratio |
| --- | --- | --- | --- |
| Three set-ups together | $0.256 | $0.265 | 0.96 |
| · GPT-6.2 Sol | $0.067 | $0.062 | 1.09 |
| · Gemini 4 Hydrogen | $0.060 | $0.063 | 0.95 |
| · Gemini 4 Neon | $0.128 | $0.140 | 0.92 |
| The handwritten page alone: Sol, Hydrogen, Neon | | | 1.18, 1.23, 1.17 |
| Interview analysis, Hydrogen | $0.021 | $0.024 | 0.91 |
| A strip of two lines, Claude Sonnet 5.5 | about $0.006 | | the whole page: $0.020 for $0.021 |
| Records, Hydrogen, 4 pages | $0.041 | $0.038 | 1.08 |
| Translation of 207 words, Hydrogen | $0.016 | $0.013 | 1.16 |
| Mistral OCR, a page | $0.004 | $0.004 | 1.00 |

GPT-6.2 Sol's *People, places and organisations* is unchanged (about $0.031 for a 5,000-word
text, billed $0.019 to $0.022): a little high, as before. The rc1 cases stay within 0.85 to 1.3:
the 36 newspaper pages are now estimated at $1.43 for $1.21 billed in rc1 (the nine issues cost
$1.35 together in rc2), the issue read by two models $0.37 for $0.31.

- The source's count is kept with it, so a job, its sample and its *Run the rest* price the
  same pages alike; a PDF added before this version has none and is priced by its prompt, as
  before.
- Counting the text layer adds to what adding a PDF reads: the input checks already extract
  the same ten pages' text.
- GPT-6.2 Sol reads a newspaper page as a far larger picture than Gemini (11,000 tokens against
  3,400); its small thinking leaves the dense pages' output allowance to cover that, about 1.1
  to 1.3 times what it was billed. Input per model can be measured the same way if it drifts.
- Claude Sonnet 5.5 sends a page as about 5,000 tokens and thinks about 250: left on the
  allowance, which balances the two (0.96 on the 1755 page).
