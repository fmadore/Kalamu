Kalamu 1.7.0 translates: a text, a PDF's own text or a result you have corrected is put into the
language you choose, page by page, and shown beside its source to check. Photographs of the
pages of one letter, register or file are read as one document, in the order you set. A
ready-made analysis describes what an image shows, and New job lists the models by tier, from
each provider's most capable to its lightest. For what 1.6.0 changed, see the
[1.6.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.6.0).

## What's new since 1.6.0

- **Translate a text**
  ([#169](https://github.com/fmadore/Kalamu/pull/169),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-11-translation.md)).
  *Translate a text* in New job reads a text or Word file, a PDF's own text layer, or the text of
  an earlier extraction, transcription or correction through *Use this text in a new job* ›
  *Translate it*, and puts it into the language you choose, whatever language or languages it is
  in. Names of people, places and organisations stay as the text spells them; an office's or an
  organisation's name is given as written, its translation in parentheses at first mention. The
  review shows each page beside its translation, and for a PDF the text the model read is a
  click away. A translation is written as `<name>.translated.<language>`, in the formats of a
  correction, and can be analysed in turn. A workflow can translate after any step that yields a
  text: a ready-made one reads scanned documents, has you check them, translates them and
  summarises the translation, the language left to you. It goes to the provider you choose as a
  correction does: nothing new is sent to anyone new, and the
  [privacy policy](https://fmadore.github.io/Kalamu/privacy) now names it.
- **Images as the pages of one document**
  ([#170](https://github.com/fmadore/Kalamu/pull/170),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-11-ordered-image-documents.md)).
  With several images in New job, choose *Pages of one document* and name it: the photographs of
  a letter, a register or a file are read in the natural order of their names (page 2 before page
  10), with a number that skips or repeats pointed out, and you move pages with the buttons or by
  dragging. Text extraction and tables of contents read the pages as they read a PDF's, and the
  result is one set of files named after the document, in the first page's folder. The review
  links each page to its original file, and `provenance.json` lists the pages in order, each with
  its path and fingerprint. Each page is kept under its own file, so a page moved, or read alone
  before, is not read or paid for again.
- **Image description and keywords**
  ([#169](https://github.com/fmadore/Kalamu/pull/169)), a ready-made analysis for photographs,
  posters, maps and other images: a description of what is visibly there, the kind of image,
  keywords, the text written on it, and the people, places and organisations that text names,
  which the project dictionary gathers. Nothing is inferred from appearance. New job
  suggests it when every file is an image.
- **Models by tier**
  ([#172](https://github.com/fmadore/Kalamu/pull/172),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-12-gemini-4-hydrogen.md)).
  Every model is marked *Most capable*, *Balanced* or *Lightweight* among its provider's own, and
  New job can list models by provider, as before, or by tier, every provider's in one list. The
  model field shows each model's tier and description, and why one cannot be chosen yet. The
  glossary explains what a tier compares.
- **Only the newest model of each line**
  ([#166](https://github.com/fmadore/Kalamu/pull/166),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-11-model-successors.md)).
  New job no longer offers a model its provider has replaced. A pipeline, workflow or comparison
  that names one keeps it and runs as before; the model is marked *replaced*, with a button to use
  its successor. A model its provider no longer serves is refused before anything is sent.
- **New models**: Claude Fable 5.2, which replaces Fable 5.1; Gemini 4 Carbon, beside Argon;
  Gemini 4 Hydrogen, beside Gemini 3.8 Flash, which stays Gemini's default (Hydrogen costs twice
  as much until 1 January 2027); Mistral Large 4, now stable and Mistral's default for analyses,
  which replaces Mistral Medium 3.5.
- **Large jobs stay quick**
  ([#169](https://github.com/fmadore/Kalamu/pull/169),
  [#170](https://github.com/fmadore/Kalamu/pull/170)). A running job's progress is counted from
  an index (on a 2,000-file job, about a sixth of the time and a twentieth of the data read before),
  the names of a job's files are worked out once per run (5,000 files: 6 ms, where it took
  7.7 s), and a job's page can list only *Files with failures*, however far down they sit.

## Fixed

- **A model that cannot read the files** is refused when the job is made, a saved workflow
  included, where each page used to fail in turn. With images among the files, New job had also
  offered models that read only text.
- **The offline language models** refused the corrections and tables of contents New job
  offered them.
- **Recordings in 24- and 32-bit AIFF**, as archives often keep them, are read: the bundled
  FFmpeg now has their decoders ([#144](https://github.com/fmadore/Kalamu/issues/144)), and the
  file picker lists `.aiff` and `.aif` files, and `.wmv` and `.m4v` videos, which before could
  only be dropped on New job.
- **Gemini 4 Argon and Carbon** ask for at most 32,768 tokens of output, the most Google accepts
  from one request, instead of 65,536.
- **A page answered from the cache** showed the cost of the call that first read it; it now says
  *From cache*.
- A workflow's refusal named its step twice, and stayed after the step was put right.
- In the export profile editor and the set-up's Thinking field, controls beside a hint now sit
  level with their neighbours.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.7.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.7.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.7.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.7.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.7.0 over an earlier version keeps your jobs, settings, presets and
keys. Once 1.7.0 has opened your job database, an earlier version refuses it.

## Known limitations

- **Translation and the image description have had few live runs**: a translation with each of
  Gemini, OpenAI, Claude and Mistral, and the image description on a printed page and a
  manuscript, not yet on a photograph. Check names, figures and dates, and try a small job first. Subtitles of a
  translated transcript, and a project glossary, are not there yet.
- **A document of images** can be read for its text or a table of contents, not analysed: an
  analysis of one is refused, with the reason. Its pages are ordered in New job and cannot be
  moved, added or removed once the job is made; a page cannot be given its own label yet, and its
  items cannot yet be read on their own.
- **Gemini 4 Hydrogen** has not yet been compared with Gemini 3.8 Flash on real pages; its
  maker reports more unjustified refusals, especially for images.
- **The offline language models are far weaker than the cloud models.** Their answers are
  plainer, they can miss or invent names and facts, and they follow long instructions less well:
  check what they write. They are slow on a processor, up to several minutes for a long text.
- **Whisper on a graphics processor has not been timed yet** on a real card: turn it off in
  Settings if a recording comes out wrong, and please say so with
  [Report a problem](https://github.com/fmadore/Kalamu/issues/new/choose).
- **The OCR models can write text that is not on the page**, as any language model can: compare
  doubtful pages with Tesseract or a cloud model.
- **The Mac versions** are built and tested on Macs in GitHub Actions and have been tried on one
  Mac with Apple Silicon (macOS 26); what 1.7.0 adds has not yet been tried on a Mac. Intel Macs
  are not supported.
- **Moving from the disk image to the Mac App Store app**: jobs made by the disk image read
  files the store app has not been given yet. To run one again, add its folder to a new job
  once. macOS may also ask once more to let Kalamu use the keys saved in your Keychain: choose
  *Always Allow*.
- **Tesseract** reads print, not handwriting, and does not detect the language.
- **Still in English**: the macOS application menu, the offline models' descriptions in part,
  the error a failed page keeps, and the files Kalamu writes. German will follow.
- **Search** finds words of three letters or more, and one word or another is not offered yet.
- **Items are whole pages of a PDF.** Two articles on one page each take the whole page.
- **Unsigned Windows installers**: Windows SmartScreen will warn. Choose *More info*, then
  *Run anyway*, or install from the Store.
- Offline Whisper, tables of contents and corrections have had fewer runs against the live
  services. Try a small job first.
- A workflow run again starts from the models' text, not the corrections made in the last run.

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
(MIT). The fonts are under the SIL Open Font License. Every other component is listed with its
licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
