# 2026-11-24 · Gemini 4 Neon in place of Argon

## Context

Since [16 November](2026-11-16-older-gemini-lines.md) New job has offered four Gemini 4 models:
Carbon, Neon, Argon and Hydrogen. Argon's price of $2 / $10 per million tokens is introductory
and ends on 30 November; from 1 December it costs $4 / $20, as Carbon does. Gemini 4 Neon,
released on 18 November ([decision](2026-11-19-gemini-4-neon.md)), is newer, sits below Carbon
among Google's top models and costs $3 / $15. With Carbon, Neon and Hydrogen offered, Argon made
Gemini's list longer without being anyone's best choice.

Google names no successor for Argon and still serves it.

## Decision

- **`successor` on Argon**, by the maintainer's choice for version 1.10.1, as on 16 November:
  Argon → Neon. New job no longer offers it; a pipeline, workflow step, comparison or job that
  names it keeps it, marked *replaced*, with Neon a click away. It is not `retired`.
- **Gemini 3.1 Pro keeps Argon as its successor.** A line follows successors to the first model
  still offered (`Catalog::replacement`), and the catalogue's test asks only that the line end
  in one; so 3.1 Pro → Argon → Neon stays true to history, and New job offers Neon in place of
  3.1 Pro.
- The catalogue's descriptions say so: Argon's that Neon replaces it and that Google still
  serves it; 3.1 Pro's that Neon replaces it. Neon's no longer says it has not read a real page,
  which stopped being true when it read the pages below.

## Consequences

- Neon has read one printed page and one handwritten page in Kalamu, no more; on the
  handwriting it reads much as Hydrogen does, at about twice the cost. It has not been compared
  with Argon, and Hydrogen stays Gemini's default.
- A researcher who wants Argon keeps it in a saved pipeline, which runs it while Google serves
  it.
- Until 30 November Argon costs less than Neon; after it, more.
- No code names either model: one catalogue field, three descriptions, the catalogue's date,
  and tests that named Argon as offered changed.
