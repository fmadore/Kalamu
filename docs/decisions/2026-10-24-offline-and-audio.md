# 2026-10-24 · Tray, offline Whisper, FFmpeg and audio

## Context

The next round after the stage 2 checkpoint: keep jobs running with the window closed, add
offline transcription downloadable from the app as in Live Translation, bundle FFmpeg for
audio and video, then summaries and keywords. Provider documentation was re-read on this date.

## Decisions

- **Tray icon.** Closing the window while jobs run offers to keep them running in the
  background; the notification-area menu shows progress and can open the window, pause all
  jobs, or pause them and quit. Ported from Live Translation's `tray.rs`; the interface pushes
  translated labels.
- **Offline Whisper is a provider.** A new `offline` crate implements the `Provider` trait with
  whisper.cpp through whisper-rs 0.16 (bundling whisper.cpp 1.8.3), CPU only. Providers now say
  whether they need a key (`needs_key`) and whether a model can run (`check_ready`), so an
  undownloaded model or an unsupported processor is reported before the job starts. The
  catalogue marks the provider `local` for the interface.
- **Models are pinned.** Large v3 Turbo, Large v3, Small and Base (quantised), from a fixed
  Hugging Face revision with SHA-256 checks, plus Silero VAD 5.1.2 (the version whisper.cpp
  1.8.3 shipped with). whisper.cpp opens the VAD model by path, which the C runtime cannot do
  for non-ASCII Windows user folders; those users go without VAD rather than failing.
- **No unsafe code for cancellation.** whisper-rs 0.16's safe abort callback casts its user
  data wrongly, and Live Translation uses the unsafe one; this workspace forbids unsafe code.
  Instead a piece is transcribed in two-minute windows cut at the quietest moment, and the job's
  new `StopSignal` is checked between windows. A piece stopped this way returns
  `ProviderError::Interrupted` and goes back to pending.
- **FFmpeg as separate programs, our own LGPL build.** Recordings are cut into pieces of
  about 20 minutes (iwac-ai-pipelines' length, less when a model's limit is lower) at the
  longest pause before each mark, from one `silencedetect` pass; with no pause the next piece
  repeats two seconds and the repeated segments are dropped when the transcript is assembled.
  The pieces are stored with the job so resuming never re-plans them. Pieces go to the cloud as
  32 kbit/s Opus (OpenAI lists neither FLAC nor Ogg, so WebM there), FLAC or WAV as fallbacks,
  and to Whisper as raw 16 kHz samples. The bundled build is audio-only, LGPL 2.1-or-later,
  with libopus, made by `.github/workflows/ffmpeg.yml`; `docs/ffmpeg.md` sets out how each
  licence obligation is met for the Microsoft Store (including our own licence terms, since
  Microsoft's standard ones forbid the reverse engineering the LGPL must allow).
- **Summaries.** A third task reads each document whole (text, Markdown, CSV and Word as text;
  PDFs as they are) and gets a summary and keywords from each provider's structured output.
  The default prompts are new, in English, keeping the principles of iwac-ai-pipelines'
  bilingual summary prompt. Summaries are named `<name>.summary.*`. Mistral's chat models and
  `.xlsx` columns are not wired up yet.
- **Searchable PDF from Mistral OCR only**, as the user decided: Mistral is the one provider
  that reports text positions (paragraph boxes, `include_blocks`). The exporter reads the
  source PDF itself and writes a glyph-less Unicode text layer, as Tesseract does.
- **Provider documentation, October 2026.** Gemini 4 Argon, GPT-6.1 Sol, GPT-6 Astra,
  gpt-transcribe and Claude Fable 5.1 added; the diarizing OpenAI model marked as retiring;
  Gemini's media resolution set per request (still high, against Google's new advice of medium
  for PDFs, because our material is dense historical print and handwriting); thinking level
  written in lower case as the docs show it. Gemini 3.5 Transcribe needs the Interactions API
  and a forum thread reports empty transcripts over REST, so it waits.
- **zip advisories ignored, with reason.** RUSTSEC-2026-0367, -0368 and -0369 affect extracting
  zip archives to disk; docx-rs 0.4.22 (the latest) still needs zip 8, and Scriptorium only
  writes Word files and reads .docx entries into memory. Remove the ignores in
  `.cargo/audit.toml` when docx-rs moves to zip 9.

## Checkpoint review

- **Dependencies still point inwards.** The new `offline` crate depends on `core` only
  (`catalog` in its tests); `media` gained docx-rs (reading Word files) and `export` gained
  lopdf (the searchable PDF). `engine` reaches FFmpeg only through `media`.
- **Files over ~500 lines** were split: the review read model out of the run loop, model
  downloads out of the Whisper model table, audio formats out of the FFmpeg runner.
- **Accepted duplication.** The searchable PDF exporter repeats about forty lines of page
  geometry (inherited attributes, page box, rotation) from `media`'s PDF reader, to keep
  `export` independent of `media`. Revisit if a third crate needs them. It is also the one
  exporter that reads a file (the source PDF) rather than only stored results.
- **Resuming, caching and provenance** cover the new paths: Whisper pieces stopped part-way go
  back to pending; recording pieces are stored with the job; volume evening is in the cache
  key and provenance; summaries are cached like pages.
- **Gaps to close.** No progress inside a long Whisper piece, and the silence analysis of a
  long recording shows no progress either; the FFmpeg build workflow, the Whisper smoke test
  with a large model and the summary requests have not run against the real thing (Whisper was
  checked with the tiny model, FFmpeg with a local full build).
