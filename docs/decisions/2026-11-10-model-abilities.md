# 2026-11-10 · A model's abilities in the catalogue, not in its name

## Context

AGENTS.md says a new model is a catalogue entry and the engine never matches on a provider's
name. The [review of 9 November](2026-11-09-review.md) found five places that decided behaviour
by matching a model's or a preset's name instead (#147), an architecture checkpoint item:

| Where | What was matched | What it decided |
| --- | --- | --- |
| `providers/anthropic.rs` | a list of two Claude ids | the server-side fallback on a refusal |
| `providers/openai.rs` | `contains("diarize")` | speaker labels, and one language or several |
| `providers/gemini/transcribe.rs` | `contains("transcribe")` | the speech-to-text request instead of the ordinary one |
| `providers/mistral/chat.rs` | `starts_with("zai-glm")`, `starts_with("deepseek")` | the lowest reasoning effort |
| `engine/group.rs` | a preset id containing `newspaper` | how much text a page is taken to hold, so how many go together in one call |

A search of `providers`, `engine`, `offline`, `catalog`, `prompts` and the shell for other
matches on a model or preset id found none that decides behaviour: the offline engines' ids name
their own model files, and the engine's tests name Whisper's provider. The interface matches no
model name.

A model that is renamed, or a new one with the same abilities, behaved differently from what the
catalogue said of it; a researcher's own newspaper prompt was sized like a letter.

## Decision

- **Providers learn a model's abilities through `Tuning`.** The engine already filled it from the
  catalogue (the reasoning level and the output limit) and passed it in every `ProcessRequest`
  and `request_settings` call; it gains three abilities and the provider trait does not change.
  `ModelInfo::tuning` fills them; a model the catalogue lacks has none of them. Each is named
  for what the model does, not for a provider:
  - `refusal_fallback`, a new optional field of a catalogue model: when its safety classifier
    refuses, the provider can answer again with another of its models. Set for Claude Opus and
    Sonnet 5.5, as before; Haiku and Fable stay without, as before.
  - `speakers`, the catalogue's existing field (whether a model labels who speaks), decides
    OpenAI's diarized request, and with it one language rather than several.
  - `no_prompt`, the catalogue's existing `accepts_prompt` turned round so that a model the
    catalogue lacks gets the ordinary request: Gemini sends its speech-to-text request to a
    model that takes no prompt, since all its other models take one.
- **Mistral guesses no level.** Its lowest effort by name applied only to a model the catalogue
  lacks: every chat model listed has its levels, and the lowest is the suggestion for an
  analysis, so GLM and DeepSeek already get `low` and Mistral's own models `none` from the
  catalogue. A model the catalogue lacks is now sent no `reasoning_effort`, and thinks as much
  as Mistral's default for it, rather than a level that one family or another refuses. Such a
  model is one a job kept after the catalogue dropped it.
- **A preset says its pages are dense.** `Preset::dense`, set on the newspaper default, sizes
  the pages sent together as a newspaper's (4,000 tokens each instead of 1,500). It is editable
  in the Prompts screen for a text extraction prompt (*Pages full of text*), kept by a copy and
  by an export, and settled into the job's `pages_cap` when the job is made; a job made before
  `pages_cap` existed reads it from the preset its set-up names.
- **The file stem** of a source's name, written seven times in the engine, is `source_stem`.

## Compatibility

- **An absent ability means no.** The catalogue is refreshed from the repository, but
  `Catalog::newest` never prefers a downloaded catalogue to the bundled one unless it is
  strictly newer, so this build never reads a catalogue older than its own, which carries
  `refusal_fallback`. A cached file from before (same date or older) loses to the bundled one;
  every catalogue published from now on carries the field. Keeping the name rule as the default
  for an absent field would have kept what #147 removes, and a new schema version would have
  stopped released versions from reading any newer catalogue. A test strips the field from the
  bundled file and checks the bundled one wins; another checks that, for today's models, each
  ability is exactly what the names decided.
- **Released versions keep their own behaviour.** No struct in the catalogue, presets,
  pipelines or workflows refuses unknown fields (none uses `deny_unknown_fields`), so an older
  version reads the new catalogue and preset files, ignores the new fields and goes on matching
  names. No model id changed. The schema version stays 1, and so do the presets', pipelines' and
  workflows' file versions.
- **A preset written before the hint has none** (`dense` is `Option<bool>` in the file). A
  default's id, a retired id it replaced, or a copy made on import reads as the default; a
  researcher's own prompt as not dense: exactly as those files were read until now, since a
  researcher's preset id (`user-` and a UUID) never contained `newspaper`. The library writes the
  hint down when it reads a preset, before an import or a copy gives it a new id. On import, a
  preset that differs from one here only in the hint is still that one, so an older pipeline
  file finds the default it was made from.
- **Cache keys do not change.** A key is made of the source, the piece, the set-up (provider,
  model, prompt text, options) and the languages; neither `Tuning` nor the density is in it.
  A test pins the key of a newspaper page read by Opus to its value computed on the commit
  before the change. The provider tests now send each model the bundled catalogue's tuning, as
  the engine does, and every request they check is the one they checked before. Provenance is
  unchanged: the settings a request carried were already recorded; whether a page was dense is
  a sizing hint, not something that shapes the answer, and `pages_cap` is in the job.

## Consequences

- Adding a model that has one of these abilities is a catalogue entry again; a new ability of
  this kind is a catalogue field, a `Tuning` field and the provider's use of it.
- A model the catalogue lacks loses what its name gave it: no Claude fallback, no OpenAI speakers
  or Gemini speech request, no Mistral level. Such a model only remains in a job made before the
  catalogue dropped it.
- `providers` uses `catalog` as a development dependency, so its tests send the catalogue's
  tuning; the architecture test does not check development dependencies.
- AGENTS.md states that a provider never matches on a model's name. The specification's
  catalogue and prompts sections and the roadmap are updated; the formats documented in
  `formats.md` are output files and did not change. The privacy policy is unchanged: nothing new
  is sent or kept.
- `providers/tests/providers.rs` and the engine's `tests.rs` were already past 500 lines; the new
  provider tests went into a file of their own (`tests/abilities.rs`).

## To check

- `crates/offline` and the shell are not compiled on Linux. The shell calls none of the changed
  functions; one test in `offline/src/llm.rs` built a `Tuning` and now fills the new fields with
  their defaults. CI is the first to compile both.
