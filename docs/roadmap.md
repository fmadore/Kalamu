# Roadmap

What is built, what is next, and what is still open. The [specification](specification.md) says
what the app must do; [decisions](decisions/) record why it is built as it is. Each stage ends
with an architecture review (see [AGENTS.md](../AGENTS.md)).

## Status (24 November 2026)

**Version 1.10.0** ([notes](release-1.10.0.md)) is the latest release. Three things, each with
its own date:

- **Published on GitHub**: 22 November 2026, as the
  [v1.10.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.10.0), after its test
  pass on `v1.10.0-rc1` and `rc2`.
- **Submitted to the Microsoft Store**: after that tag, from its `Kalamu_1.10.0.msixbundle`.
- **Live in the Microsoft Store**: seen there on 24 November 2026.

It holds what 1.9.0 brought (published on GitHub on 20 November before its test pass, and never
sent to the Store), and since then documents read against a research question, a project saved
as one file, a sample first, long interviews in parts, photographs taken twice or blurred, and
the plain-first round.

**1.10.1** ([notes](release-1.10.1.md)) is prepared as the next update, its version files set
and not yet tagged: the eight bugs a review of the repository found on 23 November
([#237](https://github.com/fmadore/Kalamu/issues/237) to
[#244](https://github.com/fmadore/Kalamu/issues/244)), all fixed on `main` that day and the
next, a project file checked without opening it
([#246](https://github.com/fmadore/Kalamu/issues/246)), Gemini 4 Neon in place of Argon, and
the review's menus on a right click. No test pass by hand has run on it. Once it is
submitted, the guide and the privacy policy move to `kalamu.frederickmadore.com`
([#228](https://github.com/fmadore/Kalamu/issues/228)), which waited for 1.10.0 to reach the
Store. The paragraph below is how 1.8.0 and 1.9.0 stood on 20 November.

**Version 1.8.0** was published on 17 November 2026: unsigned installers for
Windows x64 and ARM64 and a signed, notarised disk image for macOS on Apple Silicon, each with
FFmpeg bundled, are on the
[v1.8.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.8.0)
([notes](release-1.8.0.md)): records (one row per entry), text regions over the scan, a batch's
files deleted at the provider, and Gemini 4 Hydrogen as the default. It was tried by hand on
Windows as `v1.8.0-rc1` ([results](windows-test-pass.md#results)) and what that found fixed;
the live runs of that pass's Part F are still to do. It is live in the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H) since 17 November, Kalamu
having been there since version 1.0.0 (2 November 2026). **1.9.0** ([notes](release-1.9.0.md))
is prepared as the next update, its version files set and not yet tagged: an interview analysed
from its transcript with quotations found in the text, the search by meaning and the map, the
researcher's own regions and a region read again, a passage corrected by a model, PAGE XML, and
the fix for the review's scan, which redrew itself without end when a scrollbar came and went
outside full screen (#192). 1.8.1, prepared for that fix alone, was never published. What
1.9.0 adds has run against recorded answers only; the live runs and the test pass in the
packaged app come before its tag. **Version 1.6.0** ([notes](release-1.6.0.md)),
the first sent to the Mac App Store, is still in App Review there ([steps](mac-app-store.md)).
Everything below is tested against recorded provider
responses. Text extraction, transcription, analyses and translation have also run against the
live Gemini, OpenAI, Claude and Mistral APIs with real keys, at once and in economy mode
(`cargo run --example live`, see [development.md](development.md) and the
[live runs](decisions/2026-11-03-live-runs.md)).

**macOS** (Apple Silicon) is built and tested in CI and has a disk image from 1.1.2 on, signed
with an Apple Developer ID and notarised from 1.4.0
([decision](decisions/2026-11-04-macos-notarisation.md)). Before 1.6.0 it was tried on one Mac
with Apple Silicon (macOS 26), the sandboxed Mac App Store build included, where the App Store
screenshots were taken; what is left of a first run on a Mac is third in *Next* below. From 1.6.0
it asks for **macOS 14 or later** (13 before): the interface uses what WebKit has from Safari 17
on ([review](decisions/2026-11-09-review.md)).

A [review of the whole app on 9 November](decisions/2026-11-09-review.md) (the Rust crates, the
shell and CI, the interface's code, and every screen on its sample data) found the bugs and
improvements that open *Next*; what it did not fix at once was tracked in
[#143](https://github.com/fmadore/Kalamu/issues/143) to
[#149](https://github.com/fmadore/Kalamu/issues/149), of which #148 and #144 are still open:
both are written, and close with the test pass of 1.9.0.

A [review on 20 November](decisions/2026-11-20-review-for-researchers-new-to-computing.md),
made from the seat of a researcher in the humanities or social sciences who is new to
computing, found that Kalamu is ahead of what has been tried by hand, not short of features,
and set the order of *Next*: 1.9.0's test pass and live runs, then a round of plain-first
fixes ([#208](https://github.com/fmadore/Kalamu/issues/208) to
[#212](https://github.com/fmadore/Kalamu/issues/212)), then the research workflows in the
order given there.

## Done

- **Three items of 17 November**, after 1.8.1
  ([decision](decisions/2026-11-17-search-agreement-subtitles.md)), each tested, none yet
  tried in the packaged app or on a Mac:
  - **Project search, third delivery**: one word or phrase or another (OR or OU in capitals);
    words of two letters standing on their own; each page folded once for all its words, a
    common word on every page of 4,000 found in about 0.2 s where it took about 1 s. The
    index is prepared again once, by itself, at the first search after updating.
  - **Agreement between set-ups across a whole job**: exact agreement and Cohen's kappa for
    choices, mean overlap for lists, on the page of an analysis job with two set-ups or more,
    with a Glossary entry.
  - **Subtitles of a translated transcript** (#167): lines frozen with the transcript,
    translated as numbered lines and checked by number, times and speakers copied by number;
    a correction keeps them while it keeps one paragraph per line.
- **A name pointed at in an analysis is brought into view** in the text beside it (#194),
  after 1.8.1.
- **Records, first increment** ([#171](https://github.com/fmadore/Kalamu/issues/171),
  [decision](decisions/2026-11-15-records-regions-batch-files.md)), after 1.7.0: an analysis
  whose fields are one repeating group, chosen in the field editor with *One row per entry*,
  reads a directory, a register or a list page by page into one row per entry; the review lists
  a page's rows beside the scan, a cell corrected and a row added or removed in the correction
  layer; `.csv` and `.json` hold one row per entry with its file, page and position, a page not
  read standing as a row that says so; two ready-made examples, a directory and a register.
- **Source-linked review, first step** ([#84](https://github.com/fmadore/Kalamu/issues/84)),
  after 1.7.0: *Show text regions* in the text review draws Tesseract's blocks, lines and words
  and Mistral OCR's and LightOnOCR-3's blocks over the scan, estimated lines and words dashed;
  a region chosen with the pointer or the keyboard shows its text and marks its place in the
  page's text only where that place is certain; a page whose model placed nothing says so.
- **Economy mode, a batch's files deleted**, after 1.7.0: once a batch's answers are stored, or
  it is given up on, the files it left in the researcher's OpenAI and Mistral storage are
  deleted, by the ids the batch names; finished at the next launch after a crash, and for a job
  deleted meanwhile.
- **Translation and image description, follow-ups**
  ([decision](decisions/2026-11-11-translation-followups.md)): a job whose model cannot read its
  files (images or a whole PDF for a model that reads text only) is refused when it is made, a
  saved pipeline or workflow included, for the reason each page would have failed with, and New
  job offers only models that read PDFs whole for an analysis of a PDF; a workflow with a
  translation step and no language is not saved; a ready-made workflow reads, translates and
  summarises documents in another language, the language left to choose; New job chooses
  *Image description and keywords* when every file is an image, unless a prompt was chosen;
  the review of a PDF's translation or correction shows the text the model read at a click,
  the page a click away.
- **Ordered image documents, first increment** ([#52](https://github.com/fmadore/Kalamu/issues/52),
  [decision](decisions/2026-11-11-ordered-image-documents.md)): several images in New job are
  read either as separate documents, as before, or as the pages of one document, named (the
  folder's name unless changed) and put in the natural order of their names, page 2 before page
  10, with numbers that skip or repeat pointed out; pages move with buttons or by dragging. An
  extraction or a table of contents reads the document page by page as it reads a PDF, each page
  cached under its own file, so a page moved, or read alone before, is not read again; a page
  whose file was moved or replaced fails alone, named. The review links each page to its
  original file; the results are one document named after it, and `provenance.json` records the
  pages in order with their paths and hashes. Left for later: labels for pages in New job,
  reordering a job already made, analyses of such a document, its items, and pairing existing
  transcriptions with the scans.
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
- **Gemini 4 Hydrogen** ([decision](decisions/2026-11-12-gemini-4-hydrogen.md)): Google's fast
  Gemini 4 model added beside 3.8 Flash, which stays the default until their prices meet on
  1 January 2027; Carbon and Argon now ask for the 32K tokens Google lets one request write.
  Every model has a tier (most capable, balanced, lightweight), and New job lists models by
  provider or by tier across providers, marking the chosen one's tier beside its description.
- **Hydrogen in place of 3.8 Flash** ([decision](decisions/2026-11-16-older-gemini-lines.md)):
  Gemini 4 Hydrogen is now the default and replaces 3.8 Flash, and Gemini 4 Argon replaces
  3.1 Pro; both older models stay in the pipelines that name them. The model menu follows its
  button when the page scrolls, and shows a model's whole description under the pointer.
- **PAGE XML, a sample drawn at random, Claude's batches deleted**
  ([decision](decisions/2026-11-17-page-xml-review-sample-claude-batches.md)), each tested,
  none yet tried in the packaged app: a text extraction written as PAGE XML, one file per page
  beside the ALTO file; in *What to check*, pages nothing flags drawn at random from a seed
  kept with the project, items kept aside with a note, and the list exported as a review
  manifest ([#85](https://github.com/fmadore/Kalamu/issues/85)); a Claude batch deleted at
  Anthropic once its answers are stored; and the image description run over JPEG, PNG, TIFF
  and PDF against recorded answers, the last point of
  [#168](https://github.com/fmadore/Kalamu/issues/168).
- **Search by meaning, the map, and two additions to the review, of 18 November**
  ([decision](decisions/2026-11-18-search-by-meaning-map-passage-correction-added-items.md)), each tested,
  none yet tried in the packaged app, on a Mac or with a cloud model:
  - **Search by meaning** ([#86](https://github.com/fmadore/Kalamu/issues/86)): a project's
    text prepared with a model the researcher chooses (Gemini, OpenAI, Mistral, or
    EmbeddingGemma 2 on this computer), only when they start it and after consent for a
    cloud model, with an estimate, a pause, a resume and the cost kept; documents listed by
    their best passage whatever their length.
  - **A map of a project's documents**, one point each, grouped by collection or tag, with
    the nearest documents in a list beside it.
  - **A passage corrected by a model** from a selection in the review, as a proposal
    accepted, edited, discarded or undone.
  - **Add to a list answer** in an analysis's review: the name, keyword or theme a model
    missed, marked as the researcher's.
  - **A guide** to using Kalamu, published with the documentation
    ([fmadore.github.io/Kalamu/guide](https://fmadore.github.io/Kalamu/guide/)).
- **Five items of 18 November** ([decision](decisions/2026-11-18-gemini-file-batches-set-aside-far-apart-names-resume.md)), each tested,
  none yet tried in the packaged app, on a Mac or in a live run:
  - **Gemini batches through a file**: an economy batch above the 20 MB Google takes in one
    request goes as a JSON Lines file, and that file and the answers' are deleted at Google
    afterwards; smaller batches go as before.
  - **What to check** ([#85](https://github.com/fmadore/Kalamu/issues/85)): the results set
    aside kept with the project and named in the review manifest; a seed typed by hand.
  - **Project dictionary** ([#48](https://github.com/fmadore/Kalamu/issues/48)): in a field
    too long for one request, rows far apart in the alphabet that share a word, initials or a
    sound are sent again together, at no more than twice the cost.
  - **The interface**: a paused or interrupted job in the sidebar's foot with **Resume**; the
    edge of a button that cancels or deletes at 3:1 in both themes; colons, semicolons and
    the dash of an empty value out of the markup.
  - **The test passes written again** for Claude to run on the computer itself
    ([Windows](windows-test-pass.md), [a rented Mac](mac-test-day.md)), and `rfd` 0.18.0.
- **GPT-6.2 Sol and Astra** ([decision](decisions/2026-11-17-gpt-6-2.md)): OpenAI's two models
  of 17 November added; GPT-6.2 Sol is OpenAI's default and replaces 6.1 Sol at the same price,
  and GPT-6.2 Astra replaces GPT-6 Astra for a fifth less. Both older models stay in the
  pipelines that name them.
- **Gemini 4 Neon** ([decision](decisions/2026-11-19-gemini-4-neon.md)): Google's model of
  18 November, between Hydrogen and Carbon, added beside them at $3 / $15; Hydrogen stays the
  default. Escape while a model answers from the review now stops the wait and leaves the
  dialog open, for a passage as for a region.
- **Neon in place of Argon** ([decision](decisions/2026-11-24-gemini-4-argon-replaced-by-neon.md)):
  Gemini 4 Neon replaces Argon, by the maintainer's choice, and with it 3.1 Pro; Argon, which
  costs more than Neon from 1 December, stays in the pipelines that name it, marked replaced.
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
  recordings transcribed or translated into English; two correction prompts, for printed text
and for a transcription of handwriting (the passage dialog offers the second for a page read
as handwriting). Defaults that
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
  the functions that anchor a quotation in the frozen text, piece by piece, and check it are
  written and tested, and no result uses them yet
  ([decision](decisions/2026-10-31-reviewed-text-snapshots.md)). A snapshot also keeps its
  text's turns (each speaker's words, or each paragraph, with its place in the text and its
  times on the recording's clock), read with the text; nothing shows them yet.
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

**First, 1.10.0 tried and tagged** (tagged on 22 November 2026 and live in the Store; what
this paragraph leaves is the day on a Mac and the live runs not yet made, and 1.10.1, which
carries the fixes of 23 November, see *Status*). 1.9.0 was published on GitHub without its test pass, and
the plain-first round and the research workflows of 20 November joined `main` after it, so the
pass runs on `v1.10.0-rc1`, which holds both, and 1.10.0 is the next Store update. No new
feature goes into 1.10.0. Its
[test pass on Windows](windows-test-pass.md) on the packaged `v1.10.0-rc1`, Parts C and F with
real keys above all (an hour's interview analysed, the search by meaning's preparation and
its bill, a region read again, a passage corrected, a batch's files deleted at each provider
and Gemini's batch through a file), and its [day on a rented Mac](mac-test-day.md), where
nothing written since 1.6.0 has run. #148 and #144 close with them.

**Then, plain first: what stops a researcher new to computing** (in 1.10.0, from the
[review of 20 November](decisions/2026-11-20-review-for-researchers-new-to-computing.md)).
Small, and each one meets a colleague on their first day:

- **The first launch and a start without a key**
  ([#208](https://github.com/fmadore/Kalamu/issues/208)): Settings opened at launch only when
  there is neither a key nor a model on the computer (today a researcher who works offline is
  sent to *API keys* at every launch), a welcome there that offers both ways in, and the
  sidebar's line counting every offline engine, where it names Whisper alone.
- **Tables that open in Excel by double-click**
  ([#209](https://github.com/fmadore/Kalamu/issues/209)): an `.xlsx` beside the `.csv`, which
  Excel set to French or German shows in one column; and `corrected` per row in a records
  export. Both were "to decide" after the test pass of 17 November: decided.
- **The default prompts' fields in the interface language**
  ([#210](https://github.com/fmadore/Kalamu/issues/210)): *Summary*, *Themes*, *Research
  question* and the choices of the default analyses, contents and records, shown in French
  when the interface is; nothing stored changes.
- **The guide reached from the app, and in French**
  ([#211](https://github.com/fmadore/Kalamu/issues/211)): nothing in the app opens it today.
- **A first project that does not look like a cockpit**
  ([#212](https://github.com/fmadore/Kalamu/issues/212)): in an empty project, what to do
  first and what fills itself.

All five were written on 20 November, by agents in worktrees gathered on one branch, and none
is in 1.9.0; all are in 1.10.0 ([xlsx and `corrected` per row](decisions/2026-11-20-xlsx-tables-corrected-per-row.md),
[default fields](decisions/2026-11-20-default-fields-interface-language.md)). Still to do: the
`.xlsx` opened by double-click in an Excel set to French and on a Mac (step E2 of the test
pass); the French guide's first build on GitHub Pages, whose layout now serves two languages
and was not built locally; the welcome, the empty project and the French fields seen in the
packaged app, in WebKit on a Mac, and by one or two colleagues who have never used Kalamu.
Left of #210: a choice searched in the review by its French word, and exports' headings,
which stay as stored. Left of #212: *To check*'s boxes at zero in an empty project.

Items 1 and 2 are done in code: what they leave is to be tried by hand, gathered here, and
none of it waits on the others.

The second [test pass on Windows](windows-test-pass.md) and the second
[day on a rented Mac](mac-test-day.md) gather every line below as numbered steps, for Claude
to run on the computer itself.

- **In the packaged app on Windows**: tried on 17 November with `v1.8.0-rc1`
  ([results](windows-test-pass.md#results)). What it found is fixed (estimated lines in Mistral
  OCR's blocks, a records analysis's estimate, New job's records diagram and `.csv`, Escape in
  the language pickers, the in-use badge, the set-aside note, "om." for an empty reading).
  Still to try: a transcription from an accented data folder; Narrator; the notification and
  its icon while the window is hidden; the offline engines on a PC with a graphics processor;
  a register; Part F's live runs. Decided on 20 November: `corrected` per row in a records
  export, and an `.xlsx` for the tables, which French Excel opens in columns
  ([#209](https://github.com/fmadore/Kalamu/issues/209)).
- **On a Mac** (with item 3): all of the above in WebKit, Tab with the system's keyboard
  navigation on and off, VoiceOver, the symbols of `panic.log`, the programs Kalamu starts
  after a crash or a Force Quit, and the folders out of reach shown at launch in the Mac App
  Store build.
- **Written on 15 November**
  ([decision](decisions/2026-11-15-records-regions-batch-files.md)), to try on both systems:
  the launch now that Kalamu's setup makes the window, and its message when the webview cannot
  be made (on Windows, with `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` set to a folder that does not
  exist); an answer stopped by a provider's filter saying so. Escape in the language pickers,
  Home and End in a name's menu and an empty reading's spaces are done too.
- **What 15 November added**: the text regions on real Tesseract and Mistral OCR pages, with a
  trackpad and a screen reader; a records job on a real directory and register.
- **Interview analysis, first slice**
  ([decision](decisions/2026-11-19-interview-analysis-design.md)), to try on both systems
  and with live providers, which no run has done: an hour's interview analysed by each
  provider's default model and by a model on the computer, how many quotations each gives
  that are not in the transcript, and whether the themes are of use; a transcript in a
  language written from right to left; a quotation played from a recording cut into pieces;
  the one box and one button tried by someone who has never used the app; and a transcript
  over an hour with a model on the computer, which the dialog now says is too long before
  Start (the catalogue's `max_input_tokens`, measured on the numbered text; the engine refuses
  such a job when it is created).
- **What the evening of 18 November added**
  ([decision](decisions/2026-11-18-drawn-regions-region-read-again.md)), to try on both
  systems: a region drawn, moved and resized with a mouse, a trackpad, a touch screen and the
  keyboard alone, with Narrator and VoiceOver and in forced colours; a region read again by
  each provider and by Tesseract, its cost against the estimate; the regions on a page of
  thousands of words while one is moved; and all of it by someone who has never used the
  app, which is who it must stay plain for.

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
     notification to the shell, which follows the engine's own events. The
     [fifth round](decisions/2026-11-11-fifth-round.md) weighed Tauri's signed updater and left
     it (a new request at each launch, a signing secret, and nothing the Store copies may use).
     A launch that fails before setup (a webview that cannot be made) has its message since
     15 November. Left: the symbols of `panic.log` on a Mac, and all of it tried on a real Windows PC and Mac, with the
     window hidden for a whole job.
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
     decoders of 24- and 32-bit AIFF, built on 13 November, checked on each architecture and
     pinned as `ffmpeg-9.0.2-2` for 1.7.0, with `.aiff` and `.aif` in the file picker. Left: on
     a Mac, the programs after a crash or a Force Quit.
   - **The release pipeline** ([#145](https://github.com/fmadore/Kalamu/issues/145), closed).
     Done ([decision](decisions/2026-11-10-fourth-round.md)): signing secrets only in the job
     that notarises, which compiles nothing; one Windows build per architecture for both
     packages; a packaging dry run on pull requests that touch the packaging; checksums and an
     attestation with every release; every action pinned by commit; interaction tests in
     Chromium and WebKit, and the screens opened in French. The dry run passed on `main`, and
     v1.6.0 was the first tag it built: its first run failed on the ARM64 lane's path to
     Whisper's Vulkan helper, fixed in #163, and the second published every asset with
     `SHA256SUMS`.
   - **Large jobs, the rest** ([#143](https://github.com/fmadore/Kalamu/issues/143), closed). Done
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
     of writing a source's files is Windows making and renaming eight small files: the
     benchmark run alone on a quiet machine found the disk decides it, at 12 to 17 ms a source
     (recorded in the decision). A
     correction no longer writes its source's files on the spot: they are marked out of date
     with it and written a few seconds after the last save, before they are listed or
     exported, at quit and at the next launch (a save in a 300-page PDF went from 84 ms to
     under 0.1 ms); still to try on Windows and a Mac, where CI is the first to build the
     shell's part. Two smaller costs are left without an issue of their own: listing a job's
     files checks each one on the disk (2 s for 40,000), and the job list parses each job's
     record.
   - **A model's abilities in the catalogue**
     ([#147](https://github.com/fmadore/Kalamu/issues/147), closed). Done
     ([decision](decisions/2026-11-10-model-abilities.md)): Claude's fallback, OpenAI's speakers
     and Gemini's speech request follow the catalogue, Mistral guesses no level from a name, and
     a prompt says its pages are dense; cache keys unchanged, every test passing on CI.
   - **Keyboard and interface, the rest**
     ([#146](https://github.com/fmadore/Kalamu/issues/146),
     [#149](https://github.com/fmadore/Kalamu/issues/149), both closed). Done
     ([decision](decisions/2026-11-10-fourth-round.md)): a recording's transcript shown once,
     the passage heard marked in it and kept in view; the marks of a project's Results column
     reached from the keyboard; several jobs linked or deleted, and a name renamed across
     set-ups, each in one command and one transaction. A
     [fifth round](decisions/2026-11-11-fifth-round.md) checked every bullet of both issues
     against the code and found them done; their acceptance is now proved with the keyboard in
     Chromium and WebKit (a text page reviewed through its readings, an analysis's names renamed
     and removed, a failure that stays, Back to where a review was opened from, Ctrl F, the
     remembered choices, a window of 640 × 560), and the focus no longer falls to nothing when
     a failure is dismissed or the release line hidden. Left, with the first run on a Mac
     (item 3): all of it tried in WebKit on a Mac, where Tab skips buttons unless the system's
     keyboard navigation is on, and with VoiceOver and Narrator.
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
   drop listener left registered after a screen closed was fixed. A
   [round on 14 November](decisions/2026-11-14-checkpoint-leftovers.md) took up the nine things
   it listed: the file pickers' extensions sent from Rust, in one table the engine reads too
   (correcting and translating a text now offer `.csv`, and a dropped `.xlsx`, which was never
   read, is refused when dropped); FFmpeg's, llama.cpp's and the Whisper helper's shared
   helpers; one helper for the tests that write files for the interface; `timed_lines` in core;
   `tauri.ts`, `mock.ts`, `vision.rs`, `llm.rs` and `llama.rs` split; and one sort order for the
   reader, in the interface language. A
   [second round on 15 November](decisions/2026-11-15-checkpoint-rest.md) took up the rest: New
   job, the review, a project and a job's set-ups split into smaller files; twelve more screens'
   commands through `Action`, one `ContextMenu` and one `FindBox` for the two language pickers;
   Tesseract's engines pooled, their language data deleted at once, and the resolution a PDF
   page was drawn at recorded and used by ALTO; and Whisper's voice detection from an accented
   data folder on Windows. Left: from 4 November, the folders out of reach shown at launch (the
   Mac App Store build's, with item 3); a transcription from an accented folder tried in the
   packaged app. What the round saw and left (an empty reading's spaces, Escape in the
   language pickers, Home and End in a name's menu) was done on 15 November.
3. **A first run on a real Mac, the rest.** Before 1.6.0 the sandboxed Mac App Store build was
   opened on one Mac with Apple Silicon (macOS 26): the application menu, ⌘, and ⌘Q, and the
   menu-bar icon were tried and fixed there (#160), and the jobs of the App Store screenshots run
   ([decision](decisions/2026-11-02-macos.md)). Not yet written down as tried: the disk image
   through Gatekeeper, the Keychain's prompts, the Dock, YouTube's player in WKWebView, a Whisper
   job, the interface in WebKit at the sizes CI does not try, Tab and VoiceOver (item 1), and
   what 1.7.0 adds.
4. **Economy mode's loose ends**, a shipped feature: reading OpenAI's batch answers in a live run
   (on 3 November it had answered none after 35 minutes); a live run of the deleting of a
   batch's files, done on 15 November, which must show whether Mistral names its inline
   requests' file (951 KB for two pages) among the job's; a live run of the deleting of a Claude
   batch, done on 17 November; a live run of a Gemini batch through a JSON Lines file, done on
   18 November from Google's documentation alone, which must show whether Google lets its
   answers' file be deleted and whether the batch names its input file; the Gemini batch
   itself deleted at Google, as Claude's is; then several
   pages per call in economy mode, and Voxtral transcription in batches once its format is
   documented ([live runs](decisions/2026-11-03-live-runs.md)).
5. **Source-linked review, what is left of
   [#84](https://github.com/fmadore/Kalamu/issues/84)**, the first two steps being done (the
   second on 18 November: stable region identities with their granularity and origin, regions
   drawn or adjusted by the researcher in a layer of their own, a region read again by a
   model, and the researcher's regions in the analysis, contents and records reviews;
   [decision](decisions/2026-11-18-drawn-regions-region-read-again.md); the researcher's
   boxes and regions in ALTO, PAGE XML and the searchable PDF since the
   [checkpoint that followed](decisions/2026-11-18-checkpoint-requests-exports.md)). Left:
   the review drawing a moved block's estimated lines within its new box, as the exports do;
   the reading's identifier in the XML files; an answer of an analysis, an entry of a table of contents or a row of records tied to the
   regions it was read from (with item 6 and #47); a drawn region kept with the document
   across jobs rather than with one job; and the third step, imported ALTO and PAGE XML and a model's text aligned to OCR positions.
6. **Records, the second increment ([#171](https://github.com/fmadore/Kalamu/issues/171))**,
   the first being done: entries that run over a page (with #54), where each row sits (with
   item 5), rows across a project, a cut-off page read again in halves, error rates by row, and the
   text formats, which show only each page's first column.
   Before it, a records job on real directories and registers, and one in economy mode.
7. **What is left of the research workflows**, below.

### What is left of the research workflows

The [review of 20 November](decisions/2026-11-20-review-for-researchers-new-to-computing.md)
put four of them first, for what they give a historian, a sociologist or an anthropologist:

1. **Interview analysis, the second slice**
   ([#38](https://github.com/fmadore/Kalamu/issues/38)): windows of turns and a
   consolidation, since an interview runs from an hour and a quarter to two and a half and a
   model on the computer stops near an hour; the researcher's own themes or interview guide
   given before the run; a quotation added by the researcher from a passage of the
   transcript.
2. **Research-question screening** ([#51](https://github.com/fmadore/Kalamu/issues/51)):
   which of 400 pages or 150 articles speak of the question, why, and with which passage.
   The field that cites the text and its check, built for #38, are its foundation.
3. **Photographs from the archive**: persistent image preparation
   ([#53](https://github.com/fmadore/Kalamu/issues/53): turn, crop, move a spread's fold), an
   analysis of a document made of photographs
   ([#52](https://github.com/fmadore/Kalamu/issues/52)), and duplicate or blurred
   photographs flagged before a job is sent
   ([#216](https://github.com/fmadore/Kalamu/issues/216)).
4. **Entries that run over a page and the neighbouring pages' context**
   ([#171](https://github.com/fmadore/Kalamu/issues/171),
   [#54](https://github.com/fmadore/Kalamu/issues/54)): "the 14th of the said month" read
   with the page before it, what is written told apart from what is inferred.

Of these, written on 20 November by agents in worktrees gathered on one branch, and none yet
tried with a real model, on a real Mac or on real photographs:

- **#38, the second slice, whole**: a transcript too long for its model is read in parts of
  whole turns and brought together by one more request that names the parts' quotations by
  number, so every quotation is still the transcript's words; a part not read is named and
  read again alone; the researcher's themes or guide under *More choices*, each theme saying
  where it comes from; a quotation added from a passage selected in the transcript
  ([decision](decisions/2026-11-20-interview-windows-guide-themes.md)). To try: a
  two-hour interview with a cloud model and with one on the computer, the share of the
  model's limit a part takes (seven tenths) and the two turns repeated.
- **#51, a first increment**: one dialog and one box from a selection of a project's
  documents; relevance, a reason and up to three passages found word for word with their
  page; the table filtered by any analysis's answer, kept or set aside as tags, exported as a
  workbook and CSV ([decision](decisions/2026-11-20-research-question-screening.md)).
  Left: a document too long for the model still refuses the job (the parts built for #38 are
  the way), the question kept for the next documents, the answer shown in the table's rows.
- **#216**: New job compares its photographs on the computer and says which show the same
  page twice and which are blurred, the sharper one suggested, nothing left out unless the
  researcher chooses
  ([decision](decisions/2026-11-20-duplicate-and-blurred-photographs.md)). Its thresholds
  were set on retakes made from two sample pages: a real batch from a reading room decides
  whether they hold. About a minute for 300 photographs, read again whole when some are
  added.

A second round the same day, gathered the same way and as little tried:

- **#51, a second increment**: a long document is read in parts as a long interview is, told
  of passages and pages, and one a model cannot read even so is left out and named while the
  others are read; with an answer filter on, each row shows its answer, reason and passages,
  with Keep and Set aside; the question is asked again of the documents added since, which
  alone are paid
  ([decision](decisions/2026-11-20-screening-long-documents-rows-question.md)). Left: New
  job offers the screening prompt with no box for the question. To try: whether a real model
  follows the prompt's rule for a document read in parts (relevant if any part is).
- **#215**: New job offers, for an extraction of a document of more than 15 pages, to try
  three pages spread through it, alone or with two models; the sample's page runs the rest
  with either model, the sample's pages taken from the cache, or compares another model on
  the same pages ([decision](decisions/2026-11-20-sample-first.md)). Left: a folder of
  separate images, the diagram that still shows pages 1 to 3, economy mode with a sample.
- **#213, a first increment**: a project saved as one `.kalamu` file (SQLite, this project's
  rows only, the original files only when chosen, never a key) and opened on any computer
  as a new project beside the others ([decision](decisions/2026-11-20-project-file.md)).
  Left: pointing a project opened without its originals at the files on the new computer,
  merging a colleague's corrections, workflow runs, a file association. To try before
  anything else: a real file carried from Windows to a Mac and back. *Check a project file…*
  ([#246](https://github.com/fmadore/Kalamu/issues/246)) is written: it reads a file without
  opening it, checks each carried file's fingerprint, and names the frozen texts missing, the
  documents to find again and the originals renamed on this system; a saved file is checked
  so before it takes its name ([decision](decisions/2026-11-24-check-project-file.md)); not
  yet tried on a file carried between Windows and a Mac.

Beside them, new from that review and each to be designed in a decision first:

- **Assisted pseudonymisation** ([#214](https://github.com/fmadore/Kalamu/issues/214)):
  replacements set in the project dictionary, a pseudonymised copy as a layer of its own,
  exported or sent to a cloud model that never sees the names.
- **References from Zotero** ([#217](https://github.com/fmadore/Kalamu/issues/217)): a
  bibliography file fills the source descriptions; a file only, no account.

The rest, as it stood:

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
  request. Items of a multi-page image, and of a document of images
  ([#52](https://github.com/fmadore/Kalamu/issues/52)), can be made but not yet read on their
  own.
- **Ordered image documents, what is left of
  [#52](https://github.com/fmadore/Kalamu/issues/52)**: page labels typed in New job (the
  manifest holds them already); reordering, adding or removing pages of a job already made;
  an analysis of such a document (several images in one request); a document of images added to
  a project as such before any job reads it; several pages of images sent in one call; and the
  second increment, existing transcriptions paired with their scans by name and reviewed
  without another reading
  ([decision](decisions/2026-11-11-ordered-image-documents.md)).
- **Project dictionary, what is left of
  [#48](https://github.com/fmadore/Kalamu/issues/48)**: a live run of proposed merges with
  each provider, on a field of a few hundred rows, and on one too long for one request, to
  see what a model makes of the groups of names far apart in the alphabet, done on
  18 November (the cap of ten rows to a key, and people who share a surname); names that
  differ by their vowels (Muhammad, Mohamed) or a shortened forename, which those groups
  leave out; import with conflicts previewed; an
  accepted dictionary versioned and given to later analyses as a fingerprinted input (with
  #49); and telling apart two people who share a spelling, which needs mentions anchored in
  the text (#47).
- **What to check, what is left of [#85](https://github.com/fmadore/Kalamu/issues/85)**:
  optional field rules with their version; invalid evidence anchors, which wait for answers
  that quote their evidence (an answer holds a value and a reason today, nothing placed in
  the text: with [#47](https://github.com/fmadore/Kalamu/issues/47)). The sample, the items
  kept aside and the manifest, done on 17 November, and the results set aside kept with the
  project and the seed typed by hand, done on 18 November, are still to try in the packaged
  app.
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
  a project glossary (term and translation) from the dictionary; models trained to translate,
  offline. A live run with each provider, and one of a transcript translated with its times,
  which no real model has yet answered line by line; such passages in a batch, which economy
  mode sends at once for now, at the full price where the estimate counts half; the times of a
  transcript corrected in the review, which a segment-based reading loses.
- **Project search, the rest**: the one-time preparation after updating tried on a large real
  project; German's ODER when the interface has German.
- **Archival metadata**: [#55](https://github.com/fmadore/Kalamu/issues/55) is closed;
  normalised and inferred values beside the literal ones moved to
  [#54](https://github.com/fmadore/Kalamu/issues/54), item-level rows to #50.

### Live runs still to do

The runs of 3 November, their costs against the estimates and what they changed are in the
[decision](decisions/2026-11-03-live-runs.md). Still to run: offline Whisper, and a full
Tesseract job with networking off; GLM 5.3 through a whole job (GLM 5.3 Flash ran one); GPT-6.1
Terra on handwriting and older print before it is considered as OpenAI's default; GPT-6.2 Sol
and Astra on real pages, which no run has yet read; Gemini 4 Neon against Hydrogen on
handwriting, older print and refusals; a table of
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
- **Text extraction**: PAGE XML files imported into Transkribus, eScriptorium and Aletheia,
  which no one has yet tried (a PDF page names `file.pdf#page=N` as its image); checks for
  skewed and blurry pages, with persistent image preparation
  ([#53](https://github.com/fmadore/Kalamu/issues/53)).
- **Analyses**: a column added to an `.xlsx`; the agreement table seen with three set-ups
  and more, tried so far with two.
- **Design improvements left from [issue #1](https://github.com/fmadore/Kalamu/issues/1)**,
  closed on 28 October when the name and icon were decided (its closing comment keeps these
  here): a collation table on the job's page (pieces × set-ups, the sigla as column heads),
  which needs a command of its own, since the job's page counts by file and set-up and knows
  no piece's state; a page with a correction checked and captured; the progress bar's red
  segment beside the ink, at 2.75:1 in the light theme and 1.94:1 in the dark one, its
  failures being written beside it. Done on 18 November: the job to resume in the sidebar's
  foot, the error colour checked against the hand blue and the highlighter, and the dashes,
  colons and semicolons out of the markup.

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
in [microsoft-store.md](microsoft-store.md); 1.10.0 is live, and 1.10.1 is the next update
(1.9.0 was not submitted). The addresses given in the listing move to
`kalamu.frederickmadore.com` with a submission after it
([#228](https://github.com/fmadore/Kalamu/issues/228)).

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

Done by the maintainer: the App ID, the distribution certificates, the profile, the API key and
the app in App Store Connect, with their secrets; the screenshots, taken on a Mac from the
screened material ([store-listing.md](store-listing.md#screenshots)); and the first submission,
version 1.6.0, uploaded from its tag on 11 November 2026 and still in App Review on
16 November. Next: Apple's answer, then the latest release with a *What's New* text (1.7.0, or
1.8.0 if it is tagged by then). After it: a way to grant again the folders of jobs made by
the disk image, a French listing, and optional tips through in-app purchase, the only way
Apple allows the store build to ask for support
([#113](https://github.com/fmadore/Kalamu/issues/113)).

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
| Ordered image documents and paired transcriptions | Import photographs as one document with reviewable order (first increment done: New job's choice and order, extraction and table of contents page by page, review, one result with its pages in provenance); then pair existing text with scans | [#52](https://github.com/fmadore/Kalamu/issues/52) |
| Persistent image preparation | Crop, processing rotation and adjustable spread splitting; transformation recipes preserve originals and source coordinates | [#53](https://github.com/fmadore/Kalamu/issues/53) |
| Source-linked review | Providers' existing regions drawn over the scan, each opening its text, with a page-only fallback; then stable region identities and researcher-drawn regions. Coordinates shared with #53, text anchors from #47 | [#84](https://github.com/fmadore/Kalamu/issues/84) |
| Context-aware processing | Bounded neighboring pages; then date/place proposals with evidence, literal readings and explicit inference status, shown in export profiles beside the literal value | [#54](https://github.com/fmadore/Kalamu/issues/54) |
| Archival metadata and export profiles | Supplied source descriptions and reusable local CSV/JSON mappings (done; closed). Normalised and inferred values moved to #54, item-level rows to #50 | [#55](https://github.com/fmadore/Kalamu/issues/55) |
| Translation | A task of its own over existing text and frozen results, a workflow step after extraction or transcription, the source beside the translation in review, subtitles of a translated transcript (done); then a project glossary | [#167](https://github.com/fmadore/Kalamu/issues/167) |
| Project file | A project saved as one file with its results and the researcher's layer, opened on another computer as a new project; a merge of a colleague's corrections later | [#213](https://github.com/fmadore/Kalamu/issues/213) |
| Pseudonymisation | Replacements in the project dictionary and a pseudonymised copy of a text, a layer of its own; builds on #48 and #47 | [#214](https://github.com/fmadore/Kalamu/issues/214) |
| A sample first | Three pages tried, two models compared, the rest run from the sample's page | [#215](https://github.com/fmadore/Kalamu/issues/215) |
| Duplicate and blurred photographs | Flagged in the file list before anything is sent; with #53's checks | [#216](https://github.com/fmadore/Kalamu/issues/216) |
| References from Zotero | A CSL-JSON file matched to a project's documents, filling their descriptions | [#217](https://github.com/fmadore/Kalamu/issues/217) |
| Corrected OCR with ALTO | Tesseract, then a correction that sees the page, line for line, laid back onto the extraction's words so ALTO, PAGE XML and the searchable PDF are kept; for an OCR already readable ([decision](decisions/2026-11-24-correction-keeps-alto.md)) | [#262](https://github.com/fmadore/Kalamu/issues/262) |

Scope is local research preparation, review and export. External collection-system integration
and publishing are excluded from this work. Implementation follows the architecture checkpoints
in AGENTS.md and updates the specification and user-facing documentation when features ship.

## Later

- German interface: `de.ts` beside `en.ts` and `fr.ts`.
- OpenRouter for open-weights models.
- Linking names to imported local authority records (an id from candidate records, or none,
  with a reason), building on the project dictionary in #48.
- Tables of contents of text files (a long Word report, a transcript, read in passages).
  Ordered photographs (first increment done) and item-level extraction/export are tracked in
  #52 and #50 above.
- Post-interview analysis with research profiles and evidence-linked themes/quotations:
  [#38](https://github.com/fmadore/Kalamu/issues/38), using the shared snapshot foundation.
  Designed on 19 November ([decision](decisions/2026-11-19-interview-analysis-design.md)): an
  analysis job with one new kind of field whose entries cite the transcript, each quotation
  found in the frozen text by Kalamu before it is shown; turns in snapshots first (done, with
  the answer check as a hook of the engine), then one interview read whole (done, a transcript
  too long for the model said before Start), then windows and consolidation.
- Explore assisted TEI encoding with reusable project profiles:
  [#37](https://github.com/fmadore/Kalamu/issues/37); feasibility work, not a committed feature,
  and last in line since 20 November: those Kalamu is for do not use TEI, and correcting
  markup in a plain interface is a trap.
- Importing ALTO XML from elsewhere (a digital library's) to correct it with its coordinates
  kept; Kalamu's own extractions already keep their words' places when corrected, and #262
  gives a model's correction the same.
- French and German Store listings, with the translated interface.
- macOS: the application menu's labels through the translation files.
- Projects: nested collections and saved searches, which
  [#57](https://github.com/fmadore/Kalamu/issues/57) left to follow its first delivery.
- Search by meaning, built on 18 November, still to try: a live preparation with each cloud
  provider (Gemini's answer may not say the tokens billed; Mistral's limit for one call is a
  guess), EmbeddingGemma 2 in the packaged app and on a Mac (Metal's numbers against the
  processor's, since the model's card warns of half precision), and the estimate against a
  real invoice. Then: a projection that keeps close groups apart, the map following the
  search's filters, analyses' answers, images and sound, the dictionary's labels offered
  while adding to a list answer, a pending proposal for a passage shown again when the
  review reopens. Answers and chat stay out of scope
  ([#86](https://github.com/fmadore/Kalamu/issues/86)).
- Linux x86-64: a `.deb` and an AppImage on GitHub releases, built and tested in CI, for a
  documented range of distributions; a third file in each platform layer (`platform/linux.rs`,
  `platform.ts`, `tauri.linux.conf.json`) and WebKitGTK as a third webview. Proposed, not yet
  built: [#87](https://github.com/fmadore/Kalamu/issues/87). Low in the order since
  20 November: a third webview, keyring and set of codecs would slow every release for few of
  the researchers Kalamu is for.
