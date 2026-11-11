# 2026-11-11 · Translation and image description: what they left

## Context

The translation task ([decision](2026-11-11-translation.md),
[#167](https://github.com/fmadore/Kalamu/issues/167)) and the ready-made image description
([#168](https://github.com/fmadore/Kalamu/issues/168)) shipped with five gaps:

- The engine accepted an analysis of images on a model whose catalogue inputs list only text
  (GLM 5.3, DeepSeek V4.1 Flash), because `Task::reads` is true for an analysis. New job filtered
  them out, but a saved pipeline or workflow bypasses New job, and every piece then failed in
  `media` with "This model reads text only". The same held for a whole PDF on a model that reads
  page images but no PDF (the offline language models).
- A workflow with a translation step and no language was saved, and refused only when a run
  started.
- No ready-made workflow translated, since the examples detect the language and a translation
  needs one.
- New job always chose the task's first prompt, so the image description had to be picked by
  hand even when every file was a photograph.
- The review of a PDF's translation showed the scan, not the text the model translated.

## Decision

- **One rule for what a model can be given, in `media`.** `kalamu_media::readable(kind,
  reading, accepted)` says, from the source's kind alone, whether a model taking `accepted`
  formats can be given it, with the message a researcher reads: a text needs `text/plain`; an
  image needs an image format; a PDF read whole (an analysis, `Reading::Whole`) needs
  `application/pdf`; a PDF read page by page (`Reading::Pieces`) needs PDF pages, PNG, or text
  only, which reads its text layer. `OpenSource::prepare` asks it before preparing a piece, so
  the messages pieces failed with are now this function's, and the engine asks the same function
  before a job is made: the two cannot drift. What depends on the file itself (a scan without a
  text layer, a TIFF of several pages) is still found piece by piece.
- **The engine refuses the job up front.** `Engine::check_reading` (called from `check_new`, so
  by `create_job` and by every step of a workflow when its run starts) works out what each
  set-up's model would be sent, with the same function the run uses
  (`piece::accepted_inputs`: the provider's formats for the task, narrowed to the model's
  catalogue inputs, text only for a correction or a translation), and refuses the first source
  it cannot read: `EngineError::CannotRead`, code `provider.cannot_read`, its detail naming the
  model, the file and media's reason. A model the catalogue does not list is left to its
  provider, as before. No provider or model is named. `check_new` became a method, since the
  check needs the providers; `check_job` stays a function of the catalogue alone.
- **New job agrees for PDFs too.** `inputOf` returns every input the chosen models must read
  (`Needs`, a list) instead of one: images when an image is among the files, and, for an
  analysis, `application/pdf` when a PDF is. A correction or translation of a PDF, and an
  extraction from one, need nothing more, since a text-only model reads its text layer. The rule
  is written in TypeScript beside `readable`'s comment, not sent from Rust: it is three lines,
  and the engine refuses whatever it gets wrong.
- **A workflow's translation step needs its language when saved.** `kalamu_core::workflow::
  check_languages` refuses a step whose task needs a language (`Task::needs_language`) and has
  none: `WorkflowError::NoLanguage`, with the code of the job refused for the same reason
  (`job.no_language`) and the same words ("Step N: Choose the language to translate into"), so
  saving and starting say one thing. `Workflow::check` calls it, so saving, a copy and an import
  are refused alike; the editor shows it where it shows every step problem, above the steps
  after *Save*.
- **A ready-made workflow translates**: *Documents in another language: read, check,
  translate, then summarise* (`translated`), an extraction checked by the researcher, the
  translation (*Translation for research*) reading it, and *Summary and keywords* reading the
  translation. Examples are built in the interface (`EXAMPLES` in `src/lib/workflows.ts`) and
  open unsaved in the editor; its translation step has no language, the editor's language list
  shows *Choose a language*, and saving is refused until one is chosen. A workflow runs only
  once saved, so nothing is sent without the language. No example was given a language of its
  own: there is no right default.
- **A ready-made prompt may say the kind of source it is made for.** `Preset::made_for:
  Option<InputKind>`, set on the shipped default (`defaults::Default::made_for`) and only for
  `analysis.image_description` (images). The library sets it from the shipped default by id
  whenever a preset is listed, loaded or saved, so an edited image description keeps it and a
  copy, an import or the researcher's own prompt has none, whatever its file says. New job's
  `suggestedPreset` picks the first prompt made for the files' kind when every file is of it,
  else the task's first; it is applied when the task is chosen and again as files come and go,
  only to set-ups whose prompt is still the one Kalamu chose. A prompt the researcher picked is
  never switched. Simpler designs were weighed: a rule in the interface naming the image preset
  (matches on an id, the kind of coupling the architecture forbids), and the engine choosing
  the prompt (New job shows the choice before the job exists, so the engine would have to be
  asked anyway). The field keeps the knowledge in the catalogue of prompts, in Rust.
- **The review of a PDF's translation or correction shows the text read at a click.** The
  engine already fills `PieceView::source_text` with the text each piece was sent. When the
  source is a PDF and that text is there, the source pane's bar ends with *Show the text read*,
  which puts it in the pane as a text file's text is shown; *Show the page* brings the scan
  back. The choice holds from page to page for the review's time. A correction of a PDF gets
  it too, for the same reason: the page is not always what the model read.

## Consequences

- A pipeline or workflow saved with a text-only model for images now fails at once, on *Start*
  or when the run starts, with one message instead of a failed piece per file. An older job
  already created is not checked again; its pieces fail as they did.
- Engine test `a_text_only_model_is_not_sent_the_formats_its_provider_takes` now expects the
  refusal at creation; `tests/reading.rs` covers images, whole PDFs, a translation of a PDF on a
  text-only model (accepted) and a workflow step reading an earlier step's text (accepted).
- `Needs` replaced the one-string input in New job's helpers (`modelsFor`, `pickerModels`,
  `providerDefault`, `newDraft`) and the dictionary's proposals.
- Workflows offer four examples.

## Not tried

- A live run of any of this against a provider; everything was checked with recorded or fake
  providers and the interface's sample data.
- On a real Mac: nothing here differs by system, but the review's switch has been seen only in
  Chromium and WebKit under Playwright.
- Suggesting a prompt for other kinds (a recording, a PDF of photographs): only the image
  description sets `made_for`, and a PDF of photographs is a PDF.
- The workflow editor marking the translation step before *Save* is pressed: the language list
  shows *Choose a language*, and the refusal comes on saving.
