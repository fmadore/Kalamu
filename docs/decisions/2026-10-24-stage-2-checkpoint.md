# 2026-10-24 · Checkpoint after foundations and text extraction

## Context

Roadmap stages 1 (foundations) and 2 (text extraction) are implemented, with parts of stage 3
(audio transcription without splitting). This is the architecture review AGENTS.md asks for at
the end of a milestone.

## Review

- **Dependencies point inwards.** `core` depends on nothing internal; `catalog`, `prompts`,
  `media`, `providers`, `store` and `export` depend only on `core`; `engine` uses `catalog`,
  `export`, `media` and `prompts`, and reaches providers and the store only through the
  `Provider` and `JobStore` traits (SQLite is a test dependency only). The Tauri app wires them
  together.
- **Every provider call goes through the engine**, which records each piece before and after
  its call and caches results by a key covering the source, the piece, the model, the full
  prompt, the options and the languages.
- **Files over ~500 lines** were split: the app's `commands.rs` into one module per area, and
  `NewJob.svelte` into `FileList` and `SetupList` with its logic in `newJob.ts` (now tested).
- **Duplication**: helpers shared by several providers had been left in one provider's module;
  they moved to `http.rs`. The "test key" command built a second provider registry; it now asks
  the engine.

## Decisions taken while implementing

- **Claude: server-side fallback on.** Opus 5.5 and Sonnet 5.5 requests send
  `fallbacks: "default"`, so a safety-classifier refusal is retried on another Claude model.
  iwac-ai-pipelines skips refused pages instead, so that the wrong model is never credited; here
  the response's `model` field (the model that wrote the text) is what provenance records, which
  keeps the credit right. Remove the two lines in `providers/src/anthropic.rs` to go back.
- **Gemini 3.5 Transcribe is not in the catalogue yet.** It needs the Interactions API and the
  Files API; the Gemini chat models, OpenAI's diarizing model and Voxtral cover transcription
  meanwhile.
- **Audio is sent whole** until FFmpeg is bundled: a recording in a format the model accepts
  and within its size limit works; longer recordings and video need the splitting of stage 3.
- **Page previews are rendered by pdf.js in the interface** from the same single-page PDFs the
  providers receive, so PDFium is still not needed.
- **`#lib/` imports.** SvelteKit 3 no longer provides `$lib`; the front end uses Node subpath
  imports, as Live Translation does.
- **Provider tests use hand-written responses** in the documented shapes. Replace them with
  recorded responses once a recording run with real keys has been made; until then the OpenAI
  provider in particular has not been exercised against the live API.
- **Gemini safety filters are off** for every request, as in iwac-ai-pipelines (zmo makes it an
  opt-in): historical documents about violence, religion or politics otherwise trip them.

## Still open

- Searchable PDF and ALTO/PAGE XML exports (need coordinates; Mistral OCR's blocks are a start).
- FFmpeg bundling and splitting with overlap; the audio player in the review view.
- Tray icon so jobs continue with the window closed; the close dialog pauses them for now.
- Recorded provider responses, and an end-to-end run with real keys.
