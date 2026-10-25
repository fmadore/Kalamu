# 2026-10-25 · Analyses, line breaks, speaker names, pipelines

## Context

After the Critical Edition redesign, the researcher asked for:

- a text extraction mode that keeps the page's lines (easier to check against the PDF, worse for
  term search);
- summaries and keywords turned into configurable labels: categories such as places, persons
  and subjects, free or fixed values (positive, negative), so the same task covers named entity
  recognition and sentiment analysis, with saved configurations and a new name for the task;
- review and comparison of those results, as for OCR;
- renaming speakers during audio review;
- saved, named pipelines with quick access;
- Lucide icons in place of hand-drawn ones, a foldable side menu, and a leaner README with the
  documentation under `docs/`.

Throughout, the app must stay friendly to researchers who are not technical. Transcription
Pearl (Humphries and Leddy) and its successor Archive Studio were studied for ideas only: their
CC BY-NC licence rules out reusing their code in an MIT app.

## Decisions

- **Line breaks are a job setting, not a doubled set of presets.** `JobSettings.line_breaks`
  (`as_on_page` or `running_text`) appends a short section to whichever prompt is used, stating
  that it overrides the preset's own line rules, as the language choice already did. One
  choice then works with every preset, the user's own included. Models that take no prompt keep
  their layout. Deriving running text from diplomatic output in code was considered (one paid
  call for both versions) and left for later: a line-end hyphen cannot always be told from a
  real one, and only a model catches words broken without a hyphen.
- **Speaker names are a layer, like corrections.** A `speaker_names` table per job, set-up and
  recording; the engine applies the names when it builds the review view and the outputs, and
  the stored output keeps the model's labels. Labels are renamed only where they open a line, so
  a name in the text is never touched. A recording sent in several pieces gets a note that a
  label may change person between pieces, as zmo-ai-pipelines scopes them per request.
- **"Summarise and find keywords" becomes "Analyse texts".** `Task::Summary` is renamed
  `Task::Analysis`; stored jobs that say `summary` still read. An analysis is a schema of
  fields (`core::analysis`): text, a list of names or terms with an optional maximum, one
  choice, several choices, each with the researcher's instructions and an optional reason.
  - The schema belongs to an analysis preset, is frozen into the job's prompt snapshot and its
    cache key, described to the model in words and sent as each provider's structured output
    with every property required (OpenAI's strict mode demands it; Claude refuses length
    limits, so a list's maximum is applied on reading).
  - Answers are read back and checked: lists tidied and cut to their maximum, choices spelled as
    in the schema. An answer that breaks the fields is `ProviderError::InvalidAnswer`, which the
    engine retries, never an empty result (as iwac-ai-pipelines does for its sentiment
    consistency rules).
  - Ready-made analyses: research summary, short summary, people, places and organisations
    (from iwac-ai-pipelines' NER rules), tone (from its sentiment scales: labels, never numbers,
    with a reason). Named entity recognition and sentiment are analyses, not tasks.
  - Corrected answers are their own layer (`corrections.fields`); the model's are kept.
  - Exports: a CSV row per document with a column per field and per reason; JSON answers by
    key with their reasons.
- **The analysis review collates answers by field.** Each value carries the sigla of the
  set-ups that gave it, list items found by only some set-ups get a rubric siglum, and pointing
  at a name marks it in the document. A text source is shown as text (`source_text` command).
- **Pipelines live in the prompts crate.** A pipeline is a job's settings without its files,
  saved under a name (`PipelineLibrary`, one JSON file each), held by the app state since the
  engine does not need it. Saving under an existing name replaces that pipeline.
- **Interface icons are Lucide**, through the one `Icon` component, at an absolute 1.5 px
  stroke. Hand-drawn icons read as generic AI output to the researcher.
- **The side menu folds to icons**, remembered in the browser storage of the window (a
  per-computer convenience); open, it keeps words and no icons.
- **Documentation moves under `docs/`**: the specification, roadmap and development notes leave
  the README, which becomes a short introduction. The author's affiliation is the University of
  Bayreuth; ZMO is only the origin of one reference code base.

## Architecture checkpoint

- **Dependencies still point inwards.** The analysis schema is a domain type in `core`; the
  prompts crate describes it, providers turn it into their formats, the engine passes it
  through, exporters read it. No crate gained a dependency on another internal crate.
- **One job per crate holds**, with one stretch: `prompts` now keeps pipelines as well as
  presets. Both are named, saved user configurations; split them if pipelines grow.
- **Duplication to fix next.** `ReviewView.svelte` (817 lines) and `AnalysisReview.svelte` (665)
  share their frame: masthead, strip, panes, bars, edition page and sigla key, with styles
  copied. Extract the shared frame before the next feature that touches the review.
  `NewJob.svelte` (691 lines) should give its pipelines and line-break sections their own
  components.
- **Resuming, caching and provenance cover the new paths.** The schema is in the cache key and
  `provenance.json`; corrections and speaker names are layers applied by the engine; an invalid
  answer retries like a transient error.

## Consequences

- Output files of analyses end in `.analysis` instead of `.summary`.
- Results stored before this change lose their keywords (the field was replaced); no release
  had shipped.
- Nothing here has been run against the live APIs yet; the provider tests use documented
  response shapes.
