# 2026-10-31 · Refused answers in the cost, and the last provenance gaps

## Context

The [29 October checkpoint](2026-10-29-checkpoint.md) left two items that change no result. An
analysis answer that breaks its fields is asked for again, and the provider bills both
answers, but only the one kept reached the cost shown: the error carried no usage. And
`provenance.json` did not say whether Whisper skipped silences with its voice-activity model,
which speaker labels the researcher renamed, or what each step of a table of contents cost, and
it listed fixes the settings allowed rather than those made (`split_spreads` on a document with
no spread, a sample of a document shorter than it).

## Decisions

- **A refused answer carries its usage.** `ProviderError::InvalidAnswer` holds the usage of the
  answer it refused, set by the providers' shared `analysis::finish`, and
  `ProviderError::billed` returns it; no other error does, since the others were refused
  before the model answered. The engine records it as a `Charge` (job, set-up, source, piece,
  usage, cost) in a new `charges` table, in a single call and in a batch collected (at the
  batch's price), and adds it to the cost reported as the job runs. `JobStore::spent` and the
  job list count charges with the results kept, so the cost shown is the cost charged across
  runs. Deleting a job deletes its charges.
- **Provenance keeps the two apart.** `usage` and `cost_usd` remain what the results kept cost,
  so a document's figures still match its results; refused answers are under `refused`
  (`answers`, `usage`, `cost_usd`), absent when there were none.
- **Whisper says whether it used voice detection.** Whisper's `sent_settings` adds `vad`
  (`true` or `false`) and, when on, `vad_model`, decided as the transcription decides it: by
  whether the model is downloaded. The pipeline record's `request_settings` leaves it out, as it
  depends on the PC rather than the pipeline.
- **Renamed labels, not names.** Provenance lists the labels the researcher renamed under
  `renamed_speakers`, so it says the files' speakers are not the model's own labels. The names
  themselves stay out: they are often those of the people interviewed, they are already in
  every file, and `provenance.json` is the record a researcher shares as a methods note.
- **A table of contents' second step has its own figures.** `merge.usage` and
  `merge.cost_usd` give the combining step's share of the document's totals.
- **Only the fixes made.** The cache key still carries every fix the settings ask for, so that
  results are not read again; provenance lists `split_spreads` only where a page was split,
  `even_out_volume` only for a recording, and a sample only of a document cut into pages and
  longer than it.

## Refactor

`engine/src/output.rs` was at 498 lines and these changes add to the document it assembles:
`export_document` moved, with the helpers only it uses, to `document.rs`. `output.rs` keeps
where files go and how they are written.

## Consequences

- The comparison's cost per set-up still counts the results kept, as its scores do.
- Results stored before this have no `vad` entry; offline results transcribed earlier say
  nothing about voice detection.
- The checkpoint's list is done; the roadmap's *Next* moves on to project workspaces and the
  live runs.
