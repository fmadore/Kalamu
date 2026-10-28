# Kalamu

A Windows desktop app that lets researchers read scanned and handwritten documents, transcribe
interviews and analyse texts with the AI provider of their choice, or offline, without Python,
Colab or the command line. Every result is checked against its source side by side: where models
disagree is highlighted, and corrections never overwrite the AI's text.

*Kalamu* is Swahili for pen, from the Arabic *qalam*. The test releases up to 0.2 were called
Scriptorium; a newer version moves their jobs, settings and keys over on its first start.

**Windows 11 · x64 and ARM64 · English interface (French and German planned) · MIT licence**

**Status: early development.** Test builds (unsigned installers, Windows on ARM64) are on the
[releases page](https://github.com/fmadore/Scriptorium/releases). Text extraction,
transcription and analyses have run against the live OpenAI, Claude and Mistral APIs; Gemini,
offline Whisper and the cheaper batch mode still await a live run. A Microsoft Store release is
in preparation. See the [roadmap](docs/roadmap.md).

## What it does

- **Extract text** from PDFs and images, printed or handwritten, with Gemini, OpenAI, Claude or
  Mistral. Lines kept as on the page, or joined into running text.
- **Transcribe audio and video**, with speakers and timestamps, in the cloud or offline with
  Whisper. Speakers can be given their real names. Public YouTube videos are transcribed from
  their link with Gemini, without downloading them.
- **Analyse texts** with fields you define: a summary, the people, places and organisations a
  document names, its tone with a reason, themes from your own list. Ready-made analyses cover
  the common needs, including the question, argument, methods and findings of a scholarly
  publication for a literature review.
- **Check every result against its source**, side by side, with corrections kept apart from the
  AI output, which is never overwritten.
- **Compare models** on the same documents, with their differences highlighted in the text and
  error rates against corrected pages.
- **Never pay twice**: work is saved piece by piece, interrupted jobs resume, and nothing is
  sent again with the same settings.
- **Pay half for large collections**: send a job to the provider's batch service, answered
  within a day, and close the app in the meantime.
- **Save a job's settings as a pipeline** and start the next batch in one click. Export it,
  with its models, settings and full prompts, to cite in a methods section or share with
  colleagues.
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
The same link is in the app, at the foot of the sidebar and at the end of Settings.

<a href="https://buymeacoffee.com/fmadore"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy me a coffee" height="48"></a>

## Author and licence

By [Frédérick Madore](https://www.frederickmadore.com/), University of Bayreuth. An independent
project. [MIT licence](LICENSE).
