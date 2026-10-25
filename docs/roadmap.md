# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (25 October 2026)

Early development. Everything below is implemented and tested against recorded or documented
provider responses, but **nothing has yet been run against the live APIs with real keys**, and
there is no release yet.

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
- **Review.** Source and reading text side by side, set as a critical edition, with an
  apparatus of differences between set-ups and corrections, linked both ways to the text and its
  line numbers; corrections kept apart from the AI output; marking as reviewed; a player for
  recordings with speakers that can be named; an analysis review that collates answers by
  field and finds names in the document.
- **Comparison mode** with character error rates against corrected pages.
- **Pipelines**: a job's settings saved under a name and applied in one click.
- **Design**: the Critical Edition interface ([DESIGN.md](../DESIGN.md)), responsive from 640 to
  2560 pixels, light and dark, with a side menu that folds to icons.

## Next

- **Refactor the review**: extract the frame shared by the transcription and analysis reviews,
  and split New job into smaller components (see the
  [25 October checkpoint](decisions/2026-10-25-analyses-review-and-pipelines.md)).
- **Run every provider against the live APIs** with real keys, and fix what the documented
  response shapes got wrong. Measure costs against the estimates.
- **Build the bundled LGPL FFmpeg** with its workflow (not run yet; development uses an FFmpeg
  found on `PATH`).
- **Gemini 3.5 Transcribe** in the catalogue.
- **Text extraction**: ALTO and PAGE XML where models return coordinates; checks for skewed
  and blurry pages.
- **Analyses**: agreement between set-ups across a whole job (exact agreement and Cohen's kappa
  for choices, overlap for lists), shown in Jobs; a column added to an `.xlsx`; grouping
  variant spellings of the same name across a job, confirmed by the researcher.
- **Design decisions still open**: app icon, name, a collation table in Jobs, the interrupted
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
