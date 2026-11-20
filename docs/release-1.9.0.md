Kalamu 1.9.0 analyses an interview from its transcript: themes and passages worth quoting, each
backed by quotations Kalamu has found word for word in the text. A project can be searched by
meaning as well as by words, and seen as a map of its documents. In the review you can draw your
own regions over a scan, have a region or a passage read again by a model, and add what a model
missed to a list. Extractions are written as PAGE XML, and GPT-6.2 and Gemini 4 Neon are in the
model list. 1.8.1 was prepared but never published: its one fix is here. For what 1.8.0 changed,
see the [1.8.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.8.0).

## What's new since 1.8.0

- **Analyse this interview**
  ([#204](https://github.com/fmadore/Kalamu/pull/204),
  [#205](https://github.com/fmadore/Kalamu/pull/205),
  [#206](https://github.com/fmadore/Kalamu/pull/206),
  [#38](https://github.com/fmadore/Kalamu/issues/38),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-19-interview-analysis-design.md)).
  On a transcription, *Use this text in a new job* offers **Analyse this interview**: one box,
  *What is your research about?*, which may be left empty, the model and its estimated cost in
  one line, and one button. The review shows a short summary, themes, keywords and passages
  worth quoting, each theme a card with its quotations, their speaker and time; **Listen** opens
  the transcript at that turn and plays it. Nothing is shown as a quotation unless Kalamu found
  those words in the transcript: the speaker and the time are the transcript's, never the
  model's, and quotations found nowhere are left out and counted. You can edit an entry, remove
  an entry or a quotation, and undo; a quotation's words cannot be changed. What you typed about
  your research can be kept for the next interviews. A transcript too long for the model chosen
  is said before Start. The exports are a Markdown memo, a JSON file and a `.csv` with one row
  per quotation.
- **Search by meaning, and a map of the project**
  ([#199](https://github.com/fmadore/Kalamu/pull/199),
  [#86](https://github.com/fmadore/Kalamu/issues/86),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-18-search-by-meaning-map-passage-correction-added-items.md)).
  A project's Search has **Words**, still the default, and **Meaning**: passages that speak of
  what a question describes, in other words than its own. You choose the model: Gemini
  Embedding 2, OpenAI's two, Mistral Embed, or EmbeddingGemma 2 on your computer (a 310 MB
  download in Settings, nothing sent). Nothing is prepared until you start it, after an estimate
  of its cost; a cloud model asks your consent for that project first, naming what is sent: the
  corrected text now, each question afterwards. Preparing can be paused and goes on where it
  stopped. The map shows one point per document, grouped by collection or tag, with the nearest
  documents listed beside it.
- **Your own regions over a scan, and a region read again**
  ([#200](https://github.com/fmadore/Kalamu/pull/200),
  [#201](https://github.com/fmadore/Kalamu/pull/201),
  [#84](https://github.com/fmadore/Kalamu/issues/84)). With the regions shown, draw a region on
  any scan page, with the pointer or the keyboard alone, move and resize it, write what is
  there, or remove it. A model's region can be moved or resized too; its own box is kept and
  can be put back. **Read this region again…** cuts that part of the page from the original and
  sends it alone, after saying what is sent and what it costs; the answer is a proposal you
  accept or leave. ALTO, PAGE XML and the searchable PDF write your regions. Nothing new shows
  in a review until the regions are shown.
- **A passage corrected by a model, and items added to a list**
  ([#199](https://github.com/fmadore/Kalamu/pull/199)). In the review, select text, open the
  menu (right-click, the menu key, or the button that appears) and choose a model and a
  correction prompt: the answer is shown against your selection to accept, edit first or
  discard. In an analysis, add the name, keyword or theme a model missed to a list answer; it
  is marked as yours in the exports, the dictionary and the search.
- **PAGE XML** ([#197](https://github.com/fmadore/Kalamu/pull/197)). A text extraction can be
  written as PAGE XML, one file per page, for Transkribus, eScriptorium and Aletheia, where
  ALTO is offered: with Tesseract, Mistral OCR and LightOnOCR-3. Corrections are included.
- **What to check**
  ([#197](https://github.com/fmadore/Kalamu/pull/197),
  [#198](https://github.com/fmadore/Kalamu/pull/198),
  [#85](https://github.com/fmadore/Kalamu/issues/85)). Pages nothing flags can be drawn at
  random, from a seed shown and kept with the project, or typed to draw the pages a colleague
  drew. An item can be kept aside with a note, results set aside stay so after a restart, and
  *Export the list* writes the whole list as JSON and CSV.
- **Project search by words**
  ([#195](https://github.com/fmadore/Kalamu/pull/195)): `OR` (or `OU`) in capitals between two
  words or phrases finds either, and words of two letters (Ba, Sy) are found standing on their
  own. The index is made again by itself at the first search after updating.
- **How far two models agree**
  ([#195](https://github.com/fmadore/Kalamu/pull/195)). The page of an analysis job with two
  set-ups or more shows, field by field, how often they gave the same answer, with Cohen's
  kappa for a choice and the overlap of a list's items. The Glossary explains both.
- **Subtitles of a translated transcript**
  ([#195](https://github.com/fmadore/Kalamu/pull/195),
  [#167](https://github.com/fmadore/Kalamu/issues/167)). A transcript with times, translated,
  can be written as SRT and WebVTT: each line keeps its time and its speaker.
- **Project dictionary** ([#198](https://github.com/fmadore/Kalamu/pull/198),
  [#48](https://github.com/fmadore/Kalamu/issues/48)). In a long list, names far apart in the
  alphabet that share a word, initials or a sound ("Ouédraogo, Moussa" and "Moussa Ouédraogo")
  are sent together, so a model can propose merging them. The request says its cost first.
- **Economy mode** ([#197](https://github.com/fmadore/Kalamu/pull/197),
  [#198](https://github.com/fmadore/Kalamu/pull/198)). A Claude batch is deleted at Anthropic
  once its answers are stored. A Gemini batch too large for one request goes as one file, which
  is deleted at Google afterwards with the file of its answers, so a large job is no longer cut
  into small batches.
- **New models**: GPT-6.2 Sol, OpenAI's default, and GPT-6.2 Astra replace GPT-6.1 Sol and
  GPT-6 Astra ([#196](https://github.com/fmadore/Kalamu/pull/196)); Gemini 4 Neon is listed
  between Hydrogen, still Gemini's default, and Carbon
  ([#202](https://github.com/fmadore/Kalamu/pull/202)). A pipeline or workflow that names a
  replaced model keeps it, marked *replaced*.
- **A job to resume** is named at the foot of the sidebar, from any screen, with how far it got
  and **Resume** ([#198](https://github.com/fmadore/Kalamu/pull/198)).
- **A name pointed at in an analysis** is brought into view in the text beside it
  ([#194](https://github.com/fmadore/Kalamu/pull/194)).
- **A guide** in nine chapters, from a first job to exporting and citing:
  [fmadore.github.io/Kalamu/guide](https://fmadore.github.io/Kalamu/guide/).

## Fixed

- **The scan in the review stays still**
  ([#192](https://github.com/fmadore/Kalamu/pull/192)). When the window was not full screen,
  the page could keep resizing, flickering between two widths and staying blurry: a scrollbar
  came and went, each time changing the room the page was drawn in. The pane now keeps the
  scrollbar's room whether it shows or not. Seen on Windows; on a Mac only with scrollbars set
  to always show.
- **Escape while a model answers** a request made from the review stops the wait and leaves the
  dialog open, as *Stop* does; a second Escape closes it.
- The edge of a button that cancels or deletes is as easy to see as any other control's.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.9.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.9.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.9.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.9.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.9.0 over an earlier version keeps your jobs, settings, presets and
keys. Once 1.9.0 has opened your job database, an earlier version refuses it.

## Known limitations

- **The interview analysis has not yet run against a live provider.** How many quotations real
  models give that are not in the transcript, and how useful the themes are, is not yet known:
  read the count of quotations left out, and try one interview first. A transcript is read in
  one request, so one too long for the model is refused; with the models on the computer that
  is a little over an hour of interview. The ready-made prompt's field names (*Themes*,
  *Quotations*) stay in English in the French interface.
- **Search by meaning has not yet been prepared with a cloud model**, and EmbeddingGemma 2 has
  not yet run in the installed app: the requests follow each provider's documentation, and the
  estimate has not been set against an invoice. The map's positions are approximate, close
  groups can overlap, and a long document's point is an average.
- **A region read again and a passage corrected by a model** have run against recorded answers
  only. Check the cost shown against what your provider bills.
- **A large Gemini batch sent as a file** follows Google's documentation and has not run live;
  if Google refuses to delete the file of answers, Kalamu records it as left. Deleting a
  batch's files at OpenAI, Mistral and Anthropic has not run live either.
- **Subtitles of a translation**: economy mode sends these passages at once, at the full price,
  where the estimate counts half; a transcript corrected under some engines has no timed lines,
  and is offered no subtitles.
- **PAGE XML** writes estimated lines but no words where a model places whole blocks.
- **Records** have run on a printed directory, not yet on a handwritten register or in economy
  mode: check the rows, and try a small job first. The `.txt`, `.md` and `.docx` of a records
  job show only each row's first column; use the `.csv` or `.json`.
- **GPT-6.2 and Gemini 4 Neon have not yet read a real page in Kalamu**, and Gemini 4 Hydrogen
  has not been compared with Gemini 3.8 Flash on real pages.
- **Translation and the image description have had few live runs.** Check names, figures and
  dates, and try a small job first.
- **A document of images** can be read for its text, a table of contents or records, not
  analysed whole; its pages cannot be moved, added or removed once the job is made.
- **The offline language models are far weaker than the cloud models.** Check what they write.
  They are slow on a processor, up to several minutes for a long text.
- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model.
- **The Mac versions** are built and tested on Macs in GitHub Actions and were tried on one Mac
  with Apple Silicon before 1.6.0; what 1.7.0, 1.8.0 and 1.9.0 add has not yet been tried on a
  Mac. Intel Macs are not supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once. macOS may also ask once more to let Kalamu use the keys saved in your Keychain: choose
  *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language.
- **Still in English**: the macOS application menu, the offline models' descriptions in part,
  the error a failed page keeps, the guide, and the files Kalamu writes. German will follow.
- **A `.csv` opened by double-clicking in Excel set to French** lands in one column, since
  French Excel expects semicolons: open it with *Data* › *From Text/CSV* instead.
- **Unsigned Windows installers**: Windows SmartScreen will warn. Choose *More info*, then
  *Run anyway*, or install from the Store.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-2 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-2). The
offline OCR, language and embedding models run in llama.cpp b12110 (MIT), whose build is
described in the
[llama-b12110-2 release](https://github.com/fmadore/Kalamu/releases/tag/llama-b12110-2); the
models themselves, LightOnOCR-3 (Apache 2.0), GLM-OCR (MIT), Gemma 4 E4B and E2B (Apache 2.0)
and EmbeddingGemma 2 (Apache 2.0), are downloaded when you ask. Offline transcription uses
whisper.cpp (MIT), on Windows also built with Vulkan into `whisper-vulkan.exe` from Kalamu's own
source, with Vulkan's headers (MIT) compiled in; its voice-activity model is Silero VAD (MIT).
Offline text extraction uses Tesseract 5.5.2 and its language data (Apache 2.0) with Leptonica
(BSD), and PDF pages are drawn with hayro (MIT). Accents are compared with unicode-normalization
and texts cut into passages with unicode-segmentation (each MIT or Apache 2.0). The fonts are
under the SIL Open Font License. Every other component is listed with its licence in
`licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
