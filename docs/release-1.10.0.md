Kalamu 1.10.0 asks which of a project's documents speak of your research question, and why,
with the passages that say so. A project can be saved as one file and opened on another
computer. A long document can be tried on three pages first, an interview of two hours is read
in parts, and New job says which photographs show the same page twice or look blurred. For
colleagues new to these tools, Kalamu opens with a welcome when nothing is set up yet, every
table comes as an Excel workbook too, the ready-made fields and the guide are in French, and an
empty project says what to do first. For what 1.9.0 changed, see the
[1.9.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.9.0).

## What's new since 1.9.0

- **Which of these speak of my question?**
  ([#220](https://github.com/fmadore/Kalamu/pull/220),
  [#221](https://github.com/fmadore/Kalamu/pull/221),
  [#51](https://github.com/fmadore/Kalamu/issues/51),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-research-question-screening.md)).
  Select documents in a project and type your research question: a model reads the text of
  each, as you corrected it, and says whether it speaks of the question, why, and with up to
  three passages. Nothing is shown as a passage unless Kalamu found those words in the text,
  with their page. Filter the project's table by the answer: each row then shows the answer,
  its reason and its passages, with **Keep** and **Set aside**, kept as tags. A long document is
  read in parts, and is relevant if any part is. The question stays with the project: once
  documents are added, one line offers to read only them against it, so only they are paid
  for. The whole table exports as an Excel workbook and a CSV.
- **A project saved as one file**
  ([#221](https://github.com/fmadore/Kalamu/pull/221),
  [#213](https://github.com/fmadore/Kalamu/issues/213),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-project-file.md)).
  **Save to a file…** writes a project as one `.kalamu` file: its results, your corrections and
  review marks, collections, tags, descriptions and dictionary, and the original scans and
  recordings only if you choose. Never your keys, your settings or your other projects.
  **Open a project file…** opens it on any computer, Windows or Mac, as a new project beside
  your others, which stay as they were. A damaged or foreign file is refused in a sentence.
- **Try on a few pages first**
  ([#221](https://github.com/fmadore/Kalamu/pull/221),
  [#215](https://github.com/fmadore/Kalamu/issues/215),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-sample-first.md)).
  For a text extraction of a document of more than 15 pages, New job offers to read three pages
  spread through it, with one model or two side by side. From the sample's page, run the rest
  with the model you prefer, the three pages already read costing nothing, or try another model
  on the same pages.
- **Long interviews, your own themes, your own quotations**
  ([#220](https://github.com/fmadore/Kalamu/pull/220),
  [#38](https://github.com/fmadore/Kalamu/issues/38),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-interview-windows-guide-themes.md)).
  A transcript too long for the model chosen is no longer refused: it is read in parts of whole
  turns, then brought together, and every quotation is still the transcript's own words. Type
  your themes or interview guide under *More choices*: each theme in the answer says whether it
  comes from your guide or emerged from the interview, and a guide theme the interview does not
  take up is listed apart. Select a passage of the transcript to add it to a theme as your own
  quotation, marked as yours in every file.
- **Photographs taken twice or blurred**
  ([#220](https://github.com/fmadore/Kalamu/pull/220),
  [#216](https://github.com/fmadore/Kalamu/issues/216),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-duplicate-and-blurred-photographs.md)).
  New job compares its photographs on the computer, nothing sent, and says which look like the
  same page photographed twice and which look blurred, side by side with the sharper one
  suggested. A photograph is left out only if you choose, with Undo.
- **A first start without a key**
  ([#219](https://github.com/fmadore/Kalamu/pull/219),
  [#208](https://github.com/fmadore/Kalamu/issues/208)). Kalamu opens on Settings only while
  there is neither a key nor anything installed offline, with a welcome that offers both ways
  in: a model on this computer, or a key from an online provider. Someone who works offline is
  no longer sent to *API keys* at every launch.
- **Tables as Excel workbooks**
  ([#219](https://github.com/fmadore/Kalamu/pull/219),
  [#209](https://github.com/fmadore/Kalamu/issues/209),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-xlsx-tables-corrected-per-row.md)).
  Every table Kalamu writes as CSV is also written as an `.xlsx` workbook, which opens by
  double-click in Excel, LibreOffice or Numbers set to any language, every cell as written. A
  records export marks as corrected only the rows you typed into or added.
- **The ready-made fields in French**
  ([#219](https://github.com/fmadore/Kalamu/pull/219),
  [#210](https://github.com/fmadore/Kalamu/issues/210),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-20-default-fields-interface-language.md)).
  The field names, help and choices of the default analyses, contents and records (*Résumé*,
  *Thèmes*…) show in French when the interface is; what is stored, exported and sent stays as
  it was.
- **The guide in French, and reached from the app**
  ([#219](https://github.com/fmadore/Kalamu/pull/219),
  [#211](https://github.com/fmadore/Kalamu/issues/211)). Its nine chapters are now in French
  too, at [fmadore.github.io/Kalamu/guide/fr](https://fmadore.github.io/Kalamu/guide/fr/), and
  *Read the guide* in the welcome, the Glossary and Settings › About opens it in the interface
  language.
- **An empty project says what to do first**
  ([#219](https://github.com/fmadore/Kalamu/pull/219),
  [#212](https://github.com/fmadore/Kalamu/issues/212)), in place of its table, and its search
  and *What to check* say what will come there once a job has read the documents.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.10.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.10.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.10.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.10.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.10.0 over an earlier version keeps your jobs, settings, presets and
keys. Once 1.10.0 has opened your job database, an earlier version refuses it.

## Known limitations

- **Screening has had one live run**, on ten short documents with Gemini 4 Hydrogen, whose
  answers and passages held up. How models follow the rule that a long document is relevant if
  any part is, is not yet known: read the reasons, check the passages, and try a few documents
  first. New job offers the screening prompt without a box for the question: start a screening
  from the project. *Export the whole table* writes every document's answer, whatever filter is
  on.
- **Estimates now count the models' thinking** and a newspaper's dense pages, which made them
  three to eleven times too low before this version's test pass: Gemini 4 thinks more than
  other models on a page, a PDF page whose text layer is small print is priced as a
  newspaper's whatever the prompt, an interview analysis writes a longer answer, and a region
  read again is priced as the part of the page it sends. A newspaper scanned as images, with no
  text layer, is priced as dense pages only with the *Newspaper or magazine* prompt. They are
  still estimates: compare the first job's cost with what your provider bills.
- **GPT-6.2 Sol can leave text out, or answer nothing.** It replaced a chemistry lecture's
  description in a 1911 newspaper with a note that it had omitted it, and answered an analysis
  of that issue with every field empty. Kalamu now refuses an analysis left wholly empty for a
  long text and asks again with more thinking (*Medium*, then *High*, where the model has them),
  then says which levels were tried; a comparison of two models shows the gaps in a
  transcription.
- **A project file has not yet been carried from Windows to a Mac.** A project opened without
  its original files cannot yet be pointed at those files on the new computer; save it again
  with its originals instead. Corrections from a colleague's copy are not merged: their file
  opens as a project of its own. Workflow runs are not in the file.
- **Try on a few pages first** is offered for a PDF, not yet for a folder of separate images,
  nor in economy mode. New job's diagram still shows pages 1 to 3.
- **A long interview read in parts** has run against recorded answers only; the interview
  analysis itself has run live on a short interview, with the researcher's own themes. Read the
  count of quotations left out, and try one interview first.
- **The check of photographs** was tuned on retakes of two sample pages: a real batch from a
  reading room may need it told otherwise. It takes about a minute for 300 photographs.
- **The `.xlsx`** opens by double-click in Excel set to French; it has not yet been opened on a
  Mac. Exports' column headings stay in English.
- **Search by meaning has been prepared with Gemini Embedding 2 and with EmbeddingGemma 2** on
  the computer (about 2½ minutes for a small project on a Snapdragon's processor), not yet with
  OpenAI's or Mistral's models. The map's positions are approximate.
- **A region read again and a passage corrected by a model** have each run live once (Mistral
  OCR on two handwritten lines, Gemini on a passage). Check the cost shown against what your
  provider bills.
- **A large Gemini batch sent as a file** follows Google's documentation and has not run live.
  OpenAI's and Anthropic's batches have run live and came back within minutes; that their files
  are deleted afterwards has not been checked on the providers' pages. Mistral's batches are
  now sent as an uploaded file, since rc2's test pass found two photographed pages sent inline
  refused; that file has not yet run live.
- **GPT-6.2 Sol, GPT-6.2 Astra and Gemini 4 Neon have read one printed page and one handwritten
  page each in Kalamu.** On the handwriting Neon reads much as Gemini 4 Hydrogen does, at about
  twice the cost.
- **Records** have run on a printed directory, not yet on a handwritten register or in economy
  mode: check the rows, and try a small job first.
- **The offline language models are far weaker than the cloud models.** Check what they write.
  They are slow on a processor, up to several minutes for a long text.
- **On a Snapdragon PC, LightOnOCR-3 runs on the processor**, by design, and a dense
  newspaper page can take several minutes. Settings shows an average over the pages read so far.
- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model.
- **The Mac versions** are built and tested on Macs in GitHub Actions and were tried on one Mac
  with Apple Silicon before 1.6.0; what 1.7.0 and later add has not yet been tried on a Mac.
  Intel Macs are not supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once. macOS may also ask once more to let Kalamu use the keys saved in your Keychain: choose
  *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language.
- **Still in English**: the macOS application menu, the offline models' descriptions in part,
  the error a failed page keeps, exports' headings, and the files Kalamu writes. German will
  follow.
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
(BSD), and PDF pages are drawn with hayro (MIT). Excel workbooks are written with
rust_xlsxwriter (MIT or Apache 2.0). Accents are compared with unicode-normalization and texts
cut into passages with unicode-segmentation (each MIT or Apache 2.0). The fonts are under the
SIL Open Font License. Every other component is listed with its licence in
`licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
