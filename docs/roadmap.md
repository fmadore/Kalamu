# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (2 November 2026)

**Version 1.1.2** is the latest release: unsigned installers for Windows x64 and ARM64 and a
disk image for macOS on Apple Silicon, each with FFmpeg bundled, are on the
[v1.1.2 release](https://github.com/fmadore/Kalamu/releases/tag/v1.1.2)
([notes](release-1.1.2.md)). Kalamu has been in the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H) since version 1.0.0
(2 November 2026). Everything below is tested against
recorded provider responses. Text extraction, transcription and analyses have also run against
the live OpenAI, Claude and Mistral APIs with real keys (`cargo run --example live`, see
[development.md](development.md)); Gemini has not yet, for want of credit on the account.

**macOS** (Apple Silicon) is built and tested in CI and has a disk image from 1.1.2 on, signed
ad hoc; it has not yet been opened on a real Mac
([decision](decisions/2026-11-02-macos.md)).

## Done

- **Foundations.** Rust workspace and Tauri shell, job database with caching and resuming,
  progress and taskbar progress, model catalogue refreshed from the repository, prompt library,
  keys in Credential Manager, English interface with translation files, CI with `cargo audit`
  and `cargo deny`, Dependabot.
- **Text extraction.** Gemini, OpenAI, Claude and Mistral; PDF page cutting with PDFium, image
  preparation, input checks with splitting of double pages, language choice, line breaks kept
  as on the page or joined into running text, exports to `.txt`, `.md`, `.docx`, `.json` and
  `.csv`, searchable PDFs from Mistral OCR.
- **Audio transcription.** Recordings and videos split at pauses and converted by FFmpeg, checks
  for quiet, long or silent files with an "even out the volume" fix, speakers and timestamps,
  `.srt`, `.vtt` and `.csv`. Offline transcription with Whisper, models downloaded from
  Settings. Jobs keep running from the notification area when the window is closed. Public
  YouTube videos transcribed from their link with Gemini, without downloading them, and reviewed
  beside the video, played on the dark stage in YouTube's privacy-enhanced player, each video
  named after its title
  ([decision](decisions/2026-10-27-youtube-links.md)).
- **macOS.** Apple Silicon, built and tested in CI beside Windows x64 and ARM64: what differs
  between the systems in one place per layer (`src-tauri/src/platform/`, `src/lib/platform.ts`,
  `tauri.macos.conf.json`), the review's player given WebKit's formats, ⌘ shortcuts, Mac window,
  Dock and menu-bar conventions, FFmpeg built for macOS, a `.dmg` in each release, and the
  interface tested in WebKit on a Mac in CI
  ([decision](decisions/2026-11-02-macos.md)).
- **Analyses.** With Gemini, OpenAI, Claude and Mistral, including the Z.ai GLM and DeepSeek
  models Mistral hosts. Fields defined by the researcher
  (text, lists of names or terms, one or several choices, reasons), sent as each provider's
  structured output and checked on return; five ready-made analyses (research summary, short
  summary, scholarly publication, people, places and organisations, tone); a field editor in Prompts; CSV with a
  column per field.
- **Detailed tables of contents** of newspaper and magazine issues, archival files and books:
  each page read by one model, the pages combined by another, of any provider; presets ported
  from iwac-ai-pipelines' issue index and widened; a review of the entries beside the scan,
  with the strip of pages and the entries following the page; an answer language, as analyses
  have; a diagram in New job ([decision](decisions/2026-10-29-contents-and-correction.md),
  [first test](decisions/2026-10-29-contents-review-and-answer-language.md)).
- **Correcting existing text**: a PDF's text layer or a text file corrected page by page or in
  passages, with iwac-ai-pipelines' OCR-correction prompt among the presets, and every change
  highlighted against the original in the review.
- **Review.** The source on a dark lightbox beside the text, in panes that can be resized,
  with another set-up's whole text beside them on demand; differences between set-ups and
  corrections highlighted in the text, their readings shown on pointing, any of which can be
  taken as a correction; corrections kept apart from the AI output; marking as reviewed, with a strip of
  pages showing what is left; a player for
  recordings with speakers that can be named; an analysis review that collates answers by
  field and finds names in the document.
- **Comparison mode**, each set-up scored against the reviewed pages.
- **Error rates after review, for any job**: character and word error rates against every
  page corrected or marked as reviewed, summed over all the characters and words, for the job
  and each document on its page and for the document in the review, in plain words
  ([decision](decisions/2026-10-29-error-rates-after-review.md)).
- **Results that cannot be wrong unnoticed**: an answer cut off at the model's output limit is
  kept but marked, counted on the job's page, warned about in the review and listed in
  provenance; a file changed on disk since it was added is refused; the first answer stored
  for a fingerprint is never replaced
  ([decision](decisions/2026-10-29-result-integrity.md)).
- **Work neither lost nor paid twice**: quitting waits for the pieces already sent, saying how
  many, with "Quit now"; a pause or cancel ends a wait between retries; at launch a job with a
  waiting batch is only collected; an engine error lets the calls already started finish; the
  job's page shows what it cost over all its runs
  ([decision](decisions/2026-10-29-cost-and-quitting.md)).
- **Results that reach the researcher**: a document's files are written even when a page keeps
  failing, the page marked `[Not read: reason]` and listed in provenance; sources of the same
  name sent to one folder are numbered instead of writing over each other
  ([decision](decisions/2026-10-29-files-despite-failures.md)).
- **One frame for the reviews**: the text, analysis and table of contents reviews share their
  masthead, source pane, handles, strip, keys (Ctrl Enter to mark reviewed or save a
  correction, Page Up and Page Down to turn pages), correction editor and key to the sigla; a
  text file's correction shows the passage beside it
  ([decision](decisions/2026-10-29-review-frame.md)).
- **Search in the review** (Ctrl F): the document's text as shown, corrections included,
  searched across its pages or a recording's pieces, capitals and accents ignored; the matches
  listed by page or by time, each gone to with its words marked and a recording playing from
  its line; a table of contents' entries filtered; an analysis's fields kept where their answers
  or reasons hold the words, which are also marked in a text document
  ([decision](decisions/2026-10-29-search-in-the-review.md),
  [analyses](decisions/2026-10-30-version-1.md)).
- **The rest of the checkpoint's refactor**: New job, the job page and the sidebar in smaller
  components; the engine's job loop split into planning (`plan.rs`), one piece's call
  (`piece.rs`) and the loop (`run.rs`), sharing the piece-saving and cost code with batches,
  which are split into sending and collecting; the rules on which files, models and formats
  suit each task given to New job by the engine (`task_rules`), which checks them again when a
  job is created ([decision](decisions/2026-10-30-version-1.md)).
- **Version 1.0.0 for the Microsoft Store**: Report a problem in every review and in Settings
  (policy 11.16), the MSIX package with FFmpeg inside and the Store bundle built by the release
  workflow, third-party notices for every crate and package, sample files for the certification
  team, and the listing for 1.0.0 ([packaging-msix.md](packaging-msix.md),
  [decision](decisions/2026-10-30-version-1.md)).
- **A glossary** for colleagues new to these tools: a page beside Settings explaining sixteen
  words the app uses (OCR and HTR, LLM, Whisper, API key, prompt, set-up, pipeline, workflow, tokens,
  reasoning effort, economy mode, ground truth, error rates, hallucination, provenance,
  machine-readable) and what each export format suits, with a filter, links between the
  words, and a help mark beside the headings that use them; written in the translation files
  like all interface text ([decision](decisions/2026-10-30-glossary.md)).
- **Icons centred in their buttons**: the icon component's style no longer reaches buttons
  named `icon`, which drew their icon at the left whenever its style loaded last.
- **Analysing corrected text, first increment**
  ([#47](https://github.com/fmadore/Kalamu/issues/47)): *Analyse this text* freezes an
  extraction's, transcription's or correction's corrected text and analyses it without
  reading the originals again; a later correction marks the analysis out of date without
  changing what it read; provenance records the result read and how much was reviewed;
  quotations can be anchored in the frozen text, piece by piece, and checked
  ([decision](decisions/2026-10-31-reviewed-text-snapshots.md)).
- **Project workspaces, first increment**
  ([#57](https://github.com/fmadore/Kalamu/issues/57)): projects with flat collections, a
  document held once in several, every result of a document across jobs with its review state
  and cost, a preferred result per task, files and folders added before they are read, earlier
  jobs added after their matches are confirmed, New job started from a selection and tied to
  it, documents with the same content merged, deletions that never reach a job, missing files
  marked, archiving; a crate of its own, `projects`
  ([decision](decisions/2026-10-31-project-workspaces.md)).
- **Project dictionary, first increment**
  ([#48](https://github.com/fmadore/Kalamu/issues/48)): the names and terms every analysis
  listed, field by field, gathered through the corrected answers across the project or a
  collection; entries made by the researcher, with stable ids, a preferred label and the forms
  as written, merged, taken apart, relabelled, reviewed and kept apart, all surviving a
  restart; look-alike spellings pointed out, never merged; documents and agreement between
  results counted, not mentions; entries whose evidence is gone marked; export as JSON and CSV;
  a crate of its own, `dictionary`
  ([decision](decisions/2026-11-01-project-dictionary.md)).
- **Project dictionary, merges proposed by a model**
  ([#48](https://github.com/fmadore/Kalamu/issues/48)): a field's rows sent, with their
  spellings, the documents naming them and the pairs kept apart, as an analysis job of its own
  under a prompt that can be read and edited in Prompts, with an estimate first and economy
  mode where the model has one; the whole field in one request up to about 100,000 tokens, larger
  fields in alphabetical groups of that size sharing a tenth of their rows, the coverage said; the answers
  checked against the rows sent, references to no row counted and dropped, pairs kept apart
  never offered; each proposal accepted through the ordinary merge (some rows left out, the
  label changed), kept apart, or turned down and taken back, across restarts
  ([decision](decisions/2026-11-02-proposed-merges.md)).
- **Tags on a project's documents**: the researcher's own labels beside the collections, added
  and removed on a selection or a document, narrowing the table and found by its search; one
  spelling per project whatever the capitals, kept through a merge.
- **A project's Activity, filtered and costed**: its jobs narrowed by task, state, model,
  workflow and when they were started, and the jobs shown costed together, with those not yet
  priced (offline, or not yet answered) counted apart so a sum is never read as whole when it
  is not. A job does not record a pipeline it was filled from (New job's settings stay
  editable after one is applied), so the workflow a job is a step of stands in for it.
- **Workflows, first increment** ([#49](https://github.com/fmadore/Kalamu/issues/49)):
  pipelines run one after another on the same files, each step reading the files or an
  earlier step's corrected text, frozen when it starts; a review stop, or pages not read, holds
  the steps that read it until the researcher continues; each step is an ordinary job, and
  provenance names the run, the workflow's version and the go-ahead; definitions versioned,
  exported and imported, runs frozen; running again pays only for the steps that changed; a
  run started from a project's selection ties its jobs to the documents
  ([decision](decisions/2026-11-01-workflow-runs.md)). For colleagues new to them: *Then* at
  the end of New job analyses the text once it is checked, with no workflow to build; a banner
  on the job's page and in its review offers *Continue*, and the sidebar lists those jobs under
  *For you to check*; three ready-made examples; an animated drawing of the steps; a glossary
  entry. Any finished text goes into a new round from *Use this text in a new job* (analyse it,
  correct it, or run a workflow on it), on the job's page or on a project document's results,
  which send only that document's text.
- **Each model's own output limit and a thinking level per set-up**: answers may use the whole
  output each model is documented to give (none is set where Mistral documents none); New job
  offers each model's reasoning levels under *Thinking*, the lowest suggested for OCR, HTR and
  transcription, `low` for analyses; the level is settled when the job is created and recorded
  in provenance ([decision](decisions/2026-11-01-output-limits-and-reasoning.md)).
- **Several pages per call** for text extraction: one page at a time (the default), several,
  or the whole document, sized to what the model can write in one answer; the answer split
  back into pages at the marks the model is asked for, and a page left out or cut short read
  again alone, so the review and error rates stay page by page; not yet in economy mode
  ([decision](decisions/2026-11-01-pages-per-call.md)).
- **The last of the 29 October checkpoint**: analysis answers billed but refused for breaking
  their fields, in a single call or a batch, now count in what a job cost, on its page and in
  the job list, and `provenance.json` lists them apart (`refused`); provenance also records
  whether Whisper skipped silences with its voice-activity model, which speaker labels were
  renamed (not the names), a table of contents' second step's usage and cost, and only the
  fixes that were made to a source
  ([decision](decisions/2026-10-31-refused-answers-and-provenance.md)).
- **Economy mode and prompt caching**: a job sent as batches to Gemini, OpenAI, Claude or
  Mistral at half price, answered within a day and collected after the app was closed; cached
  input billed at its own price; both costs shown before starting
  ([decision](decisions/2026-10-27-economy-mode-and-prompt-caching.md)).
- **Pipelines**: a job's settings saved under a name and applied in one click; exported as an
  importable JSON file and a Markdown record for a methods section, and imported.
- **Provenance of each request**: `provenance.json` records the settings each request carried
  (reasoning effort, output limit, media resolution), for the step that reads and the step
  that combines, kept with each result when it was sent; offline results are marked `offline`
  ([decision](decisions/2026-10-29-provenance-request-settings.md)).
- **Design**: the Graphite and highlighter interface ([DESIGN.md](../DESIGN.md)), responsive
  from 640 to 2560 pixels, light and dark, with a sidebar that lists the jobs and folds to a
  rail, and an app icon
  drawn from the same mark. Under the tasks in New job, a diagram of how each task splits
  and joins the work, drawn with the user's own files.
- **Kalamu**: the app renamed from Scriptorium, with the "two witnesses" icon and warm graphite
  neutrals; installs of 0.1 and 0.2 have their jobs, settings and keys moved over on the first
  start; the strip of pages shows thumbnails of scans and images
  ([decision](decisions/2026-10-28-kalamu.md)).

## Next

The [2 November checkpoint](decisions/2026-11-02-checkpoint.md), after version 1.0.0, is done:
pages read together settled when a job is created and a call that keeps no page counted,
workflow steps made once and tied to their project at launch, the proposal job's files apart,
rules and estimates from Rust, and the duplication it found removed. Next comes the release that
carries it, then [#50](https://github.com/fmadore/Kalamu/issues/50).

- **Project workspaces, what is left of
  [#57](https://github.com/fmadore/Kalamu/issues/57)**: a document's descriptive metadata
  (with [#55](https://github.com/fmadore/Kalamu/issues/55)); renaming a tag across a project;
  and later, search through the text of every result.
- **Project dictionary, what is left of
  [#48](https://github.com/fmadore/Kalamu/issues/48)**: a live run of proposed merges with
  each provider, on a field of a few hundred rows; comparing rows far apart in the alphabet
  (shared words, sound), which the groups leave out; import with conflicts previewed; an
  accepted dictionary versioned and given to later analyses as a fingerprinted input (with
  #49); and telling apart two people who share a spelling, which needs mentions anchored in
  the text (#47).
- **Workflows, what is left of [#49](https://github.com/fmadore/Kalamu/issues/49)**: steps
  that read several inputs (the text and an accepted dictionary, with #48), project-level
  steps, an export step that gathers results across steps, the whole run's estimate computed
  in the engine, and restarting a run from a step after the definition changed; a new run
  that keeps the corrections made in the last one, whose steps after a review are otherwise
  read again from uncorrected text. A live run of
  a two-step workflow, and one in economy mode across a restart. Try *Then* and the examples
  with one or two colleagues before going further. Several jobs' or documents' texts sent into
  one new round together, from a project's selection.
- **Prompts, house cleaning**: to scope with the maintainer before the next prompt work.
- **Run Gemini against the live API**, as OpenAI, Claude and Mistral were, and measure costs
  against the estimates. Offline Whisper still needs a live run too, as do the models added on
  26 October: Mistral's chat models for analyses and GPT-6.1 Terra (see the
  [decision](decisions/2026-10-26-terra-and-mistral-chat.md)); Terra on historical pages
  before it is considered as OpenAI's default. GLM 5.3 and GLM 5.3 Flash were checked with
  single live calls on 27 October, not yet through a whole job; DeepSeek V4.1 Flash, added on
  28 October, not yet at all: its `reasoning_effort` and economy mode are taken from Mistral's
  documentation ([decision](decisions/2026-10-28-deepseek-through-mistral.md)).
- **Gemini 3.5 Transcribe, live**: added on 2 November as Gemini's model for recordings, with a
  request of its own and its 77 languages, from Google's documentation
  ([decision](decisions/2026-11-02-gemini-transcribe.md)). Run it on a real recording with
  speakers and timestamps, and check the answer's shape and a minute's cost.
- **Economy mode, live**: a small batch with each provider, to check the documented shapes
  (Gemini's and Mistral's pages disagree on some), what cached input really saves, and
  Claude's one-hour cache inside batches. Later, Gemini batches through a JSONL file, to lift
  the 20 MB inline limit, and Voxtral transcription in batches once its format is documented.
  Delete the JSONL files a batch leaves in the user's OpenAI and Mistral storage once its
  answers are collected.
- **YouTube**: a live run on long and multilingual videos; long videos in pieces once Gemini's
  clipping of YouTube links covers the sound track again; the cost before the run from a
  `countTokens` call; check the embedded player in the packaged app, whose `tauri.localhost`
  origin YouTube must accept, and the links that open a video on YouTube, which every build up
  to 0.4.0 blocked (the opener allowed no address)
  ([decision](decisions/2026-10-27-youtube-links.md)).
- **Text extraction**: ALTO and PAGE XML where models return coordinates; checks for skewed
  and blurry pages.
- **Tables of contents and corrections, live**: a run of each against the live APIs, mixing
  providers (Gemini reading, Mistral combining), and a test of the economy path for a table of
  contents. The first live run, Claude Sonnet 5.5 on a 12-page issue of *Islam Info*, found
  articles on two pages only: read the other pages' answers, and try Claude at a higher effort
  than `low` for reading pages
  ([decision](decisions/2026-10-29-contents-review-and-answer-language.md)).
- **Analyses**: agreement between set-ups across a whole job (exact agreement and Cohen's kappa
  for choices, overlap for lists), shown in Jobs; a column added to an `.xlsx`. Project-wide
  entity/label reconciliation is tracked in [#48](https://github.com/fmadore/Kalamu/issues/48)
  and the research-workflow sequence below.
- **Design improvements left from [issue #1](https://github.com/fmadore/Kalamu/issues/1)**,
  closed on 28 October when the name and icon were decided (its closing comment keeps these
  here): a collation table in Jobs (pieces × set-ups, the sigla as column heads); an
  interrupted or paused job in the sidebar's foot with **Resume** (the sidebar already groups
  unfinished jobs, without the button); a page with a correction checked and captured; the
  error colour checked against the hand blue and the highlighter; and the placeholder dashes
  and the apparatus's `]` and `:`, still in the markup, either declared typesetting or moved
  to the translation files.

## Toward the Store release

The first Microsoft Store submission, version 1.0.0. What the Store checks and how to submit
are in [microsoft-store.md](microsoft-store.md), the package in
[packaging-msix.md](packaging-msix.md), the listing text in [store-listing.md](store-listing.md)
and the [privacy policy](privacy.md).

Done: the name (reserved on 28 October 2026) and the package identity in the manifest
(`49346FMadore.Kalamu`), Report a problem, MSIX packaging with FFmpeg and
the licences inside, the `.msixbundle` and `kalamu-samples.zip` in the release workflow,
version 1.0.0, third-party notices generated at build time, the sample files (with a 1755
manuscript and a 1915 newspaper in the public domain), and the listing, What's new and
certification notes for 1.0.0, the privacy policy on GitHub Pages, the screenshots and the
submission. Version 1.0.0 passed certification and is
[in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H) since 2 November 2026. Later versions follow the update steps
in [microsoft-store.md](microsoft-store.md).

## Research workflows (planned)

The following improvements extend Kalamu from individual jobs to reusable research workflows.
They are agreed product directions, not shipped features or release-date commitments. The
current release checks and reliability work above remain first. Issues hold the detailed scope,
dependencies and acceptance criteria; this roadmap holds the sequence, without duplicating them.

The three central goals are:

- **Project workspaces**: organise collections and documents, with the original sources and
  every associated extraction, correction and analysis accessible from each document. A
  document can belong to several collections without duplicating files or results. Activity
  keeps job controls available; the document view brings work from different runs together.
- **A project dictionary**: review all entities and analytical labels across an explicitly
  selected set of jobs/documents, ask an LLM for proposed merges, then accept, reject or adjust
  them. Keep original expressions and source evidence, stable IDs, aliases and reversible
  decisions. Entity identity and thematic-label synonymy have separate rules.
- **Reusable multi-step pipelines**: save a workflow from extraction through user-defined
  analyses and export. Each step chooses its model, prompt and upstream inputs, with optional
  review checkpoints. Existing single-job pipelines remain supported.

Example: extract text → optional text review → extract entities → review dictionary merges →
run another analysis using the text and accepted dictionary → export. Each run records the
exact inputs used at every step and resumes without repeating completed unchanged work.
A pending merge decision is never treated as approval.

### Shared foundation and first workflow

| Order | Improvement | Tracking |
| --- | --- | --- |
| 1 | Project workspaces, collections and stable document identity; group originals and results across jobs (first increment done) | [#57](https://github.com/fmadore/Kalamu/issues/57) |
| 2 | Frozen reviewed-result snapshots, stable source anchors and linked downstream jobs; shared with post-interview analysis (first increment done) | [#47](https://github.com/fmadore/Kalamu/issues/47), existing [#38](https://github.com/fmadore/Kalamu/issues/38) |
| 3 | Reusable workflow definitions and runs; begin with extraction → analysis → export, then add review checkpoints and project-level steps (first increment done, with review checkpoints) | [#49](https://github.com/fmadore/Kalamu/issues/49) |
| 4 | Project-wide entity/label inventory, model-proposed merges, researcher decisions and local dictionary import/export (done: inventory, researcher decisions, export, model-proposed merges) | [#48](https://github.com/fmadore/Kalamu/issues/48) |
| 5 | Reviewed document units from contents entries, with item-level extraction, analysis and export | [#50](https://github.com/fmadore/Kalamu/issues/50) |

The project workspace can ship incrementally before the complete workflow builder. It owns
project/document identity and collection membership (#57); result snapshots and dependencies
are owned by #47. Workflows freeze the documents and result revisions selected at launch,
so later collection edits do not rewrite earlier runs or start extra processing. Dictionary
review can be project-wide or scoped to a visible collection/selection. Existing standalone
jobs/results can be attached without rerunning them. Tags remain useful filters alongside
collections. Later text edits flag dependent findings as out of date while retaining their
original snapshots. Arbitrary scripts, loops and a general graph editor remain outside the
first workflow increment.

### Supporting improvements

| Improvement | First useful increment and dependency | Tracking |
| --- | --- | --- |
| Research-question screening | Relevance preset, explanations, filters and researcher selection; verified evidence spans build on #47 | [#51](https://github.com/fmadore/Kalamu/issues/51) |
| Ordered image documents and paired transcriptions | Import photographs as one document with reviewable order; then pair existing text with scans | [#52](https://github.com/fmadore/Kalamu/issues/52) |
| Persistent image preparation | Crop, processing rotation and adjustable spread splitting; transformation recipes preserve originals and source coordinates | [#53](https://github.com/fmadore/Kalamu/issues/53) |
| Context-aware processing | Bounded neighboring pages; then date/place proposals with evidence, literal readings and explicit inference status | [#54](https://github.com/fmadore/Kalamu/issues/54) |
| Archival metadata and export profiles | Supplied source descriptions and reusable local CSV/JSON mappings, separate from extracted/inferred values | [#55](https://github.com/fmadore/Kalamu/issues/55) |

Scope is local research preparation, review and export. External collection-system integration
and publishing are excluded from this work. Implementation follows the architecture checkpoints
in AGENTS.md and updates the specification and user-facing documentation when features ship.

## Later

- French and German interface.
- OpenRouter for open-weights models.
- **Free offline text extraction with Tesseract** for clean printed PDFs and images, without
  an account, API key or per-page API charges; language data installed through the app, then
  processing entirely on the PC. The same review, exports, caching and resume as cloud
  extraction, with the engine and language data recorded in provenance. Handwriting still
  needs a suitable cloud model. Planned, not available in 1.0.0; no release date set:
  [issue #46](https://github.com/fmadore/Kalamu/issues/46).
- Linking names to imported local authority records (an id from candidate records, or none,
  with a reason), building on the project dictionary in #48.
- Tables of contents of text files (a long Word report, a transcript, read in passages).
  Ordered photographs and item-level extraction/export are tracked in #52 and #50 above.
- Post-interview analysis with research profiles and evidence-linked themes/quotations:
  [#38](https://github.com/fmadore/Kalamu/issues/38), using the shared snapshot foundation.
- Explore assisted TEI encoding with reusable project profiles:
  [#37](https://github.com/fmadore/Kalamu/issues/37); feasibility work, not a committed feature.
- Correcting ALTO XML with its coordinates kept, as iwac-ai-pipelines does.
- French and German Store listings, with the translated interface.
- macOS: signing and notarisation with an Apple Developer ID, so Gatekeeper does not ask; the
  application menu's labels through the translation files; a first test on a real Mac (the
  Keychain's prompts, ⌘Q, the Dock and the menu-bar icon, YouTube's player in WKWebView).
