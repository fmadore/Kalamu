Kalamu 1.8.0 reads registers, directories and lists as tables: one row per entry, page by page,
with the file, page and place of each, to correct beside the scan and export for a spreadsheet.
The review can draw where each block, line and word sits on the scan and find it in the text.
Economy mode deletes what a batch left in your OpenAI or Mistral storage once its answers are
safe, and Gemini 4 Hydrogen is Gemini's default. For what 1.7.0 changed, see the
[1.7.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.7.0).

## What's new since 1.7.0

- **Records: one row per entry**
  ([#184](https://github.com/fmadore/Kalamu/pull/184),
  [#171](https://github.com/fmadore/Kalamu/issues/171),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-15-records-regions-batch-files.md)).
  An analysis whose fields are one repeating group (*One row per entry* in the field editor)
  reads each page of a PDF, an image or a document of images on its own and lists its entries in
  reading order, in your columns. Two are ready-made: *Directory or list* and *Register*, each
  reading every entry as written, ditto marks included and nothing carried down from a heading.
  The review shows a page's rows as a table beside the scan: correct a cell, add or remove a row,
  each change kept apart from the model's rows. The `.csv` has one row per entry with its `file`,
  `page` and `position`, then your columns, `corrected`, `reviewed` and `not_read`; a page that
  could not be read stands as one row saying so. The `.json` lists the same rows. New job draws
  this reading, ticks the spreadsheet for it, and prices it page by page.
- **Text regions over the scan**
  ([#184](https://github.com/fmadore/Kalamu/pull/184),
  [#84](https://github.com/fmadore/Kalamu/issues/84)). *Show text regions* in the review draws
  where the engine placed the text: blocks at first, lines from 175% and words from 350%. Choose
  one with the pointer or the arrow keys, and its place is marked in the text. Tesseract's lines
  and words are its own and drawn solid; within the blocks Mistral OCR and LightOnOCR-3 place
  whole they are estimated from the block's shape and dashed. A region whose words you corrected
  shows its own text and says it has no certain place; a page read by a model that places nothing
  says so.
- **Economy mode cleans up after itself**
  ([#184](https://github.com/fmadore/Kalamu/pull/184)). Once a batch's answers are stored, or it
  is given up on, Kalamu deletes what it left in your account: at OpenAI the uploaded requests,
  the answers and the errors file; at Mistral the files the job names. A job deleted while its
  batch waits has its files deleted too, at the next launch if need be. Claude and Gemini leave
  no files. Kalamu reads these files from the batch itself and never lists your storage.
- **Gemini 4 Hydrogen is Gemini's default**, and replaces Gemini 3.8 Flash; Gemini 4 Argon
  replaces Gemini 3.1 Pro ([#185](https://github.com/fmadore/Kalamu/pull/185),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-16-older-gemini-lines.md)).
  New job no longer offers the two older models; a pipeline or workflow that names one keeps it,
  marked *replaced*, with its replacement a click away. Until 1 January 2027 Hydrogen costs about
  twice what 3.8 Flash did.
- **Why an answer stopped.** An answer cut off by the provider's content filter now says so in
  the review, the queue and the error, rather than blaming the output limit, and
  `provenance.json` lists it.
- **The model menu** follows its button when the page scrolls, and pointing at a model, or
  reaching it with the keyboard, shows its whole description.

## Fixed

- **A window that cannot be made** (WebView2 missing or damaged) is explained at launch in
  Kalamu's own message, with what to do on each system and where Kalamu's logs are.
- **Tesseract**: deleting a language right after a job works at once; a PDF page drawn below 300
  dpi is named in ALTO, which measures each page at the resolution it was drawn at.
- **Whisper's voice detection** runs when Kalamu's data folder has an accent in its path, as
  under a Windows account named with one.
- **Correcting and translating a text** offer `.csv` files in the picker, and an `.xlsx`, which
  no task reads, is refused as soon as it is added, with the reason.
- **The keyboard**: Escape in a language picker closes its list and keeps what was typed; a
  name's menu takes Home and End; the prompt and thinking lists of a set-up have a name for
  screen readers again.
- **An empty reading** in a comparison says "(no text)" instead of "om.", and taking it leaves
  one space, not two.
- Settings no longer shows a language as in use by a job that has ended.
- Lists (jobs, a project's documents and tags, filters, a model's languages) are sorted in the
  interface's language.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.8.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.8.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.8.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.8.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.8.0 over an earlier version keeps your jobs, settings, presets and
keys. Once 1.8.0 has opened your job database, an earlier version refuses it.

## Known limitations

- **Records are new.** They have run on a printed directory, not yet on a handwritten register
  or in economy mode: check the rows, and try a small job first. A page-level field (a heading,
  a year above the entries) is not there yet, so nothing is carried down from a heading; an
  entry that runs over the end of a page is given as the part on each page. The `.txt`, `.md`
  and `.docx` of a records job show only each row's first column; use the `.csv` or `.json`.
- **Text regions** follow the engine's own placing. Lines and words estimated inside a block are
  approximate, and the more so in headlines and mixed type sizes.
- **Deleting a batch's files** has run against recorded answers, not yet against the live
  services; whether Mistral names the file of a job's inline requests among them is still to be
  seen.
- **Gemini 4 Hydrogen** has not yet been compared with Gemini 3.8 Flash on real pages; its
  maker reports more unjustified refusals, especially for images.
- **Translation and the image description have had few live runs.** Check names, figures and
  dates, and try a small job first.
- **A document of images** can be read for its text, a table of contents or records, not
  analysed whole; its pages cannot be moved, added or removed once the job is made.
- **The offline language models are far weaker than the cloud models.** Check what they write.
  They are slow on a processor, up to several minutes for a long text.
- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model.
- **The Mac versions** are built and tested on Macs in GitHub Actions and were tried on one Mac
  with Apple Silicon before 1.6.0; what 1.7.0 and 1.8.0 add has not yet been tried on a Mac.
  Intel Macs are not supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once. macOS may also ask once more to let Kalamu use the keys saved in your Keychain: choose
  *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language.
- **Still in English**: the macOS application menu, the offline models' descriptions in part,
  the error a failed page keeps, and the files Kalamu writes. German will follow.
- **A `.csv` opened by double-clicking in Excel set to French** lands in one column, since
  French Excel expects semicolons: open it with *Data* › *From Text/CSV* instead.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Unsigned Windows installers**: Windows SmartScreen will warn. Choose *More info*, then
  *Run anyway*, or install from the Store.

## Included software

Kalamu bundles FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD). The
exact source of the bundled FFmpeg, its build configuration and licences are in the
[ffmpeg-9.0.2-2 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-2). The
offline OCR and language models run in llama.cpp b12110 (MIT), whose build is described in the
[llama-b12110-1 release](https://github.com/fmadore/Kalamu/releases/tag/llama-b12110-1); the
models themselves, LightOnOCR-3 (Apache 2.0), GLM-OCR (MIT) and Gemma 4 E4B and E2B (Apache
2.0), are downloaded when you ask. Offline transcription uses whisper.cpp (MIT), on Windows also
built with Vulkan into `whisper-vulkan.exe` from Kalamu's own source, with Vulkan's headers (MIT)
compiled in; its voice-activity model is Silero VAD (MIT). Offline text extraction uses Tesseract
5.5.2 and its language data (Apache 2.0) with Leptonica (BSD), and PDF pages are drawn with hayro
(MIT). Accents are compared with unicode-normalization (MIT or Apache 2.0). The fonts are under
the SIL Open Font License. Every other component is listed with its licence in
`licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
