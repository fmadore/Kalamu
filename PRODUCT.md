# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(A Tauri 2 desktop app for Windows, x64 and ARM64; the interface is a SvelteKit web front end in
WebView2, distributed through the Microsoft Store and as unsigned installers on GitHub.)

## Users

Researchers who are not technical: historians and social scientists, many uncomfortable
installing or configuring software, working on their own Windows laptops. Their material is
archival: scanned newspapers and reports, handwritten letters, Arabic and Ajami manuscripts,
and oral-history interviews, often in French, English, Arabic and West African languages. They
run long batches (a folder of interviews, a 400-page scan) over hours, then check the results
against the sources.

## Product Purpose

Scriptorium turns recordings and scanned or handwritten documents into text (transcription,
OCR and HTR in one "extract text" task) and analyses texts with fields the researcher defines
(a summary, the people and places named, a tone), using the AI
provider the researcher chooses (Google Gemini, OpenAI, Anthropic Claude, Mistral) or offline
Whisper. Success is trustworthy text the researcher has checked against the source, produced
without losing work or paying twice.

## Positioning

- **Checking against the source.** Results are reviewed side by side with the page or the
  recording; corrections are kept apart from the AI output, which is never overwritten, and
  `provenance.json` records model, prompt, settings, and what was reviewed or corrected.
- **Your files, your provider.** A desktop app: files go straight from the PC to the provider
  the researcher picks, under their own key, with nothing sent to the author; offline Whisper
  keeps sensitive recordings on the PC.
- **Never lose or repay work.** Every piece is cached and resumable after a close, a crash or a
  lost connection; costs are estimated before starting; jobs can keep running from the
  notification area.

## Operating Context

- Long-running jobs watched intermittently; the window may be closed while work continues.
- The review view is where most attention goes: a scan or player on one side, text on the other,
  page by page, with comparison of two or more models on difficult material.
- Inputs: PDF, JPEG, PNG, TIFF, audio (MP3, M4A, WAV, …), video, plain text, Word.
- Outputs written beside the sources: `.txt`, `.md`, `.docx`, `.json`, `.srt`, `.vtt`, `.csv`,
  searchable PDF (Mistral OCR), each with `provenance.json`.

## Capabilities and Constraints

- Three tasks: extract text (printed and handwritten), transcribe audio or video, analyse
  texts. Comparison mode runs several models over the same sources. A job's settings can be
  saved as a pipeline.
- Must stay friendly to researchers who are not technical: plain words, ready-made starting
  points, advanced choices folded away.
- English interface first; every visible string lives in translation files so French and
  German can follow.
- Front end talks to the Rust core only through typed commands and events; no business logic
  in the interface.
- Must work in light and dark themes and follow the Windows theme by default.
- Open decision: the product name. "Scriptorium" is the working name and may change.

## Brand Commitments

- An independent project by Frédérick Madore (University of Bayreuth). It is not associated
  with Live Translation or with ZMO, and must not borrow their identity; `zmo-ai-pipelines` is
  only one of its reference code bases.
- No binding visual identity: the current look, icon and even the name may be replaced.

## Evidence on Hand

- Real sample material appears only in the development preview (`?mock`): a 1957 bulletin and
  an interview; no user testimonials, statistics or institutional endorsements exist and none
  may be invented.

## Product Principles

1. The source is the authority: every result can be checked against the page or recording it
   came from, and the AI's own output is never silently replaced.
2. Nothing is lost: interruptions, closures and failures cost at most the piece in progress.
3. The researcher stays in control of where files go and what it costs.
4. Plain words for non-specialists: warnings and tips explain what to do, not how it works.
