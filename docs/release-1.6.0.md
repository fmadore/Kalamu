Kalamu 1.6.0 analyses texts offline: two small language models from Google, Gemma 4 E4B and
E2B, fill in your analyses' fields, make tables of contents and correct text on your computer,
with nothing sent and no key. Whisper transcribes on a Windows PC's graphics processor, really
skips silence and music now, and stops within moments when a job is paused. For what 1.5.0
changed, see the [1.5.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.5.0).

## What's new since 1.5.0

- **Offline analyses with Gemma 4**
  ([#136](https://github.com/fmadore/Kalamu/pull/136),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-07-offline-language-models.md)).
  Download **Gemma 4 E4B** (6.1 GB) or, for smaller computers, **Gemma 4 E2B** (4.3 GB) once in
  Settings › Offline models, then choose *Language models (on this computer)* in New job for an
  analysis, a table of contents or a correction. They answer in the fields you defined, checked
  as the cloud models' answers are, and read texts of up to about 11,000 words and photographed
  pages; to analyse a PDF, extract its text first. They are Google's own 4-bit files, checked
  against their fingerprints before use, and run in llama.cpp, shipped with the app: on the
  graphics processor where Kalamu found one for the OCR models (as much of the model as its
  memory holds), on a Mac's through Metal, on the processor otherwise. Each needs several
  gigabytes of free memory while it runs.
- **Whisper on a Windows PC's graphics processor**
  ([#137](https://github.com/fmadore/Kalamu/pull/137),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-08-whisper-graphics-processor.md)).
  On Windows, Whisper now runs on the graphics processor through Vulkan, in `whisper-vulkan.exe`,
  a small program installed with the app and started only by it: a card from NVIDIA, AMD or
  Intel's Arc A and B series first, otherwise integrated graphics. Settings' graphics-processor
  choice, now *Run Whisper, the OCR and the language models on the graphics processor*, covers
  it. A recording that fails there is transcribed again on the processor, which Kalamu then keeps
  to until it restarts; each result's `provenance.json` names the device that transcribed it.
  On a Mac Whisper already ran on the graphics processor (Metal). Snapdragon PCs stay on the
  processor.
- **Whisper decodes only the speech.** With the voice-activity model downloaded (it comes with
  the first Whisper model), each two-minute stretch is searched for speech, only the speech is
  transcribed, and every line keeps its real time in the recording. A stretch of silence or
  music is not transcribed at all, so it costs nothing and Whisper invents no lines over it.
- **Pausing stops Whisper within moments**, after the thirty seconds of audio it is reading,
  instead of at the end of a two-minute stretch.
- **Whisper drops what it invents.** A line it was unsure of in a stretch it judged silent (the
  "Thank you." over room noise), and a line that only reads back the names and words you gave
  it, are left out. Those names and words now reach Whisper as the model's own tokens, cut to
  what it can hold from the top of your list.
- **On a Mac, Settings… and Help**
  ([#160](https://github.com/fmadore/Kalamu/pull/160)). The application menu has *Settings…*
  (⌘,), which opens Settings, also from the menu bar while the window is hidden; ⌘, works in the
  window too. The Help menu, empty until now, opens Kalamu's website and *Report a Problem…*.
  The menu-bar icon is drawn as macOS's own are, dark or light to suit the bar.

## Fixed

- **Quitting on a Mac soon after a Whisper transcription** no longer ends in a crash report
  ([#159](https://github.com/fmadore/Kalamu/pull/159)): the Whisper model, kept in memory for a
  few minutes after a recording, is released before the app exits.

- **Speech detection did nothing before this version.** When the voice-activity model was
  downloaded, Kalamu set whisper.cpp's switch for it and `provenance.json` said `"vad": true`,
  but through the interface Kalamu uses whisper.cpp ignored the switch and transcribed every
  stretch whole. Results made before 1.6.0 keep what their provenance says; transcribe a
  recording again to have its silences skipped.
- **Long pages with Gemini 4 Argon** no longer time out: the deadline for a provider's answer is
  now 15 minutes, as Google advises for hard prompts, instead of 10, so a long handwritten page
  at high thinking is no longer cut off and sent again from scratch.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.6.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.6.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.6.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.6.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.6.0 over an earlier version keeps your jobs, settings, presets and
keys.

## Known limitations

- **The offline language models are far weaker than the cloud models.** Their answers are
  plainer, they can miss or invent names and facts, and they follow long instructions less well:
  check what they write. They are slow on a processor, up to several minutes for a long text.
  They have not yet been run on a full job with a real document on each system, timed on a
  processor and a graphics processor, or compared with the cloud models on the ready-made
  analyses.
- **Whisper on a graphics processor has not been timed yet.** GitHub's machines have none; it was
  tried on a software renderer, where it wrote the same transcript as the processor. How much
  faster it is on a real card, and whether every driver handles it, is still to be seen: turn it
  off in Settings if a recording comes out wrong, and please say so with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose).
- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model. NVIDIA and AMD cards have not been timed with
  them yet; on a Snapdragon laptop only GLM-OCR's page image uses the graphics processor.
- **The Mac versions** are built and tested on Macs in GitHub Actions, Whisper on Metal
  included, and have now been tried on one Mac with Apple Silicon (macOS 26), the Mac App Store
  build included; they have had far fewer hours of use than the Windows versions. Intel Macs
  are not supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once. macOS may also ask once more to let Kalamu use the keys saved in your Keychain: choose
  *Always Allow*.
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
offline OCR and language models run in llama.cpp b12110 (MIT), whose build is described in the
[llama-b12110-1 release](https://github.com/fmadore/Kalamu/releases/tag/llama-b12110-1); the
models themselves, LightOnOCR-3 (Apache 2.0), GLM-OCR (MIT) and Gemma 4 E4B and E2B (Apache
2.0), are downloaded when you ask. Offline transcription uses whisper.cpp (MIT), on Windows also
built with Vulkan into `whisper-vulkan.exe` from Kalamu's own source, with Vulkan's headers (MIT)
compiled in; its voice-activity model is Silero VAD (MIT). Offline text extraction uses Tesseract
5.5.2 and its language data (Apache 2.0) with Leptonica (BSD), and PDF pages are drawn with hayro
(MIT). The fonts are under the SIL Open Font License. Every other component is listed with its
licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
