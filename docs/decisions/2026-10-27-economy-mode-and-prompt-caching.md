# 2026-10-27 · Economy mode (batches) and prompt caching

## Context

Large collections are expensive: hundreds of pages to read, or an analysis over a whole archive
(issue #7). Two provider features lower the price of the calls that still have to be made after
Scriptorium's own result cache: **batch services**, which answer within a day at half price, and
**prompt caching**, which bills a repeated prefix at a fraction of the input price. This entry
draws on each provider's documentation as read on 27 October 2026 (Gemini batch mode and
caching, OpenAI Batch API and prompt caching, Claude Message Batches and prompt caching, Mistral
batch jobs and prompt caching). These details change often; check them again before relying on
the figures below.

## What the providers offer

| | Gemini | OpenAI | Claude | Mistral |
| --- | --- | --- | --- | --- |
| Batch covers | `generateContent` | Responses, not transcription | Messages | OCR, chat, transcription |
| Discount | 50% | 50% | 50% | 50% |
| Window | 24 h target, expires at 48 h | 24 h | 24 h, most within 1 h | `timeout_hours`, 24 by default |
| Sent as | inline (under 20 MB) or a JSONL file | a JSONL file | inline, up to 256 MB | inline (under 10,000) or a JSONL file |
| Results kept | 6 weeks | 30 days | 29 days from creation | unclear (24 h or no limit) |
| Caching | implicit, 4,096-token minimum on current models | automatic, 1,024-token minimum | explicit marker, 512-token minimum on the 5.x models | `prompt_cache_key` |
| Cached input | about 10% (Argon 5%) | 10% (GPT-6.1 Sol and Terra 5%), writes 1.25× | 10% (Opus and Sonnet 5.5 5%, Fable 2.5%), writes 1.25× | 10% |
| Cached tokens reported in | `cachedContentTokenCount` (part of the prompt count) | `input_tokens_details.cached_tokens`, `.cache_write_tokens` (part of `input_tokens`) | `cache_read_input_tokens`, `cache_creation_input_tokens` (not in `input_tokens`) | `prompt_tokens_details.cached_tokens` (part of `prompt_tokens`) |

Claude states that cache and batch discounts stack; OpenAI's price list implies it; Gemini's
pages disagree with each other; Mistral says nothing.

## Decisions

- **A choice when starting a job: Faster or Cheaper.** `JobSettings.pace` is `fast` (as before)
  or `economy`. New job offers the choice, with the estimated cost of each, when one of the
  chosen models can be batched, and names the models that still run page by page. A pipeline
  saves the choice with the rest of the settings. The pace is not part of a result's cache key:
  a page read in a batch is the same answer as one read alone.
- **Batching is a capability of the `Provider` trait**: `batches(task, model)`, `batch_limits`,
  `submit_batch`, `poll_batch` and `cancel_batch`, with defaults that refuse. The engine never
  matches on a provider's name. Each provider reuses its own request body and response parsing,
  so a batched answer is read exactly as a single one, an analysis's fields included.
- **What is batched.** Claude and OpenAI: text extraction and analyses. Gemini: text extraction
  and analyses, inline, in batches of up to 13 MB of pages (under the 20 MB inline limit once
  encoded); recordings are too large to send inline, and YouTube links are not documented in
  batches. Mistral: OCR and analyses, inline; Voxtral's batch format for audio is not
  documented. Offline engines never batch. Claude batches go without `fallbacks`, which batches
  refuse.
- **The catalogue carries the prices.** `cached_input_per_mtok`, `cache_write_per_mtok` and
  `batch_discount` per model. `actual_cost` bills cached and written input at their own prices
  and takes the batch discount off everything, as the providers bill it; the estimate gives a
  second figure, `economy_usd`, which the app shows only where the provider really batches the
  model. GLM 5.3 has no cached price yet: issue #6 quotes one for GLM 5.2 only, so its cached
  input is costed at the full price until Mistral publishes one.
- **`Usage` records what was billed**: `input_tokens` is now always the whole input, with
  `cached_input_tokens` and `cache_write_tokens` as parts of it (Claude's figures are added up to
  match the others), and `batch` says the call went through a batch service. Provenance writes
  these fields, so the cost on record is the cost charged.
- **Batches survive closing the app.** A batch is recorded in the store (`batches` table)
  before it is sent, and its pieces are marked `submitted`. Once the provider accepts it, its id
  is written and the batch is `waiting`. At startup, jobs with a waiting batch start again by
  themselves and ask the provider at once, so answers that came in while the app was closed are
  collected. A batch still `sending` at restart was never confirmed: it is abandoned and its
  pieces are sent again. The one gap is a crash between the provider accepting a batch and its
  id being written, which would send those pieces twice; the window is a single HTTP response.
  Only a rate-limit refusal is retried when sending, since a timed-out request may have created
  the batch.
- **Waiting, pausing, cancelling.** The engine asks about waiting batches every minute. The job
  view says how many pieces the provider holds and when they are due (24 hours after sending).
  Pausing stops the asking but leaves the batches with the provider; resuming collects them.
  Cancelling or deleting a job asks the provider to cancel its batches. An expired or refused
  batch fails its pieces with the provider's reason; Retry sends them in a new batch.
- **The request layout already puts the shared part first.** The instructions go as the system
  prompt (Gemini `systemInstruction`, OpenAI `instructions`, Claude `system`) and the schema in
  the request's settings, ahead of the page; the 26 October rule, document first and the request
  last, only orders the user turn. Nothing was reordered, so the quality of results is
  unchanged.
- **Caching per provider.** Claude needs a marker: it goes on the system block, not at the
  request's top level, where automatic caching would place the breakpoint after the page and
  write every page to the cache at 1.25× for nothing. OpenAI and Gemini cache by themselves.
  Mistral's `prompt_cache_key` is documented for the Conversations API; Scriptorium uses Chat
  Completions, so no key is sent, and cached tokens are read from the response when Mistral
  reports them.

## Consequences

- Most prompts are short: a text-extraction prompt is a few hundred tokens, under Gemini's and
  OpenAI's minimum, so caching will mostly help analyses with long instructions and many fields,
  and Claude with its 512-token minimum. The batch discount is the larger saving.
- Claude's cache lasts 5 minutes by default. Inside a batch, Anthropic recommends the 1-hour
  cache (written at 2×), since requests run in any order. Not adopted yet: the catalogue has one
  write price, and the gain depends on how long the system prompt is.
- Gemini batches are capped by the 20 MB inline limit, so a large job makes several batches. A
  JSONL file through the Files API would lift this, but uploaded files expire after 48 hours,
  as long as a batch may wait.
- **Not yet run against the live APIs.** The four batch clients follow the documentation and are
  tested against responses in the documented shapes, read defensively where Google's and
  Mistral's pages disagree. A live run of each, with a small job, is on the roadmap before the
  next release.
- New: `core::batch` (records, phases, progress), `providers::batch` (shared readers), a batch
  module per provider, `engine::batch` (gathering, sending, waiting, collecting) and store
  migration 4. Writing a source's outputs moved to `engine::output`. `engine::run` and
  `engine::batch` sit just above 500 lines, each with one job; `NewJob.svelte` grows past 850
  lines, so its split, already on the roadmap, is more pressing.
