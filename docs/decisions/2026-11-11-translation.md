# 2026-11-11 · Translating a text: a task of its own

## Context

Researchers often work on sources in a language their readers, co-authors or they themselves
read less easily: a newspaper in Arabic, an interview in Hausa, a file in German. Until now
Kalamu translated only while transcribing a recording, and only into English (the *Interview or
recording, translated* preset and Whisper's own translation). A text that was extracted,
transcribed or corrected could not be translated without leaving the app, and a workflow could
not end with a translation ([#167](https://github.com/fmadore/Kalamu/issues/167)).

A correction already does most of what a translation needs: it reads text that exists (a PDF's
text layer, a text file's pages or passages, a frozen result), sends it page by page as an
analysis whose one field is the new text, stores that field as the page's text, and keeps the
text it read for the review ([decision](2026-10-29-contents-and-correction.md)). The question
was whether a translation is a correction preset or a task.

## Decision

- **A task of its own, `Task::Translation` (`translation`)**, offered in New job right after
  *Correct existing text*. A correction keeps the source's words and its review shows every
  change against the original; a translation answers in another language, and two languages
  have no differences to show. A preset could not change either: the review, the answer
  language, the line breaks and the file names all follow the task.
- **It reaches providers as a correction does.** `request_task` is `Analysis`, the answer is the
  one field `text` (`correction::translation_schema`, labelled *Translated text*), and the engine
  stores it as the page's text. Providers add `Translation` beside `TextCorrection` in the arms
  where they already name it, and nothing else; no provider or engine code matches a provider's
  or model's name. The catalogue needs no entry: a translation takes the analysis models that
  read text, the same as a correction.
- **What it shares with a correction is one predicate, `Task::rewrites_text`**: it reads the text
  there is (`plan`), sends text whatever the model reads (`accepted`), stores the answer's field
  as text (`finish`), and keeps the text each piece had for the review (`source_pieces`). The
  engine asks the predicate instead of naming either task, so a third such task would touch
  only `core`.
- **The job's language is the target, one only, and must be chosen.** `answers_in_language` is
  true, as for an analysis; unlike an analysis, a translation has no language of its own to fall
  back on, so `Task::needs_language` is true: New job hides *Detect automatically*, a workflow
  step made from a prompt asks for the language, and `Engine::check_job` refuses a job without
  one (`job.no_language`) before anything is sent. The preset names the target through
  `{languages}`, as analysis presets name the answer language.
- **No choice of lines.** `has_line_breaks` is false: a translation follows the layout of the
  text it reads, which already made that choice (or kept the page's lines), and a page's lines do
  not carry over into another language. The preset keeps paragraphs, headings and short lines.
- **Page by page, with samples.** `takes_samples` is true, as for a correction; a PDF is read
  for its text layer (`reads_text_layer`), never split into double pages.
- **A ready-made prompt, *Translation for research*** (`translation.research`): everything
  translated in order, nothing summarised or left out; names of people, places and
  organisations kept as written; a culturally specific term translated with the original in
  parentheses; an unsure reading marked `[word?]`; paragraphs, headings and a transcript's
  speaker labels and times kept exactly; no notes of the translator's own; a machine-read
  source translated for what is meant where a misreading is obvious, never invented. It carries
  the translated interview's rules over to any text.
- **Results never replace their source**: `<name>.translated.<code>.*`, the language's code in
  the name so that translations into two languages do not replace each other. The formats are a
  correction's: text, Markdown, Word, JSON, CSV.
- **The review shows the source beside the translation.** The engine fills
  `PieceView::source_text` for a translation as for a correction; the interface puts it in the
  source pane (where a text file's text already stood) and does not add it as a witness, so no
  differences are drawn. The translation is corrected as any text is; in a comparison, two
  models' translations are compared as two readings are. A PDF keeps its scan in the source
  pane.
- **Its text yields a text.** `yields_text` is true: *Use this text in a new job* and workflow
  steps can read a translation (an analysis of the translation), and its errors are scored
  against the researcher's corrections like any text's. *Use this text in a new job* offers
  *Translate it* beside *Analyse it* and *Correct it*.
- **The cost estimate** is a correction's, with the answer a quarter longer than the text read
  rather than a tenth: a translation runs longer than its source in most pairs of languages.
- **Offline language models translate.** The offline language model provider matched the job's
  task against `Analysis`, so it refused corrections and tables of contents that New job offered
  it; it now matches `request_task()`, and translations come with them.

## Consequences

- Provenance, the cache, resuming and the frozen text need nothing new: a translation is a job
  like a correction, and the engine's tests run one over a text file, a PDF's text layer and a
  frozen correction, resume it after a crash without sending anything again, and freeze its
  text for a next round.
- No ready-made workflow translates yet: the examples run on the models New job would offer and
  detect the language, so a translation example needs a language chosen first. A researcher's
  own workflow takes a translation step after any step that yields a text.
- A translation's review of a PDF shows the scan, not the text layer the model read. Showing
  that text would need the source pane to switch between the scan and the text.

## Not tried

- **Subtitles of a translated transcript.** A transcript's times are kept in the translated
  text, but the segments are not carried over, so SRT and WebVTT are not offered yet.
- **A per-project glossary** (a term and its translation) from the project dictionary, sent with
  the prompt.
- **Models trained to translate**, offline or in the cloud, beside the language models.
- **Names to spell from the project dictionary**: the prompt keeps names as written instead.
