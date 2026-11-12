# Specification

What Kalamu must do, and why. The [README](../README.md) is the short introduction; the
[roadmap](roadmap.md) says what is built. Keep this file in step with decisions as they are made
([decisions](decisions/)).

## Why

More and more colleagues ask how to use AI to transcribe their recordings and documents. The
existing tools work, but they assume some technical comfort:

- [zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines) runs in Google Colab: no
  installation, but files must be uploaded to Colab and the steps followed in order.
- [iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines) offers many more tasks and
  providers, but needs Python, a terminal and an `.env` file.

Kalamu packages the same workflows as an ordinary desktop app for Windows and macOS, built with
Rust and Tauri.
Install it, paste an API key, pick a file or a folder, press **Start**. Its users are
researchers, historians first, who are not technical: every screen must stay plain and
friendly, with the advanced choices folded away.

## Starting point: existing code

Kalamu does not start from scratch. Three existing projects are reviewed for each part
and their tested pieces carried over:

- **[iwac-ai-pipelines](https://github.com/fmadore/iwac-ai-pipelines)** (Python), the main
  reference for the AI side: prompts for each task, provider clients and their quirks, model
  choices, chunking of long audio and PDFs, retry handling, entity extraction and sentiment
  scales, output formats and the `provenance.json` format. These are ported to Rust, not
  rewritten from memory.
- **[zmo-ai-pipelines](https://github.com/fmadore/zmo-ai-pipelines)** (Colab notebooks), for the
  simplified workflows and default settings that already suit non-technical colleagues.
- **[Live Translation](https://github.com/fmadore/Live-translation)** (Rust, Tauri 2), for the
  app's plumbing: Tauri setup, API keys in Windows Credential Manager, whisper.cpp with model
  download and verification, x64 and ARM64 builds, and MSIX packaging. Kalamu shares code
  with it, not its look or identity. None of the three runs on macOS: Kalamu's port follows
  Tauri's own conventions there ([decision](decisions/2026-11-02-macos.md)).

Where Kalamu departs from these projects, the reason is written down in
[decisions](decisions/).

## Tasks

| Task | Input | Output |
| --- | --- | --- |
| **Extract text** (printed and handwritten) | PDF, JPEG, PNG, TIFF, or several images read as the pages of one document | Text per page, its lines kept as on the page or joined into running text |
| **Transcribe audio or video** | `.mp3`, `.wav`, `.m4a`, `.mp4`…, or a public YouTube link | Transcript with optional speakers and timestamps; speakers can be given names |
| **Analyse texts** | Text, Word, PDF, photographed pages, photographs and other images, and the results of earlier jobs | The researcher's own fields for each document: a summary, the people and places it names, its tone, what an image shows… |
| **Detailed table of contents** | A long PDF, a multi-page image or several images read as one document: a newspaper or magazine issue, an archival file scanned in one piece, a book or report | One entry per article or item, with its title, its pages and the researcher's fields |
| **Correct existing text** | A PDF's text layer, or a text or Word file from a digital library | The text corrected page by page or passage by passage, its spelling of the time kept |
| **Translate a text** | A PDF's text layer, a text or Word file, or the text of an earlier result | The text in the language chosen, page by page or passage by passage, nothing summarised or left out |

**OCR and HTR are one task.** Current vision models read print and handwriting with the same
call, so the app has a single **Extract text** task and the difference lies in the prompt:
users pick a preset, *printed or typed document*, *newspaper or magazine* or *handwritten
document or manuscript*, or write their own. There is one preset per kind of material, never
one per language or script: each reads any language, left to right or right to left (Arabic
and Ajami included), while the language and the lines are chosen in New job
([decision](decisions/2026-11-02-fewer-default-prompts.md)). Mixed material, such as a typed
report with handwritten notes, needs no separate pass.

**YouTube videos are transcribed from their link.** New job offers *Transcribe YouTube* beside
the three tasks: paste one or more watch, `youtu.be`, Shorts or live links, and the transcript
comes out as for a recording, in the same formats. Gemini fetches the video itself, so it is
never downloaded or stored, which also keeps within YouTube's terms; only models whose catalogue
entry reads YouTube links are offered (Gemini today). Only public videos can be read. A video
is identified by its id, so the same video under another form of link comes from the cache, and
`provenance.json` records its address, title and when it was read. Each video is named after its
title, which the app asks of YouTube when the link is added, and a private or removed video is
refused then. Results go to a chosen folder, since a video has no folder of its own; the folder
last chosen is offered. In the review the video sits on the dark stage like a scan, behind a
poster: YouTube's privacy-enhanced player loads only when the researcher plays, the line being
spoken is highlighted and kept in view, and clicking a line plays the video from there. Videos
whose owner forbids embedding, or an offline PC, get a card that opens the video on YouTube.

**Line breaks are a choice of the job**, offered in plain words for every text extraction:

- *As on the page*: one line per line in the source, end-of-line hyphens kept. Easiest to check
  against the scan, and the review's line numbers then match it; a word split across two lines
  is harder to find by search.
- *Running text*: lines joined into paragraphs and split words rejoined. Best for reading and
  searching.

The choice is added to whichever prompt is used, so it works with every preset, the user's own
included. Models that take no prompt (Mistral OCR) keep their own layout, which New job says.

**An analysis is a set of fields** the researcher defines, filled in for each document. Each
field has a name, a kind of answer and, in the researcher's words, what belongs in it:

- *Text*: a summary, the main subject, a date.
- *List of names or terms*: people, places, organisations, keywords, spelled as in the document,
  with an optional maximum.
- *One choice* from a fixed list: a tone (positive, neutral, negative), a relevance.
- *Several choices* from a fixed list: themes from the researcher's own vocabulary.

Any field can ask for a one-sentence reason. Ready-made analyses cover the common needs (a
summary with keywords, its first sentence short enough for a catalogue, a social-science publication's
question, argument, theory, methods, data, case and findings for a literature review, people,
places and organisations, the tone of a document, and an image's description, kind, keywords,
written text and the names that text gives), and common fields are one click away when building
one. The image description says only what can be seen: people by number, activity and dress,
never an identity guessed from a face, and no date, place or name the image does not itself
state ([#168](https://github.com/fmadore/Kalamu/issues/168)). When every source is an image,
New job chooses it for the researcher, unless they chose a prompt themselves. When the sources
include images, New job offers only models that read images, and for an analysis of a PDF only
models that read PDFs; a job that asks a model for what it cannot read (a saved pipeline or
workflow, an older window) is refused before anything is sent, for the reason each page would
have failed with ([decision](decisions/2026-11-11-translation-followups.md)). The app turns
the fields into each provider's structured output, so the answer always has exactly these
fields, and checks it again before storing it: an answer that breaks the fields is asked for
again, never stored empty. Named entity recognition and sentiment analysis are analyses, not
separate tasks.

**A detailed table of contents is read in two steps**, as iwac-ai-pipelines indexes magazine
issues. A model that reads page images lists the articles or items on each page (title, a note
such as "continued on page 5", the researcher's fields for that part); then a model that reads
text combines the pages into one entry per article or item, with every page it is on. Reading
the pages is the hard part and combining them the easy one, so the second step can run on a
smaller, cheaper model, from any provider: Gemini Pro may read and Mistral combine. New job
offers the reading provider's cheapest model by default, so one key is enough. Presets cover a
newspaper or magazine issue, an archival file item by item (kind of document, date, sender,
addressee, summary, names: an archivist's inventory) and a book, report or thesis; the entry's
fields are edited in Prompts as an analysis's are, title and pages always included
([decision](decisions/2026-10-29-contents-and-correction.md)).

**A table of contents' entries become items read on their own** ([#50](https://github.com/fmadore/Kalamu/issues/50)).
Once the entries are checked, *Items* in the review makes one item per entry: its title, its
pages in reading order and its fields, kept apart from the table of contents, which never
changes. Items are then the researcher's: renamed, given other pages (`1, 3-5, 8 (left)`, as
the file numbers its pages), joined to the next (an article and its continuation), split in
two, added and removed; entries whose pages the document does not have are reported. An item
is made of whole pages, so two items on one page each take all of it, and the list says so.
Selected items go to a new job, *Extract the text* or *Analyse*, each as a PDF of its own pages
cut from the source and frozen like corrected text: the job's results are written beside the
source under the item's name, `provenance.json` names the item, and an item given other pages
afterwards marks the job out of date without changing what it read. *Export* writes each
item's pages as a PDF with a list of the items as CSV and JSON
([decision](decisions/2026-11-02-document-units.md)). Only a PDF's items can be cut out so far.

**A result's text can go into a new round.** *Use this text in a new job* on an extraction, a
transcription, a correction or a translation, on the job's page or among a project document's
results, freezes the text as the researcher left it, corrections and speaker names included
(from a document, only that document's), and opens an analysis of it, a correction or a
translation of it, or Workflows to run a
saved workflow or an example on it: nothing is read again or exported and imported. Workflows
that start by reading scans or recordings are shown but cannot be chosen for text. The frozen copy never changes, so a correction made afterwards does not
alter what the analysis read; the analysis is marked out of date, with a way to analyse the
corrected text, whose answers are new ones beside the old
([decision](decisions/2026-10-31-reviewed-text-snapshots.md),
[#47](https://github.com/fmadore/Kalamu/issues/47)). Pages that could not be read are left
out and named in `provenance.json`, with how much of the text had been reviewed.

**Existing text is corrected, not read again.** Many researchers already have machine-read
text, often noisy: the text layer of a PDF from a digital library, or a text file downloaded
beside it. *Correct existing text* sends that text, page by page (or in passages of a few
pages for a text file without page markers), with a prompt that fixes reading errors only and
keeps the document's spelling, wording and style; its preset keeps iwac-ai-pipelines'
OCR-correction rules, those for newspapers included. The review highlights every change against
the original. A scan with
no text layer needs *Extract text* instead.

**A text is translated as a task of its own** ([#167](https://github.com/fmadore/Kalamu/issues/167),
[decision](decisions/2026-11-11-translation.md)). *Translate a text* reads what a correction reads
(a PDF's text layer, a text or Word file) and, above all, a result's text frozen through *Use
this text in a new job*: an extraction, a transcription or a correction as the researcher left
it. It sends that text page by page, or in passages, and the answer comes back as the one field
of a correction's, so every model that answers in fields and reads text can translate, cloud
or offline, with nothing new asked of providers. The job's language is the language translated
into, one only and always chosen: New job and a workflow's step ask for it, a workflow with a
translation step and no language is not saved, and a job without one is refused before
anything is sent. Its preset, *Translation for research*, translates
everything in order, summarising and leaving out nothing; keeps names of people, places and
organisations as written; gives the original beside a culturally specific term; marks an
unsure reading `[word?]`; keeps paragraphs, headings and a transcript's speaker labels and
times exactly; adds no notes of its own; and, since the source may be machine-read, translates
what is meant where a misreading is obvious and never invents. The translation follows the
layout of the text it reads, so it offers no choice of lines. It is a result of its own, never
replacing the source, written as `<name>.translated.<code>` in the formats of a correction; the
review shows each page of the source beside its translation (for a PDF, its page or, at a click,
the text the model read), which is corrected as any text is,
with no differences view between two languages. Its text yields a text: it can be analysed, or
read by a later workflow step. Subtitles of a translated transcript come later.

## Choosing and starting a job

- **One file or a whole folder.** Choose files from the PC or drop a folder; the app processes
  them one by one, shows progress, and picks up where it stopped after an interruption. Files
  the task chosen does not read, and folders holding none it reads, are named rather than
  dropped unseen, with the task that reads them offered instead.
- **Several images, one document** ([#52](https://github.com/fmadore/Kalamu/issues/52),
  [decision](decisions/2026-11-11-ordered-image-documents.md)). With two images or more, an
  extraction or a table of contents asks whether they are separate documents (as before) or the
  pages of one: the photographs of a letter, a register, an archival file. As one, the document
  is named (the folder's name unless changed; the name of its result files, kept to what Windows
  and macOS both accept) and its pages are put in the natural order of their names: numbers read
  as numbers (page 2 before page 10, `p001` with `p9`), capitals ignored, spaces, `_`, `-` and
  `.` taken alike, digits of other scripts compared as letters. Numbers that skip or repeat in
  the names are pointed out beside their page, never fixed. Pages move with buttons, which the
  keyboard reaches, or by dragging. Each page keeps its identity, its file's path and content,
  apart from its place: it is read under its own file's fingerprint, so a page moved, or read
  alone in another job, is never read again, while a table of contents combines its pages again
  in their new order. The order and the files are kept with the job, after a restart too; the
  images are never copied, moved or changed, and a page whose file was moved, deleted or
  replaced since fails alone, named, the others read. The document is read page by page as a
  PDF is, with the same sample of first pages; its pages go one per call. An analysis reads
  each image as a document of its own. Typing a label for each page, reordering a job already
  made and pairing existing transcriptions with the scans come later.
- **Choice of provider.** Google Gemini, OpenAI, Anthropic Claude, Mistral, offline Whisper,
  Tesseract, OCR models and language models,
  and later OpenRouter for open-weights models. The app shows only the providers and models
  that can handle the chosen task: Claude, for instance, reads documents but does not accept
  audio.
- **Language choice with auto-detect.** Extraction and transcription list only the languages
  the chosen model supports, with **Auto-detect** as the default; several can be chosen for
  material that mixes languages (French and Mooré, say). Tesseract offers no Auto-detect: its
  languages are chosen, and their data downloaded once in Settings › Offline models. Where a provider publishes no list
  (GPT Transcribe, Voxtral), every language is offered and the model's description says so.
  New job says when a model's API takes fewer than were chosen: Whisper, Voxtral and OpenAI's
  diarizing model take one, so with several they detect the language; Voxtral takes none when
  it labels speakers or times the lines; OpenAI's speech-to-text takes only two-letter codes,
  so Mooré is not passed on. The catalogue records this per model (`language_hints`,
  `two_letter_codes`). Analyses and tables of contents have
  an answer language instead, by default the document's own and then said outright so that a
  model reading English instructions does not drift into English; names and titles stay as the
  document spells them, and a table of contents' entries are combined in one language. A
  translation's language is the one it translates into, and has no default: it is chosen.
- **Pages per call.** A text extraction sends a PDF one page at a time, or several pages, or
  as many as the model can write in one answer, so it sees an article that runs over the next
  page. Each page is still reviewed, corrected and exported on its own: the model marks where
  each page begins, and a page it leaves out or cuts short is read again alone
  ([decision](decisions/2026-11-01-pages-per-call.md)). A prompt marked *Pages full of text*
  (the newspaper default, and any the researcher marks) sends fewer pages together.
- **How much the model thinks.** For a model that reasons, *Thinking* offers the levels its
  provider documents, set to a suggestion: the lowest for reading pages and recordings, `low`
  for analyses. More thinking can help a hard hand or an analysis, and costs more. Every answer
  may use the whole output the model is documented to give
  ([decision](decisions/2026-11-01-output-limits-and-reasoning.md)).
- **Current models.** The latest models for each task, updated as better ones appear (see
  [Keeping models current](#keeping-models-current)). Very expensive models carry a note that
  they are most likely more than the task needs.
- **Estimated cost before starting**, from file length and the provider's published prices.
- **Faster or cheaper.** A job runs at once, page by page, or, for large collections, through
  the provider's batch service at about half the price, with answers within a day. Both costs
  are shown before starting; offline models and those without a batch service run at once
  either way. Where providers cache a repeated prompt, the cached part is billed at its lower
  price and the cost shown is the cost charged
  ([decision](decisions/2026-10-27-economy-mode-and-prompt-caching.md)).
- **Pipelines.** A job's settings (task, models, prompts or analysis, languages, line breaks,
  formats, destination) can be saved under a name and are listed at the top of New job; one
  click fills the form for the next batch of files. A pipeline can be exported to cite or share:
  a JSON file another Kalamu imports, prompts included, and the same as Markdown for a
  methods section, with each model, the settings its provider sends (reasoning effort, output
  limit, media resolution), the full text of every prompt with the hash its results' provenance
  carries, and the fields of an analysis. The output folder is left out.
- **Workflows.** Pipelines run one after another on the same files
  ([decision](decisions/2026-11-01-workflow-runs.md), [#49](https://github.com/fmadore/Kalamu/issues/49)).
  Each step is a saved pipeline, or a prompt (one Kalamu ships or the researcher's own) on the
  model New job would offer for it; a step of one set-up can take another prompt of its task,
  and keeps its name unless it was still named after the old prompt. Each reads either the
  files or the text of an earlier extraction, transcription, correction or translation step, frozen as the researcher corrected it when the
  step starts; a step that reads one after it, or a table of contents or analysis's answers, is
  refused. A step can stop for a review: the steps that read it wait until the researcher says
  to continue. They also wait when some of its pages could not be read, until those are retried
  or the researcher continues with what was read; nothing else counts as going on. Before starting, Workflows shows what each step sends and to which provider, and
  what the steps that read the files will cost; a step that reads text costs what that text
  holds, known once it is read, and a step that runs on this PC followed by one that sends its
  text to the cloud is pointed out. Each step is an ordinary job, with its progress, files,
  review and provenance (which names the run, the workflow's version and the go-ahead given).
  Starting freezes the steps and their prompts, so saving the workflow again (a new version)
  or editing a prompt never changes a run already started; running again pays only for steps
  whose model, prompt or text changed. Corrections belong to the job they were made in, so a
  new run starts from the models' text: a step that read corrected text reads it uncorrected,
  and is paid for again (*Use this text in a new job* goes on from the corrected text). A
  step that could not start, for want of any text read, starts once its input is. A workflow is exported and imported like a pipeline,
  and a run can start from a project's selection, its jobs tied to the documents.
- **The simplest workflow needs no building.** The last section of New job, *Then*, offers to
  analyse the text once it is read (an extraction, a transcription, a correction or a translation), by default
  after the researcher has checked it; nothing is sent for the analysis before they continue.
  Ready-made examples (letters, interviews, a digital library's text, documents in another
  language read, translated and summarised) open in Workflows on the models the researcher has
  keys for, the translating one asking for the language to translate into before it is saved, and an animated drawing of the steps, the researcher's
  check among them, shows how each workflow runs.
- **Continuing happens where the researcher already is.** A job that is a workflow's step says
  so on its page and in its review, with what comes next and, when the next step waits for
  them, a *Continue* button; such jobs are listed first in the sidebar, under *For you to
  check*, and the notification says the text is ready to check rather than done.
- **Works without an API key.** An offline mode keeps the app useful without an account, a key
  or an internet connection, and keeps sensitive material on the PC.
- **Interface in English and French**, with German to follow. All interface text lives in
  translation files (`src/lib/i18n/`), so adding a language needs no code changes. The
  language is chosen in Settings › General; by default it is the first of the system's languages that
  Kalamu has, else English. Prompts, the models' answers and the files Kalamu writes keep their
  own language, and the default prompts stay in English, which the models follow best; their
  names and descriptions are translated.

## Interruptions and resuming

A folder of interviews or a 400-page scan can take hours. Closing the app, a crash, a lost
connection, a sleeping laptop or a provider's rate limit must never mean starting again, or
paying twice.

- **Work is saved in small pieces.** Long recordings are split into chunks and PDFs into pages.
  Each finished piece is stored at once in a local job database (SQLite, in the app's data
  folder), so an interruption loses at most the piece in progress.
- **Jobs survive closing the app.** On the next launch, the app lists unfinished jobs and offers
  to resume them. Closing the window during a job asks first, and can keep the job running in
  the background from the notification area (the menu bar on a Mac, where closing the window
  never quits and ⌘Q asks first too). Quitting pauses the jobs and waits for the pieces
  already sent, which are paid for, to come back and be stored, saying how many; "Quit now"
  stops waiting. A pause or cancel also ends a wait between retries, so no further attempt is
  sent.
- **Nothing is processed twice.** Results are cached by a fingerprint of the source file plus
  the model, prompt, analysis fields and settings. Re-running a folder skips what was already
  done with the same settings; changing the prompt or model reprocesses only what it affects.
  A file changed on disk since it was added to a job is refused rather than read under its
  old fingerprint, and the first answer stored for a fingerprint is kept, never replaced. A page
  of a document of images has the fingerprint of its own file, wherever it stands.
- **Batches outlive the app.** In the cheaper mode, a batch is recorded before it is sent;
  the app can be closed while the provider works, and collects the answers when it opens
  again, without sending anything twice, nor anything new: the rest of the job waits for the
  researcher to resume it.
- **Failures are retried, then reported.** Rate limits, network errors and answers that break
  an analysis's fields are retried with increasing delays; an answer refused for breaking its
  fields was billed all the same, so it counts in the job's cost and `provenance.json` lists
  it apart. A piece that still fails is marked
  as failed with a readable reason, the rest of the queue continues, and failed pieces can be
  retried later on their own. An answer that stops at the model's length limit is kept but
  marked as cut off: the job's page counts such pieces, the review warns on each, and
  `provenance.json` lists them. A piece that still fails does not keep its document from disk:
  the files are written with `[Not read: reason]` in its place, `provenance.json` lists it, and
  retrying it writes them again, whole (a table of contents, which has nothing to write without
  its combined entries, waits for all its pages).
- **Exports come from the cache.** Choosing another output format later regenerates the file
  from stored results, without calling the provider again.
- **A job's files stay at hand.** Its page lists the files it wrote that are still on disk,
  after a restart too. Each opens in the app the system has for it (JSON in Notepad on Windows,
  TextEdit on a Mac, which every computer has) or in its folder; only the job's own files can be opened this way. A job can be renamed at any
  time: its name is only a label, as the files it writes are named after their sources.

## Tips and checks for better results

Many disappointing results come from the input, not the model. The app teaches good practice
with short tips where they are relevant, and checks each file before a job starts:

| Problem | Why it matters | Suggested fix |
| --- | --- | --- |
| **Two pages on one PDF page or image** (a spread) | Models read straight across both pages, and page numbers no longer match | Split each spread into two pages |
| **Low resolution** (below about 200 dpi) | Small letters, diacritics and handwriting become guesswork | Rescan; or continue anyway |
| **Rotated or skewed pages** | Lines are read in the wrong order or skipped | Rotate or straighten automatically |
| **Blurry, dark or low-contrast photos** | Characters are misread | Retake the photo |
| **PDF that already contains text** | Extraction may not be needed | Use the existing text, or extract anyway |
| **Very long or very quiet recordings, long silences** | Transcription drifts or skips quiet speech | Even out the volume; recordings are split at pauses |
| **Auto-detect on short or mixed recordings** | Detection is unreliable on little speech | Choose the language(s) explicitly |
| **Several documents in one file, or one document over several files** (analyses) | An analysis reads each file whole, however many pages, and gives it one set of answers, so keywords and names mix or split | One file per document; a tip says so, as it cannot be checked. A file that holds many documents (an issue, an archival file) suits a detailed table of contents |
| **A scan without a text layer** (correction, translation) | There is no text to correct or translate | Extract text instead; the page is reported with that advice |

Checks never block a job: users can dismiss a warning. Fixes are applied to working copies,
never to the originals, and recorded in `provenance.json`.

Under the tasks, a short diagram shows how the chosen task splits the work and puts it back
together, so that a researcher knows what is sent and what comes back:

- **Extract text**: a PDF or images split one piece per page (double pages in two first), each
  page read by the model, joined into one text per file with its page numbers. When a chosen
  model reports where text sits on the page (Tesseract, Mistral OCR, LightOnOCR-3), its pages are drawn with their
  blocks boxed and a second result follows, joined by a plus rather than an arrow: ALTO XML with
  each word in its place, and the original PDF with an invisible text layer, for PDF files
  only. With models that return text only (the language models), a note under the reading says
  so, and that they give no searchable PDF or ALTO. The
  diagram follows the same capability in the catalogue as the formats offered, never the
  provider's name.
- **Analyse texts**: each document read whole, as one piece; the model fills in the prompt's
  fields; one row of answers per document, as in the CSV export.
- **Transcribe audio**: a recording cut at pauses into pieces of at most 20 minutes, each
  transcribed, joined with timestamps; speaker labels may differ between pieces.
- **Transcribe YouTube**: only the link leaves the PC; Gemini watches the whole video at once.
- **Detailed table of contents**: each page read for its articles or items, their titles marked
  and an article continued overleaf followed; the pages combined by the second model, named;
  a table of entries with their pages and the prompt's fields.
- **Correct existing text**: the text there is, its misreadings marked; one piece per page or
  passage; the model corrects them; every change shown against the original.
- **Translate a text**: the text there is; one piece per page or passage; the model translates
  each, leaving nothing out; each page of the text beside its translation.

Once files are added, it uses them: the first file's name, pages or length, its pieces (counted
by the engine, as for the long-recording check), the chosen model and the prompt's fields. It
plays once (about 2.5 s) when a task is chosen and again on request, never loops, and shows its
last frame at rest and under reduced motion. Its steps are a list of plain sentences, so a
screen reader reads them in order.

## Review view

Checking AI output against the source is what makes results trustworthy, so the review is the
heart of the app (see [DESIGN.md](../DESIGN.md)).

- **Source and result side by side**: the page image on a dark lightbox on the left, the text on
  the right with line numbers in the margin, page navigation, zoom (Ctrl and the wheel or a
  pinch, about the point under the pointer; a double click on a place) with the page moved by
  dragging it, and a strip of pages,
  thumbnails for scans and images, showing which are reviewed. A page of a document of images
  opens its original file from the masthead. The boundary between them can be
  dragged, and the job list folded away (Ctrl B), for more room.
- **Differences in the text.** Where another model, or the model under a correction, reads
  differently, the words are highlighted. Pointing at them shows the readings over the text,
  each named by its set-up; clicking them, or stepping through them with F or the arrows above
  the text, keeps them open, and the readings can be dragged aside to show the sentence they
  cover.
  The researcher chooses the correct reading, or types one, and it is taken into the text as a
  correction.
- **Compare**: another set-up's whole text in a third pane beside the text, its differences
  highlighted the same way; pointing at one marks it in both texts, and choosing one in either
  brings it level in the other.
- **Audio too**: a player next to the transcript, which is shown once, as the text read and
  corrected; under the player its turns (time, speaker, opening words) say where to listen
  from, and clicking one plays from it. The passage being heard is marked in the text and kept
  in view while it plays. A piece plays at once, from the recording itself (MP3, M4A,
  AAC, WAV, Ogg, Opus, FLAC, WebM, MP4); other formats are converted once and the copy kept. When a model labelled speakers, the researcher can give each
  label a name ("Speaker 2" → "Imam Ouédraogo"); names replace the labels in the review and in
  every file, and the model's own output keeps its labels.
- **Tables of contents**: the scan beside the entries, with the strip of pages below;
  clicking an entry shows its first page, turning the page brings its entries into view,
  marked, and what the first step read on that page is shown below. Entries can be corrected,
  added and removed. *Items* shows the items made from them instead, marked when on the page
  shown, each opening its first page.
- **Corrections of existing text**: the original text is a witness of its own, so every change
  the model made is highlighted and the original reading can be taken back in one click.
- **Translations**: the text translated stands in the source pane beside the translation, page
  by page, as a text file's text does; a PDF keeps its page there, and *Show the text read*
  puts the text the model was given in its place, for a translation or a correction (*Show the
  page* brings the page back); it is not a witness,
  since two languages have no differences to mark. The translation is corrected as any text is,
  and in a comparison two models' translations are compared as any two readings are.
- **Analyses**: the document beside its answers, field by field. In a comparison every value
  carries the sigla of the set-ups that gave it, so agreement reads at a glance; reasons are
  shown below; pointing at a name finds it in the document. A right click on a name in a list
  renames or removes it at once, in the corrected answers of every set-up that gave it, each
  left reviewed or not as it was.
- **Correct in place.** Corrections are kept apart from the AI output, which is never
  overwritten. Exports use the corrected text or answers, and `provenance.json` records which
  pieces were reviewed or corrected. A correction or speakers' names mark the source's files
  out of date in the job database as they are saved; the files are written a few seconds after
  the last save, before the job's files are listed, opened or exported, when the app quits and
  at the next launch, so a crash or a folder out of reach never leaves them old for good.
- **Search** (Ctrl F) through the whole document as the review shows it, corrections included,
  ignoring capitals, accents and Arabic vowel marks: every page of a scan or every piece of a
  recording, the matches counted and listed by page or by the time of their line, Enter going
  from one to the next, each shown with its words marked (and a recording playing from its
  line). In a table of contents, the search keeps the entries that hold the words; in an
  analysis, the fields whose answers or reasons hold them, marked where they stand, and in a
  text document the words are marked and counted in the document too. Nothing is sent: the
  search runs on what the app already keeps.
- **Report a problem**: a flag in every review's header, and a button in Settings › About,
  open GitHub's choice of issue forms in the browser: a bug or feedback first, and output that
  is offensive or harmful, which Microsoft Store policy 11.16 asks apps to let users flag. A
  misreading is corrected in the review, not reported. The app sends nothing itself, and the
  forms warn not to paste confidential material.
- **Mark as reviewed** (Ctrl Enter), so long jobs can be checked in several sittings and
  progress shows what has been verified, not only what has been processed.
- **Error rates after review.** Every page or part the researcher corrected, or marked as
  reviewed without changing it, is a reference: extracted text, transcriptions and corrections
  are scored against it in character and word error rates, summed over every character and
  word rather than averaged by page, runs of white space counted as one space, and a curly
  quote or apostrophe the same as a straight one (neither an error nor a highlighted
  difference). The job's
  page gives them for the job and for each document, the review's foot for the document open,
  in plain words ("1.4% of characters and 5.2% of words wrong (about 1 character in 71), on 12
  reviewed pages"). They are said to count only the errors the researcher caught: the
  reference is a correction of the model's own text.

The review opens while a job is still running, so the first pages can be checked before the
rest of the folder is processed.

## Comparison mode

Which model reads a given archive best is an empirical question. Comparison mode runs the same
documents through several set-ups, each a provider, model and prompt or analysis, and shows the
results together.

- **Try a sample first.** Comparison can run on a few pages before committing a whole folder,
  and the cost estimate shows each set-up separately.
- **Scores when a reference exists.** A page corrected or reviewed in any set-up is the
  reference for all of them, a correction before a text reviewed as it stands: each set-up's
  error rates are given side by side; without one, the highlighted differences show where
  they disagree.
- **Every version is kept**, with the set-up in the file name.
- **Built on the cache.** Adding a third model to an existing comparison only runs the new one.

## Progress

The user always sees what the app is doing, how far it is, and what it costs (the job's own
calls over all its runs, not only the current one, answers billed but refused included): a list of jobs
with their status, a bar per file and set-up, a live preview of the text as pieces arrive,
pause and cancel that keep everything finished, progress on the taskbar or Dock icon, and a
notification when a job finishes or needs attention. A job waiting for a batch says how many
pieces the provider holds and when they are due.

- **The sidebar's list of jobs** is grouped by what each needs from the researcher: *For you to
  check* (workflow steps waiting for them), *Running*, *Unfinished* (interrupted, paused or not
  yet started) and *Finished*. The first three are always listed in full. *Finished* lists the
  8 most recent, and the job open in the main area when it is older, so the one being read or
  reviewed is never hidden; its heading counts them all, and a link under the list,
  *All jobs (243)*, opens the Jobs page. Folded to a rail, the sidebar keeps that page as an
  icon. A right click on a job opens its menu: rename, add to a project, delete.
- **The Jobs page** lists every job in a table: name (with how many files and the workflow it
  is a step of), task, status, pieces, cost and date, fewer columns as the window narrows. A
  search finds a job by its name, its workflow or the name of a file it read (a YouTube
  video's title too), capitals and accents ignored; filters narrow it to a task or a status
  (*For you to check* among them), and it sorts newest or oldest first or by name. Clicking a
  row opens the job; each row's menu is the sidebar's, and rows chosen together are added to a
  project or deleted at once (a running job cannot be deleted until it is paused or
  cancelled).

## Projects

A researcher's material outlives any one job: the same letter is extracted, corrected,
analysed, compared. A **project** keeps it together
([decision](decisions/2026-10-31-project-workspaces.md), [#57](https://github.com/fmadore/Kalamu/issues/57)).

- **Documents, not files.** A document is a research item: its original, referenced where it
  lies on the PC (never copied, moved or changed), its title, and every result read from it.
  The same file at the same place is added once; two files with the same content are pointed
  out but stay two documents until the researcher merges them, since a copy and a second item
  can share their bytes. Images read as the pages of one document are one document, missing
  when any of its files is.
- **In the sidebar**, the projects are listed under *Projects*. A finished job dragged onto one
  is added to it, as its menu's *Add to a project* does.
- **Collections** group documents as the researcher likes (correspondence, a newspaper's
  issues, the interviews of one village); a document can be in several, held once. Leaving a
  collection, or deleting it, keeps the document in the project.
- **Tags** are the researcher's own labels, further filters beside the collections: a
  selection or a document is tagged and untagged, the table narrowed by a tag, and the search
  finds tags too. A tag differing only in capitals from one the project uses takes its
  spelling, so one label is never split in two; merging two documents keeps the tags of both.
  A tag is renamed across the project from the table's tag filter; renamed onto one in use, the
  two become one.
- **A source's description** is the researcher's, as an archive or library gives it: the
  repository, the fonds or collection, the series, the box or folder, the shelfmark, the
  creator, the date as written, a web address, how to cite it and its rights. The project holds
  the parts its documents share (the repository, the fonds) and each document its own, a part
  it leaves empty taking the project's; the document's page shows which is which. Written by
  the researcher only: no model reads or writes it and no result changes it. *Export
  descriptions* writes the documents shown (the project's or a collection's) as JSON and CSV,
  each value beside where it came from (`document` or `project`), with the documents' ids,
  titles, files, collections and tags
  ([decision](decisions/2026-11-03-source-descriptions.md), [#55](https://github.com/fmadore/Kalamu/issues/55)).
  Each result's `provenance.json`, and the `.json` export that holds it, names the project
  documents its source is, with their titles and descriptions as they stand. When a title, a
  description, the project's defaults or name, or a tie between a job and a document changes,
  those two files are written again for the results already written, without reading anything
  again; the text, Word, subtitle and CSV files are left as they are
  ([decision](decisions/2026-11-03-descriptions-in-provenance.md)).
- **Export profiles** write the same descriptions in the researcher's own columns, for a
  spreadsheet or another tool's import: each column a heading and where its value comes from
  (the document's id, title, file, project, collections or tags, a part of the description,
  where that part came from, or a fixed text), in the researcher's order, with how lists are
  joined, what an empty cell holds and whether line breaks become spaces. A profile is checked
  as it is edited (no heading twice, whatever its capitals; each on one line), saved for every
  project, duplicated, shared as a template without its fixed texts unless asked, and imported.
  Starting points: every column, or Dublin Core terms. CSV follows the profile; JSON keeps
  lists as arrays and empty values as `null`. Exporting again never calls a model
  ([decision](decisions/2026-11-03-export-profiles.md)). A column can also read the document's
  result for a task: how far it was reviewed (`reviewed`, `partly_reviewed`, `not_reviewed`),
  the model that read it, its text as corrected (for a task that writes one), or an analysis's
  answer to a field, named by its key (the project's fields offered), a list staying a list,
  either as the model gave it or under the project dictionary's labels: each form an entry
  holds written under its label, in that field's entries only, a form no entry holds as
  written, a label reached twice written once; the answer itself is not changed
  ([decision](decisions/2026-11-03-profile-dictionary-labels.md)). The result read is the one
  the researcher preferred for the task or, where none is preferred, the only one; with several
  and none preferred the cell is left empty, never chosen for the researcher
  ([decision](decisions/2026-11-03-profile-result-columns.md)).
- **Every result of a document together**, whatever job made it: each set-up of each job that
  read it, with its progress, how much was reviewed and corrected, what it cost, opening its
  review. The researcher can mark one per task as **preferred**: never chosen by the app, and
  a newer run does not take its place.
- **Adding work.** Files and folders are added before anything is read. A selection opens New
  job, and the job is tied to those documents when it starts. A job run before is added after
  its files are matched to documents, shown for the researcher to confirm: the same file is
  proposed, the same content elsewhere only offered. Its results come as they are; nothing is
  sent again.
- **Activity**: the jobs that read the project's documents, with their controls as in Jobs,
  narrowed by task, state, model, workflow and date started; the jobs shown are costed
  together, and those without a price yet are counted apart.
- **Workflows on a selection**: a selection can run a workflow; every step's job is tied to
  the documents it read, and the run keeps the files it started with.
- **What to check**: one list, for the project, a collection or a selection, of the pages and
  answers that need the researcher, each with its reason
  ([decision](decisions/2026-11-03-what-to-check.md), [#85](https://github.com/fmadore/Kalamu/issues/85)).
  Pieces that failed or are not read yet come first, apart from the answers; an answer is
  listed when it was cut off at the model's limit, when another set-up of its job read it
  differently (words in a text; names, choices or entries in an analysis or a table of
  contents, never prose), when it read a text corrected since (or an item of a table of contents since given other
  pages), and while it is not reviewed.
  Where the researcher preferred a result, only it is read; elsewhere every result is, listed
  so that some can be set aside, and a document read by several models is still one document.
  Each item names its document, page, set-up and stored answer, and opens the review at its
  page, from which the previous and next items are a click away. The list is read from what
  is stored, so it calls no model. Only the researcher's review resolves an item, which then
  stays listed as reviewed; agreement between models is never taken for one, and an answer
  read from a changed text stays open. Counts of documents, pages read, failed, reviewed and
  flagged are given over everything in scope, whatever the filters show.
- **Organising never changes work.** Collections, titles, links and preferences call no model.
  A job keeps the files it was started with, so a later change to a collection does not rewrite
  a past run or start new work. Deleting a project, a collection or a document removes only how
  it was organised; deleting a job unties it from its documents, which stay. A file moved since
  it was added is marked missing, its results kept. A project can be archived and restored.
- **Search through every document's text**: see [Project search](#project-search).

### Project dictionary

The names and terms a project's analyses listed, gathered for the researcher to reconcile
([#48](https://github.com/fmadore/Kalamu/issues/48)).

- **Read from the results, never written into them.** The dictionary is gathered from the list
  fields (people, places, keywords) of every analysis tied to the project's documents, through
  the researcher's corrected answers where there are some. It maps the forms the models wrote,
  kept as written, to entries; the answers themselves are never changed.
- **One field, one namespace.** People are never mixed with places, even when spelled alike;
  fields of the same key in different analyses share one namespace.
- **Entries are the researcher's.** An entry gathers forms under a preferred label of the
  researcher's choosing, with a stable id. Forms are merged, taken out again, relabelled,
  marked reviewed and **kept apart**; each decision is kept, survives a restart and is undone
  by its opposite. A form no entry holds is its own, not yet reviewed.
- **Look-alikes are pointed out, never merged.** Forms that read the same once capitals,
  accents, punctuation and word order are set aside are shown as possibly the same, as
  iwac-ai-pipelines folds names before matching them; the same spelling is not taken as
  evidence that two mentions are one person, and pairs kept apart are not offered again.
- **Documents, not mentions.** A list answer says that a document names someone, not where, so
  the dictionary counts documents, and how many of the results that read them listed the form:
  two models listing a name is agreement, not a second mention.
- **Coverage and stale evidence.** The view says from how many of the documents in scope it was
  gathered, and marks an entry none of whose forms is listed any longer. It covers the whole
  project or one collection, and each row opens the results that listed it.
- **Export** of the entries as JSON (ids, labels, forms as written, review status, pairs kept
  apart) and of every form found as CSV, beside the entry and label it maps to.
- **Merges proposed by a model.** For one field, the researcher chooses a provider and a model
  and sees the estimate; the rows go as an analysis job of its own, through the engine, so it
  is cached, resumed, costed and recorded in provenance as any job, in economy mode where the
  model has one. Each row goes as a reference (`R1`) with its label, its other spellings, the
  titles of up to three documents naming it, and the pairs kept apart. The whole field goes in
  one request up to about 100,000 tokens of input (two to three thousand rows), which every
  model offered for analyses reads. A larger field is cut into requests of that size in
  alphabetical order (accents and capitals set aside), each sharing a tenth of its rows with the
  next, and the interface says so before sending: names far apart in the alphabet are then not
  compared. The prompt asks for groups with a preferred label, a reason and a certainty in
  words (certain, likely, possible), which is not a probability. It is listed in Prompts under
  Dictionary, linked from the request, and can be edited and reset like any default; its
  answer's fields are fixed, since the dictionary reads them, and New job never offers it.
  The job writes no export, since the dictionary reads its answers from the store, and keeps
  its `provenance.json` in the app's data folder.
- **Answers checked, nothing applied.** References to no row sent are counted and left out; a
  group of one row, or of rows already one entry, is no proposal; rows the researcher kept
  apart are never offered together, even if the model groups them. Each row is kept by its
  forms, so a proposal still finds its rows after other merges. The researcher accepts a
  proposal through the ordinary merge, with some of its rows left out or its label changed,
  keeps two apart, or turns it down; a proposal turned down stays listed as such, survives a
  restart and can be taken back. The same proposal from two requests is offered once.
- **Later**: import; an accepted dictionary as a fingerprinted input to later analyses;
  comparing names far apart in the alphabet (by sound or by shared words); and telling apart
  two people who share a spelling, which needs mentions anchored in the text (#47).

### Project search

Every passage of a project's documents that holds some words, found without opening each
document and opened where it stands
([decision](decisions/2026-11-03-project-search.md), [second delivery](decisions/2026-11-03-search-words-and-preparation.md),
[#86](https://github.com/fmadore/Kalamu/issues/86)).

- **The text as the researcher corrected it.** Each piece is searched as the review shows it:
  a correction in place of the model's text, speakers under the names given them. Matching is
  the review's (capitals, accents and Arabic vowel marks set aside, line breaks as spaces),
  found anywhere in the text, inside words too.
- **Three ways to read the words.** The exact phrase, three letters at least (the default);
  all the words anywhere in the same page, part or answer, each word's places listed; or the
  words with at most a chosen number of words between the first and the last (10 unless
  changed), each such stretch one match with only the words looked for marked. In the last
  two, words between quotes (straight, curly or « ») are one term, and words shorter than three
  letters are left out and named.
- **Read text apart from written text.** Extractions, transcriptions and corrections are
  searched; analyses' answers and reasons and tables of contents' entries, which models wrote
  about the documents, only on request, each match naming its field.
- **Which readings.** By default the preferred result of each task where the researcher chose
  one, and every result where none is preferred, which the view counts; or every reading. Two
  readings of a document stay two results of one document.
- **Filters**: a collection, a tag, a task, pieces reviewed or not yet reviewed.
- **Each match** shows its document, its page, passage or the time of its line, the words
  around it, the result it is in (task, model, job), and whether it was corrected, reviewed,
  preferred, or is an analysis out of date. Clicking it opens the review on that place with its
  words in the review's search (for several words, the word found, or the first of those near
  each other), a recording playing from its line, another reading in the compare pane, an
  analysis at its field, a table of contents at its entry and that entry's first page. A match
  moved by a correction since is reported, never replaced silently.
- **Coverage**: the documents searched out of those in scope, those with nothing read yet, the
  pages or parts still to read or failed; finding nothing is said not to mean absence.
- **Local and derived.** Nothing is sent. The text is kept, folded for matching, in an index
  beside the job database, brought up to date with the results, corrections, preferences and
  links before each search, and made again from them when it is lost or damaged: it holds
  nothing they do not. Opening Search prepares the text of every result tied to the project in
  the background, its progress shown, so that the first search need not wait; a search begun
  meanwhile takes over from where it got to.

## Export formats

Results are stored once and exported to several formats at the same time; the default selection
is remembered per task. Details are in [formats.md](formats.md).

| Task | Formats |
| --- | --- |
| **Text extraction** | `.txt`, `.md`, `.docx`, `.json`, `.csv`; a searchable PDF (invisible text layer over the original, from PDF sources) and ALTO XML (each block, line and word with its place) only from models that report where text sits on the page: Tesseract, word by word, and Mistral OCR and LightOnOCR-3, by block (in ALTO their lines and words are then estimated); a corrected page keeps its words' places; PAGE XML later |
| **Transcription** | `.txt`, `.md`, `.docx`, `.json` (segments with speakers and timestamps), `.srt`, `.vtt`, `.csv` |
| **Analysis** | `.txt`, `.md`, `.docx`, `.json` (answers by field, with reasons), `.csv` (one row per document, one column per field); later a column added to an `.xlsx` |
| **Detailed table of contents** | `.md` and `.docx` (a heading per entry), `.csv` (one row per entry, a column per field), `.txt`, `.json`; its items' pages as one PDF each, with a list of them as `.csv` and `.json` |
| **Correction** | As text extraction, without the searchable PDF or ALTO XML |
| **Translation** | As a correction; subtitles of a translated transcript later |

## Prompts and analyses

- **Defaults per task**, ported and tested from `zmo-ai-pipelines` and `iwac-ai-pipelines`.
- **Edit, save and name** a prompt or an analysis, duplicate one as a starting point, and reset
  any default. They are stored locally and kept across updates.
- **Write a new one** with *New prompt*: the researcher chooses the task it belongs to, which
  New job then offers it for, and writes its instructions; its request, and an analysis's or a
  table of contents' fields, start from that task's default. New job's help on *Prompt* says
  where the prompts are kept and links to them.
- **Variables** `{languages}` and `{speakers}` are filled from the job's settings, so one prompt
  works across projects.
- **Pages full of text.** A text extraction prompt can say its pages are dense (the newspaper
  default does), so fewer go together when a job reads several pages in one call. The hint is
  the prompt's, never read from its name, and travels with it when it is copied or shared.
- **Share** prompts and analyses by exporting and importing them as small JSON files.
- **Recorded in provenance.** Each output's `provenance.json` stores the full prompt text and
  the analysis's fields, so results stay reproducible after a preset is edited.

## Keeping models current

- **A model catalogue as data, not code**: each provider's models, tasks, languages, limits and
  prices in a catalogue file the app refreshes from this repository without a new release. What
  shapes a model's request (whether it takes a prompt, labels speakers, has a server-side
  fallback on a refusal) is a field of its entry, never read from its name, so a new model is
  a new entry ([decision](decisions/2026-11-10-model-abilities.md)).
- **A recommended model per task**, reviewed whenever a provider releases something better.
- **A tier for every model**, where it stands among its provider's own as the provider presents
  it: most capable, balanced or lightweight; an offline model among its own family. New job
  lists models by provider (a provider field, then its models under their tiers) or by tier (one
  list of every provider's models, most capable first, those on this computer apart), as the
  researcher last chose, and marks the chosen model's tier beside its description with three
  bars. A tier compares a provider's models with each other, never with another provider's
  ([decision](decisions/2026-11-12-gemini-4-hydrogen.md)).
- **Only the newest model of each line is offered.** When a provider names a newer model as the
  replacement of an older one (Claude Fable 5.2 for 5.1), the older entry stays in the catalogue
  with a `successor` and is no longer offered for new work; models of separate lines (Gemini 4
  Carbon and Argon, Claude Opus and Sonnet) are all offered. A pipeline, comparison, workflow or
  job that names a replaced model keeps it, shows its name, and runs it while its provider
  serves it, so its results stay comparable; New job marks it and offers its replacement in one
  click, never switching on its own. A model its provider no longer serves is marked `retired`:
  a job naming it is refused before anything is sent. No entry is ever removed, and an id the
  catalogue corrected is kept as an alias of the model's id now
  ([decision](decisions/2026-11-11-model-successors.md)).
- **The exact model version is recorded** in `provenance.json`, with the settings each request
  carried (reasoning effort, output limit, media resolution), kept with each result when it
  was sent so that an update of the app never rewrites them.

## Offline mode

- **Audio transcription with Whisper** (whisper.cpp), with models downloaded once from Hugging
  Face from Settings and verified before use. It runs on the graphics processor: through Metal
  on a Mac, and on Windows through Vulkan in a helper program shipped beside the app, when it
  finds a usable graphics processor and Settings allows it; otherwise on the processor, and a
  recording that fails on the graphics processor is transcribed again on the processor
  ([decision](decisions/2026-11-08-whisper-graphics-processor.md)). Where the voice-activity
  model is downloaded, only each window's speech is decoded, its times mapped back to the
  recording. Each window of about two minutes is decoded in the language chosen, or in the
  language detected on its loudest thirty seconds of speech, so a recording that changes
  language is followed. Lines are kept to subtitle length, cut between words at
  the times Whisper gives each word; the text joins them into one sentence a line. New job
  takes a list of names and words to spell (Whisper's prompt, fitted to the model's own
  tokenizer; lines that only read it back are dropped), which a job started from
  a project can fill from the project's dictionary, and for the models trained to translate
  (not Turbo), can have the speech written in English. Whisper's loops (a word or a line said
  over and over, subtitle credits written over silence) are cut back, and lines it was unsure of
  in a window it judged silent are dropped. Pausing stops a transcription within one pass of
  the model. Provenance records each of these settings and the device.
- **Text extraction with Tesseract** (Tesseract 5, built into the app), for clean print: PDFs
  and images read page by page on the computer, each word with its place on the page, so ALTO
  and a searchable PDF come from it too. Its language data is downloaded once per language from
  Settings, from a pinned revision of Tesseract's own `tessdata_best`, and verified before use;
  Settings lists the languages downloaded and finds the others by name in a search box;
  languages written in Latin letters that Tesseract has no model for (Hausa, Mooré, Wolof,
  Bambara…) read with its Latin script model. Tesseract does not detect the language, so New
  job asks for it. Its speed depends on the computer's processor, from about a second to
  several seconds a page, and Settings and the model's description say so. Handwriting presets
  are offered with cloud models only
  ([decision](decisions/2026-11-04-tesseract.md)).
- **Text extraction with small OCR vision models** (LightOnOCR-3 1B, GLM-OCR), for print
  Tesseract reads badly: degraded type, old newspapers in columns, mixed layouts, tables. Each
  model and its image encoder are downloaded once from Settings, from a pinned revision on
  Hugging Face (about 1.1 and 1.4 GB), and verified before use. They run in llama.cpp's own
  program, shipped next to the app rather than linked into it, one page at a time: on the
  processor, from tens of seconds to over a minute a page, or on the graphics processor,
  several times faster: through Metal on a Mac, and on Windows through Vulkan with a second
  build of the program, used when it finds a graphics processor with memory enough (a discrete
  card before integrated graphics), and on Snapdragon's Adreno, where Vulkan reads wrongly,
  through OpenCL for GLM-OCR's image encoder alone, the only model and part measured correct
  there. Settings names that graphics processor and can keep the
  models on the processor; a page that fails on the graphics processor is read again on the
  processor, which the rest of the session keeps to. Provenance records the device that read
  each page (`cpu`, `metal`, or `vulkan` or `opencl` with the device's name, and for OpenCL
  that it ran the image encoder only). Settings and New job show the
  seconds a page took on this computer, on the device the next page goes to, once it has read
  some, rather than a promise. They take a fixed
  instruction, no prompt and no language; LightOnOCR-3 places each block of text on the page,
  which gives ALTO and a searchable PDF by block, while GLM-OCR returns the text alone. Like
  any language model they can write text that is not on the page, which their descriptions
  say: compare doubtful pages with Tesseract or a cloud model. A page whose answer reaches the
  output limit is marked as cut short, and a line repeated over and over at its end is kept
  once. Pausing ends the page being read, which is read again on resuming
  ([decision](decisions/2026-11-06-ocr-vision-models.md),
  [#105](https://github.com/fmadore/Kalamu/issues/105)).

- **Analyses, tables of contents and corrections with small language models** (Gemma 4 E4B
  and E2B, from Google), for material that must not leave the computer. Each model and its
  encoder are downloaded once from Settings, from a pinned revision of Google's own repository
  on Hugging Face (about 6.1 and 4.3 GB), and verified before use. They run in llama.cpp's
  programs, shared with the OCR models, which they take turns with: a text through the text
  program, which reads the instructions, the text and the fields from files; a photographed page
  or a page of an issue through the multimodal program. The answer is held to the analysis's
  fields as it is written (llama.cpp turns their schema into a grammar), then checked as every
  provider's is; decoding is greedy, thinking off. They read a text of up to about 11,000 words
  at once (a context of 32K tokens) and say so when one is longer; a PDF is analysed once its
  text is extracted. On a Mac they run on Metal; on Windows they follow the OCR models' graphics
  processor and Settings choice, with as many of their layers there as its memory holds and the
  rest on the processor, so a card with little memory of its own still helps. Snapdragon's
  Adreno runs them on the processor until OpenCL is measured correct for them. They are far
  weaker than the cloud models, which their descriptions say
  ([decision](decisions/2026-11-07-offline-language-models.md)).

Offline results are a fallback, not the default for difficult material: the app says which
engine produced a result and records it like any other.

## Privacy

In offline mode, nothing leaves the computer. With a cloud provider, files go directly from the
computer to
that provider; there is no intermediate server. For a YouTube video only its link is sent, and
Google reads the video from YouTube; the app also asks YouTube for the video's title when the
link is added, and loads YouTube's player in the review only when the video is played. API keys
stay in the system's keychain: Credential Manager on Windows, the Keychain on macOS.
The developer receives no files, keys, results or telemetry.

Before the first job, workflow run or dictionary proposal sent to a cloud provider, the app asks
once for that provider: it names it, says what is sent (the pages, recordings or text, with any
personal information they hold), that it goes under the researcher's own account and the
provider's terms, and links the privacy policy. Nothing is sent until the researcher agrees,
which the settings keep; the shell refuses work for a provider not agreed to, so no path skips
the question. Offline engines are never asked about. Apple's Guideline 5.1.2(i) asks for this in
the Mac App Store; it applies on Windows too
([decision](decisions/2026-11-04-ai-consent.md)).

Sending research material to an AI provider is still a transfer of data to a third party.
Researchers remain responsible for consent, confidentiality, copyright and any approval their
institution requires, and should prefer paid API tiers whose terms exclude training on inputs.

## What each task accepts

Which kinds of file a task reads, which models can do it and which formats it writes are the
engine's rules, given to the interface so New job offers only what will run, and checked again
when a job is created: a source the task does not read, or a model that does not do the task or
cannot read a YouTube link, is refused with a plain message, and formats the task does not
write (from a pipeline saved for another task) are left out, as are a searchable PDF and ALTO XML when no
model of the job reports where text sits on the page. Where line breaks, samples and pages per
call apply, which models are the app's and each provider's choice, and what a job will cost
at each pace are the engine's to say too; the interface only shows them.

## AI output is not ground truth

Transcriptions, extracted text and analyses can be fluent and wrong: misheard names, skipped
passages, invented words, a tone misjudged. Check a sample against the source before relying on
the results, and keep the originals.

## Glossary

Many researchers meet these tools for the first time. A glossary beside Settings explains the
words the app uses, from OCR and tokens to ground truth, error rates and hallucination, in
plain language, and what each export format suits. A small help mark beside the headings that
use a word (pipelines, workflows, the model and prompt, speed and price, the estimate, formats,
the files written, the errors found in review) opens the glossary on it. Like all interface text, it is
written in the translation files.

## Bundled conversion tools

Providers accept different inputs, so the app prepares each file for the chosen model; users
never convert anything themselves. What a model is sent is what both its provider and its own
catalogue entry accept: a text-only model behind a provider that takes PDFs and images is sent
text alone, and a scan has its text extracted first. The tools ship inside the app for Windows
(x64 and ARM64) and macOS (Apple Silicon).

- **PDF to images** with [hayro](https://github.com/LaurenzV/hayro), a PDF rasterizer written
  in Rust, page by page at 300 dpi, for engines that read images only (Tesseract, the offline
  OCR models); previews are
  drawn in the interface with pdf.js.
- **Image preparation**: multi-page TIFFs split, HEIC, WebP and BMP converted, oversized scans
  resized, orientation fixed from EXIF data.
- **Audio and video with FFmpeg**: the audio track extracted from video, files converted to a
  format the provider accepts, and long recordings split at silences with a short overlap.
- **Reading inputs** for analyses: Word, PDF, Markdown, CSV and plain text, and a photographed
  page as one image (a multi-page TIFF is refused, with a hint to save it as a PDF).

Licences matter for an MIT app in the Store. hayro is MIT or Apache 2.0; Tesseract (Apache 2.0)
and Leptonica (BSD-2-Clause) are compiled into the app, their licences in the third-party
notices, and Tesseract's language data is Apache 2.0. llama.cpp (MIT) is our own build of its
multimodal program, shipped beside the app with its licence (on Windows also its Vulkan build,
with Vulkan's headers, MIT, compiled in, and on Windows ARM64 its OpenCL build, with Khronos's
OpenCL headers, Apache 2.0; the loaders come with the graphics driver);
LightOnOCR-3 (Apache 2.0) and
GLM-OCR (MIT) are downloaded at the researcher's request. FFmpeg is our own
audio-only build, LGPL v2.1 or later, run as separate programs rather than linked; its source is
published with each release, and the Store listing carries our own licence terms because
Microsoft's standard terms forbid the reverse engineering the LGPL must allow. Details are in
[ffmpeg.md](ffmpeg.md) and [store-license-terms.md](store-license-terms.md).

## Technology

- [Tauri 2](https://tauri.app/) with a Rust back end and a Svelte front end.
- Provider calls in Rust; no bundled Python.
- SQLite for the job database and result cache, and its full-text search (FTS5) for project
  search; exporters in Rust.
- [whisper-rs](https://crates.io/crates/whisper-rs) (whisper.cpp) for offline transcription.
- [Tesseract](https://github.com/tesseract-ocr/tesseract) 5, through
  [kreuzberg-tesseract](https://crates.io/crates/kreuzberg-tesseract), for offline text
  extraction, and [hayro](https://crates.io/crates/hayro) to draw PDF pages for it.
- [llama.cpp](https://github.com/ggml-org/llama.cpp)'s `llama-mtmd-cli`, built from a pinned
  release and run as a separate program, for the offline OCR models; on Windows a second build
  of it with Vulkan for the graphics processor, and on Windows ARM64 a third with OpenCL for
  Snapdragon's Adreno.
- Native builds for Windows x64 and ARM64, and macOS on Apple Silicon. What differs between the
  systems lives in one place per layer ([AGENTS.md](../AGENTS.md)).

The crate layout, modularity rules and architecture reviews are in [AGENTS.md](../AGENTS.md).

## Distribution

The [Microsoft Store](https://apps.microsoft.com/), from version 1.0.0: Store packages are
signed, install without a security warning and update themselves, which matters for colleagues
who are not comfortable installing software. The MSIX package (x64 and ARM64 in one bundle)
carries FFmpeg and every licence, declares network access and no microphone, and keeps keys in
Credential Manager; the privacy policy is published with the listing, and every installation
lists its third-party components with their licences. Unsigned installers are also attached to
GitHub releases. The submission, the policies it must meet, the package and the listing text are
in [microsoft-store.md](microsoft-store.md), [packaging-msix.md](packaging-msix.md),
[store-listing.md](store-listing.md) and [privacy.md](privacy.md).

On macOS (Apple Silicon, macOS 14 and later), a disk image attached to each GitHub release,
with FFmpeg and every licence inside. From 1.4.0 it is signed with an Apple Developer ID and notarised, so
Gatekeeper opens it without a detour through System Settings
([decision](decisions/2026-11-04-macos-notarisation.md)). Keys are
kept in the Keychain. A Mac App Store build of the same code is being prepared: it runs in the
App Sandbox, keeps the folders and files the researcher grants as security-scoped bookmarks,
asks for a source's folder before writing results beside it, and has no support link
([feasibility](decisions/2026-11-04-mac-app-store-feasibility.md),
[build](decisions/2026-11-04-mac-app-store-build.md)). Its first submission is version 1.4.1;
the setup at Apple, the answers in App Store Connect and the steps are in
[mac-app-store.md](mac-app-store.md).
