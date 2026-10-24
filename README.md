# Scriptorium

A Windows desktop app that lets researchers transcribe interviews, read scanned or handwritten
documents, and summarise texts with the AI provider of their choice — without Python, Colab or
the command line.

**Windows 11 · x64 and ARM64 · English, French and German interface · MIT license**

**Status: concept.** Nothing is built yet. This README describes what the app should do.

## Why

More and more colleagues ask how to use AI to transcribe their recordings and documents. The
existing tools work, but they assume some technical comfort:

- [zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines) runs in Google Colab: no
  installation, but files must be uploaded to Colab and the steps followed in order.
- [iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines) offers many more tasks and
  providers, but needs Python, a terminal and an `.env` file.

Scriptorium packages the same workflows as an ordinary Windows app, built like
[Live Translation](https://github.com/fmadore/Live-translation) with Rust and Tauri. Install it,
paste an API key, pick a file or a folder, press **Start**.

## Starting point: existing code

Scriptorium does not start from scratch. Before any code is written, three existing projects
are reviewed and their tested parts carried over:

- **[iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines)** (Python), the main
  reference for the AI side: prompts for each task, provider clients and their quirks, model
  choices, chunking of long audio and PDFs, retry handling, output formats and the
  `provenance.json` format. These are ported to Rust, not rewritten from memory.
- **[zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines)** (Colab notebooks), for the
  simplified workflows and default settings that already suit non-technical colleagues.
- **[Live Translation](https://github.com/fmadore/Live-translation)** (Rust, Tauri 2), the
  template for the app itself: project layout, Tauri setup and front end, API keys in Windows
  Credential Manager, whisper.cpp integration with model download and verification, the
  English/French/German interface, x64 and ARM64 builds, and MSIX packaging for the Microsoft
  Store. Code is reused or shared with it wherever possible.

Where Scriptorium departs from these projects, the reason should be written down.

## What it should do

| Task | Input | Output |
| --- | --- | --- |
| **Transcribe audio or video** | `.mp3`, `.wav`, `.m4a`, `.mp4`… | Speakers and timestamps optional; see [Export formats](#export-formats) |
| **Extract text** (printed and handwritten) | PDF, JPEG, PNG, TIFF | Per page; diplomatic or normalized transcription |
| **Summarise and extract keywords** | Text, Word, PDF, `.xlsx` | Summary and keywords per file, or a column added to the spreadsheet |

**OCR and HTR are one task.** Current vision models read print and handwriting with the same
call, so the app has a single **Extract text** task and the difference lies in the prompt:
users pick a preset such as *printed document*, *newspaper (columns)*, *handwritten letter*,
*Arabic or Ajami manuscript*, *diplomatic* or *normalized*, or write their own. Mixed material,
such as a typed report with handwritten notes, needs no separate pass.

- **One file or a whole folder.** Choose files from the PC or drop a folder; the app processes
  them one by one, shows progress, and picks up where it stopped after an interruption (see
  [Interruptions and resuming](#interruptions-and-resuming)).
- **Choice of provider.** Google Gemini, OpenAI, Anthropic Claude, Mistral, or OpenRouter for
  open-weights and other models. The app shows only the providers and models that can handle the
  chosen task: Claude, for instance, reads documents but does not accept audio.
- **Language choice with auto-detect.** Audio transcription and text extraction have a language
  dropdown listing only the languages the chosen model or engine supports, with
  **Auto-detect** as the default. Picking the language explicitly helps with short or noisy
  recordings, and Tesseract needs it to load the right language data. Recordings or documents
  that mix languages (French and Mooré, say) can take several. Summaries have an output-language
  setting, by default the source's language. The detected language is shown in the results and
  recorded in `provenance.json`.
- **Current models.** The app offers the latest models for each task and is updated as better
  ones appear (see [Keeping models current](#keeping-models-current)).
- **Works without an API key.** An offline mode keeps the app useful without an account, a key
  or an internet connection, and keeps sensitive material on the PC (see below).
- **Prompt library.** Every task ships with tested default prompts that users can adapt and
  save (see [Custom prompts](#custom-prompts)).
- **Results next to the sources.** Outputs are written beside the original files or into a chosen
  folder, each with a `provenance.json` recording the model, prompt and settings used.
- **Estimated cost before starting**, based on file length and the provider's published prices.
- **English, French and German interface.**

## Interruptions and resuming

A folder of interviews or a 400-page scan can take hours. Closing the app, a crash, a lost
connection, a sleeping laptop or a provider's rate limit must never mean starting again, or
paying twice.

- **Work is saved in small pieces.** Long recordings are split into chunks and PDFs into pages.
  Each finished chunk or page is stored at once in a local job database (SQLite, in the app's
  data folder), so an interruption loses at most the piece in progress.
- **Jobs survive closing the app.** On the next launch, the app lists unfinished jobs and offers
  to resume, review or discard each one. Closing the window during a job asks first, and can keep
  the job running in the background from the notification area.
- **Nothing is processed twice.** Results are cached by a fingerprint of the source file plus the
  model, prompt and settings. Re-running a folder skips files and pages already done with the same
  settings; changing the prompt or model reprocesses only what it affects. A **Redo** button
  forces a fresh run of one file or page.
- **Failures are retried, then reported.** Rate limits and network errors are retried with
  increasing delays. A file that still fails is marked as failed with a readable reason and the
  rest of the queue continues; failed items can be retried later on their own.
- **Exports come from the cache.** Choosing another output format later regenerates the file from
  stored results, without calling the provider again.

## Progress

The user should always see what the app is doing, how far it is, and what it costs.

- **A queue view** listing every file with its status (waiting, running, done, failed, skipped
  because cached), and per file a bar for pages or audio chunks.
- **An overall bar** with files and pages done, elapsed time, estimated time remaining, and the
  cost so far against the estimate shown before starting.
- **A live preview** of the text as each page or chunk arrives, next to the page image or an
  audio player at the matching timestamp, so problems show up early rather than at the end.
- **Pause and cancel** that take effect after the current piece, keeping everything finished.
- **Progress on the Windows taskbar icon**, and a notification when a job finishes or needs
  attention.

## Export formats

Results are stored once and can be exported to several formats at the same time; the default
selection is remembered per task.

| Task | Formats |
| --- | --- |
| **Transcription** | `.txt`, `.md`, `.docx` (Word), `.json` (segments with speakers and timestamps), `.srt`, `.vtt`, `.csv` |
| **Text extraction** | `.txt`, `.md`, `.docx`, `.json` (pages, blocks and confidence where available), searchable PDF (text layer over the original scan), ALTO XML and PAGE XML where the engine returns coordinates |
| **Summaries and keywords** | `.txt`, `.md`, `.docx`, `.json`, `.csv`, or a column added to the original `.xlsx` |

Options include one file per source or one combined file per folder, page or timestamp markers,
and whether to include speaker labels. JSON follows a documented schema, so results can feed
other tools such as `iwac-ai-pipelines` or Omeka S imports.

## Custom prompts

- **Defaults per task and model**, ported and tested from `zmo-ai-pipelines` and
  `iwac-ai-pipelines`: interview transcription; text extraction for printed documents,
  newspapers, handwriting and manuscripts, diplomatic or normalized; summaries of different
  lengths.
- **Edit, save and name** a prompt as a preset, duplicate it as a starting point, and reset any
  default. Presets are stored locally and kept across updates.
- **Variables** such as `{language}`, `{speakers}` or `{summary_length}` filled from the job's
  settings, so one prompt works across projects.
- **Share presets** by exporting and importing them as small JSON files, for example across a
  research team.
- **Recorded in provenance.** Each output's `provenance.json` stores the full prompt text used,
  not only the preset name, so results stay reproducible after a prompt is edited.

## Keeping models current

AI models improve every few months; the app must not stay stuck on the models it shipped with.
In October 2026, for example, Mistral released
[Voxtral Transcribe 3](https://www.marktechpost.com/2026/10/20/mistral-voxtral-3-speech-models-voice-agent-sdk/)
(Small for accuracy, Mini for cheaper batch work, 50 languages), a clear step up for interview
transcription.

- **A model catalogue as data, not code.** Each provider's models, the tasks they suit, their
  supported languages, limits and prices live in a small catalogue file that the app can refresh from this repository
  without a new release.
- **Recommended model per task**, reviewed whenever a provider releases something better, and
  chosen by default; older models remain selectable.
- **Discovery from the providers' own model lists**, so a new model can be tried before the
  catalogue lists it, and retired models are flagged before they stop working.
- **The exact model version is recorded** in `provenance.json`, since the same name can point to
  a newer model later.

## Offline mode

Some tasks can run entirely on the PC, with no key, no cost and no data leaving the computer:

- **Audio transcription with Whisper**, as in Live Translation's offline subtitles: the same
  whisper.cpp engine, with models downloaded once from Hugging Face and verified before use.
  Larger models suit recorded interviews, where speed matters less than in live captioning.
- **Text extraction with Tesseract**, bundled with the app and its language data (French,
  English, Arabic, German…). It handles clean printed text reasonably well but is far weaker
  than cloud models on degraded scans and complex layouts, and cannot read handwriting: the
  handwriting presets are only offered with cloud models.

Offline results are a fallback, not the default for difficult material: the app should say
clearly which engine produced a result, and record it in `provenance.json` like any other.
Summaries still need a cloud model.

## Privacy

In offline mode, nothing leaves the PC. With a cloud provider, files go directly from the PC to
that provider; there is no intermediate server. API keys
stay in Windows Credential Manager. The developer receives no files, keys, results or telemetry.

Sending research material to an AI provider is still a transfer of data to a third party.
Researchers remain responsible for consent, confidentiality, copyright and any approval their
institution requires, and should prefer paid API tiers whose terms exclude training on inputs.

## AI output is not ground truth

Transcriptions, extracted text and summaries can contain fluent but wrong text: misheard names, skipped
passages, invented words. Check a sample against the source before relying on the results, and
keep the originals.

## Planned technology

- [Tauri 2](https://tauri.app/) with a Rust back end and a web front end, as in Live Translation.
- Provider calls in Rust, porting the prompts, model choices and provenance format from
  `zmo-ai-pipelines` and `iwac-ai-pipelines`; no bundled Python.
- FFmpeg to convert and split long recordings for providers with file-size limits.
- SQLite for the local job database and result cache; exporters in Rust for Word, searchable
  PDF, subtitles and XML.
- [whisper-rs](https://crates.io/crates/whisper-rs) (whisper.cpp) for offline transcription,
  reusing Live Translation's model download and verification code.
- Tesseract for offline text extraction, bundled as a sidecar with its language data.
- Native builds for x64 and ARM64.

## Distribution

The goal is the [Microsoft Store](https://apps.microsoft.com/), as for Live Translation: Store
packages are signed, install without a security warning and update themselves, which matters for
colleagues who are not comfortable installing software. This means packaging the app as MSIX
from the start, keeping keys in Credential Manager, publishing a privacy policy, and declaring
the app's network access. Unsigned installers will also be attached to GitHub releases.

## Author and license

[Frédérick Madore](https://www.frederickmadore.com/), Leibniz-Zentrum Moderner Orient (ZMO), Berlin.
[MIT license](LICENSE).
