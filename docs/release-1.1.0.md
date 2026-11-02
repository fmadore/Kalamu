Kalamu 1.1.0 keeps a researcher's material together across jobs: projects with their
collections and documents, a dictionary of the names your analyses found, and workflows that
run one step after another, stopping where you want to check.

## What's new since 1.0.0

- **Projects.** Gather documents in collections and tags, a document in several without being
  copied. Each document shows every result read from it, whatever job made it, with how much
  was reviewed and what it cost; mark the one you prefer. Add files before reading them, add
  jobs you ran before, start a new job or a workflow from a selection, and see the project's
  activity filtered by task, model or date, costed together.
- **A project dictionary** of the people, places and terms your analyses listed, across the
  project or one collection: merge the spellings that are one person or place under a label of
  your choosing, take them apart again, keep two apart, mark them reviewed, and export the
  result as JSON and CSV. A model can propose merges for a field; nothing is merged until you
  accept a proposal.
- **Workflows**: pipelines run one after another on the same files, each step reading the files
  or an earlier step's corrected text. A step can stop for your review, and the next waits until
  you continue. *Then*, at the end of New job, analyses the text once you have checked it, with
  no workflow to build, and three ready-made examples show the rest.
- **Use this text in a new job**: send any finished extraction, transcription or correction, as
  you corrected it, into an analysis, a correction or a workflow, without reading the originals
  again. An analysis of text corrected since is marked out of date.
- **Several pages per call** for text extraction, so a model sees an article that runs over the
  next page; each page is still reviewed and exported on its own.
- **Thinking** for models that reason, set to a suggestion for each task, and every answer
  allowed the whole output its model is documented to give.

## Fixed

- A job reading several pages per call no longer sends pages already read again after the model
  catalogue is updated, and a call whose answer could not be split into pages now counts in what
  the job cost.
- A workflow step started at launch is tied to its project's documents; a step is never made
  twice, nor started after its run is cancelled, and a step that could not start for want of
  text starts once the text is read.
- A searchable PDF is written only by a model that reports where text sits on the page, whatever
  a pipeline asked for.
- Values not known yet (a cost, an error rate) are read out as such by screen readers rather
  than as a dash.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- `Kalamu_1.1.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.1.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `kalamu-samples.zip`: a short recording, a scanned page and a text to try the app with.
- `Kalamu_1.1.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg and offline Whisper; nothing else needs to be installed.
Installing 1.1.0 over 1.0.0 keeps your jobs, settings, presets and keys.

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
