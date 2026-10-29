# 2026-10-29 · Results that cannot be wrong unnoticed

## Context

The [checkpoint of the same day](2026-10-29-checkpoint.md) found three ways a result could be
wrong without the researcher or `provenance.json` knowing: an answer that stopped at the model's
output limit was stored as complete (Gemini's code said the engine marked it; nothing did); a
file changed on disk after it was added to a job was read without being hashed again, so its new
content was cached under the old fingerprint and served to every later job; and a second answer
under the same fingerprint replaced the first (two identical set-ups, a batch collected after a
fast run), leaving the researcher's corrections over a text they had not seen.

## Decisions

- **A cut-off answer is kept and marked.** `ProcessOutput::truncated` is set by each provider
  from its own signal: Gemini's `MAX_TOKENS`, Claude's `max_tokens`, OpenAI's incomplete
  `max_output_tokens`, Mistral's `length`. Transcriptions and Mistral OCR have no such limit,
  and a cut-off analysis or table of contents fails its fields and is retried as before. The
  text is not retried, since the same request would stop at the same place: it is kept, as
  what was read is still useful. The flag is stored with the result (and in a `truncated`
  column of `results`, so the job list counts such pieces without reading every answer); the
  job's page shows "2 cut off" beside its pieces, the review warns on the page, naming the
  set-ups in a comparison, and `provenance.json` lists the pieces under `truncated`.
- **A changed file is refused.** A source now records its modification time when it is added.
  Before a run reads it, `kalamu_media::check_unchanged` compares its size and that time, and
  hashes it again only when the time differs or is unknown (jobs set up before this change),
  so a file touched but not changed is still read and a large recording is not hashed on every
  resume. A changed file fails with a sentence that says to start a new job with it; nothing
  is sent or cached.
- **The first answer is kept.** `put_result` inserts or ignores: a later answer under the same
  fingerprint is still paid for and counted, but never replaces the stored one.

## Consequences

- The store's sixth migration adds `results.truncated`; answers stored before it count as not
  cut off, which they may have been.
- `JobSummary` gains `pieces_truncated`, `PieceView` and `SourceFile` a field each; the privacy
  policy lists the files' sizes and modification times among what the job database keeps.
- Tested in the providers (each one's signal, Mistral's in its unit tests), in `media` (a file
  touched but unchanged, changed at the same size, changed in size), in the store (the first
  answer kept) and in the engine (a replaced file never sent; a cut-off page counted, marked in
  the review and listed in provenance).
- Not done here: two identical set-ups in one job still both pay before the second answer is
  dropped; worth a check in `create_job` if it happens in practice.
