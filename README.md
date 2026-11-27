# Kalamu

A desktop app for Windows and macOS for reading scanned and handwritten documents, transcribing
interviews, and analysing texts with your choice of AI provider (or offline), without Python,
Colab, or the command line. Every result opens side by side with its original source: differences
between models are highlighted, and your corrections never overwrite the AI output.

*Kalamu* is Swahili for pen, from the Arabic *qalam*.

**Windows 11 (x64 and ARM64) · macOS 14 and later (Apple Silicon) · English and French interface (German planned) · MIT licence**

<a href="https://apps.microsoft.com/detail/9MXT5555BZ1H?referrer=appbadge&mode=direct"><img src="https://get.microsoft.com/images/en-us%20dark.svg" alt="Get it from Microsoft" height="48"></a>&nbsp;&nbsp;<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

Free, with every feature included. If Kalamu is useful to your work, you can
[support its development](#support-development).

**Status: version 1.10.1** is available for Windows and macOS on the
[releases page](https://github.com/fmadore/Kalamu/releases) ([release notes](docs/release-1.10.1.md)).
The Microsoft Store carries 1.10.0 while 1.10.1 follows; the Mac App Store build is in Apple's
review ([progress](docs/mac-app-store.md)). Text extraction, transcription, analysis, and
translation have been tested live against the Gemini, OpenAI, Claude, and Mistral APIs. Offline
Whisper and the cheaper batch mode have had fewer live runs, so try a small job first. See the
[roadmap](docs/roadmap.md) and [Install](#install).

## What it does

### Read, transcribe, and analyse

- **Extract text** from printed or handwritten PDFs and images using Gemini, OpenAI, Claude, or
  Mistral. Offline, Tesseract handles clean print for free with word-level coordinates on the page
  (speed depends on your CPU), while local OCR vision models (LightOnOCR-3, GLM-OCR) handle
  degraded print, multi-column newspapers, and tables. Vision models run slowly on a CPU and much
  faster on a GPU (Apple Silicon or Vulkan on Windows), and like any language model they can
  hallucinate text that is not on the page. Keep original line breaks or join lines into running
  text, process pages individually or in small batches while still reviewing page by page, or read
  a folder of photographed pages as a single document in natural filename order (page 2 before
  page 10) or your own order. You can also control reasoning effort and let answers use a model's
  full output limit.
- **Transcribe audio and video** with speaker labels and timestamps, in the cloud or offline with
  Whisper (GPU-accelerated via Metal on macOS and Vulkan on Windows), automatically skipping
  silence and music. Rename speakers with their real names, or transcribe public YouTube videos
  directly from their URL with Gemini without downloading them.
- **Analyse texts** with structured fields you define: summaries, named people, places, and
  organisations, tone with justifications, or themes from your own controlled list. Built-in
  presets cover common research tasks, including extracting a scholarly publication's research
  question, argument, methods, and findings for a literature review, or describing photographs,
  posters, and maps with keywords. Offline, Google's small language models (Gemma 4 E4B and E2B)
  can analyse texts and photographed pages, build tables of contents, and correct text locally;
  they are weaker than cloud models and slower on a CPU, so check their output carefully.
- **Turn directories, registers, and lists into tables**: an analysis with *One row per entry*
  extracts each page into structured rows using your own columns (such as surname, trade, and
  address, or date, name, and place), ready to verify beside the scan and export as a single
  spreadsheet.
- **Build tables of contents** for long documents such as newspaper issues, multi-item archival
  files, books, or reports. One model reads each page and a second, cheaper model (from any
  provider) merges them into one entry per article or item with page numbers, authors, dates, and
  summaries. Any entry can then be split out as its own document to extract, analyse, or export
  separately.
- **Correct existing text**: clean up noisy OCR from a PDF text layer or digital-library export
  page by page while preserving historical spelling, with every edit shown against the original.
- **Translate texts**: translate text files, PDF text layers, or your reviewed extractions,
  transcriptions, and corrections page by page into your chosen language, without summarising or
  dropping content and keeping proper names intact. Translations appear beside the source text and
  can be analysed in turn.

### Organise and search projects

- **Work in projects**: group documents into collections (a document can belong to several) and
  keep every extraction, transcription, and analysis tied to its source, whichever job produced
  it. Add files before running anything, attach earlier jobs without paying again, launch a job on
  a selection, and mark the preferred result for each document. Record archival metadata
  (repository, fonds, shelfmark, citation), enter shared fields once for the whole project, and
  export everything as an Excel workbook, CSV, or JSON using Kalamu's columns or a custom layout
  saved for future projects, alongside corrected text, review status, and analysis fields (raw or
  normalised through your dictionary).
- **Search across a project**: find every mention of a person or organisation, or words appearing
  near one another, across your corrected text. Search runs locally, ignores accents and Arabic
  diacritics, and jumps straight to the matching passage in its source.
- **Search by meaning and map a project**: find passages that discuss a topic in different words
  using Gemini, OpenAI, Mistral, or local EmbeddingGemma 2 (which sends nothing off your
  computer), and view your project's documents as a 2D map where similar texts cluster together.
  Nothing is indexed until you start it, and the estimated cost is shown upfront.
- **Reconcile names in a project dictionary**: gather the people, places, and terms extracted by
  your analyses under canonical labels. A model can suggest merges for spelling variants, which
  you accept, edit, or reject.
- **Track what still needs review**: open one project-wide list of pages and answers that failed,
  were truncated, differed between two models, relied on text edited since, or have not yet been
  reviewed. Clicking an item opens the exact page, with nothing sent to a model.
- **Manage jobs at a glance**: search the Jobs table by name, workflow, or filename, filter by
  task or status, sort columns, and rename, assign to a project, or delete multiple jobs at once.

### Review, compare, and export

- **Check every result against its source** side by side, keeping your edits separate from the
  immutable AI output. Zoom the scan around the cursor, click a difference in one pane to align it
  in the other, and inspect word or block regions overlaid on the scan where the model provides
  them (Tesseract, Mistral OCR, LightOnOCR). Draw a custom box on any scan to re-read just a
  marginal note or faint line, search across all pages or timestamps (`Ctrl+F` / `⌘F`), and report
  inappropriate output directly from the review screen.
- **Compare models** on the same sources with inline diff highlighting and character/word error
  rates measured against your corrected pages.
- **Never pay twice**: results are cached piece by piece, interrupted jobs resume where they left
  off, and identical requests are never sent twice.
- **Save half on large batches**: submit jobs to a provider's batch service (answered within 24
  hours) at roughly half the standard price, and close Kalamu while you wait.
- **Reuse pipelines and chain workflows**: save a job's models, settings, and full prompts as a
  pipeline to rerun in one click or export for a methods section. Chain an analysis directly after
  a reading job in New job, or run multi-step workflows across folders and project selections:
  each step is a separate job, so rerunning a workflow only pays for steps that changed.
- **Export with full provenance**: export to plain text, Markdown, Word, JSON, Excel workbooks,
  CSV, subtitles, searchable PDF, ALTO XML, and PAGE XML (preserving word coordinates alongside
  your corrections). Every export includes a `provenance.json` file recording the model, prompt,
  and settings used.
- **Built-in glossary**: plain-language explanations of OCR, tokens, error rates, hallucination,
  and related terms for colleagues new to AI workflows.

Files travel directly from your computer to the provider you select, authenticated with your own
API key stored in Windows Credential Manager or the macOS Keychain. In offline mode, nothing
leaves your machine. Not yet in a release: a server of your own, such as Ollama or a
university's OpenAI-style service, added in Settings by its address, each of its models tried
before it is offered; it has not yet been tried against a real server
([roadmap](docs/roadmap.md)). AI output can sound fluent while being wrong, so always check a sample
against the original source before relying on it.

## Install

**Windows 11 (x64 and ARM64):** install Kalamu from the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H) for signed builds and automatic
updates. Alternatively, download `Kalamu_<version>_x64-setup.exe` (Intel and AMD PCs) or
`Kalamu_<version>_arm64-setup.exe` (Snapdragon laptops) from the
[releases page](https://github.com/fmadore/Kalamu/releases). Because the standalone installers are
unsigned, Windows SmartScreen will warn you on first launch: click **More info**, then **Run
anyway**.

**macOS 14 and later (Apple Silicon):** download `Kalamu_<version>_aarch64.dmg` from the
[releases page](https://github.com/fmadore/Kalamu/releases) and drag Kalamu into Applications. The
app is signed with an Apple Developer ID and notarised by Apple, so macOS opens it after the
standard downloaded-app prompt. If you are upgrading from version 1.3.0 or earlier (which were not
Developer ID signed), macOS will ask once for permission to access your saved Keychain items:
choose **Always Allow**. Shortcuts use ⌘ in place of Ctrl; closing the window leaves Kalamu in the
Dock, and ⌘Q prompts before quitting while jobs are running.

## Documentation

- [Website](https://kalamu.frederickmadore.com/): what Kalamu does, a tour and the downloads
  ([its page](docs/index.html)).
- [Guide](https://kalamu.frederickmadore.com/guide/): how to use Kalamu, from a first job to
  exporting checked results ([its pages](docs/guide/)); in French as the
  [Guide en français](https://kalamu.frederickmadore.com/guide/fr/).
- [Specification](docs/specification.md): what the app must do, and why.
- [Roadmap](docs/roadmap.md): what is built and what is next.
- [Development](docs/development.md): building, checks, the browser preview.
- [Output formats](docs/formats.md), [decisions](docs/decisions/), [design](DESIGN.md), and
  [everything else](docs/README.md).
- [Privacy policy](docs/privacy.md) and [security policy](SECURITY.md).

## Development

```bash
npm install
```

```bash
npm run tauri dev
```

Requirements, checks and the sample-data preview are in [docs/development.md](docs/development.md).

## Support development

Kalamu is free, and every feature stays free. If it is useful to you, you can help fund its
development on [Buy Me a Coffee](https://buymeacoffee.com/fmadore). Supporting is entirely
optional and does not unlock additional features; AI providers bill their own usage separately.
The same link is in the app, at the foot of the sidebar and in Settings › About.

<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

## Author and licence

By [Frédérick Madore](https://www.frederickmadore.com/), University of Bayreuth. An independent
project. [MIT licence](LICENSE).
