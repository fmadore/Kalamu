# 2026-11-16 · Gemini 4 Hydrogen in place of 3.8 Flash, Argon in place of 3.1 Pro

## Context

The [Hydrogen decision](2026-11-12-gemini-4-hydrogen.md) kept Gemini 3.8 Flash as the
recommended model and Gemini's default until 1 January 2027, because Google had named no
successor and Hydrogen costs twice as much until then. Gemini 3.1 Pro stayed offered beside
Gemini 4 Argon for the same reason: Google names no successor for it either, though its own
description already said Argon does the same for less.

Neither is retired: Google still serves both. But with Gemini 4 out, they made New job's Gemini
list longer without being anyone's best choice, and every new pipeline started on a model of the
previous generation.

## Decision

- **`successor` on both**, by the maintainer's choice rather than Google's word, which the
  [successors decision](2026-11-11-model-successors.md) required until now: 3.8 Flash →
  Hydrogen, 3.1 Pro → Argon. New job no longer offers them; a pipeline, workflow step or
  comparison that names one keeps it, marked *replaced*, with its replacement a click away, as
  for Fable 5.1.
- **Hydrogen is the recommended model and Gemini's default** for text extraction and analysis,
  so a new job, and with it every new pipeline, starts on it.
- The catalogue's descriptions of both say which model replaces them and that Google still
  serves them.

## Consequences

- Until 1 January 2027 a new job on Gemini's default costs about twice what it did on 3.8 Flash.
- Hydrogen has still not been compared with 3.8 Flash on real pages, handwriting and refusals
  included; that comparison stays to do, and a researcher who wants 3.8 Flash can still keep it
  in a saved pipeline.
- No code names either model: two catalogue fields and a test changed.
