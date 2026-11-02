Kalamu 1.1.2 is the first release for Macs with Apple Silicon, and makes Gemini 3.5 Transcribe,
Google's speech-to-text model, Gemini's model for recordings. For what 1.1.1 changed, see the
[1.1.1 release](https://github.com/fmadore/Kalamu/releases/tag/v1.1.1).

## What's new since 1.1.1

- **Kalamu on a Mac**: a disk image for Apple Silicon, macOS 13 and later, with FFmpeg and
  offline Whisper inside, as on Windows. Shortcuts use ⌘ where Windows uses Ctrl; keys are kept
  in the macOS Keychain; closing the window keeps Kalamu in the Dock, the menu-bar icon shows
  running jobs, and ⌘Q asks first while jobs run. Recordings the Mac's player cannot play as
  they are (Ogg, WebM) are converted to FLAC for the review.
- **Gemini 3.5 Transcribe** is Gemini's model for recordings, in place of Gemini 3.8 Flash,
  which is still offered. It labels speakers and times every line, and a chosen language is
  passed on to it; with none, it detects the language. YouTube links still go to Gemini 3.8
  Flash, which watches the video from its link.
- **The languages a model supports**: under Gemini 3.5 Transcribe and Whisper, New job lists the
  languages they document, in alphabetical order, and offers only those.
- The interface names the keychain and the notification area as each system calls them.

## Fixed

- A folder added from a drive a Mac has used no longer brings in the hidden `._name` files
  macOS leaves there.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.1.2_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.1.2_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.1.2_aarch64.dmg`: macOS 13 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.1.2.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing 1.1.2 over an earlier version keeps your jobs, settings, presets and keys.

## Known limitations

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
- **Gemini 3.5 Transcribe** follows Google's documentation and has not yet run against the live
  service: try a short recording first, and check its estimated cost against your Google bill.
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
