# 2026-11-11 · Only the newest model of each line offered, older ones kept

## Context

New job listed every model in the catalogue, so a provider's previous model (Claude Fable 5.1)
stood beside its replacement, and a new one meant either a longer list or deleting the old entry.
Deleting it would break what names it: a pipeline, preset, workflow step or comparison saved with
Fable 5.1 would lose its name (New job showed the raw id and a blank choice), its prices, its
output limit and reasoning levels, and run with none of them. The catalogue had already dropped
an id once without a trace: DeepSeek V4.1 Flash's, corrected from `deepseek-v4p1-flash-1017`.
The prompts crate already solved the same problem for retired default prompts
(`defaults::RETIRED`, [decision](2026-11-02-fewer-default-prompts.md)).

The providers' pages on 11 November 2026: Anthropic lists Fable 5.2 (`claude-fable-5-2`, $8 /
$40) as current and Fable 5.1 as legacy, still served; Google added Gemini 4 Carbon
(`gemini-4-carbon`), its strongest reasoning model, beside Gemini 4 Argon, its efficient one, at
the same price, neither replacing the other, and names no successor for Gemini 3.1 Pro; Mistral
made Large 4 generally available on 5 November and deprecated Medium 3.5 ("Use Mistral Large 4
for new integrations"), retiring it on 30 April 2027; OpenAI's GPT-6.1 Sol and Terra and GPT-6
Astra and Luna are each the newest of their line.

## Decision

- **`successor` on a catalogue entry**, the id of the model its provider names as its
  replacement, from the same provider. Only the provider's word counts: Fable 5.1 → 5.2 and
  Medium 3.5 → Large 4 now; Argon, Carbon and 3.1 Pro keep no successor. A model with one is
  hidden from New job's choices, never recommended nor a provider's default, and still found by
  `Catalog::model`, so whatever names it keeps its name, prices and settings.
- **`retired`** for a model its provider no longer serves. The entry stays, for old jobs' names
  and costs; `Engine::check_job` refuses a job naming it (`provider.retired`) before anything is
  sent. `status` keeps its meaning (the provider's own stable, preview or deprecated).
- **`aliases`**, ids the catalogue once gave a model it now lists under another. `Catalog::model`
  follows them; a job freezes its set-ups under the id now (`canonical_id`), so the right one is
  sent; New job loads a pipeline's models under it.
- **Kept, not upgraded.** A set-up that names a replaced model runs it while the provider serves
  it: a silent switch would change results and cost unseen, against provenance's promise that a
  pipeline reproduces. New job lists it, marked *replaced*, with a button for its replacement
  (`Catalog::replacement` follows a line of successors to the first model still offered; the
  engine passes it to the interface as `ModelRef::replaced_by`).
- **No entry is ever removed.** A test lists every id shipped since and fails when one stops
  resolving; another checks that each successor exists, ends in a model still offered that does
  all the old one did, and loops nowhere.
- **The catalogue refresh is unchanged.** All three fields are optional and serde skips fields it
  does not know, so 1.6.0, already in the stores, reads the refreshed catalogue and simply goes on
  offering every model. No new `ModelStatus` value and no new schema version, either of which
  would make 1.6.0 discard the download and stay on its bundled catalogue.

## Consequences

- A new model of an existing line is its entry plus one `successor` on the one it replaces;
  nothing in the engine or interface changes.
- GPT-6 Sol (`gpt-6-sol`), dropped from the catalogue in October when GPT-6.1 Sol replaced it,
  is not restored: a pipeline naming it still runs, as any id the catalogue does not list, but
  without its name or prices. It can come back as an entry with a successor once its prices are
  checked.
- Gemini 4's prices double on 1 December 2026 and Gemini 3.8 Flash's on 1 January 2027; the
  catalogue's notes say so, and the entries are updated on those dates.
