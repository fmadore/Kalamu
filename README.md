# Kalamu

A desktop app for Windows and macOS that lets researchers read scanned and handwritten
documents, transcribe interviews and analyse texts with the AI provider of their choice, or
offline, without Python, Colab or the command line. Every result is checked against its source
side by side: where models disagree is highlighted, and corrections never overwrite the AI's
text.

*Kalamu* is Swahili for pen, from the Arabic *qalam*.

**Windows 11 (x64 and ARM64) · macOS 13 and later (Apple Silicon) · English and French interface (German planned) · MIT licence**

<a href="https://apps.microsoft.com/detail/9MXT5555BZ1H?referrer=appbadge&mode=direct"><img src="https://get.microsoft.com/images/en-us%20dark.svg" alt="Get it from Microsoft" height="48"></a>&nbsp;&nbsp;<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

Free, with every feature included. If it is useful to you, you can
[support its development](#support-development).

**Status: version 1.4.1**, for Windows and macOS: from the Microsoft Store or the
[releases page](https://github.com/fmadore/Kalamu/releases), see [Install](#install). Text
extraction, transcription and analyses have run against the live OpenAI, Claude and Mistral
APIs; Gemini, offline Whisper and the cheaper batch mode have had fewer live runs, so try a
small job first. See the
[roadmap](docs/roadmap.md) and the [1.4.1 release notes](docs/release-1.4.1.md). The Mac App
Store version is being prepared ([steps](docs/mac-app-store.md)).

## What it does

- **Extract text** from PDFs and images, printed or handwritten, with Gemini, OpenAI, Claude or
  Mistral, or offline with Tesseract for clean print: free, nothing sent, each word with its
  place on the page; its speed depends on your computer's processor. For degraded print, old
  newspapers in columns and tables, small OCR vision models (LightOnOCR-3, GLM-OCR) also run
  offline: much slower on a processor, faster on a graphics processor (a Mac's, or a PC's
  through Vulkan), and, like
  any language model, able to write text that is not on the page. Lines kept as on the
  page, or joined into running text; pages read one at a time or several together, still
  reviewed page by page. Choose how much each model thinks; every answer may use the whole
  output the model allows.
- **Transcribe audio and video**, with speakers and timestamps, in the cloud or offline with
  Whisper. Speakers can be given their real names. Public YouTube videos are transcribed from
  their link with Gemini, without downloading them.
- **Analyse texts** with fields you define: a summary, the people, places and organisations a
  document names, its tone with a reason, themes from your own list. Ready-made analyses cover
  the common needs, including the question, argument, methods and findings of a scholarly
  publication for a literature review. Offline, small language models from Google (Gemma 4 E4B
  and E2B) analyse texts and photographed pages, make tables of contents and correct text with
  nothing sent: far weaker than the cloud models and slow on a processor, faster on a graphics
  processor, so check their answers.
- **Make a detailed table of contents** of a long document: a newspaper or magazine issue, an
  archival file scanned in one piece, a book or report. One model reads each page, another,
  cheaper one (from any provider) combines the pages into one entry per article or item, with
  its pages, authors, date or summary. Each entry can then become an item of its own, an
  article or a letter, whose pages are extracted, analysed or exported apart from the rest.
- **Correct existing text**: the noisy OCR of a PDF's text layer or of a digital library's text
  file, corrected page by page with its spelling of the time kept, every change shown against
  the original.
- **Organise your material in projects**: documents grouped in collections (one document can
  be in several), each with every transcription, extraction and analysis made of it, whatever
  job made it. Add files before reading them, add jobs you ran before without paying again,
  start a job on a selection, and mark the result you rely on. Describe each source as an
  archive would (repository, fonds, shelfmark, citation), with what a project's documents share
  given once, and export the descriptions as CSV or JSON, in Kalamu's columns or in your own,
  saved for the next project, beside each document's corrected text, review status or analysis
  answers, these as the model gave them or under your dictionary's labels.
- **Search a whole project**: every passage of its documents that names a person or an
  organisation, or holds several words near each other, in the text as you corrected it, accents
  and Arabic vowel marks ignored, each opening its source where it stands. It runs on your
  computer.
- **Reconcile names in a project's dictionary**: the people, places and terms its analyses
  listed, their spellings gathered into entries under the label you choose, with merges a
  model can propose and that you accept, change or turn down.
- **See what needs checking across a project**: one list of the pages and answers that failed,
  were cut off, read differently by two models, read from a text corrected since, or are not
  yet reviewed, each opening its review at the right page. Nothing is sent to a model.
- **Find any job** on the Jobs page: every job in a table, searched by name, workflow or file
  name, filtered by task or status and sorted; rename, add to a project or delete several at
  once.
- **Check every result against its source**, side by side, with corrections kept apart from the
  AI output, which is never overwritten. Zoom the scan about the pointer, and choose a
  difference in either text to bring it level in the other. Search a document or a recording across all its pages
  (Ctrl F, or ⌘F on a Mac), and report inappropriate output from the review.
- **Compare models** on the same documents, with their differences highlighted in the text and
  error rates against corrected pages.
- **Never pay twice**: work is saved piece by piece, interrupted jobs resume, and nothing is
  sent again with the same settings.
- **Pay half for large collections**: send a job to the provider's batch service, answered
  within a day, and close the app in the meantime.
- **Save a job's settings as a pipeline** and start the next batch in one click. Export it,
  with its models, settings and full prompts, to cite in a methods section or share with
  colleagues.
- **Then analyse the text**: at the end of New job, choose an analysis to run once the text is
  read and you have checked it. For longer chains, **workflows** run your pipelines one after
  another, from ready-made examples or your own, on a folder or a project's selection; each
  step is a job of its own, and running again pays only for the steps that changed.
- **Export** to text, Markdown, Word, JSON, CSV, subtitles, searchable PDF and ALTO XML (each
  word with its place on the page, corrections included), each with a `provenance.json`
  recording the model, prompt and settings.
- **A glossary** explains the words these tools use, from OCR and tokens to error rates and
  hallucination, for colleagues new to them.

Files go straight from your computer to the provider you choose, with your own API key, kept in
Windows Credential Manager or the macOS Keychain. Offline, nothing leaves the computer. AI
output can be fluent and wrong: check a sample against the source before relying on it.

## Install

**Windows 11** (x64 and ARM64): install Kalamu from the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H), signed and updated through
the Store. Or download `Kalamu_<version>_x64-setup.exe` (Intel and AMD PCs) or
`Kalamu_<version>_arm64-setup.exe` (Snapdragon laptops) from the
[releases page](https://github.com/fmadore/Kalamu/releases). These installers are not signed,
so Windows SmartScreen warns on the first launch: choose **More info**, then **Run anyway**.

**macOS 13 and later** (Apple Silicon): download `Kalamu_<version>_aarch64.dmg` from the
[releases page](https://github.com/fmadore/Kalamu/releases) and drag Kalamu to Applications.
The app is signed with an Apple Developer ID and notarised by Apple, so macOS opens it after
the usual question about an app downloaded from the internet. If you used version 1.3.0 or
earlier, which were not signed this way, macOS asks once more to let Kalamu use the keys saved in
your Keychain: choose **Always Allow**. Shortcuts
use ⌘ where Windows uses Ctrl; closing the window keeps Kalamu in the Dock, and ⌘Q asks first
while jobs run.

## Documentation

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
