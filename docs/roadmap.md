# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (25 October 2026)

Early development; [v0.1.0](https://github.com/fmadore/Scriptorium/releases/tag/v0.1.0) is
the first test build, for Windows on ARM64, with FFmpeg bundled. Everything below is tested
against recorded provider responses. Text extraction, transcription and analyses have also run
against the live OpenAI, Claude and Mistral APIs with real keys (`cargo run --example live`,
see [development.md](development.md)); Gemini has not yet, for want of credit on the account.

## Done

- **Foundations.** Rust workspace and Tauri shell, job database with caching and resuming,
  progress and taskbar progress, model catalogue refreshed from the repository, prompt library,
  keys in Credential Manager, English interface with translation files, CI with `cargo audit`
  and `cargo deny`, Dependabot.
- **Text extraction.** Gemini, OpenAI, Claude and Mistral; PDF page cutting with PDFium, image
  preparation, input checks with splitting of double pages, language choice, line breaks kept
  as on the page or joined into running text, exports to `.txt`, `.md`, `.docx`, `.json` and
  `.csv`, searchable PDFs from Mistral OCR.
- **Audio transcription.** Recordings and videos split at pauses and converted by FFmpeg, checks
  for quiet, long or silent files with an "even out the volume" fix, speakers and timestamps,
  `.srt`, `.vtt` and `.csv`. Offline transcription with Whisper, models downloaded from
  Settings. Jobs keep running from the notification area when the window is closed.
- **Analyses.** Fields defined by the researcher (text, lists of names or terms, one or several
  choices, reasons), sent as each provider's structured output and checked on return; four
  ready-made analyses (research summary, short summary, people, places and organisations, tone);
  a field editor in Prompts; CSV with a column per field.
- **Review.** The source on a dark lightbox beside the text; differences between set-ups and
  corrections highlighted in the text, each opening its readings, any of which can be taken as
  a correction; corrections kept apart from the AI output; marking as reviewed, with a strip of
  pages showing what is left; a player for
  recordings with speakers that can be named; an analysis review that collates answers by
  field and finds names in the document.
- **Comparison mode** with character error rates against corrected pages.
- **Pipelines**: a job's settings saved under a name and applied in one click.
- **Design**: the Graphite and highlighter interface ([DESIGN.md](../DESIGN.md)), responsive
  from 640 to 2560 pixels, light and dark, with a sidebar that lists the jobs, and an app icon
  drawn from the same mark.

## Next

- **Refactor the review**: extract the frame shared by the transcription and analysis reviews,
  and split New job into smaller components (see the
  [25 October checkpoint](decisions/2026-10-25-analyses-review-and-pipelines.md)).
- **Run Gemini against the live API**, as OpenAI, Claude and Mistral were, and measure costs
  against the estimates. Offline Whisper still needs a live run too.
- **Gemini 3.5 Transcribe** in the catalogue.
- **Text extraction**: ALTO and PAGE XML where models return coordinates; checks for skewed
  and blurry pages.
- **Analyses**: agreement between set-ups across a whole job (exact agreement and Cohen's kappa
  for choices, overlap for lists), shown in Jobs; a column added to an `.xlsx`; grouping
  variant spellings of the same name across a job, confirmed by the researcher.
- **Design decisions still open**: name, a collation table in Jobs, the interrupted
  job with Resume in the side menu: [issue #1](https://github.com/fmadore/Scriptorium/issues/1).

## Later

- French and German interface.
- OpenRouter for open-weights models.
- Offline text extraction with Tesseract.
- Linking names to authority records (an id from candidate records, or none, with a reason).
- Microsoft Store release, with the privacy policy and own licence terms. Store policy 11.16
  (live generative AI content) also asks to disclose the use of generative AI in the listing and
  in Partner Center, and to give users a way to report inappropriate output to the developer:
  add a "Report a problem" link to the app before submitting.
