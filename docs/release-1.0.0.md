The first version of Kalamu for the Microsoft Store, and the first that is not a test build:
read scanned and handwritten documents, transcribe interviews and analyse texts with the AI
provider of your choice, or offline, and check every result against its source.

## What's new since 0.4.0

- **Search in the review** (Ctrl F): find a name or a phrase across every page of a document
  or every piece of a recording, corrections included, and go to each match, a recording
  playing from its line. Capitals, accents and line breaks are forgiven. In an analysis, the
  fields whose answers or reasons hold the words are kept, and in a text document the words are
  marked in the document too; in a table of contents, the entries are filtered.
- **A glossary**, beside Settings, for colleagues new to these tools: OCR and HTR, LLM,
  tokens, reasoning effort, ground truth, error rates, hallucination, provenance and more, in
  plain words, and what each export format suits. A small help mark beside the headings that
  use these words opens it on the right one.
- **Report a problem.** A flag in every review's header, and a button in Settings › About,
  open a GitHub form to report output that is inappropriate, harmful or invented, or a bug. The
  form opens in your browser; Kalamu sends nothing itself.
- **One frame for the three reviews**: the text, analysis and table of contents reviews share
  their header, source pane, handles, strip of pages and keys (Ctrl Enter to mark reviewed,
  Page Up and Page Down to turn pages).
- **Results that cannot be wrong unnoticed**: an answer cut off at the model's output limit is
  marked and listed in provenance; a file changed on disk since it was added is refused.
- **Work neither lost nor paid twice**: quitting waits for the pieces already sent, and a job
  shows what it cost over all its runs.
- **Results that reach you**: a document's files are written even when one page keeps failing,
  that page marked `[Not read: reason]`.
- **Error rates after review, for any job**, against every page you corrected or marked as
  reviewed.
- **Provenance of each request**: `provenance.json` records the settings each request carried
  (reasoning effort, output limit, media resolution), for both steps of a table of contents.
- **Tables of contents**: the review follows the page shown, and the entries can be written in
  the language you choose.
- **Links open in your browser** in the installed app: YouTube links, and the links in
  Settings, did nothing in 0.4.0.
- **New job offers only what will run**: which files, models and formats suit each task now
  comes from the engine, which checks them again when a job is created, so a pipeline saved
  for another task cannot start a job a model cannot do.
- **Icons centred in their buttons** everywhere: an icon-only button could draw its icon off to
  the left.
- **Third-party notices**: every crate and package built into the app, with its licence, in the
  `licenses` folder beside FFmpeg's.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself (link once
  the listing is live).
- `Kalamu_1.0.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.0.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `kalamu-samples.zip`: a short recording, a scanned page and a text to try the app with.
- `Kalamu_1.0.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing the 1.0.0 installer over 0.4.0 keeps your jobs, settings, presets and keys.

## Known limitations

- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*,
  or install from the Store.
- Gemini, offline Whisper, economy mode, tables of contents and corrections have had fewer runs
  against the live services than OpenAI, Claude and Mistral text extraction: try a small job
  first.
- English interface only; French and German will follow.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT). The fonts are under the SIL Open Font License. Every other
component is listed with its licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
