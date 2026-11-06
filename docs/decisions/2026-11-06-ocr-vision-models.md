# 2026-11-06 · Offline OCR vision models, run by llama.cpp's own program

## Context

[#105](https://github.com/fmadore/Kalamu/issues/105) asks for a second offline engine for
*Extract text*: a small vision-language model trained for OCR, for the print Tesseract reads
badly (degraded type, old newspapers in columns, mixed layouts, tables), still with no account,
key, charge or upload. It named three candidates (PaddleOCR-VL, GLM-OCR, LightOnOCR) and four
conditions: one copy of ggml, the existing paths (provider, catalogue, media, engine, cache,
provenance), honesty about speed and invented text, and the three systems built in CI.

The checkpoint due before a third offline engine was held first, against the specification,
the roadmap, AGENTS.md and the `offline`, `core`, `media` and `engine` crates.

## Checkpoint findings

- **The provider trait holds.** The engine is one more `Provider` in `offline` (`vision.rs`),
  registered in one line; nothing in `engine` names it. Its readiness check uses the same
  `NotInstalled` path as Whisper and Tesseract, so New job and Settings point to the download.
- **A model can be two files.** A vision model is a language model and an image encoder
  (`mmproj`), each its own download. `ModelFile` gains `pair`: the file it cannot run without,
  downloaded before it, counted in its size, reported as one model and removed with it. Whisper
  and Tesseract keep `None`; `ModelSet`'s shared `companion` stays for Whisper's voice-activity
  model. Nothing was duplicated.
- **Verifying a gigabyte.** `ModelLease::read_verified` reads the whole file to hash it, which
  suits Whisper, whose engine loads from memory. The new engine opens its files itself, so
  `verify` streams the hash instead, once per session for each file unless its size or time
  changes.
- **A shared helper moved to `core`.** Mistral OCR's reading of a block's Markdown or HTML as
  plain words is what LightOnOCR's blocks need too: it is `kalamu_core::plain_words` now, used
  by both. Whisper's thread count and the device it runs on moved to `offline/src/cpu.rs`,
  shared by both engines.
- **Dependencies still point inwards.** `offline` depends on `core` only; `app` wires the engine
  in and finds the program next to the executable, as it finds FFmpeg.
- **Resume, cache, progress and provenance cover the new path.** A page is a piece like any
  other, stored before and after. Decoding is greedy (temperature 0), so a cached page is what a
  new run would read. Provenance records the engine's build, both files and their SHA-256, the
  quantisation, the instruction, the output limit, the image size and the device. Pausing ends
  the program at once; the page counts as interrupted and is read again on resuming.

## Decisions

### llama.cpp as a separate program, not linked

whisper.cpp (through whisper-rs) already links one copy of ggml into the app. llama.cpp brings
its own, of another version: linked statically beside it, the two would define the same
symbols, which the linker either refuses or resolves to one copy for both, an ABI mismatch.
Three ways out were weighed:

- **A shared ggml build** for both: whisper-rs-sys vendors whisper.cpp 1.8.3 with its ggml, and
  llama.cpp moves its ggml weekly; keeping one ggml both accept would tie Whisper's updates to
  llama.cpp's.
- **llama.cpp as shared libraries** inside the app: each library resolves its own ggml, but the
  app's own references from whisper.cpp could still bind to the newer one, and the Mac App Store
  build would have more nested code to sign.
- **llama.cpp's own program, run per page**, as FFmpeg already is. Chosen. It needs no unsafe
  code in the workspace, gives its memory back after each page, cannot take the app down if a
  page crashes it, and its build is independent of Whisper's: the Mac's copy uses Metal without
  touching how Whisper is built. The cost, loading the model for every page, is a second or two
  against tens of seconds of reading on a processor.

The program is `llama-mtmd-cli`, llama.cpp's multimodal command line, given the page and the
model's fixed instruction once (`-p "<__media__>grounding"`), so it answers once and exits.
llama-server was set aside: it would keep the model loaded, but opening a local port needs the
network-server entitlement in the Mac App Store's sandbox, and a server to start, watch and stop.

`.github/workflows/llama.yml` builds it from the source of the pinned release (b12110, which
llama.cpp calls v0.9.0), checked against its SHA-256: one static executable per system, the C
runtime included, without OpenMP or networking, for the same processors as Whisper (AVX2 with
FMA and F16C on x64, `armv8.2-a+dotprod+fp16` on ARM64, so `cpu.rs` checks both), and with Metal
and its shaders built in on macOS. The archives are published as a GitHub release and pinned in
`scripts/llama.json`, and `scripts/fetch-llama.mjs` installs them as FFmpeg's are; the program is
one more `externalBin`, signed to inherit the sandbox in the Mac App Store build. llama.cpp's
official builds were set aside because they ship as many libraries beside each program.

### Models: LightOnOCR-3 1B and GLM-OCR

- **LightOnOCR-3 1B** (LightOn, Apache 2.0, October 2026) is trained end to end on whole pages:
  it needs no separate layout model, reads multi-column pages, tables and handwriting, and was
  trained with European languages in mind. Its `grounding` mode places each block of text on
  the page in thousandths, which becomes a `TextRegion` per block, so it reports layout and
  gives ALTO and a searchable PDF as Mistral OCR does. Picture descriptions and chart tables,
  which it also writes in that mode, are left out: they are its reading of the page, not words
  on it. The 1B rather than the 0.8B or the 4B: the 1B keeps LightOnOCR-2's architecture, which
  llama.cpp's own maintainers have converted (`ggml-org/LightOnOCR-2-1B-GGUF`), and the 4B is
  four times slower on a processor. LightOn publishes no GGUF of its own, so the files are a
  community conversion (`prithivMLmods/LightOnOCR-3-1B-GGUF`), pinned by commit and hash.
- **GLM-OCR** (Z.ai, MIT) has an official GGUF from ggml-org and is strong on tables, formulas
  and Chinese, Japanese and Korean. It returns the text alone (`layout: false`).
- **PaddleOCR-VL 1.6** (Apache 2.0, official GGUF) is left out for now: it is built to read the
  regions a separate layout model (PP-DocLayout) cuts out of a page, which Kalamu does not run,
  and its two files come to 1.8 GB. GLM-OCR also ships with a layout model in its own SDK, but
  reads whole pages with `Text Recognition:`; if the benchmark below shows it reads columns
  badly without one, it may be dropped in turn.

Q8_0 for both: at about a billion parameters a lower quantisation saves little and costs
accuracy (#105 asked for Q4_K_M or better). Downloads: 1.1 GB for LightOnOCR-3, 1.4 GB for
GLM-OCR.

### Running them

One page at a time, each run given half the logical processors on x64 and all but one on
ARM64, at most eight, as Whisper; every layer on the graphics processor where there is one.
Pages larger than a model reads best are resized first (1540 pixels on the longest side for
LightOnOCR, its makers' advice; 1800 for GLM-OCR). The answer may run to 6,144 tokens, in a
context of 10,240 with the image's tokens read in one batch of 4,096 (llama.cpp would otherwise
lower the image's resolution to fit its default batch). A page that reaches the limit is marked
cut short, and a line written over and over at its end is kept once.

### Speed, measured rather than promised

The seconds each page took are kept per model in the models' folder (`speeds.json`, a running
average weighted to the latest pages). Settings and New job show them once the model has read a
page on this computer, and before that say it has not been timed yet; the descriptions give the
range (tens of seconds to over a minute a page on a processor, several times faster on a Mac's
graphics processor). The test page #105 suggested after installation was set aside for this:
the first pages of a real job measure the same thing without a step of its own.

### Benchmark: not yet run

#105 asks for character error rates and seconds per page on real pages before choosing models.
The models were downloaded on the development machine (Windows ARM64), but running llama.cpp's
downloaded program there was not permitted in the session that built this, so no measurement
was made. The workflow above times both models on the sample page on each of its three runners
(Windows x64, Windows ARM64, macOS) and writes the result to its summary; the benchmark on IWAC
newspapers, typescripts and French and Arabic print remains to do before release, and may change
which models are offered.

## Consequences

- An extraction of degraded print can stay on the computer: a PDF or images in, Markdown text
  out, and with LightOnOCR-3, ALTO and a searchable PDF by block.
- The privacy policy names the two model repositories on Hugging Face and the new folder
  (`ocr-models`); the third-party notices and Settings › About credit llama.cpp and the models.
- Release builds need the pinned program: until `scripts/llama.json` holds the archives'
  SHA-256, `fetch-llama.mjs` stops the release workflows.
- The catalogue refreshed from the repository reaches 1.4.1 and earlier, which do not know the
  `local-ocr` provider: their New job offers its models, and starting a job with one stops with
  the message for an unknown provider, before anything is read; their Settings show nothing for
  it. Tesseract's arrival did the same to 1.3.0. Harmless, and gone with the next release; the
  catalogue change could instead wait for the release if that message is judged too confusing.
- Not yet tried: any run of the engine on a real page (see the benchmark), a real Mac, and
  Windows x64.
