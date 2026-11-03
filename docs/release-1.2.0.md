Kalamu 1.2.0 reads the articles of a newspaper issue, or the letters of an archival file, on
their own: a table of contents' entries become items you can correct, then extract, analyse or
export one by one. It also brings simpler default prompts and a data folder named for Kalamu.
For what 1.1.2 changed, see the
[1.1.2 release](https://github.com/fmadore/Kalamu/releases/tag/v1.1.2).

## What's new since 1.1.2

- **Items of a table of contents**: in a table of contents' review, *Make items from the
  entries* turns each checked entry into an item: an article, a letter, a chapter, with its
  title and pages. Correct an item's pages (`1, 3-5, 8 (left)`), join a continuation, split,
  add or remove items; a page shared by several items is named as such, since a whole-page cut
  cannot isolate one article's text. Send the items you select to a new text extraction or
  analysis: each is read as a PDF of its own pages, its results are written under the item's
  name beside the source, and `provenance.json` names the item. Giving an item other pages
  marks the jobs that read it out of date. Export writes each selected item's pages as a PDF,
  with the list as CSV and JSON.
- **One default prompt per kind of material**: text extraction keeps three, for a printed or
  typed document, a newspaper or magazine, and a handwritten document or manuscript, each for
  any language and either reading direction, Arabic and Ajami included. The lines (as on the
  page, or as running text) are chosen in New job for every prompt. Thirteen defaults replace nineteen,
  with one shared set of marks (`[illegible]`, `[word?]`, `[margin: ...]`). Saved pipelines,
  workflows and shared files that named a retired prompt use its replacement; an edited copy
  of one stays your own prompt.
- **The data folder is called Kalamu Data**: on Windows, `%LOCALAPPDATA%\Kalamu Data` instead
  of `io.github.fmadore.kalamu`; on macOS, `~/Library/Application Support/Kalamu`. The first
  start of 1.2.0 moves the earlier folder once, with your jobs, results and settings; if it
  cannot, the old folder stays in use. Settings shows where it is, also for the Store version.
- **Searchable PDFs, shown before you start**: New job's diagram for text extraction says
  whether the chosen model also gives a searchable PDF (today Mistral OCR) or text only.
- **Clearer interface text**: a copy-editing pass over the whole interface; the glossary names
  the controls it explains as New job does.

## Fixed

- Mistral Large 4 is estimated at its current price, half what the catalogue had.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.2.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.2.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.2.0_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.2.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing 1.2.0 over an earlier version keeps your jobs, settings, presets and keys.

## Known limitations

- **Items are whole pages of a PDF.** Two articles on one page each take the whole page; a
  column or passage cannot be cut out yet. A multi-page image's items can be made and
  corrected, but not read or exported on their own. The numbers printed on the pages are not
  read yet: an item's pages are numbered as the file's.
- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*,
  or install from the Store.
- **The Mac app is not signed with an Apple Developer ID**, so the first time macOS refuses to
  open it: open **System Settings → Privacy & Security**, choose **Open Anyway** beside Kalamu,
  and confirm. macOS may also ask once, and again after an update, to let Kalamu use the key
  saved in your Keychain.
- **The Mac version is new**: it is built and tested on Macs in GitHub Actions, but has not yet
  been used on a real Mac. Please report what does not work with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose). Intel Macs are not
  supported, and the application menu is in English only.
- Gemini (Gemini 3.5 Transcribe included), offline Whisper, economy mode, tables of contents,
  corrections and the dictionary's proposed merges have had fewer runs against the live
  services than OpenAI, Claude and Mistral text extraction: try a small job first.
- A workflow run again starts from the models' text, not the corrections made in the last run.
- English interface only; French and German will follow.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT). The fonts are under the SIL Open Font License. Every other
component is listed with its licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
