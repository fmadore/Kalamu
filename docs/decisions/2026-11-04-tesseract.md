# 2026-11-04 · Tesseract, page images, word positions and ALTO

## Context

[#46](https://github.com/fmadore/Kalamu/issues/46) asks for free text extraction on the
researcher's computer, as Whisper already gives for recordings, so that someone who wants to stay
local can do so without an account, a key or a provider. Tesseract was planned in the
specification's *Offline mode*; ALTO XML was planned "where models return coordinates", and
correcting ALTO with its coordinates kept is what iwac-ai-pipelines' `02_correct_alto_xml.py`
does. A small OCR vision model (PaddleOCR-VL, GLM-OCR, LightOnOCR) is the next step, tracked in
[#105](https://github.com/fmadore/Kalamu/issues/105).

The issue asks for the architecture checkpoint due before a second implementation of an
offline engine. It was held before writing code, against the specification, the roadmap,
AGENTS.md and the `offline`, `media`, `export` and `engine` crates.

## Checkpoint findings

- **The provider trait holds.** Tesseract is one more `Provider` in `offline`, registered in one
  line; the engine does not name it. Two things the trait lacked were added in `core`, where
  every engine benefits: the readiness check now receives the job's languages (an engine whose
  language data is a download per language must check that data before the job starts, as
  Whisper checks its model), and `TextRegion` gains the lines and words an engine places.
- **Model downloads were Whisper's.** `ModelManager` held Whisper's list and its voice-activity
  model by name. It now takes a `ModelSet` (the files and an optional companion downloaded with
  the first of them), so Tesseract's language data uses the same pinned, hashed, resumable and
  leased downloads. Nothing was duplicated.
- **PDFs were never drawn.** Every model so far read PDF pages, so the specification's *PDF to
  images with PDFium* had been deferred
  ([foundations](2026-10-24-foundations.md)) until "a provider or Tesseract needs images". That
  time has come; see *PDF pages drawn with hayro* below.
- **Dependencies still point inwards.** `offline` depends on `core` only among internal crates,
  `media` and `export` likewise; the realignment of corrections is in `engine`, beside the
  export document it serves.
- **Resume, cache, progress and provenance cover the new path.** A Tesseract page is a piece
  like any other, stored before and after; its cache key names the model `tesseract-5`, whose
  data revision is fixed, and each result records the engine's version, the data revision, the
  languages loaded and the page segmentation.

## Decisions

### Tesseract built from source and linked in

Tesseract 5.5.2 and Leptonica 1.87 are compiled by `kreuzberg-tesseract` (MIT) and linked into
the app, through its safe API: the workspace forbids unsafe code. Tesseract is Apache 2.0 and
Leptonica BSD-2-Clause, so linking is allowed, unlike FFmpeg's LGPL; nothing is shipped beside
the executable. Their licences are kept in `crates/offline/licenses/`, since the build script
downloads their sources, and `third-party-notices.mjs` adds them to the notices.

Three things the build needed:

- **Windows ARM64.** CMake calls the processor `ARM64`; Tesseract's CMakeLists matches only
  `arm64` or `aarch64`, so it left its NEON code out while its processor check, which sees
  `__aarch64__` under clang-cl, still called it, and linking failed. A CMake toolchain file
  for that target alone (`crates/offline/cmake/windows-arm64.cmake`, named in
  `.cargo/config.toml`) gives the lower-case name. whisper.cpp builds with it too, and its
  smoke test transcribes as before. Worth reporting upstream.
- **`zip` stays on 8.** The build script accepts any `zip` from 7 up and breaks on 9;
  `Cargo.lock` keeps 8.6, as the crate was released with. A Dependabot update to 9 will fail
  its build and should wait for a new `kreuzberg-tesseract`.
- **Its build folder** is `target/tesseract` rather than its default `C:\tess`.

The crate's build script downloads Tesseract and Leptonica's tagged sources from GitHub without
a checksum. They come from the projects' own repositories at fixed tags; vendoring them was
judged heavier than the risk, and is the fallback if the crate stops being maintained.

### Language data

One file per language from `tesseract-ocr/tessdata_best` at a pinned commit, size and SHA-256
checked as for Whisper, downloaded from Settings into the app's data folder
(`tesseract-data`). `tessdata_best` rather than `tessdata_fast`: the most accurate models, at 3
to 15 MB a language. The 26 languages of Kalamu's list that Tesseract has a model for get
theirs; the Latin-script languages of West and Central Africa it has none for (Hausa, Igbo,
Fula, Wolof, Bambara, Dioula, Mooré, Ewe, Fon, Twi, Somali, Lingala, Zulu, Xhosa, Malagasy)
read with Tesseract's Latin script model (101 MB), which covers every language written in
Latin letters.

Tesseract does not detect the language. The catalogue says so per model (`detects_language`);
New job then offers no *Detect automatically*, asks for the languages, and points to Settings
for any whose data is missing. Several languages are loaded together (`fra+eng`).

Tesseract opens its data by a path in the system's code page. On Windows the app's manifest
now declares UTF-8 as that code page, so a user folder with accents (`C:\Users\Frédérick`)
works; Tauri's own manifest content is kept. The manifest is set in `src-tauri/build.rs`,
since Tauri's configuration has no field for it.

### PDF pages drawn with hayro

A PDF page is drawn at 300 dpi when the model reads images only, from the same single-page PDF
the other models are sent, so a split spread is drawn as the half they read. hayro (MIT or
Apache 2.0) is a PDF rasterizer written in Rust that forbids unsafe code; PDFium would have
meant a library shipped beside the app on each of the three systems, with its loading and
packaging. hayro does not open encrypted PDFs (neither does `lopdf`, which already refuses them)
and its authors warn that malformed files can still exhaust time or memory; a page that fails
is reported like any other failed piece. The specification now names hayro.

### Word positions and corrections

`TextRegion` keeps its block and gains `lines`, each with its `words` and the engine's
confidence. Tesseract fills them from its TSV output; Mistral OCR still gives blocks only.
A corrected page used to collapse into one block over the page. Its words are now laid back
onto the boxes of the words the engine read (`engine/src/realign.rs`), ported from
`02_correct_alto_xml.py`: there the model corrects line by line and the corrected words are
written into the original `String` elements; here the researcher's whole page is matched to the
engine's words by a diff, a changed word taking the box of the one it replaces, added words
sharing the box before them, removed words leaving theirs empty. Kalamu differs in matching by
diff because its corrections are free text typed by a person, not one answer per line.

### ALTO

ALTO 4, one file per document, a `Page` per page, positions in pixels (of the image, or of the
page at 300 dpi for a PDF), each word with its confidence where the engine gave one. Offered,
like the searchable PDF, only from a model that reports layout (`ExportFormat::needs_layout`).
From Mistral OCR the lines share their block's height and the words their line's width, and the
file says those positions are estimated. The searchable PDF now sets each of Tesseract's lines
over its own box.

## Consequences

- An offline text extraction runs end to end on the computer: PDF or image in, text, word
  positions, ALTO and a searchable PDF out, at no cost, after one download per language.
- A third offline engine (#105) reuses `ModelSet`, the readiness check with languages and the
  page images. llama.cpp would bring a second copy of ggml beside whisper.cpp's; #105 must
  settle that before linking it.
- The catalogue refreshed from the repository reaches 1.3.0 and earlier, which do not know
  Tesseract: their Settings list it, and downloading it says the model is not known to this
  version. Harmless, and gone with the next release.
- Not yet tried: a full job with networking disabled after installation, a real Mac, and
  Windows x64 (CI builds and tests both).
