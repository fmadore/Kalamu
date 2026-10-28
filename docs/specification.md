# Specification

What Kalamu must do, and why. The [README](../README.md) is the short introduction; the
[roadmap](roadmap.md) says what is built. Keep this file in step with decisions as they are made
([decisions](decisions/)).

## Why

More and more colleagues ask how to use AI to transcribe their recordings and documents. The
existing tools work, but they assume some technical comfort:

- [zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines) runs in Google Colab: no
  installation, but files must be uploaded to Colab and the steps followed in order.
- [iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines) offers many more tasks and
  providers, but needs Python, a terminal and an `.env` file.

Kalamu packages the same workflows as an ordinary Windows app, built with Rust and Tauri.
Install it, paste an API key, pick a file or a folder, press **Start**. Its users are
researchers, historians first, who are not technical: every screen must stay plain and
friendly, with the advanced choices folded away.

## Starting point: existing code

Kalamu does not start from scratch. Three existing projects are reviewed for each part
and their tested pieces carried over:

- **[iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines)** (Python), the main
  reference for the AI side: prompts for each task, provider clients and their quirks, model
  choices, chunking of long audio and PDFs, retry handling, entity extraction and sentiment
  scales, output formats and the `provenance.json` format. These are ported to Rust, not
  rewritten from memory.
- **[zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines)** (Colab notebooks), for the
  simplified workflows and default settings that already suit non-technical colleagues.
- **[Live Translation](https://github.com/fmadore/Live-translation)** (Rust, Tauri 2), for the
  app's plumbing: Tauri setup, API keys in Windows Credential Manager, whisper.cpp with model
  download and verification, x64 and ARM64 builds, and MSIX packaging. Kalamu shares code
  with it, not its look or identity.

Where Kalamu departs from these projects, the reason is written down in
[decisions](decisions/).

## Tasks

| Task | Input | Output |
| --- | --- | --- |
| **Extract text** (printed and handwritten) | PDF, JPEG, PNG, TIFF | Text per page, its lines kept as on the page or joined into running text |
| **Transcribe audio or video** | `.mp3`, `.wav`, `.m4a`, `.mp4`…, or a public YouTube link | Transcript with optional speakers and timestamps; speakers can be given names |
| **Analyse texts** | Text, Word, PDF, photographed pages, and the results of earlier jobs | The researcher's own fields for each document: a summary, the people and places it names, its tone… |

**OCR and HTR are one task.** Current vision models read print and handwriting with the same
call, so the app has a single **Extract text** task and the difference lies in the prompt:
users pick a preset such as *printed document*, *newspaper (columns)*, *handwritten letter* or
*Arabic or Ajami manuscript*, or write their own. Mixed material, such as a typed report with
handwritten notes, needs no separate pass.

**YouTube videos are transcribed from their link.** New job offers *Transcribe YouTube* beside
the three tasks: paste one or more watch, `youtu.be`, Shorts or live links, and the transcript
comes out as for a recording, in the same formats. Gemini fetches the video itself, so it is
never downloaded or stored, which also keeps within YouTube's terms; only models whose catalogue
entry reads YouTube links are offered (Gemini today). Only public videos can be read. A video
is identified by its id, so the same video under another form of link comes from the cache, and
`provenance.json` records its address, title and when it was read. Each video is named after its
title, which the app asks of YouTube when the link is added, and a private or removed video is
refused then. Results go to a chosen folder, since a video has no folder of its own; the folder
last chosen is offered. In the review the video sits on the dark stage like a scan, behind a
poster: YouTube's privacy-enhanced player loads only when the researcher plays, the line being
spoken is highlighted and kept in view, and clicking a line plays the video from there. Videos
whose owner forbids embedding, or an offline PC, get a card that opens the video on YouTube.

**Line breaks are a choice of the job**, offered in plain words for every text extraction:

- *As on the page*: one line per line in the source, end-of-line hyphens kept. Easiest to check
  against the scan, and the review's line numbers then match it; a word split across two lines
  is harder to find by search.
- *Running text*: lines joined into paragraphs and split words rejoined. Best for reading and
  searching.

The choice is added to whichever prompt is used, so it works with every preset, the user's own
included. Models that take no prompt (Mistral OCR) keep their own layout, which New job says.

**An analysis is a set of fields** the researcher defines, filled in for each document. Each
field has a name, a kind of answer and, in the researcher's words, what belongs in it:

- *Text*: a summary, the main subject, a date.
- *List of names or terms*: people, places, organisations, keywords, spelled as in the document,
  with an optional maximum.
- *One choice* from a fixed list: a tone (positive, neutral, negative), a relevance.
- *Several choices* from a fixed list: themes from the researcher's own vocabulary.

Any field can ask for a one-sentence reason. Ready-made analyses cover the common needs (a
research summary with keywords, a short catalogue summary, a social-science publication's
question, argument, theory, methods, data, case and findings for a literature review, people,
places and organisations, the tone of a document), and common fields are one click away when building one. The app turns
the fields into each provider's structured output, so the answer always has exactly these
fields, and checks it again before storing it: an answer that breaks the fields is asked for
again, never stored empty. Named entity recognition and sentiment analysis are analyses, not
separate tasks.

## Choosing and starting a job

- **One file or a whole folder.** Choose files from the PC or drop a folder; the app processes
  them one by one, shows progress, and picks up where it stopped after an interruption.
- **Choice of provider.** Google Gemini, OpenAI, Anthropic Claude, Mistral, offline Whisper,
  and later OpenRouter for open-weights models. The app shows only the providers and models
  that can handle the chosen task: Claude, for instance, reads documents but does not accept
  audio.
- **Language choice with auto-detect.** Extraction and transcription list only the languages
  the chosen model supports, with **Auto-detect** as the default; several can be chosen for
  material that mixes languages (French and Mooré, say). Analyses have an answer language, by
  default the document's own; names stay as the document spells them.
- **Current models.** The latest models for each task, updated as better ones appear (see
  [Keeping models current](#keeping-models-current)). Very expensive models carry a note that
  they are most likely more than the task needs.
- **Estimated cost before starting**, from file length and the provider's published prices.
- **Faster or cheaper.** A job runs at once, page by page, or, for large collections, through
  the provider's batch service at about half the price, with answers within a day. Both costs
  are shown before starting; offline models and those without a batch service run at once
  either way. Where providers cache a repeated prompt, the cached part is billed at its lower
  price and the cost shown is the cost charged
  ([decision](decisions/2026-10-27-economy-mode-and-prompt-caching.md)).
- **Pipelines.** A job's settings (task, models, prompts or analysis, languages, line breaks,
  formats, destination) can be saved under a name and are listed at the top of New job; one
  click fills the form for the next batch of files. A pipeline can be exported to cite or share:
  a JSON file another Kalamu imports, prompts included, and the same as Markdown for a
  methods section, with each model, the settings its provider sends (reasoning effort, output
  limit, media resolution), the full text of every prompt with the hash its results' provenance
  carries, and the fields of an analysis. The output folder is left out.
- **Works without an API key.** An offline mode keeps the app useful without an account, a key
  or an internet connection, and keeps sensitive material on the PC.
- **Interface in English first**, with French and German to follow. All interface text lives
  in translation files, so adding a language needs no code changes.

## Interruptions and resuming

A folder of interviews or a 400-page scan can take hours. Closing the app, a crash, a lost
connection, a sleeping laptop or a provider's rate limit must never mean starting again, or
paying twice.

- **Work is saved in small pieces.** Long recordings are split into chunks and PDFs into pages.
  Each finished piece is stored at once in a local job database (SQLite, in the app's data
  folder), so an interruption loses at most the piece in progress.
- **Jobs survive closing the app.** On the next launch, the app lists unfinished jobs and offers
  to resume them. Closing the window during a job asks first, and can keep the job running in
  the background from the notification area.
- **Nothing is processed twice.** Results are cached by a fingerprint of the source file plus
  the model, prompt, analysis fields and settings. Re-running a folder skips what was already
  done with the same settings; changing the prompt or model reprocesses only what it affects.
- **Batches outlive the app.** In the cheaper mode, a batch is recorded before it is sent;
  the app can be closed while the provider works, and collects the answers when it opens
  again, without sending anything twice.
- **Failures are retried, then reported.** Rate limits, network errors and answers that break
  an analysis's fields are retried with increasing delays. A piece that still fails is marked
  as failed with a readable reason, the rest of the queue continues, and failed pieces can be
  retried later on their own.
- **Exports come from the cache.** Choosing another output format later regenerates the file
  from stored results, without calling the provider again.

## Tips and checks for better results

Many disappointing results come from the input, not the model. The app teaches good practice
with short tips where they are relevant, and checks each file before a job starts:

| Problem | Why it matters | Suggested fix |
| --- | --- | --- |
| **Two pages on one PDF page or image** (a spread) | Models read straight across both pages, and page numbers no longer match | Split each spread into two pages |
| **Low resolution** (below about 200 dpi) | Small letters, diacritics and handwriting become guesswork | Rescan; or continue anyway |
| **Rotated or skewed pages** | Lines are read in the wrong order or skipped | Rotate or straighten automatically |
| **Blurry, dark or low-contrast photos** | Characters are misread | Retake the photo |
| **PDF that already contains text** | Extraction may not be needed | Use the existing text, or extract anyway |
| **Very long or very quiet recordings, long silences** | Transcription drifts or skips quiet speech | Even out the volume; recordings are split at pauses |
| **Auto-detect on short or mixed recordings** | Detection is unreliable on little speech | Choose the language(s) explicitly |
| **Several documents in one file, or one document over several files** (analyses) | An analysis reads each file whole, however many pages, and gives it one set of answers, so keywords and names mix or split | One file per document; a tip says so, as it cannot be checked |

Checks never block a job: users can dismiss a warning. Fixes are applied to working copies,
never to the originals, and recorded in `provenance.json`.

## Review view

Checking AI output against the source is what makes results trustworthy, so the review is the
heart of the app (see [DESIGN.md](../DESIGN.md)).

- **Source and result side by side**: the page image on a dark lightbox on the left, the text on
  the right with line numbers in the margin, page navigation, zoom and a strip of pages,
  thumbnails for scans and images, showing which are reviewed. The boundary between them can be
  dragged, and the job list folded away (Ctrl B), for more room.
- **Differences in the text.** Where another model, or the model under a correction, reads
  differently, the words are highlighted. Pointing at them shows the readings over the text,
  each named by its set-up; clicking them, or stepping through them with F, keeps them open.
  The researcher chooses the correct reading, or types one, and it is taken into the text as a
  correction.
- **Compare**: another set-up's whole text in a third pane beside the text, its differences
  highlighted the same way; pointing at one marks it in both texts.
- **Audio too**: a player next to the transcript; the line being spoken is highlighted, and
  clicking a line jumps to it. When a model labelled speakers, the researcher can give each
  label a name ("Speaker 2" → "Imam Ouédraogo"); names replace the labels in the review and in
  every file, and the model's own output keeps its labels.
- **Analyses**: the document beside its answers, field by field. In a comparison every value
  carries the sigla of the set-ups that gave it, so agreement reads at a glance; reasons are
  shown below; pointing at a name finds it in the document.
- **Correct in place.** Corrections are kept apart from the AI output, which is never
  overwritten. Exports use the corrected text or answers, and `provenance.json` records which
  pieces were reviewed or corrected.
- **Mark as reviewed** (Ctrl Enter), so long jobs can be checked in several sittings and
  progress shows what has been verified, not only what has been processed.

The review opens while a job is still running, so the first pages can be checked before the
rest of the folder is processed.

## Comparison mode

Which model reads a given archive best is an empirical question. Comparison mode runs the same
documents through several set-ups, each a provider, model and prompt or analysis, and shows the
results together.

- **Try a sample first.** Comparison can run on a few pages before committing a whole folder,
  and the cost estimate shows each set-up separately.
- **Scores when a reference exists.** Corrected pages serve as references: the app computes
  character error rates for each set-up; without one, the highlighted differences show where
  they disagree.
- **Every version is kept**, with the set-up in the file name.
- **Built on the cache.** Adding a third model to an existing comparison only runs the new one.

## Progress

The user always sees what the app is doing, how far it is, and what it costs: a list of jobs
with their status, a bar per file and set-up, a live preview of the text as pieces arrive,
pause and cancel that keep everything finished, progress on the taskbar icon, and a
notification when a job finishes or needs attention. A job waiting for a batch says how many
pieces the provider holds and when they are due.

## Export formats

Results are stored once and exported to several formats at the same time; the default selection
is remembered per task. Details are in [formats.md](formats.md).

| Task | Formats |
| --- | --- |
| **Text extraction** | `.txt`, `.md`, `.docx`, `.json`, `.csv`; a searchable PDF (invisible text layer over the original) only from models that report where text sits on the page, today Mistral OCR; ALTO and PAGE XML later, likewise |
| **Transcription** | `.txt`, `.md`, `.docx`, `.json` (segments with speakers and timestamps), `.srt`, `.vtt`, `.csv` |
| **Analysis** | `.txt`, `.md`, `.docx`, `.json` (answers by field, with reasons), `.csv` (one row per document, one column per field); later a column added to an `.xlsx` |

## Prompts and analyses

- **Defaults per task**, ported and tested from `zmo-ai-pipelines` and `iwac-ai-pipelines`.
- **Edit, save and name** a prompt or an analysis, duplicate one as a starting point, and reset
  any default. They are stored locally and kept across updates.
- **Variables** `{languages}` and `{speakers}` are filled from the job's settings, so one prompt
  works across projects.
- **Share** prompts and analyses by exporting and importing them as small JSON files.
- **Recorded in provenance.** Each output's `provenance.json` stores the full prompt text and
  the analysis's fields, so results stay reproducible after a preset is edited.

## Keeping models current

- **A model catalogue as data, not code**: each provider's models, tasks, languages, limits and
  prices in a catalogue file the app refreshes from this repository without a new release.
- **A recommended model per task**, reviewed whenever a provider releases something better;
  older models remain selectable and retired ones are flagged.
- **The exact model version is recorded** in `provenance.json`.

## Offline mode

- **Audio transcription with Whisper** (whisper.cpp), with models downloaded once from Hugging
  Face from Settings and verified before use.
- **Text extraction with Tesseract** (planned), for clean print only; handwriting presets are
  offered with cloud models only.

Offline results are a fallback, not the default for difficult material: the app says which
engine produced a result and records it like any other. Analyses need a cloud model.

## Privacy

In offline mode, nothing leaves the PC. With a cloud provider, files go directly from the PC to
that provider; there is no intermediate server. For a YouTube video only its link is sent, and
Google reads the video from YouTube; the app also asks YouTube for the video's title when the
link is added, and loads YouTube's player in the review only when the video is played. API keys stay in Windows Credential Manager.
The developer receives no files, keys, results or telemetry.

Sending research material to an AI provider is still a transfer of data to a third party.
Researchers remain responsible for consent, confidentiality, copyright and any approval their
institution requires, and should prefer paid API tiers whose terms exclude training on inputs.

## AI output is not ground truth

Transcriptions, extracted text and analyses can be fluent and wrong: misheard names, skipped
passages, invented words, a tone misjudged. Check a sample against the source before relying on
the results, and keep the originals.

## Bundled conversion tools

Providers accept different inputs, so the app prepares each file for the chosen model; users
never convert anything themselves. What a model is sent is what both its provider and its own
catalogue entry accept: a text-only model behind a provider that takes PDFs and images is sent
text alone, and a scan has its text extracted first. The tools ship inside the app for x64 and ARM64.

- **PDF to images** with PDFium, page by page, for providers that do not accept PDF, for
  previews and for searchable PDF exports.
- **Image preparation**: multi-page TIFFs split, HEIC, WebP and BMP converted, oversized scans
  resized, orientation fixed from EXIF data.
- **Audio and video with FFmpeg**: the audio track extracted from video, files converted to a
  format the provider accepts, and long recordings split at silences with a short overlap.
- **Reading inputs** for analyses: Word, PDF, Markdown, CSV and plain text, and a photographed
  page as one image (a multi-page TIFF is refused, with a hint to save it as a PDF).

Licences matter for an MIT app in the Store. PDFium is BSD/Apache-licensed. FFmpeg is our own
audio-only build, LGPL v2.1 or later, run as separate programs rather than linked; its source is
published with each release, and the Store listing carries our own licence terms because
Microsoft's standard terms forbid the reverse engineering the LGPL must allow. Details are in
[ffmpeg.md](ffmpeg.md) and [store-license-terms.md](store-license-terms.md).

## Technology

- [Tauri 2](https://tauri.app/) with a Rust back end and a Svelte front end.
- Provider calls in Rust; no bundled Python.
- SQLite for the job database and result cache; exporters in Rust.
- [whisper-rs](https://crates.io/crates/whisper-rs) (whisper.cpp) for offline transcription.
- Native builds for x64 and ARM64.

The crate layout, modularity rules and architecture reviews are in [AGENTS.md](../AGENTS.md).

## Distribution

The goal is the [Microsoft Store](https://apps.microsoft.com/): Store packages are signed,
install without a security warning and update themselves, which matters for colleagues who are
not comfortable installing software. This means MSIX packaging, keys in Credential Manager, a
privacy policy, and the app's network access declared. Unsigned installers will also be
attached to GitHub releases. The submission, the policies it must meet and the listing text are
in [microsoft-store.md](microsoft-store.md), [store-listing.md](store-listing.md) and
[privacy.md](privacy.md).
