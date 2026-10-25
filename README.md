# Scriptorium

A Windows desktop app that lets researchers read scanned and handwritten documents, transcribe
interviews and analyse texts with the AI provider of their choice, or offline, without Python,
Colab or the command line. Every result is checked against its source in a review set like a
critical edition.

**Windows 11 · x64 and ARM64 · English interface (French and German planned) · MIT licence**

**Status: early development.** The features below are implemented and tested, but have not yet
been run against the live provider APIs, and there is no release yet. See the
[roadmap](docs/roadmap.md).

## What it does

- **Extract text** from PDFs and images, printed or handwritten, with Gemini, OpenAI, Claude or
  Mistral. Lines kept as on the page, or joined into running text.
- **Transcribe audio and video**, with speakers and timestamps, in the cloud or offline with
  Whisper. Speakers can be given their real names.
- **Analyse texts** with fields you define: a summary, the people, places and organisations a
  document names, its tone with a reason, themes from your own list. Ready-made analyses cover
  the common needs.
- **Check every result against its source**, side by side, with corrections kept apart from the
  AI output, which is never overwritten.
- **Compare models** on the same documents, with their differences set out as an apparatus and
  error rates against corrected pages.
- **Never pay twice**: work is saved piece by piece, interrupted jobs resume, and nothing is
  sent again with the same settings.
- **Save a job's settings as a pipeline** and start the next batch in one click.
- **Export** to text, Markdown, Word, JSON, CSV, subtitles and searchable PDF, each with a
  `provenance.json` recording the model, prompt and settings.

Files go straight from your PC to the provider you choose, with your own API key, kept in
Windows Credential Manager. Offline, nothing leaves the PC. AI output can be fluent and wrong:
check a sample against the source before relying on it.

## Documentation

- [Specification](docs/specification.md): what the app must do, and why.
- [Roadmap](docs/roadmap.md): what is built and what is next.
- [Development](docs/development.md): building, checks, the browser preview.
- [Output formats](docs/formats.md), [decisions](docs/decisions/), [design](DESIGN.md), and
  [everything else](docs/README.md).

## Development

```bash
npm install
```

```bash
npm run tauri dev
```

Requirements, checks and the sample-data preview are in [docs/development.md](docs/development.md).

## Author and licence

By [Frédérick Madore](https://www.frederickmadore.com/), University of Bayreuth. An independent
project. [MIT licence](LICENSE).
