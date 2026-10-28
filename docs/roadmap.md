# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (27 October 2026)

Early development; [v0.2.0](https://github.com/fmadore/Scriptorium/releases/tag/v0.2.0) is
the latest test build, for Windows on ARM64, with FFmpeg bundled. Everything below is tested
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
- **Analyses.** With Gemini, OpenAI, Claude and Mistral, including the Z.ai GLM models Mistral
  hosts. Fields defined by the researcher
  (text, lists of names or terms, one or several choices, reasons), sent as each provider's
  structured output and checked on return; five ready-made analyses (research summary, short
  summary, scholarly publication, people, places and organisations, tone); a field editor in Prompts; CSV with a
  column per field.
- **Review.** The source on a dark lightbox beside the text, in panes that can be resized,
  with another set-up's whole text beside them on demand; differences between set-ups and
  corrections highlighted in the text, their readings shown on pointing, any of which can be
  taken as a correction; corrections kept apart from the AI output; marking as reviewed, with a strip of
  pages showing what is left; a player for
  recordings with speakers that can be named; an analysis review that collates answers by
  field and finds names in the document.
- **Comparison mode** with character error rates against corrected pages.
- **Economy mode and prompt caching**: a job sent as batches to Gemini, OpenAI, Claude or
  Mistral at half price, answered within a day and collected after the app was closed; cached
  input billed at its own price; both costs shown before starting
  ([decision](decisions/2026-10-27-economy-mode-and-prompt-caching.md)).
- **Pipelines**: a job's settings saved under a name and applied in one click; exported as an
  importable JSON file and a Markdown record for a methods section, and imported.
- **Design**: the Graphite and highlighter interface ([DESIGN.md](../DESIGN.md)), responsive
  from 640 to 2560 pixels, light and dark, with a sidebar that lists the jobs and folds to a
  rail, and an app icon
  drawn from the same mark.

## Next

- **Refactor the review**: extract the frame shared by the transcription and analysis reviews,
  and split New job into smaller components (see the
  [25 October checkpoint](decisions/2026-10-25-analyses-review-and-pipelines.md)).
- **Run Gemini against the live API**, as OpenAI, Claude and Mistral were, and measure costs
  against the estimates. Offline Whisper still needs a live run too, as do the models added on
  26 October: Mistral's chat models for analyses and GPT-6.1 Terra (see the
  [decision](decisions/2026-10-26-terra-and-mistral-chat.md)); Terra on historical pages
  before it is considered as OpenAI's default. GLM 5.3 and GLM 5.3 Flash were checked with
  single live calls on 27 October, not yet through a whole job.
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
  origin YouTube must accept
  ([decision](decisions/2026-10-27-youtube-links.md)).
- **Text extraction**: ALTO and PAGE XML where models return coordinates; checks for skewed
  and blurry pages.
- **Analyses**: agreement between set-ups across a whole job (exact agreement and Cohen's kappa
  for choices, overlap for lists), shown in Jobs; a column added to an `.xlsx`; grouping
  variant spellings of the same name across a job, confirmed by the researcher.
- **Design decisions still open**: a collation table in Jobs, the interrupted
  job with Resume in the side menu: [issue #1](https://github.com/fmadore/Scriptorium/issues/1).

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
- **Release workflow**: tag, native x64 and ARM64 builds with FFmpeg and whisper.cpp, installers
  and one `.msixbundle` attached to a draft release published once complete.
- **Version 1.0.0** in every manifest: the Store refuses a first segment of 0.
- **Privacy policy online**: GitHub Pages from `docs/` on `main`, so it is served at
  `fmadore.github.io/Scriptorium/privacy`.
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
- French and German Store listings, with the translated interface.
