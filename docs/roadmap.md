# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (10 November 2026)

**Version 1.5.0** is the latest release: unsigned installers for Windows x64 and ARM64 and a
signed, notarised disk image for macOS on Apple Silicon, each with FFmpeg bundled, are on the
[v1.5.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.5.0)
([notes](release-1.5.0.md)), and it is the version sent to the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H), where Kalamu has been since
version 1.0.0 (2 November 2026). **Version 1.6.0** is prepared, not yet tagged
([notes](release-1.6.0.md)): offline analyses with Gemma 4 and Whisper on a Windows PC's graphics
processor. Version 1.4.1 was the first prepared for the Mac App Store
([steps](mac-app-store.md)). Everything below is tested against recorded provider responses. Text
extraction, transcription and analyses have also run against the live Gemini, OpenAI, Claude and
Mistral APIs with real keys, at once and in economy mode (`cargo run --example live`, see
[development.md](development.md) and the [live runs](decisions/2026-11-03-live-runs.md)).

**macOS** (Apple Silicon) is built and tested in CI and has a disk image from 1.1.2 on, signed
with an Apple Developer ID and notarised from 1.4.0
([decision](decisions/2026-11-04-macos-notarisation.md)); it has not yet been opened on a real
Mac, which is third in *Next* below. From the version after 1.5.0 it asks for **macOS 14 or
later** (13 before): the interface uses what WebKit has from Safari 17 on
([review](decisions/2026-11-09-review.md)).

A [review of the whole app on 9 November](decisions/2026-11-09-review.md) (the Rust crates, the
shell and CI, the interface's code, and every screen on its sample data) found the bugs and
improvements that open *Next*; what it did not fix at once is tracked in
[#143](https://github.com/fmadore/Kalamu/issues/143) to
[#149](https://github.com/fmadore/Kalamu/issues/149).

## Done

- **Image description and keywords** ([#168](https://github.com/fmadore/Kalamu/issues/168)): a
  ready-made analysis for photographs, posters, maps and other images, giving a description of
  what is visibly there (its first sentence a summary on its own), the kind of image, keywords,
  the text written on the image, and the people, places and organisations that text names, so
  the project dictionary gathers them. Nothing is inferred from appearance. With images among
  the sources, New job now offers only models that read images: the text-only ones (GLM 5.3,
  DeepSeek V4.1 Flash) were offered before and failed on each image.
- **Translate a text, first increment** ([#167](https://github.com/fmadore/Kalamu/issues/167),
  [decision](decisions/2026-11-11-translation.md)): a task of its own after *Correct existing
  text*, reading a text file, a PDF's text layer or a result's text frozen through *Use this
  text in a new job*, page by page into the one language chosen, with the *Translation for
  research* prompt; the review shows each page of the source beside its translation, with no
  differences view; the formats of a correction, written as `.translated.<code>`; a workflow
  step after any step that yields a text, and its own text analysed in turn. Offline language
  models now take corrections, tables of contents and translations, which they refused before.
- **Only the newest models offered, older ones kept**
  ([decision](decisions/2026-11-11-model-successors.md)): Claude Fable 5.2 and Gemini 4 Carbon
  added, Mistral Large 4 out of preview and Mistral's default; Fable 5.1 and Mistral Medium 3.5
  give way to their successors in New job but keep working in the pipelines and comparisons
  that name them, marked, with their replacement a click away.
- **Foundations.** Rust workspace and Tauri shell, job database with caching and resuming,
  progress and taskbar progress, model catalogue refreshed from the repository, prompt library,
  keys in Credential Manager, English interface with translation files, CI with `cargo audit`
  and `cargo deny`, Dependabot.
- **Text extraction.** Gemini, OpenAI, Claude and Mistral; PDF pages cut with `lopdf`, image
  preparation, input checks with splitting of double pages, language choice, line breaks kept
  as on the page or joined into running text, exports to `.txt`, `.md`, `.docx`, `.json` and
  `.csv`, searchable PDFs and ALTO XML from Mistral OCR and Tesseract.
- **Offline text extraction with Tesseract**
  ([#46](https://github.com/fmadore/Kalamu/issues/46),
  [decision](decisions/2026-11-04-tesseract.md)), in 1.4.0: Tesseract 5 built into the
  app, language data from `tessdata_best` downloaded per language from Settings (the Latin
  script model for Hausa, Mooré, Wolof and the other Latin-script languages without a model of
  their own), PDF pages drawn at 300 dpi with hayro, the languages asked for in New job since
  Tesseract does not detect them, and its speed, which depends on the processor, said in
  Settings and in its description. Each word comes back with its place on the page, so it
  gives a searchable PDF, each line over its own box, and **ALTO XML**, also from Mistral OCR
  with estimated lines and words. A corrected page keeps its words' places, as
  iwac-ai-pipelines' ALTO correction does. Built and tested on Windows ARM64; Windows x64 and
  macOS in CI; a full job with networking off is among the live runs still to do.
- **Offline text extraction with small OCR vision models**
  ([#105](https://github.com/fmadore/Kalamu/issues/105),
  [decision](decisions/2026-11-06-ocr-vision-models.md)), for 1.5.0
  ([notes](release-1.5.0.md)): LightOnOCR-3 1B (blocks placed on the page, so ALTO and a
  searchable PDF) and GLM-OCR (text only), downloaded from Settings with their image encoders,
  run by llama.cpp's own program shipped next to the app (b12110, built by
  `.github/workflows/llama.yml`, published as `llama-b12110-1` and pinned), on the processor on
  Windows and with Metal on a Mac; the seconds a page took on this computer shown in Settings
  and New job. Both read the sample page on the three CI runners, LightOnOCR with its blocks
  placed, in two to three minutes on those small Windows machines. Still to do: a benchmark on
  real archive pages (newspapers, typescripts, French and Arabic print) with error rates,
  a full job on a real Mac and on Windows x64; PaddleOCR-VL left out until then (see the
  decision). The graphics processor on Windows
  ([#122](https://github.com/fmadore/Kalamu/issues/122),
  [decision](decisions/2026-11-06-ocr-graphics-processor.md)): a second build of the program
  with Vulkan for Windows x64 and ARM64, run when it finds a graphics processor with memory
  enough, a discrete card first, unless Settings keeps the models on the processor; a page that
  fails there is read again on the processor, which the session then keeps to; provenance
  names the device, and the seconds a page took are kept per device. On a Snapdragon X Elite
  Vulkan read wrongly on the Adreno, which is never chosen; through OpenCL, GLM-OCR's image
  encoder reads there correctly, about 37 s a page against 58 s
  ([decision](decisions/2026-11-06-ocr-opencl-adreno.md)), while LightOnOCR-3 stays on the
  processor. On Intel's integrated Arc graphics (Core Ultra 7 155H) Vulkan reads both models
  correctly in less than half the processor's time (51.5 s against 116.8 s for LightOnOCR-3,
  43.4 s against 98.8 s for GLM-OCR). #122 is closed with this. Still to do, among the live
  runs below: timings on an NVIDIA or AMD card, a second Snapdragon driver and more pages, and
  NPUs (Hexagon, OpenVINO), to be measured before anything is built.
- **Offline analyses with small language models**
  ([decision](decisions/2026-11-07-offline-language-models.md)), after 1.5.0: Gemma 4 E4B and
  E2B, Google's own 4-bit files downloaded from Settings with their encoders, for analyses,
  tables of contents and corrections with nothing sent. A text goes to llama.cpp's text program
  (`llama-completion`, now built and shipped beside the multimodal program), a page image to the
  multimodal one; the answer is held to the fields' schema as it is written. The two offline
  engines on llama.cpp share its programs and take turns (`LlamaHost`), and the analysis schema
  moved to `core` for both. On Windows they use the graphics processor the OCR models found,
  with as many layers as its memory holds; Metal on a Mac. Still to do: the llama.cpp workflow's
  run that builds and pins the text program, a full job on a real document on each system,
  their speed on a processor and a graphics processor, a comparison of their answers with the
  cloud models' on the ready-made analyses, and OpenCL on Snapdragon's Adreno, measured first.
- **Audio transcription.** Recordings and videos split at pauses and converted by FFmpeg, checks
  for quiet, long or silent files with an "even out the volume" fix, speakers and timestamps,
  `.srt`, `.vtt` and `.csv`. Offline transcription with Whisper, models downloaded from
  Settings: names and words to spell (from a project's dictionary too), English translation
  except with Turbo, each window's language detected on its loudest stretch, lines of subtitle
  length timed by word, loops cut back, and Metal on a Mac (built in CI, not yet run on a real
  Mac). On Windows, Whisper on the graphics processor through a Vulkan helper; speech detection
  that really skips silence (before, the setting did nothing); pausing within one pass of the
  model; Live Translation's token prompt, echo and silence filters; and a smoke test on every CI
  lane ([decision](decisions/2026-11-08-whisper-graphics-processor.md)). Speeds on real graphics
  processors are still to be measured. Jobs keep running from the notification area when the window is closed. Public
  YouTube videos transcribed from their link with Gemini, without downloading them, and reviewed
  beside the video, played on the dark stage in YouTube's privacy-enhanced player, each video
  named after its title
  ([decision](decisions/2026-10-27-youtube-links.md)).
- **French interface.** Every screen, message, tip and the glossary in French, written in the
  terms of archival practice and the digital humanities; the default prompts' names and
  descriptions too, the prompts themselves staying in English. Chosen in Settings, by default
  the system's language when Kalamu has it; dates, numbers and costs written the French way
  ([decision](decisions/2026-11-03-french-interface.md)).
- **macOS.** Apple Silicon, built and tested in CI beside Windows x64 and ARM64: what differs
  between the systems in one place per layer (`src-tauri/src/platform/`, `src/lib/platform.ts`,
  `tauri.macos.conf.json`), the review's player given WebKit's formats, ⌘ shortcuts, Mac window,
  Dock and menu-bar conventions, FFmpeg built for macOS, a `.dmg` in each release, and the
  interface tested in WebKit on a Mac in CI
  ([decision](decisions/2026-11-02-macos.md)).
- **Analyses.** With Gemini, OpenAI, Claude and Mistral, including the Z.ai GLM and DeepSeek
  models Mistral hosts. Fields defined by the researcher
  (text, lists of names or terms, one or several choices, reasons), sent as each provider's
  structured output and checked on return; five ready-made analyses (summary and keywords,
  scholarly publication, people, places and organisations, tone, image description and
  keywords); a field editor in Prompts;
  CSV with a column per field.
- **Default prompts, one per kind of material**: printed or typed documents, newspapers and
  handwriting for text extraction, each for any language and either reading direction;
  recordings transcribed or translated into English; one correction prompt. Defaults that
  repeated another for a language or a variant were retired, and anything that names one runs
  with its replacement ([decision](decisions/2026-11-02-fewer-default-prompts.md)).
- **Detailed tables of contents** of newspaper and magazine issues, archival files and books:
  each page read by one model, the pages combined by another, of any provider; presets ported
  from iwac-ai-pipelines' issue index and widened; a review of the entries beside the scan,
  with the strip of pages and the entries following the page; an answer language, as analyses
  have; a diagram in New job ([decision](decisions/2026-10-29-contents-and-correction.md),
  [first test](decisions/2026-10-29-contents-review-and-answer-language.md)).
- **Correcting existing text**: a PDF's text layer or a text file corrected page by page or in
  passages, with iwac-ai-pipelines' OCR-correction prompt among the presets, and every change
  highlighted against the original in the review.
- **Review.** The source on a dark lightbox beside the text, in panes that can be resized,
  with another set-up's whole text beside them on demand; differences between set-ups and
  corrections highlighted in the text, their readings shown on pointing, any of which can be
  taken as a correction; corrections kept apart from the AI output; marking as reviewed, with a strip of
  pages showing what is left; a player for
  recordings with speakers that can be named; an analysis review that collates answers by
  field and finds names in the document.
- **A smoother review and wider views** (1.4.0): the scan zooms about the pointer (Ctrl + wheel
  or a pinch, 25% to 600%), moves by dragging and zooms on a place with a double click; choosing
  a difference in either text brings it level in the other; the readings box can be dragged or
  moved with the keyboard; curly and straight quotes and apostrophes count as the same reading,
  in the differences and the error rates. Every view takes the window's width, with larger type
  (body 16 px, the text being checked 19 px) and an 18rem sidebar listing the projects, onto
  which a finished job can be dragged. *New prompt* writes a prompt from nothing, and Settings
  is in tabs (API keys, Offline, General, About).
- **The Jobs page** (1.4.0): every job in a table, searched by name, workflow or file name,
  filtered by task or status and sorted, with actions on several rows at once; the sidebar's
  *Finished* group shows the eight most recent.
- **Comparison mode**, each set-up scored against the reviewed pages.
- **Error rates after review, for any job**: character and word error rates against every
  page corrected or marked as reviewed, summed over all the characters and words, for the job
  and each document on its page and for the document in the review, in plain words
  ([decision](decisions/2026-10-29-error-rates-after-review.md)).
- **Results that cannot be wrong unnoticed**: an answer cut off at the model's output limit is
  kept but marked, counted on the job's page, warned about in the review and listed in
  provenance; a file changed on disk since it was added is refused; the first answer stored
  for a fingerprint is never replaced
  ([decision](decisions/2026-10-29-result-integrity.md)).
- **Work neither lost nor paid twice**: quitting waits for the pieces already sent, saying how
  many, with "Quit now"; a pause or cancel ends a wait between retries; at launch a job with a
  waiting batch is only collected; an engine error lets the calls already started finish; the
  job's page shows what it cost over all its runs
  ([decision](decisions/2026-10-29-cost-and-quitting.md)).
- **Results that reach the researcher**: a document's files are written even when a page keeps
  failing, the page marked `[Not read: reason]` and listed in provenance; sources of the same
  name sent to one folder are numbered instead of writing over each other
  ([decision](decisions/2026-10-29-files-despite-failures.md)).
- **One frame for the reviews**: the text, analysis and table of contents reviews share their
  masthead, source pane, handles, strip, keys (Ctrl Enter to mark reviewed or save a
  correction, Page Up and Page Down to turn pages), correction editor and key to the sigla; a
  text file's correction shows the passage beside it
  ([decision](decisions/2026-10-29-review-frame.md)).
- **Search in the review** (Ctrl F): the document's text as shown, corrections included,
  searched across its pages or a recording's pieces, capitals and accents ignored; the matches
  listed by page or by time, each gone to with its words marked and a recording playing from
  its line; a table of contents' entries filtered; an analysis's fields kept where their answers
  or reasons hold the words, which are also marked in a text document
  ([decision](decisions/2026-10-29-search-in-the-review.md),
  [analyses](decisions/2026-10-30-version-1.md)).
- **The rest of the checkpoint's refactor**: New job, the job page and the sidebar in smaller
  components; the engine's job loop split into planning (`plan.rs`), one piece's call
  (`piece.rs`) and the loop (`run.rs`), sharing the piece-saving and cost code with batches,
  which are split into sending and collecting; the rules on which files, models and formats
  suit each task given to New job by the engine (`task_rules`), which checks them again when a
  job is created ([decision](decisions/2026-10-30-version-1.md)).
- **Version 1.0.0 for the Microsoft Store**: Report a problem in every review and in Settings
  (policy 11.16), the MSIX package with FFmpeg inside and the Store bundle built by the release
  workflow, third-party notices for every crate and package, sample files for the certification
  team, and the listing for 1.0.0 ([packaging-msix.md](packaging-msix.md),
  [decision](decisions/2026-10-30-version-1.md)).
- **A glossary** for colleagues new to these tools: a page beside Settings explaining seventeen
  words the app uses (OCR and HTR, LLM, Whisper, Tesseract, API key, prompt, set-up, pipeline, workflow, tokens,
  reasoning effort, economy mode, ground truth, error rates, hallucination, provenance,
  machine-readable) and what each export format suits, with a filter, links between the
  words, and a help mark beside the headings that use them; written in the translation files
  like all interface text ([decision](decisions/2026-10-30-glossary.md)).
- **Icons centred in their buttons**: the icon component's style no longer reaches buttons
  named `icon`, which drew their icon at the left whenever its style loaded last.
- **Analysing corrected text, first increment**
  ([#47](https://github.com/fmadore/Kalamu/issues/47)): *Analyse this text* freezes an
  extraction's, transcription's or correction's corrected text and analyses it without
  reading the originals again; a later correction marks the analysis out of date without
  changing what it read; provenance records the result read and how much was reviewed;
  quotations can be anchored in the frozen text, piece by piece, and checked
  ([decision](decisions/2026-10-31-reviewed-text-snapshots.md)).
- **Project workspaces, first increment**
  ([#57](https://github.com/fmadore/Kalamu/issues/57)): projects with flat collections, a
  document held once in several, every result of a document across jobs with its review state
  and cost, a preferred result per task, files and folders added before they are read, earlier
  jobs added after their matches are confirmed, New job started from a selection and tied to
  it, documents with the same content merged, deletions that never reach a job, missing files
  marked, archiving; a crate of its own, `projects`
  ([decision](decisions/2026-10-31-project-workspaces.md)).
- **Project dictionary, first increment**
  ([#48](https://github.com/fmadore/Kalamu/issues/48)): the names and terms every analysis
  listed, field by field, gathered through the corrected answers across the project or a
  collection; entries made by the researcher, with stable ids, a preferred label and the forms
  as written, merged, taken apart, relabelled, reviewed and kept apart, all surviving a
  restart; look-alike spellings pointed out, never merged; documents and agreement between
  results counted, not mentions; entries whose evidence is gone marked; export as JSON and CSV;
  a crate of its own, `dictionary`
  ([decision](decisions/2026-11-01-project-dictionary.md)).
- **Project dictionary, merges proposed by a model**
  ([#48](https://github.com/fmadore/Kalamu/issues/48)): a field's rows sent, with their
  spellings, the documents naming them and the pairs kept apart, as an analysis job of its own
  under a prompt that can be read and edited in Prompts, with an estimate first and economy
  mode where the model has one; the whole field in one request up to about 100,000 tokens, larger
  fields in alphabetical groups of that size sharing a tenth of their rows, the coverage said; the answers
  checked against the rows sent, references to no row counted and dropped, pairs kept apart
  never offered; each proposal accepted through the ordinary merge (some rows left out, the
  label changed), kept apart, or turned down and taken back, across restarts
  ([decision](decisions/2026-11-02-proposed-merges.md)).
- **Project search, first delivery**
  ([#86](https://github.com/fmadore/Kalamu/issues/86)): every passage of a project's documents
  that holds some words, in the corrected text, capitals, accents and Arabic vowel marks set
  aside; analyses and tables of contents only on request, their field named; the preferred
  readings or every one, by collection, tag, task and review state; each match opening the
  review where it stands; what was searched said with every search; a local index (SQLite FTS5,
  trigrams) derived from the results and made again when lost; a crate of its own, `search`
  ([decision](decisions/2026-11-03-project-search.md)).
- **Project search, second delivery**: all the words of a query anywhere in a page or answer,
  or near each other within a chosen number of words, quoted words kept together and words too
  short named; the project's text prepared in the background when Search opens, its progress
  shown; an analysis's match opening at its field and a table of contents' at its entry
  ([decision](decisions/2026-11-03-search-words-and-preparation.md)).
- **Source descriptions**, first increment of
  [#55](https://github.com/fmadore/Kalamu/issues/55) and what was left of
  [#57](https://github.com/fmadore/Kalamu/issues/57): each document's description of its source
  (repository, fonds, series, box, shelfmark, creator, date, web address, citation, rights), the
  parts a project's documents share, both exported as JSON and CSV with where each value came
  from; renaming a tag across a project
  ([decision](decisions/2026-11-03-source-descriptions.md)).
- **Export profiles**, second increment of
  [#55](https://github.com/fmadore/Kalamu/issues/55): the descriptions export's columns chosen,
  ordered and renamed, from the document, its description, where each part came from or a
  fixed text; lists, empty cells and line breaks written as the profile says; checked as it is
  edited, saved for every project, duplicated, shared without its fixed texts unless asked, and
  imported; starting from every column or from Dublin Core terms
  ([decision](decisions/2026-11-03-export-profiles.md)).
- **What to check, first increment**
  ([#85](https://github.com/fmadore/Kalamu/issues/85)): one list per project, collection or
  selection of the pages and answers that need the researcher, from stored results only:
  failures and pages not read apart from answers, answers cut off, set-ups that disagree (words,
  or list, choice and entry fields), answers read from a text corrected since or from an item given other pages, and pages not
  reviewed; preferred results where chosen, otherwise every result, each listed to be set
  aside, documents counted once; each item opens its review at its page with the next and
  previous items at hand; coverage counted whatever the filters
  ([decision](decisions/2026-11-03-what-to-check.md)).
- **Tags on a project's documents**: the researcher's own labels beside the collections, added
  and removed on a selection or a document, narrowing the table and found by its search; one
  spelling per project whatever the capitals, kept through a merge.
- **A project's Activity, filtered and costed**: its jobs narrowed by task, state, model,
  workflow and when they were started, and the jobs shown costed together, with those not yet
  priced (offline, or not yet answered) counted apart so a sum is never read as whole when it
  is not. A job does not record a pipeline it was filled from (New job's settings stay
  editable after one is applied), so the workflow a job is a step of stands in for it.
- **Workflows, first increment** ([#49](https://github.com/fmadore/Kalamu/issues/49)):
  pipelines run one after another on the same files, each step reading the files or an
  earlier step's corrected text, frozen when it starts; a review stop, or pages not read, holds
  the steps that read it until the researcher continues; each step is an ordinary job, and
  provenance names the run, the workflow's version and the go-ahead; definitions versioned,
  exported and imported, runs frozen; running again pays only for the steps that changed; a
  run started from a project's selection ties its jobs to the documents
  ([decision](decisions/2026-11-01-workflow-runs.md)). For colleagues new to them: *Then* at
  the end of New job analyses the text once it is checked, with no workflow to build; a banner
  on the job's page and in its review offers *Continue*, and the sidebar lists those jobs under
  *For you to check*; three ready-made examples; an animated drawing of the steps; a glossary
  entry. Any finished text goes into a new round from *Use this text in a new job* (analyse it,
  correct it, or run a workflow on it), on the job's page or on a project document's results,
  which send only that document's text.
- **Each model's own output limit and a thinking level per set-up**: answers may use the whole
  output each model is documented to give (none is set where Mistral documents none); New job
  offers each model's reasoning levels under *Thinking*, the lowest suggested for OCR, HTR and
  transcription, `low` for analyses; the level is settled when the job is created and recorded
  in provenance ([decision](decisions/2026-11-01-output-limits-and-reasoning.md)).
- **Several pages per call** for text extraction: one page at a time (the default), several,
  or the whole document, sized to what the model can write in one answer; the answer split
  back into pages at the marks the model is asked for, and a page left out or cut short read
  again alone, so the review and error rates stay page by page; not yet in economy mode
  ([decision](decisions/2026-11-01-pages-per-call.md)).
- **The last of the 29 October checkpoint**: analysis answers billed but refused for breaking
  their fields, in a single call or a batch, now count in what a job cost, on its page and in
  the job list, and `provenance.json` lists them apart (`refused`); provenance also records
  whether Whisper skipped silences with its voice-activity model, which speaker labels were
  renamed (not the names), a table of contents' second step's usage and cost, and only the
  fixes that were made to a source
  ([decision](decisions/2026-10-31-refused-answers-and-provenance.md)).
- **Economy mode and prompt caching**: a job sent as batches to Gemini, OpenAI, Claude or
  Mistral at half price, answered within a day and collected after the app was closed; cached
  input billed at its own price; both costs shown before starting
  ([decision](decisions/2026-10-27-economy-mode-and-prompt-caching.md)).
- **Pipelines**: a job's settings saved under a name and applied in one click; exported as an
  importable JSON file and a Markdown record for a methods section, and imported.
- **Provenance of each request**: `provenance.json` records the settings each request carried
  (reasoning effort, output limit, media resolution), for the step that reads and the step
  that combines, kept with each result when it was sent; offline results are marked `offline`
  ([decision](decisions/2026-10-29-provenance-request-settings.md)).
- **Design**: the Graphite and highlighter interface ([DESIGN.md](../DESIGN.md)), responsive
  from 640 to 2560 pixels, light and dark, with a sidebar that lists the jobs and folds to a
  rail (its finished jobs the 8 most recent, then a link to the Jobs page, a table of every job
  with a search on names and files, filters by task and status, sorting, and the job menu's
  actions on one row or several), and an app icon
  drawn from the same mark. Under the tasks in New job, a diagram of how each task splits
  and joins the work, drawn with the user's own files; for text extraction it shows whether
  the chosen model can also give ALTO XML and a searchable PDF.
- **Items read on their own, first increment**
  ([#50](https://github.com/fmadore/Kalamu/issues/50)): a table of contents' checked entries
  made into items once, then corrected, joined, split, added and removed by the researcher,
  with pages shared between items named; selected items sent to a new extraction or analysis,
  each as a PDF of its pages frozen in the data folder, results written under the item's name
  beside the source and the item named in provenance; an item given other pages marks the
  job out of date; export of their PDFs with a list as CSV and JSON
  ([decision](decisions/2026-11-02-document-units.md)).
- **Kalamu**: the app renamed from Scriptorium, with the "two witnesses" icon and warm graphite
  neutrals; installs of 0.1 and 0.2 have their jobs, settings and keys moved over on the first
  start; the strip of pages shows thumbnails of scans and images
  ([decision](decisions/2026-10-28-kalamu.md)).

## Next

In this order. Work that keeps what has shipped sound comes before new features.

1. **What the [review of 9 November](decisions/2026-11-09-review.md) found.** Fixed at once,
   each with a test: structured answers cut off at the output limit sent once instead of four
   times; batches that can no longer be read given up on; `Retry-After` bounded; answers stopped
   early marked as cut off; a recording's plan recorded whole; a format that cannot be written
   reported; an unsaved correction, prompt or workflow asked about before it is left; New job's
   form kept across Settings; the review in a short window; an estimate, evidence or search
   overtaken by an older answer; a description lost to a tag; folders dropped on a project;
   dialogs that hold the shortcuts; the job page's errors in a row of their own. macOS 14 is the
   minimum. Still to check: these on Windows and in WebKit, since the shell and `offline` do not
   build where the review ran, and the cut-off message for an analysis stopped by a filter
   rather than the limit. A [second round on 10 November](decisions/2026-11-10-second-round.md)
   took up part of what was left: FFmpeg stopped by a pause and by a watchdog, 1-bit, palette and
   CMYK TIFF pages, a source's failure kept across a restart; exports that read only their
   source's rows (a 2,000-page job's files written in 3.6 s where it took 55), costs read from
   an index, a result cache the engine can prune; the readings, an analysis's names and a zoomed
   scan reached from the keyboard, controls' edges at 3:1, failures that stay and are read out;
   Back, remembered filters, panes and rotation, Ctrl F on the tables, a shortcut overview, a
   drop target that shows, undo for two removals; the screens opened in Chromium in CI, the app
   linked on every system, a weekly audit. Left, in this order:
   - **The shell** ([#148](https://github.com/fmadore/Kalamu/issues/148)). Done on
     [10 November](decisions/2026-11-10-shell.md), first compiled by CI: a start-up that cannot
     open its folders, its job database (a newer version's is now refused by the store) or its
     libraries says why in a native message, in English or French, naming the logs folder, and
     ends; unreadable settings moved aside and said once; the keychain read off the window's
     thread, a failed read no longer kept as a missing key; an exported record's Markdown never
     replacing a file of the researcher's; FFmpeg and llama.cpp chosen by an environment
     variable in development builds only; a month of logs; *Clear unused results* in Settings
     ([#143](https://github.com/fmadore/Kalamu/issues/143)); and the installer's and disk
     image's copies told of a newer release from the model list. The
     [fourth round](decisions/2026-11-10-fourth-round.md) moved the tray, the taskbar and the
     notification to the shell, which follows the engine's own events. Left: all of it tried on
     a real Windows PC and Mac, with the window hidden for a whole job.
   - **The offline engines** ([#144](https://github.com/fmadore/Kalamu/issues/144)). Done
     ([decision](decisions/2026-11-10-offline-robustness.md)), built and tested on Linux: llama.cpp's
     runs on `media`'s stoppable runner, ended after ten minutes of silence, its version bounded;
     a graphics processor given up only when the processor answers what failed on it, at once
     when it hangs; model downloads that continue where they stopped and check the drive's free
     space first; the Whisper helper's device listing no longer stalled by its own diagnostics;
     a failed engine's last diagnostic lines in the log; page images and texts in a folder of
     the app's own, emptied at start-up. Still to check on a PC with a graphics processor and on
     a Mac, where CI is the first to build the shell's one-line part. The
     [fourth round](decisions/2026-11-10-fourth-round.md) added: the programs the app starts
     ended with it (a Job Object on Windows, however the app ends; on a Mac when it quits by its
     own ways), a stopped download's progress in Settings, and FFmpeg's build list with the
     decoders of 24- and 32-bit AIFF. Left: that FFmpeg build run, published and pinned; on a
     Mac, the programs after a crash or a Force Quit.
   - **The release pipeline** ([#145](https://github.com/fmadore/Kalamu/issues/145)). Written
     ([decision](decisions/2026-11-10-fourth-round.md)), not yet run: signing secrets only in
     the job that notarises, which compiles nothing; one Windows build per architecture for
     both packages; a packaging dry run on pull requests that touch the packaging; checksums and
     an attestation with every release; every action pinned by commit; interaction tests in
     Chromium and WebKit, and the screens opened in French. Left: the dry run, then a
     validation tag, before the next release.
   - **Large jobs, the rest** ([#143](https://github.com/fmadore/Kalamu/issues/143)). Done
     ([decision](decisions/2026-11-10-fourth-round.md)): the long tables drawn 200 rows at a
     time, a running job's page reading counts instead of every piece, thumbnails drawn small
     by the app and one pdf.js worker; economy batches of at most 48 MiB, their body written
     once (213 MiB above the input before, 46 after), an upload given time for its size. The
     [third round](decisions/2026-11-11-large-jobs-third.md) had the store count a job's
     progress from an index of its own, added *Files with failures* to a job's page so failed
     files are reached however far down they sit, and removed the unused `job_pieces`. The
     [export names round](decisions/2026-11-11-export-names.md) worked out a job's file names
     once per run, in one pass (7.7 s down to 7 ms at 5,000 sources), stopped asking after the
     folder for every file, and raised the benchmark to the issue's 5,000 sources. What remains
     of writing a source's files is Windows making and renaming eight small files. Left: a
     quiet rerun of the benchmark. A
     correction no longer writes its source's files on the spot: they are marked out of date
     with it and written a few seconds after the last save, before they are listed or
     exported, at quit and at the next launch (a save in a 300-page PDF went from 84 ms to
     under 0.1 ms); still to try on Windows and a Mac, where CI is the first to build the
     shell's part.
   - **A model's abilities in the catalogue**
     ([#147](https://github.com/fmadore/Kalamu/issues/147)). Done
     ([decision](decisions/2026-11-10-model-abilities.md)): Claude's fallback, OpenAI's speakers
     and Gemini's speech request follow the catalogue, Mistral guesses no level from a name, and
     a prompt says its pages are dense; cache keys unchanged. Still to compile on CI: one test in
     `offline`.
   - **Keyboard and interface, the rest**
     ([#146](https://github.com/fmadore/Kalamu/issues/146),
     [#149](https://github.com/fmadore/Kalamu/issues/149)). Done
     ([decision](decisions/2026-11-10-fourth-round.md)): a recording's transcript shown once,
     the passage heard marked in it and kept in view; the marks of a project's Results column
     reached from the keyboard; several jobs linked or deleted, and a name renamed across
     set-ups, each in one command and one transaction. Left: all of it tried in WebKit on a
     Mac and with a screen reader.
   - CSV cells that begin with `=`, `+`, `-` or `@` (a telephone number, a list item) were read
     as formulas by a spreadsheet. Done ([decision](decisions/2026-11-10-fourth-round.md)): they
     are written with an apostrophe before them, numbers excepted, in every table but an export
     profile's, which another tool imports. Still to look at in a real spreadsheet.
2. **What the [8 November checkpoint](decisions/2026-11-08-checkpoint.md) left.** The
   checkpoint after 1.5.0 folded the duplication the 4 November one listed (one UTC date in
   core, every CSV through the `export` crate, one batch file reader, the engine's and the
   offline engines' locks and idle sweepers, one write-then-rename in the app), moved the
   descriptions export off the window's thread, split `catalog` and Whisper's audio windows,
   and shared the interface's pickers, path names, players' clock and start-up questions; a
   drop listener left registered after a screen closed was fixed. Left: the file picker's
   extensions sent from Rust (they have drifted: `.aiff`, `.wmv`, `.m4v`, `.xlsx` are hidden),
   FFmpeg's and llama.cpp's shared helpers, the tests that write files for the interface,
   `tauri.ts`, `mock.ts`, `vision.rs`, `llm.rs` and the larger screens split, and from 4
   November Whisper's voice detection on accented data folders on Windows, the folders out of
   reach shown at launch, and Tesseract's drawing resolution in provenance and its engines
   pooled (all listed in the decision).
3. **A first run on a real Mac.** The disk image is signed and notarised and a sandboxed build is
   ready for the Mac App Store, but Kalamu has not yet been opened on a Mac
   ([decision](decisions/2026-11-02-macos.md)). Before TestFlight: the disk image through
   Gatekeeper, the Keychain's prompts, ⌘Q, the Dock and the menu-bar icon, YouTube's player in
   WKWebView, a Whisper and a Tesseract job, and the interface in WebKit at the sizes CI does not
   try.
4. **Economy mode's loose ends**, a shipped feature: reading OpenAI's batch answers in a live run
   (on 3 November it had answered none after 35 minutes); deleting the files a batch leaves in
   the user's OpenAI and Mistral storage once its answers are collected (Mistral's inline batches
   leave their requests as a file too, 951 KB for two pages); then Gemini batches through a JSONL
   file, to lift the 20 MB inline limit, several pages per call in economy mode, and Voxtral
   transcription in batches once its format is documented
   ([live runs](decisions/2026-11-03-live-runs.md)).
5. **Source-linked review, first step
   ([#84](https://github.com/fmadore/Kalamu/issues/84))**: Tesseract's words and Mistral OCR's
   estimated lines drawn over the scan, each opening its text, with the page as a fallback where
   a provider returns none. Kalamu already keeps these boxes for ALTO and the searchable PDF.
6. **What is left of the research workflows**, below.

### What is left of the research workflows

- **Analysing corrected text, what is left of
  [#47](https://github.com/fmadore/Kalamu/issues/47)**: frozen structured results (an
  analysis's corrected answers, a table of contents' entries), a snapshot of only the reviewed
  pieces, and a retention policy for snapshot files
  ([decision](decisions/2026-10-31-reviewed-text-snapshots.md)).
- **Items, what is left of [#50](https://github.com/fmadore/Kalamu/issues/50)**: finer
  boundaries than the page (a column, a passage of text) for articles that share a page; the
  numbers printed on the pages beside the file's own; an item's text taken from an extraction of
  the whole issue rather than read again; items as a project's documents and as rows of the
  descriptions export (from [#55](https://github.com/fmadore/Kalamu/issues/55)); importing an
  exported list back; a large issue's pages combined in bounded groups rather than in one
  request. Items of a multi-page image wait for ordered image documents
  ([#52](https://github.com/fmadore/Kalamu/issues/52)).
- **Project dictionary, what is left of
  [#48](https://github.com/fmadore/Kalamu/issues/48)**: a live run of proposed merges with
  each provider, on a field of a few hundred rows; comparing rows far apart in the alphabet
  (shared words, sound), which the groups leave out; import with conflicts previewed; an
  accepted dictionary versioned and given to later analyses as a fingerprinted input (with
  #49); and telling apart two people who share a spelling, which needs mentions anchored in
  the text (#47).
- **What to check, what is left of [#85](https://github.com/fmadore/Kalamu/issues/85)**:
  items kept aside with a note; a reproducible random sample of unflagged pages; a review
  manifest export (identities, reasons, decisions); invalid evidence anchors; optional field
  rules with their version.
- **Workflows, what is left of [#49](https://github.com/fmadore/Kalamu/issues/49)**: steps
  that read several inputs (the text and an accepted dictionary, with #48), project-level
  steps, an export step that gathers results across steps, the whole run's estimate computed
  in the engine, and restarting a run from a step after the definition changed; a new run
  that keeps the corrections made in the last one, whose steps after a review are otherwise
  read again from uncorrected text; several jobs' or documents' texts sent into one new round
  together, from a project's selection. A live run of a two-step workflow, and one in economy
  mode across a restart. Try *Then* and the examples with one or two colleagues before going
  further.
- **Translation, what is left of [#167](https://github.com/fmadore/Kalamu/issues/167)**:
  subtitles (SRT, WebVTT) of a translated transcript, its times kept; a project glossary (term
  and translation) from the dictionary; models trained to translate, offline; a ready-made
  workflow that translates, once an example can ask for its language; the text layer beside a
  PDF's translation in the review, which shows the scan. A live run with each provider.
- **Project search, the rest** (its first and second deliveries closed
  [#86](https://github.com/fmadore/Kalamu/issues/86)): alternatives (one word or another);
  searching two letters, and less folding work for very common words
  ([decision](decisions/2026-11-03-search-words-and-preparation.md)).
- **Archival metadata**: [#55](https://github.com/fmadore/Kalamu/issues/55) is closed;
  normalised and inferred values beside the literal ones moved to
  [#54](https://github.com/fmadore/Kalamu/issues/54), item-level rows to #50.

### Live runs still to do

The runs of 3 November, their costs against the estimates and what they changed are in the
[decision](decisions/2026-11-03-live-runs.md). Still to run: offline Whisper, and a full
Tesseract job with networking off; GLM 5.3 through a whole job (GLM 5.3 Flash ran one); GPT-6.1
Terra on handwriting and older print before it is considered as OpenAI's default; a table of
contents and a correction mixing providers (Gemini reading, Mistral combining), and a table of
contents in economy mode, after the first run found articles on two pages only of a 12-page
issue with Claude Sonnet 5.5 at `low` (read the other pages' answers, and try a higher effort;
[decision](decisions/2026-10-29-contents-review-and-answer-language.md)); Claude's one-hour
cache only if a batch of a few hundred pages shows a need.

### Also open

- **YouTube**: a live run on long and multilingual videos; long videos in pieces once Gemini's
  clipping of YouTube links covers the sound track again; the cost before the run from a
  `countTokens` call; the embedded player in the packaged app, whose `tauri.localhost` origin
  YouTube must accept, and the links that open a video on YouTube
  ([decision](decisions/2026-10-27-youtube-links.md)).
- **Text extraction**: PAGE XML where models return coordinates (ALTO is done); checks for
  skewed and blurry pages, with persistent image preparation
  ([#53](https://github.com/fmadore/Kalamu/issues/53)).
- **Analyses**: agreement between set-ups across a whole job (exact agreement and Cohen's kappa
  for choices, overlap for lists), shown on the job's page; a column added to an `.xlsx`.
- **Design improvements left from [issue #1](https://github.com/fmadore/Kalamu/issues/1)**,
  closed on 28 October when the name and icon were decided (its closing comment keeps these
  here): a collation table on the job's page (pieces × set-ups, the sigla as column heads); an
  interrupted or paused job in the sidebar's foot with **Resume** (the sidebar already groups
  unfinished jobs, without the button); a page with a correction checked and captured; the
  error colour checked against the hand blue and the highlighter; and the placeholder dashes
  and the apparatus's `]` and `:`, still in the markup, either declared typesetting or moved
  to the translation files.

## Toward the Store release

The first Microsoft Store submission, version 1.0.0. What the Store checks and how to submit
are in [microsoft-store.md](microsoft-store.md), the package in
[packaging-msix.md](packaging-msix.md), the listing text in [store-listing.md](store-listing.md)
and the [privacy policy](privacy.md).

Done: the name (reserved on 28 October 2026) and the package identity in the manifest
(`49346FMadore.Kalamu`), Report a problem, MSIX packaging with FFmpeg and
the licences inside, the `.msixbundle` and `kalamu-samples.zip` in the release workflow,
version 1.0.0, third-party notices generated at build time, the sample files (with a 1755
manuscript and a 1915 newspaper in the public domain), and the listing, What's new and
certification notes for 1.0.0, the privacy policy on GitHub Pages, the screenshots and the
submission. Version 1.0.0 passed certification and is
[in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H) since 2 November 2026. Later versions follow the update steps
in [microsoft-store.md](microsoft-store.md); 1.5.0 is the latest sent.

## Toward the Mac App Store

The same app, sandboxed, beside the notarised disk image
([feasibility](decisions/2026-11-04-mac-app-store-feasibility.md),
[build](decisions/2026-11-04-mac-app-store-build.md)).

Done: grants kept as security-scoped bookmarks and opened again at launch, the folder asked for
before results are written beside sources or beside an export, the support link hidden in the
store build, FFmpeg and ffprobe signed to inherit the sandbox, the data folder copied into the
container, and a *Mac App Store* workflow that starts the sandboxed app in CI and, on demand,
signs and uploads the package (with `altool --upload-package`). For the first submission,
version 1.4.1: a question before files are first sent to each cloud provider (Guideline
5.1.2(i), on Windows too), the listing, review notes, age rating and App Privacy answers, the
privacy policy's lines on the store build, and the steps in [mac-app-store.md](mac-app-store.md).

Next, by the maintainer: the App ID, the two distribution certificates, the profile, the API key
and the app in App Store Connect, with their secrets and `APPLE_APP_ID`; screenshots taken on a
Mac from the screened material ([store-listing.md](store-listing.md#screenshots)); a first TestFlight build opened on a real Mac (*Next*, item 2); then the submission. After
it: a way to grant again the folders of jobs made by the disk image, and a French listing.

## Research workflows (planned)

The following improvements extend Kalamu from individual jobs to reusable research workflows.
They are agreed product directions, not shipped features or release-date commitments. The
current release checks and reliability work above remain first. Issues hold the detailed scope,
dependencies and acceptance criteria; this roadmap holds the sequence, without duplicating them.

The three central goals are:

- **Project workspaces**: organise collections and documents, with the original sources and
  every associated extraction, correction and analysis accessible from each document. A
  document can belong to several collections without duplicating files or results. Activity
  keeps job controls available; the document view brings work from different runs together.
- **A project dictionary**: review all entities and analytical labels across an explicitly
  selected set of jobs/documents, ask an LLM for proposed merges, then accept, reject or adjust
  them. Keep original expressions and source evidence, stable IDs, aliases and reversible
  decisions. Entity identity and thematic-label synonymy have separate rules.
- **Reusable multi-step pipelines**: save a workflow from extraction through user-defined
  analyses and export. Each step chooses its model, prompt and upstream inputs, with optional
  review checkpoints. Existing single-job pipelines remain supported.

Example: extract text → optional text review → extract entities → review dictionary merges →
run another analysis using the text and accepted dictionary → export. Each run records the
exact inputs used at every step and resumes without repeating completed unchanged work.
A pending merge decision is never treated as approval.

### Shared foundation and first workflow

| Order | Improvement | Tracking |
| --- | --- | --- |
| 1 | Project workspaces, collections and stable document identity; group originals and results across jobs (done, with source descriptions; closed) | [#57](https://github.com/fmadore/Kalamu/issues/57) |
| 2 | Frozen reviewed-result snapshots, stable source anchors and linked downstream jobs; shared with post-interview analysis (first increment done) | [#47](https://github.com/fmadore/Kalamu/issues/47), existing [#38](https://github.com/fmadore/Kalamu/issues/38) |
| 3 | Reusable workflow definitions and runs; begin with extraction → analysis → export, then add review checkpoints and project-level steps (first increment done, with review checkpoints) | [#49](https://github.com/fmadore/Kalamu/issues/49) |
| 4 | Project-wide entity/label inventory, model-proposed merges, researcher decisions and local dictionary import/export (done: inventory, researcher decisions, export, model-proposed merges) | [#48](https://github.com/fmadore/Kalamu/issues/48) |
| 5 | Reviewed document units from contents entries, with item-level extraction, analysis and export (first increment done: whole-page items from a PDF) | [#50](https://github.com/fmadore/Kalamu/issues/50) |

The project workspace can ship incrementally before the complete workflow builder. It owns
project/document identity and collection membership (#57); result snapshots and dependencies
are owned by #47. Workflows freeze the documents and result revisions selected at launch,
so later collection edits do not rewrite earlier runs or start extra processing. Dictionary
review can be project-wide or scoped to a visible collection/selection. Existing standalone
jobs/results can be attached without rerunning them. Tags remain useful filters alongside
collections. Later text edits flag dependent findings as out of date while retaining their
original snapshots. Arbitrary scripts, loops and a general graph editor remain outside the
first workflow increment.

### Supporting improvements

| Improvement | First useful increment and dependency | Tracking |
| --- | --- | --- |
| Research-question screening | Relevance preset, explanations, filters and researcher selection; verified evidence spans build on #47 | [#51](https://github.com/fmadore/Kalamu/issues/51) |
| Ordered image documents and paired transcriptions | Import photographs as one document with reviewable order; then pair existing text with scans | [#52](https://github.com/fmadore/Kalamu/issues/52) |
| Persistent image preparation | Crop, processing rotation and adjustable spread splitting; transformation recipes preserve originals and source coordinates | [#53](https://github.com/fmadore/Kalamu/issues/53) |
| Source-linked review | Providers' existing regions drawn over the scan, each opening its text, with a page-only fallback; then stable region identities and researcher-drawn regions. Coordinates shared with #53, text anchors from #47 | [#84](https://github.com/fmadore/Kalamu/issues/84) |
| Context-aware processing | Bounded neighboring pages; then date/place proposals with evidence, literal readings and explicit inference status, shown in export profiles beside the literal value | [#54](https://github.com/fmadore/Kalamu/issues/54) |
| Archival metadata and export profiles | Supplied source descriptions and reusable local CSV/JSON mappings (done; closed). Normalised and inferred values moved to #54, item-level rows to #50 | [#55](https://github.com/fmadore/Kalamu/issues/55) |
| Translation | A task of its own over existing text and frozen results, a workflow step after extraction or transcription, the source beside the translation in review (first increment done); then subtitles of a translated transcript and a project glossary | [#167](https://github.com/fmadore/Kalamu/issues/167) |

Scope is local research preparation, review and export. External collection-system integration
and publishing are excluded from this work. Implementation follows the architecture checkpoints
in AGENTS.md and updates the specification and user-facing documentation when features ship.

## Later

- German interface: `de.ts` beside `en.ts` and `fr.ts`.
- OpenRouter for open-weights models.
- Linking names to imported local authority records (an id from candidate records, or none,
  with a reason), building on the project dictionary in #48.
- Tables of contents of text files (a long Word report, a transcript, read in passages).
  Ordered photographs and item-level extraction/export are tracked in #52 and #50 above.
- Post-interview analysis with research profiles and evidence-linked themes/quotations:
  [#38](https://github.com/fmadore/Kalamu/issues/38), using the shared snapshot foundation.
- Explore assisted TEI encoding with reusable project profiles:
  [#37](https://github.com/fmadore/Kalamu/issues/37); feasibility work, not a committed feature.
- Importing ALTO XML from elsewhere (a digital library's) to correct it with its coordinates
  kept; Kalamu's own extractions already keep their words' places when corrected.
- French and German Store listings, with the translated interface.
- macOS: the application menu's labels through the translation files.
- Projects: nested collections and saved searches, which
  [#57](https://github.com/fmadore/Kalamu/issues/57) left to follow its first delivery.
- Semantic search beside the project's word search, evaluated separately: only with explicit
  local or cloud execution, consent to sending text, versions, costs and coverage; answers and
  chat stay out of scope ([#86](https://github.com/fmadore/Kalamu/issues/86)).
- Linux x86-64: a `.deb` and an AppImage on GitHub releases, built and tested in CI, for a
  documented range of distributions; a third file in each platform layer (`platform/linux.rs`,
  `platform.ts`, `tauri.linux.conf.json`) and WebKitGTK as a third webview. Proposed, not yet
  built: [#87](https://github.com/fmadore/Kalamu/issues/87).
