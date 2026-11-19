# 2026-11-19 · Gemini 4 Neon, and Escape while a model answers

## Context

Google released Gemini 4 Neon (`gemini-4-neon`) on 18 November 2026
([announcement](https://blog.google/innovation-and-ai/models-and-research/gemini-models/introducing-gemini-4-neon/)),
stable. Google sets it between its two other lines: "Carbon tackles your hardest, long-horizon
projects and Hydrogen handles fast everyday execution". It names no model Neon replaces and
deprecates nothing.

- **Prices**: $3 / $15 per million tokens, $0.15 cached, half in batch mode, flat across its
  1M-token context. That is twice Hydrogen ($1.50 / $7.50), and below what Carbon and Argon cost
  from 1 December ($4 / $20).
- **Inputs**: text, images, PDFs, audio and video, as the other Gemini 4 models.
- **One request writes at most 65,536 tokens**, as Hydrogen; more only through long decodes,
  which Kalamu does not use ([12 November](2026-11-12-gemini-4-hydrogen.md)).
- **Thinking levels**: `low`, `medium`, `high` (the default) and `xhigh`, as Hydrogen.
- Google says it "reads uploads like spreadsheets, PDFs, and photos more precisely"; the
  announcement says nothing of handwriting, older print or transcription.

## Decision

- **Neon is a catalogue entry, and nothing else changed**: no code names a model. It takes
  Hydrogen's inputs, limits and levels.
- **Its tier is `top`**, after Carbon and before Argon, and it is marked costly like them.
  Google's own page on choosing a model
  ([Gemini Help](https://support.google.com/gemini/answer/18648722)) calls Hydrogen the model
  that "balances performance and responsiveness", Neon "frontier-level intelligence, with high
  performance for deeper analysis and complex workflows", and Carbon "our strongest model";
  its plans open Hydrogen first, then Neon, then Carbon. A first version of this entry classed
  Neon `balanced` from the announcement alone.
- **Hydrogen stays Gemini's default** and the recommended model: it costs half, and Neon has
  not read a real page in Kalamu. Neon's description says so.
- **No `successor` anywhere**: Neon replaces no model, by Google's word or by price.

## Escape while a model answers

The two dialogs that ask a model from the review differed: Escape stopped the wait and closed
the dialog for a passage, and only stopped it for a region
([checkpoint](2026-11-18-checkpoint-requests-exports.md)). Both now do what the *Stop* button
does: **Escape stops the wait and the dialog stays**, saying it was stopped and that an answer
on its way may still be charged; a second Escape closes it. A key pressed once should not both
give up a request that may be billed and hide the sentence that says so. The rule lives once,
in the state the two dialogs share (`ModelRequest.escape`).

## Consequences

- An installed copy learns of Neon from the model list it downloads, without an update.
- Neon joins the roadmap's live runs: handwriting, older print and refusals against Hydrogen,
  before it is recommended for anything.
- Carbon's and Argon's introductory prices end on 30 November: the catalogue's prices for both
  are due on 1 December.
