Kalamu 1.1.1 keeps your place: the glossary, correcting a text and zooming a page no longer
send you back to the start, and each job in the list has its own menu. For what 1.1.0 added
(projects, a project dictionary, workflows), see the
[1.1.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.1.0).

## What's new since 1.1.0

- **A menu on each job in the list**: right-click a job, or press the keyboard's menu key, to
  rename it in place, add it to one of your projects, or delete it.
- **Words explained where they are**: the ? beside a word opens its definition in a bubble beside
  it, so a job being set up in New job stays as it was. A link in the bubble opens the whole
  glossary, now in alphabetical order.
- **Correct where you are reading**: *Correct* opens the editor on the word you clicked or
  selected, or on the first line in view, and keeps that line where it stood on screen. The
  editor is as tall as the text, so it scrolls as the text did.
- **Large windows**: the Workflows page sets its examples side by side, as many as fit.

## Fixed

- A page zoomed wider than its pane can be scrolled to its left edge again.
- New job no longer scrolls sideways when an estimate line is long, and its cost column reaches
  the foot of the page however long the form.
- *Thinking* lists each level once, the suggested one marked, rather than the suggested level
  twice.
- `kalamu-samples.zip` includes a whole 16-page newspaper issue of 1915, scans only, to try a
  table of contents with.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.1.1_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.1.1_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.1.1.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing 1.1.1 over 1.1.0 or 1.0.0 keeps your jobs, settings, presets and keys.

## Known limitations

- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*,
  or install from the Store.
- Gemini, offline Whisper, economy mode, tables of contents, corrections and the dictionary's
  proposed merges have had fewer runs against the live services than OpenAI, Claude and Mistral
  text extraction: try a small job first.
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
