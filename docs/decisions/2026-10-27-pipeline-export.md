# 2026-10-27 · Exporting a pipeline, and providers that state what they send

## Context

The researcher asked for an export button on saved pipelines, so a scholar can report or share
exactly how results were produced: which model, which settings, which prompt. A pipeline stored
only a preset id and the job's settings, and the settings each provider sends (reasoning effort,
output limit, media resolution) existed only inside each provider's request code. Sampling
temperature is being deprecated by the providers and none of them sends it, although
`SetupOptions` still has the field.

## Decisions

- **One export writes two files side by side**: `<name>.json`, which another Scriptorium imports,
  and `<name>.md`, a methods record (task, languages, formats; per set-up the provider, model,
  request settings, the full instructions and per-piece request, the prompt's SHA-256 and the
  analysis fields as a table).
- **The record comes from the engine** (`Engine::pipeline_file`), which freezes each prompt with
  the same code as `create_job` (`frozen_prompt`, factored out of it), so the record states what a
  job would send. The hash is `ProvenancePrompt::from_snapshot`, now shared with
  `provenance.json`, so a result can be matched to the pipeline that produced it.
- **`Provider::request_settings(task, model)`**, with an empty default, states the fixed settings
  a provider sends. Each setting is named by its path in the request body
  (`output_config.effort`), and one integration test sends a request from every cloud provider to
  a mock server and checks each declared value against the body it received, so the record
  cannot drift from the code. Whisper states its beam size, fresh context per window and window
  length. Temperature is not reported, since nothing sends it.
- **The file bundles its presets.** On import a preset identical to a local one (same task,
  instructions, request and fields) is used as it is; any other becomes a new user preset, a
  changed default named after the pipeline. An import never replaces a pipeline or a preset: a
  name already taken gets a number.
- **The output folder is left out**: it is a local path, and may hold the Windows user name.
- **The pipeline buttons move into `PipelineList.svelte`**, taking New job from 829 to about 750
  lines. The section now shows when no pipeline is saved yet, for the import button.

## Consequences

- A new provider should implement `request_settings` and add itself to the integration test.
- The Markdown record is in English, like the prompts it quotes.
- OpenAI's reasoning effort is not set, so OpenAI models run at their own default; the record
  says only what is sent.
- The unused `temperature` option could be removed from `SetupOptions` in a later clean-up.
