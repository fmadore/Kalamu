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

## What it should do

| Task | Input | Output |
| --- | --- | --- |
| **Transcribe audio or video** | `.mp3`, `.wav`, `.m4a`, `.mp4`… | Text, Markdown, Word, SRT/WebVTT; optional speakers and timestamps |
| **Read documents** (OCR/HTR) | PDF, JPEG, PNG, TIFF | Text or Markdown per page; diplomatic or normalized transcription |
| **Summarise and extract keywords** | Text, Word, PDF, `.xlsx` | Summary and keywords per file, or a column added to the spreadsheet |

- **One file or a whole folder.** Choose files from the PC or drop a folder; the app processes
  them one by one, shows progress, and can resume after an interruption.
- **Choice of provider.** Google Gemini, OpenAI, Anthropic Claude, Mistral, or OpenRouter for
  open-weights and other models. The app shows only the providers and models that can handle the
  chosen task: Claude, for instance, reads documents but does not accept audio.
- **Works without an API key.** An offline mode keeps the app useful without an account, a key
  or an internet connection, and keeps sensitive material on the PC (see below).
- **Customisable prompts.** Each task ships with a tested default prompt (language, transcription
  conventions, summary length). Users can edit it, save it as a named preset, and reset it.
- **Results next to the sources.** Outputs are written beside the original files or into a chosen
  folder, each with a `provenance.json` recording the model, prompt and settings used.
- **Estimated cost before starting**, based on file length and the provider's published prices.
- **English, French and German interface.**

## Offline mode

Some tasks can run entirely on the PC, with no key, no cost and no data leaving the computer:

- **Audio transcription with Whisper**, as in Live Translation's offline subtitles: the same
  whisper.cpp engine, with models downloaded once from Hugging Face and verified before use.
  Larger models suit recorded interviews, where speed matters less than in live captioning.
- **OCR with Tesseract**, bundled with the app and its language data (French, English, Arabic,
  German…). It handles clean printed text reasonably well but is far weaker than cloud models on
  degraded scans, complex layouts and handwriting.

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

Transcriptions, OCR and summaries can contain fluent but wrong text: misheard names, skipped
passages, invented words. Check a sample against the source before relying on the results, and
keep the originals.

## Planned technology

- [Tauri 2](https://tauri.app/) with a Rust back end and a web front end, as in Live Translation.
- Provider calls in Rust, porting the prompts, model choices and provenance format from
  `zmo-ai-pipelines` and `iwac-ai-pipelines`; no bundled Python.
- FFmpeg to convert and split long recordings for providers with file-size limits.
- [whisper-rs](https://crates.io/crates/whisper-rs) (whisper.cpp) for offline transcription,
  reusing Live Translation's model download and verification code.
- Tesseract for offline OCR, bundled as a sidecar with its language data.
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
