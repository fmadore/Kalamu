Kalamu 1.4.1 is the first version for the Mac App Store, beside the disk image and the Microsoft
Store. It asks once for each AI provider before sending your files to it, and fixes what an
architecture review after 1.4.0 found: a page typed by hand missing from ALTO and the searchable
PDF, word positions guessed but presented as measured, upside-down phone photos read as they
were, and the window freezing while results were written. For what 1.4.0 changed, see the
[1.4.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.4.0).

## What's new since 1.4.0

- **In the Mac App Store**: the same app, run in macOS's App Sandbox. It asks for a folder the
  first time it needs to save results beside your files there, and remembers it at later
  launches, also for a folder on a disk that is not connected at the moment. Your jobs,
  settings and presets from the disk image are copied over the first time it starts.
- **Asked before anything is sent**: before the first job, workflow or dictionary proposal sent
  to Google Gemini, OpenAI, Anthropic Claude or Mistral, Kalamu says what goes to that provider
  (the pages, recordings or text, with the personal information they hold), that it goes under
  your own account and the provider's terms, and where the privacy policy is. You are asked
  once for each provider; Whisper and Tesseract, which send nothing, are never asked about. The
  same on Windows and macOS.

## Fixed

- **A page typed by hand reaches ALTO and the searchable PDF.** When Tesseract or Mistral OCR
  read a page as empty and you typed its text, the `.txt` had it but ALTO and the searchable
  PDF left the page blank. It is now one block of text over the page, marked as estimated.
- **Word positions are no longer guessed and shown as measured.** When a correction added text
  the engine never placed, a skipped column for instance, its words were squeezed into the box
  of the word before them. Such a page is now placed as one block, marked as estimated in ALTO.
- **Photos taken upside down or mirrored** (a phone's EXIF orientation) are turned upright
  before they are read. Tesseract, which ignores that information, read them as stored.
- **The window no longer freezes** while a correction, speakers' names or an export is saved
  during a running job, or while error rates, a comparison, a review or a folder of many files
  is prepared.
- **A file dropped with its folder** is listed once, not twice.
- **Dialogs keep the keyboard inside**: the export, attach and close dialogs move the focus into
  themselves, keep Tab within them and give the focus back when they close.
- **What to check counts every reason** (failed and not read included) as the engine found them.
- Retrying a running job's failed pages no longer puts them back behind its back.

## Download

- **Mac App Store**: once Apple has reviewed it.
- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.4.1_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.4.1_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.4.1_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.4.1.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, offline Whisper and offline Tesseract; their models and language
data are downloaded from Settings when wanted. Installing 1.4.1 over an earlier version keeps
your jobs, settings, presets and keys.

## Known limitations

- **The Mac versions** are built and tested on Macs in GitHub Actions, but have not yet been
  used on a real Mac. Please report what does not work with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose). Intel Macs are not
  supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, or write its results again,
  add its folder to a new job once, so macOS lets the app reach it. macOS may also ask once
  more to let Kalamu use the keys saved in your Keychain: choose *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language. It has not yet
  been tried with networking off in the installed app, nor from a user folder whose name has
  accents.
- **Still in English**: the macOS application menu, model descriptions, the error a failed page
  keeps, and the files Kalamu writes. German will follow.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Items are whole pages of a PDF.** Two articles on one page each take the whole page.
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
