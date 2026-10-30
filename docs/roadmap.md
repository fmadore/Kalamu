# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (30 October 2026)

**Version 1.0.0**, the first Microsoft Store release, is ready to build: the app, its package
and its listing are done, and what is left before submitting needs Partner Center, the
repository's settings and real keys ([Toward the Store release](#toward-the-store-release)).
[v0.4.0](https://github.com/fmadore/Kalamu/releases/tag/v0.4.0) is the latest public test
build, for Windows on x64 and ARM64, with FFmpeg bundled. Everything below is tested against
recorded provider responses. Text extraction, transcription and analyses have also run against
the live OpenAI, Claude and Mistral APIs with real keys (`cargo run --example live`, see
[development.md](development.md)); Gemini has not yet, for want of credit on the account.

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
- **A glossary** for colleagues new to these tools: a page beside Settings explaining fifteen
  words the app uses (OCR and HTR, LLM, Whisper, API key, prompt, set-up, pipeline, tokens,
  reasoning effort, economy mode, ground truth, error rates, hallucination, provenance,
  machine-readable) and what each export format suits, with a filter, links between the
  words, and a help mark beside the headings that use them; written in the translation files
  like all interface text ([decision](decisions/2026-10-30-glossary.md)).
- **Icons centred in their buttons**: the icon component's style no longer reaches buttons
  named `icon`, which drew their icon at the left whenever its style loaded last.
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

The [29 October checkpoint](decisions/2026-10-29-checkpoint.md)'s fixes and refactor are done,
but for these two, which change no result:

- **The cost of failed answers**: answers billed but failing their fields counted in the cost
  shown, which needs a failed call to carry its usage.
- **Smaller provenance gaps** from the same checkpoint: whether Whisper used voice detection,
  speaker names, a table of contents' usage and cost per step.
- **A home for jobs**: a page listing every job with its task, models, sources, date, status,
  review progress and cost; search by name, file name and tag; sorting and filters by task,
  model, pipeline, date and status; tags the researcher gives jobs, a tag's filter serving as
  a folder. New jobs record the pipeline they were started from. Later, a search through the
  text of every result.
- **Run Gemini against the live API**, as OpenAI, Claude and Mistral were, and measure costs
  against the estimates. Offline Whisper still needs a live run too, as do the models added on
  26 October: Mistral's chat models for analyses and GPT-6.1 Terra (see the
  [decision](decisions/2026-10-26-terra-and-mistral-chat.md)); Terra on historical pages
  before it is considered as OpenAI's default. GLM 5.3 and GLM 5.3 Flash were checked with
  single live calls on 27 October, not yet through a whole job; DeepSeek V4.1 Flash, added on
  28 October, not yet at all: its `reasoning_effort` and economy mode are taken from Mistral's
  documentation ([decision](decisions/2026-10-28-deepseek-through-mistral.md)).
- **Gemini 3.5 Transcribe** in the catalogue.
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
  for choices, overlap for lists), shown in Jobs; a column added to an `.xlsx`; grouping
  variant spellings of the same name across a job, confirmed by the researcher.
- **Design decisions still open**: a collation table in Jobs, the interrupted
  job with Resume in the side menu: [issue #1](https://github.com/fmadore/Kalamu/issues/1).

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
certification notes for 1.0.0. The [v1.0.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.0.0)
is built and published, its `.msixbundle` ready to upload.

Left, in this order:

- **Privacy policy online**: GitHub Pages from `docs/` on `main`, so it is served at
  `fmadore.github.io/Kalamu/privacy`.
- **Live runs and screenshots** from the installed package, a decision on a capped test key for
  the certification team, then the acceptance checklist and the submission.

## Later

- French and German interface.
- OpenRouter for open-weights models.
- Offline text extraction with Tesseract.
- Linking names to authority records (an id from candidate records, or none, with a reason).
- Tables of contents of text files (a long Word report, a transcript, read in passages), and of
  a folder of photographs taken as one document; cutting a PDF into one file per entry, to
  analyse each article.
- Correcting ALTO XML with its coordinates kept, as iwac-ai-pipelines does.
- French and German Store listings, with the translated interface.
