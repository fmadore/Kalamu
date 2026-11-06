# llama.cpp, bundled for the offline OCR models

Kalamu runs its offline OCR models (LightOnOCR-3, GLM-OCR) in llama.cpp's multimodal program,
`llama-mtmd-cli`, shipped next to the app's executable as FFmpeg is and started once per page.
It is never linked into the app: whisper.cpp already links a copy of ggml, and llama.cpp's own
copy would clash with it ([decision](decisions/2026-11-06-ocr-vision-models.md)). llama.cpp is
MIT licensed; its licence and the exact build line are installed in the app's licences folder.

## Where the app looks

Next to the executable (Tauri's sidecar layout), or in `KALAMU_LLAMA_DIR`; development builds
also accept one on `PATH`. Without it, Settings › Offline says this copy has no engine for OCR
models, and New job's readiness check refuses them; everything else works.

## The build

`.github/workflows/llama.yml` builds the program from the source archive of a pinned llama.cpp
release (`LLAMA_TAG`, checked against `LLAMA_SHA256`) for Windows x64, Windows ARM64 and macOS
on Apple Silicon:

- one static executable, the C runtime included, with nothing shipped beside it; the workflow
  fails if it needs a library the system lacks;
- no OpenMP, no networking (the app downloads the models), no server, tests or examples;
- the processors Whisper is built for (`.cargo/config.toml`): AVX2 with FMA and F16C on x64,
  `armv8.2-a+dotprod+fp16` on ARM64, which `crates/offline/src/cpu.rs` checks before a page is
  read;
- on macOS, Metal with its shaders built in, for macOS 13 and later.

Each run then times both models on `samples/bulletin-page.jpg` with the arguments the app gives,
and writes the seconds and the start of each answer to the job's summary.

## Updating it

1. Set `LLAMA_TAG` and `LLAMA_SHA256` in the workflow to the new release and its source
   archive's SHA-256, and run it (by hand, or through a pull request that changes it).
2. Publish the three `llama-<tag>-<target>.zip` archives as assets of a GitHub release named
   `llama-<tag>-<n>`.
3. Write the release name, the tag and each archive's SHA-256 (from the `.zip.sha256` files)
   into `scripts/llama.json`. `npm run fetch:llama -- <target>` then installs exactly that
   archive into `src-tauri/binaries/`, and refuses one whose hash differs.
4. Release builds add `--config src-tauri/tauri.ffmpeg.conf.json`, which lists the program
   among the external binaries; the Mac App Store build signs it to inherit the app's sandbox
   (`scripts/sign-app-store.sh`).

A new llama.cpp may read a page differently. The cache keys results by model, not by engine
build, so when an update changes what a model writes, give the model a new id in the catalogue
and in `crates/offline/src/vision.rs`, as for Tesseract.
