The first test build under the name **Kalamu**, formerly Scriptorium, and the first for both x64
and ARM64. **This is still an early pre-release**: expect rough edges, and check results against
your sources.

## What's new

- **A new name, Kalamu**, Swahili for pen, from the Arabic *qalam*, with a new icon (a
  handwritten sheet behind its transcription, one line under the highlighter) and warmer,
  paper-toned greys in light and dark. On its first start, Kalamu moves the jobs, settings,
  presets, Whisper models and API keys of Scriptorium 0.1 or 0.2 over to its own name.
- **Economy mode**: send a job to the provider's batch service, with Gemini, OpenAI, Claude or
  Mistral, at half the price. Answers come within a day, and the app can be closed in the
  meantime. Cached input is billed at its own, lower price, and both costs are shown before
  starting.
- **YouTube videos** are transcribed from their link with Gemini, without downloading them, and
  reviewed beside the video.
- **Pipelines can be exported and imported**: a job's models, settings and full prompts as a
  JSON file to share with colleagues, and a Markdown record to cite in a methods section.
- **Analyses**:
  - A ready-made analysis of scholarly publications, giving their question, argument, methods
    and findings for a literature review.
  - Photographed pages and scanned images can be analysed directly with Gemini, OpenAI or
    Claude, as PDFs already could, without extracting their text first.
- **More models**: GPT-6.1 Terra, Mistral's chat models for analyses, and the Z.ai GLM models
  hosted by Mistral. A model is sent only the inputs it accepts: a text-only model asks for the
  text to be extracted first.
- **The strip of pages shows thumbnails** of scans and images in the review.
- **About** in Settings links each project Kalamu credits, and an optional Buy Me a Coffee link
  sits at the foot of the sidebar. It unlocks nothing: every feature stays free.

## Fixes

- Scans compressed with JPEG 2000 or JBIG2, common in library PDFs, now show in the review.
- New job's tips match the task: analyses no longer get the tips for recordings.

## Download

- `Kalamu_0.3.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_0.3.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).

Both bundle FFmpeg and offline Whisper; nothing else needs to be installed.

**Coming from Scriptorium 0.1 or 0.2?** Windows treats Kalamu as a separate app, so it installs
beside Scriptorium rather than over it. Install Kalamu and start it once, so it moves your jobs,
settings and keys over, then uninstall Scriptorium from Settings → Apps.

## Known limitations

- **Unsigned installers**: Windows SmartScreen will warn. Choose *More info*, then *Run anyway*.
- Gemini, offline Whisper and economy mode have not yet run against the live services; OpenAI,
  Claude and Mistral have. Try a small job first.
- The YouTube player inside the app is not yet checked in the installed build; if it stays
  blank, the transcript is still made.
- English interface only; French and German will follow.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-1 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-1). Offline
transcription uses whisper.cpp (MIT). The fonts are under the SIL Open Font License.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
