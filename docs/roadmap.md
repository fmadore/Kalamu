# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (29 October 2026)

Early development; [v0.4.0](https://github.com/fmadore/Kalamu/releases/tag/v0.4.0), with
detailed tables of contents and the correction of existing text, is the latest test build, for
Windows on x64 and ARM64, with FFmpeg bundled. Everything below is tested
against recorded provider responses. Text extraction, transcription and analyses have also run
against the live OpenAI, Claude and Mistral APIs with real keys (`cargo run --example live`,
see [development.md](development.md)); Gemini has not yet, for want of credit on the account.

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
  its line; a table of contents' entries filtered
  ([decision](decisions/2026-10-29-search-in-the-review.md)).
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

The [29 October checkpoint](decisions/2026-10-29-checkpoint.md) puts these before any new
feature (results that could be wrong unnoticed, work lost or paid twice, results that did not
reach the researcher, and the reviews' frame, are done):

- **The cost of failed answers**: answers billed but failing their fields counted in the cost
  shown, which needs a failed call to carry its usage.
- **The rest of the refactor**: New job, the job page and the sidebar in smaller components;
  `run.rs` and `batch.rs` split; the rules on which files and models suit a task given to the
  front end by the engine, which also checks them.
- **A home for jobs**: a page listing every job with its task, models, sources, date, status,
  review progress and cost; search by name, file name and tag; sorting and filters by task,
  model, pipeline, date and status; tags the researcher gives jobs, a tag's filter serving as
  a folder. New jobs record the pipeline they were started from. Later, a search through the
  text of every result.
- **A glossary** for colleagues new to these tools: a page beside Settings explaining the
  words the app uses (OCR and HTR, LLM, prompt, pipeline, tokens, reasoning effort, ground
  truth, character and word error rates, hallucination, provenance, machine-readable), and
  each export format with what it suits; linked from where those words appear, and written in
  the translation files like all interface text.
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
are in [microsoft-store.md](microsoft-store.md); the listing text in
[store-listing.md](store-listing.md) and the [privacy policy](privacy.md) are drafted.

- **Name**: done. Kalamu, reserved in Partner Center on 28 October 2026
  ([decision](decisions/2026-10-28-kalamu.md)); copy the package identity into
  [microsoft-store.md](microsoft-store.md#store-identity).
- **Report a problem**: a link in the review and in Settings that opens a GitHub issue form in
  the browser, so researchers can report inappropriate output (Store policy 11.16).
- **MSIX packaging**, ported from Live Translation: `@choochmeque/tauri-windows-bundle`, a
  committed `src-tauri/gen/windows/` with the reserved identity, taskbar icon variants and
  `resources.pri`, and the FFmpeg sidecars and licences inside the package.
- **Release workflow**: installers done. A tag builds the x64 and ARM64 installers natively,
  with FFmpeg and whisper.cpp, into a draft release published once both are attached
  (`.github/workflows/release.yml`, [decision](decisions/2026-10-28-release-workflow.md)); the
  `.msixbundle` is still to add, with the MSIX packaging.
- **Version 1.0.0** in every manifest: the Store refuses a first segment of 0.
- **Privacy policy online**: GitHub Pages from `docs/` on `main`, so it is served at
  `fmadore.github.io/Kalamu/privacy`.
- **Third-party notices** generated at build time for every Rust crate and npm package, PDF.js
  included, and installed with the licences.
- **Reviewer route**: sample files attached to the release, and a decision on giving the
  certification team a capped test key for the cloud features.
- **Live runs and screenshots** from the packaged app, then the acceptance checklist.

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
