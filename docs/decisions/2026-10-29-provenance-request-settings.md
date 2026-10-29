# 2026-10-29 · The settings each request carried, in provenance

## Context

A pipeline's record states the settings each provider sends besides the prompt (reasoning
effort, output limit, media resolution: `Provider::request_settings`, see the
[pipeline export](2026-10-27-pipeline-export.md)), but `provenance.json` did not, so a result
could not say at what effort it was read. Those settings live in each provider's code and change
with it, and `provenance.json` is written again from the cache whenever a format is exported
later. Reading them at export time would let an update of the app rewrite an older result's
record. The first live table of contents also pointed at Claude's `low` effort as the main
suspect for pages read short, so researchers may soon choose the effort; the record has to come
first.

## Decisions

- **Kept with each result, when it is sent.** `StoredResult::request` holds what the provider
  stated for that task and model when the piece was processed, in a new nullable `request`
  column of `results` (the store's fifth migration). A batch records them in its `BatchRecord` when it is sent,
  and its answers are stored with those, so an update between sending and collecting changes
  nothing.
- **`provenance.json` gathers them from the pieces** it describes: `request` for the step that
  reads, `merge.request` for the step that combines a table of contents. Each name and value
  appears once, in the order first seen; a name listed twice means pieces were read with
  different values (cached pieces from an earlier version, say). Results stored before this
  change add nothing, so the field is absent rather than guessed.
- **Not in the cache key.** A result is still reused across jobs whatever settings the code now
  sends, and its record says what it was read with. Whether a change to what a provider sends
  should make cached results stale stays a matter for `PIPELINE_VERSION`, as for any other
  change; this one invalidates no cache.
- **Offline results are marked `offline`.** `provenance.json` wrote `engine: "cloud"` for every
  result, Whisper's included; it now says `offline` for providers that need no key.

## Consequences

- `schema_version` stays 1: the fields are added, none changes meaning.
- Tested in the store (an older row reads back with no settings), in the engine (each step of a
  table of contents carries its own provider's settings; a batch collected by an updated app
  keeps the effort it was sent with) and in the exporters' golden files.
- Choosing the effort per set-up, in New job and in pipelines, is now possible without losing
  track of what each result was read with; it is not part of this change.
