Kalamu 1.5.0 reads difficult print offline: two small OCR models trained on whole pages,
LightOnOCR-3 and GLM-OCR, run on your computer for the old newspapers, faded type and tables that
Tesseract reads badly, with nothing sent and no key. Whisper learns the names you give it and can
write a recording in English, and Whisper runs on a Mac's graphics processor. For what 1.4.1
changed, see the [1.4.1 release](https://github.com/fmadore/Kalamu/releases/tag/v1.4.1).

## What's new since 1.4.1

- **Offline OCR models** for *Extract text*: **LightOnOCR-3 1B** (1.1 GB) reads whole pages,
  columns and tables, and places each block of text on the page, so it gives ALTO XML and a
  searchable PDF by block; **GLM-OCR** (1.4 GB) is strong on tables and on Chinese, Japanese and
  Korean, and returns the text alone. Download them once in Settings › Offline, under *Text
  extraction with OCR models*; each is checked against its fingerprint before use. They run in
  llama.cpp, shipped inside the app: on the processor on Windows and on the graphics processor
  (Metal) on a Mac. Pausing a job stops the page being read, which is read again on resuming.
- **Speed measured on your computer.** These models are much slower than Tesseract on a PC's
  processor: from tens of seconds to over a minute a page. Settings and New job show the seconds
  a page took on your own computer once a model has read a few pages.
- **Whisper: names and words to spell.** New job takes a list of names, places and terms, one a
  line, which Whisper reads before each two minutes of recording; a job started from a project
  can fill it from the project's dictionary.
- **Whisper: written in English.** Large v3, Small and Base can write a recording in English
  whatever language is spoken (Turbo was trained without translation).
- **Whisper follows a recording that changes language**, detecting it again on the loudest
  thirty seconds of each two-minute stretch, and keeps each line to subtitle length, cut between
  words at the times Whisper gives them, so SRT and WebVTT cues fall where the words do. Words or
  lines said over and over, and subtitle credits written over silence, are cut back.
- **Whisper on a Mac's graphics processor** (Metal).
- **Workflows**: a step can be made from any prompt, shipped or your own, and a step's prompt
  can be changed.
- **Analyses**: right-click a name in a list to rename or remove it in every set-up's corrected
  answers.
- **Settings › Offline** lists only the Tesseract languages you downloaded; find the others by
  name.

## Fixed

- **New job**: a job's suggested name follows the files when the one it came from is removed;
  files and folders the task cannot read are named, with the task that reads them, instead of
  being dropped unseen; a compared set-up starts on the same prompt as the others.
- **French interface**: the cloud models' descriptions and the price note in Settings are in
  French.
- Menus and buttons that overflowed sideways (the job menu's projects, *Use this text*, the
  document view's actions, the transcription options) now wrap.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.5.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.5.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.5.0_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.5.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract; their models and
language data are downloaded from Settings when wanted. Installing 1.5.0 over an earlier version
keeps your jobs, settings, presets and keys.

## Known limitations

- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model, which the review makes easy. They have been
  tried on Kalamu's sample page on Windows x64, Windows ARM64 and macOS in GitHub Actions, where
  both read it correctly, but not yet measured on a range of archive pages (error rates against
  Tesseract and the cloud models are to come), nor run on a real Mac's graphics processor. On
  those small test machines a page took two to three minutes on Windows; a recent computer
  should be several times faster, which Settings will show. Using a PC's graphics processor is
  planned: [#122](https://github.com/fmadore/Kalamu/issues/122).
- **The Mac versions** are built and tested on Macs in GitHub Actions, but have not yet been
  used on a real Mac, Whisper's Metal mode included. Please report what does not work with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose). Intel Macs are not
  supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once, so macOS lets the app reach it. macOS may also ask once more to let Kalamu use the keys
  saved in your Keychain: choose *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language.
- **Still in English**: the macOS application menu, the offline models' descriptions in part,
  the error a failed page keeps, and the files Kalamu writes. German will follow.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Items are whole pages of a PDF.** Two articles on one page each take the whole page.
- **Unsigned Windows installers**: Windows SmartScreen will warn. Choose *More info*, then
  *Run anyway*, or install from the Store.
- Offline Whisper, tables of contents and corrections have had fewer runs against the live
  services. Try a small job first.
- A workflow run again starts from the models' text, not the corrections made in the last run.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). The
offline OCR models run in llama.cpp b12110 (MIT), whose build is described in the
[llama-b12110-1 release](https://github.com/fmadore/Kalamu/releases/tag/llama-b12110-1); the
models themselves, LightOnOCR-3 (Apache 2.0) and GLM-OCR (MIT), are downloaded when you ask.
Offline transcription uses whisper.cpp (MIT); offline text extraction also uses Tesseract 5.5.2
and its language data (Apache 2.0) with Leptonica (BSD), and PDF pages are drawn with hayro
(MIT). The fonts are under the SIL Open Font License. Every other component is listed with its
licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
