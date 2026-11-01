# 2026-11-01 · Each model's own output limit, and a reasoning level per set-up

## Context

Kalamu capped every answer with a limit of its own per provider (16,000 tokens for Claude,
32,000 for OpenAI, 65,535 for Gemini, 16,000 for Mistral's chat models) and fixed the
reasoning level per provider (Claude `low`, Gemini `low`, Mistral the lowest, OpenAI each
model's default). The maintainer asked for neither: an answer should be allowed the whole
output the model is documented to give, and the researcher should choose how much each model
thinks, the lowest suggested for OCR and HTR. Sending several pages in one call (the next
change) depends on the first: a cap of Kalamu's own would cut a long answer for no reason.
iwac-ai-pipelines sets these per script, as constants; nothing there chooses them per model.

## Decisions

- **The catalogue holds both, per model.** `limits.max_output_tokens` is the provider's
  documented maximum, checked on 1 November 2026: 65,536 for the four Gemini models, 128,000
  for the four OpenAI models and the four Claude models, 128,000 for DeepSeek V4.1 Flash.
  `reasoning` lists the levels the model accepts, least first, and the provider's default where
  documented. OCR and speech-to-text models have neither. A model is updated by editing its
  catalogue entry, which the app refreshes from the repository.
- **Where a provider documents no limit, none is sent.** Mistral's chat models (Medium 3.5,
  Large 4, Small 4, GLM 5.3 and its Flash) have no published output maximum: `max_tokens` is
  left out and the model writes until it is done or its context is full.
- **The level is the researcher's, with a suggestion.** New job shows *Thinking* for a model
  that reasons, set to the suggestion: the lowest level for reading pages, recordings and text
  to correct, `low` for analyses and tables of contents (the lowest where a model has no
  `low`). The suggestion is the engine's (`Reasoning::suggested`), given to the interface with
  the task rules, so the two cannot disagree. A level a model lacks becomes the nearest it has,
  the lower of two equally near, which also covers a table of contents' combining model.
- **Settled at job creation.** The level is written into the set-up's options when the job is
  created, so the job, its cache keys and provenance hold the level sent; a different level is
  a different answer, read again. Jobs made before keep no level, and their keys, unchanged.
- **One `Tuning` per request.** The engine fills it from the catalogue and the set-up; each
  provider sends it, and reports it in the settings provenance records. Its old constants
  remain only for a model the catalogue lacks.
- **Changes in what is sent.** OpenAI now receives `reasoning.effort` (the suggestion: `none`
  where a model has it, else `low`, where it ran at its default `medium`); Gemini's limit is
  65,536 (was 65,535); Claude, OpenAI and DeepSeek may write up to 128,000 tokens.

## Consequences

- Claude is still called without streaming. A long answer (tens of thousands of tokens) may
  near the client's ten-minute request timeout; if live runs hit it, Claude moves to streaming.
- DeepSeek V4.1 Flash's levels are taken as GLM's (`low`, `high`, `max`), as on 28 October,
  since Mistral's page lists none for it. GPT-6 Astra's and Luna's defaults are not documented.
- The pipeline record shows each set-up's level and limit.
