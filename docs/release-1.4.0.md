Kalamu 1.4.0 reads printed pages on your own computer with Tesseract, free and with nothing
sent, and writes ALTO XML with every word in its place. It adds a page listing every job, gives
the views the whole window with larger type, makes the review smoother, and the Mac app is now
signed and notarised by Apple. For what 1.3.0 changed, see the
[1.3.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.3.0).

## What's new since 1.3.0

- **Offline text extraction with Tesseract**: printed pages read on the computer, with no key,
  no account and no cost, and without an internet connection once a language is downloaded in
  **Settings › Offline**. Choose the languages in New job: Tesseract does not detect them.
  It suits clean print; handwriting, faded or damaged pages and complex layouts still read
  better with a cloud model. Its speed depends on the processor, from about a second to several
  seconds a page, and up to four pages are read at once. Latin-script languages without a model
  of their own (Hausa, Mooré, Wolof, Bambara…) are read with Tesseract's Latin script model.
- **ALTO XML**: each page's text with where every block, line and word sits, in the standard
  digital libraries, eScriptorium and Transkribus read. Tesseract places each word; from Mistral
  OCR the lines and words are estimated within its paragraphs, and the file says so. Tesseract
  also gives a searchable PDF, and a corrected page keeps its words' places in both.
- **All jobs**: a page with every job in a table, searched by name, workflow or the name of a
  file it read, filtered by task or status, and sorted by date or name. Open, rename, add to a
  project or delete from each row, or several rows at once. The sidebar's *Finished* group now
  lists the eight most recent jobs, with a link to the page.
- **The whole window**: New job, Settings, the glossary, a job's page, pipelines and the review
  use the full width, while paragraphs keep a comfortable line length. Type is larger, never
  below 14 px, and the text being checked is set at 19 px. Grids, tables and diagrams fit narrow
  windows without scrolling sideways.
- **A smoother review**:
  - Zoom the scan with Ctrl + wheel (⌘ on a Mac) or a pinch, about the point under the pointer;
    drag to move the page, double-click to zoom on a place or back.
  - Choosing a difference in either text brings it level in the other.
  - The readings box can be dragged, or moved with the arrow keys, to show the sentence it
    covers.
  - Curly and straight quotes and apostrophes count as the same reading, in the differences and
    in the error rates.
- **New prompt** writes a prompt from nothing for a chosen task, starting from that task's
  default request and fields.
- **Projects in the sidebar**: listed under Projects, and a finished job dragged onto one is
  added to it.
- **Settings in tabs**: API keys, Offline, General and About.
- **The Mac app is signed with an Apple Developer ID and notarised**, so macOS opens it after
  its usual question about an app downloaded from the internet, without a detour through System
  Settings.

## Fixed

- The difference arrows in the review always started again at the first difference.
- The pipeline examples showed vertical scroll bars.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.4.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.4.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.4.0_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.4.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, offline Whisper and offline Tesseract; their models and language
data are downloaded from Settings when wanted. Installing 1.4.0 over an earlier version keeps
your jobs, settings, presets and keys.

## Known limitations

- **On a Mac updated from 1.3.0 or earlier**, macOS asks once more to let Kalamu use the keys
  saved in your Keychain, because the earlier versions were not signed with the Developer ID:
  choose *Always Allow*. Later updates no longer ask.
- **The Mac version** is built and tested on Macs in GitHub Actions, but has not yet been used
  on a real Mac. Please report what does not work with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose). Intel Macs are not
  supported.
- **Tesseract** reads print, not handwriting, and does not detect the language. It has not yet
  been tried with networking off in the installed app, nor from a user folder whose name has
  accents.
- **Still in English**: the macOS application menu, model descriptions, the error a failed page
  keeps, and the files Kalamu writes. German will follow.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Items are whole pages of a PDF.** Two articles on one page each take the whole page; a
  multi-page image's items can be made and corrected, but not read or exported on their own.
- **Unsigned Windows installers**: Windows SmartScreen will warn. Choose *More info*, then
  *Run anyway*, or install from the Store.
- Offline Whisper, tables of contents and corrections have had fewer runs against the live
  services; OpenAI's batches had not answered within 35 minutes in the last check. Try a small
  job first.
- A workflow run again starts from the models' text, not the corrections made in the last run.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT); offline text extraction uses Tesseract 5.5.2 and its
language data (Apache 2.0) with Leptonica (BSD), and PDF pages are drawn with hayro (MIT). The
fonts are under the SIL Open Font License. Every other component is listed with its licence in
`licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
