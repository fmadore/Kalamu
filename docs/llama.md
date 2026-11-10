# llama.cpp, bundled for the offline OCR and language models

Kalamu runs its offline OCR models (LightOnOCR-3, GLM-OCR) in llama.cpp's multimodal program,
`llama-mtmd-cli`, shipped next to the app's executable as FFmpeg is and started once per page.
Its offline language models (Gemma 4 E4B and E2B) read page images with the same program and
texts with llama.cpp's text program, `llama-completion`, shipped beside it, which takes the
instructions, the text and the answer's schema from files: the multimodal program answers once
only about an image, and takes its request on the command line, which Windows limits to about
32,000 characters ([decision](decisions/2026-11-07-offline-language-models.md)). Both engines
share the programs and take turns (`crates/offline/src/host.rs`).
It is never linked into the app: whisper.cpp already links a copy of ggml, and llama.cpp's own
copy would clash with it ([decision](decisions/2026-11-06-ocr-vision-models.md)). llama.cpp is
MIT licensed; its licence and the exact build line are installed in the app's licences folder.

On Windows a second build, `llama-mtmd-cli-vulkan` with `llama-completion-vulkan`, runs on the
graphics processor through Vulkan ([decision](decisions/2026-11-06-ocr-graphics-processor.md)), and on Windows ARM64 a
third, `llama-mtmd-cli-opencl`, runs GLM-OCR's image encoder on Snapdragon's Adreno through
OpenCL ([decision](decisions/2026-11-06-ocr-opencl-adreno.md)). On a Mac the one build already
uses the graphics processor through Metal.

## Where the app looks

Next to the executable (Tauri's sidecar layout), or in `KALAMU_LLAMA_DIR`; development builds
also accept one on `PATH`. Without it, Settings › Offline models says this copy has no engine for OCR
models, and New job's readiness check refuses them; everything else works. Each build's text
program is looked for beside its multimodal program, with the same suffix; without it the
language models are refused the same way.

The Vulkan build is looked for in the same places. At start-up the app asks it, off the
interface's thread, which devices it finds (`--list-devices`, given half a minute), and keeps
one with at least 2 GB of memory, a discrete card before integrated graphics, software renderers
and Snapdragon's Adreno, which reads wrongly through Vulkan, never
(`crates/offline/src/device.rs`). It then reads every page there, passing that device with
`--device`, unless Settings › Offline models turns it off. When the program does not start (no Vulkan
loader on this PC) or finds nothing usable, the processor build reads instead. When a page or a
request fails there, it is sent again to the processor: if the processor answers, the device was
the cause and the rest of the session runs on the processor; if it fails there too, the request
was, and the device is used again. A run that hangs there gives the device up at once
([decision](decisions/2026-11-10-offline-robustness.md)).

The OpenCL build is asked the same way and keeps the Adreno only. Each model lists the graphics
builds it was measured correct with (`graphics` in `crates/offline/src/vision.rs` and
`llm.rs`): LightOnOCR-3 the Vulkan build, GLM-OCR the Vulkan and OpenCL builds, the Gemma models
the Vulkan build. The OCR models put every layer on the device (`-ngl 99`); the Gemma models,
several times larger, as many as its memory holds beside their fixed context, the rest on the
processor (`-ngl auto --fit on`), so a card with little memory of its own still speeds them up.
On a Mac both run on Metal. A device given up after a failure sends both engines back to the
processor for the rest of the session. Through OpenCL only the image encoder
runs on the Adreno (`--device GPUOpenCL -ngl 0`), the language model on the processor. Its
compiled kernels are kept in the models' folder (`ocr-models/opencl-kernels`, through
`GGML_OPENCL_KERNEL_CACHE_DIR`), so only the first page compiles them.

## How a run is watched

Each run goes through `media`'s program runner, as FFmpeg's do (`kalamu_media::program`): both
pipes read as it runs, a pause or a cancel ending it within a tenth of a second, and a run that
writes nothing on either pipe for ten minutes ended as hung (`llama::SILENCE`): a step of
computation (encoding a page image, a batch of a text's tokens) is the longest the programs stay
silent, a few minutes at most on a weak processor, and the answer comes token by token. There is
no limit on a whole run. `--version` and `--list-devices` are given 30 s in all. The page image,
or the text with its instructions and schema, is written in a folder of its own inside
`offline-scratch` in the app's data folder, deleted after the run; what a crash left there is
deleted at the next start. When a run fails or hangs, the last lines of its standard error that
read as the program's own messages go to the log at `warn`; its answer never does.

## The builds

`.github/workflows/llama.yml` builds the programs from the source archive of a pinned llama.cpp
release (`LLAMA_TAG`, checked against `LLAMA_SHA256`):

- the processor build for Windows x64, Windows ARM64 and macOS on Apple Silicon, and the Vulkan
  build for Windows x64 and ARM64, with LunarG's Vulkan SDK (`VULKAN_SDK_VERSION`, each
  installer checked against the SHA-256 LunarG publishes);
- each the multimodal and the text program, static executables, the C runtime included, with
  nothing shipped beside them; the workflow fails if one needs a library the system lacks. The Vulkan build needs only the Vulkan
  loader (`vulkan-1.dll`), which graphics drivers install;
- no OpenMP, no networking (the app downloads the models), no server, tests or examples;
- the processors Whisper is built for (`.cargo/config.toml`): AVX2 with FMA and F16C on x64,
  `armv8.2-a+dotprod+fp16` on ARM64, which `crates/offline/src/cpu.rs` checks before a page is
  read; the Vulkan build keeps them, so what it leaves to the processor runs the same;
- on macOS, Metal with its shaders built in, for macOS 13 and later.

It also builds, for Windows ARM64 only, the OpenCL build for Snapdragon's Adreno
(`llama-mtmd-cli-opencl`, with Qualcomm's own kernels and Khronos's OpenCL headers, version
`OPENCL_TAG`). It needs only the OpenCL loader Qualcomm's driver installs (`OpenCL.dll`); the
loader built from Khronos's source is used for its import library and to start the program on
the runner, and is not shipped.

Each run then times both models on `samples/bulletin-page.jpg` with the processor builds and
the arguments the app gives, and writes the seconds and the start of each answer to the job's
summary. GitHub's runners have no graphics processor: the Vulkan and OpenCL builds list the
devices they find and read the page once with GLM-OCR, on the processor, to show they work.
The processor builds also ask Gemma 4 E2B for a short analysis through the text program, with
the app's arguments, and fail if its answer is not the schema's JSON. The release's
`scripts/llama.json` lists the programs its archives hold (`programs`); a pin without the text
program is built again.

## Updating it

1. Set `LLAMA_TAG` and `LLAMA_SHA256` in the workflow to the new release and its source
   archive's SHA-256, and run it: through a pull request that changes it, or by hand. A new
   Vulkan SDK means a new `VULKAN_SDK_VERSION`, both installers' SHA-256 and
   `VULKAN_HEADERS_TAG`.
2. When the builds and their timings pass, its `publish` job does the rest: it publishes the
   three `llama-<tag>-<target>.zip`, the two `llama-vulkan-<tag>-<target>.zip` and the
   `llama-opencl-<tag>-aarch64-pc-windows-msvc.zip` archives as a
   GitHub pre-release named `llama-<tag>-<n>` (the next free number, never marked as the app's
   latest release), writes the release name, the tag and each archive's SHA-256 into
   `scripts/llama.json`, commits that to the branch that ran it, and runs the Mac App Store
   sandbox check again on that commit. It does nothing when `scripts/llama.json` already pins
   the tag with every archive, and never runs for a pull request from a fork.
3. `npm run fetch:llama -- <target>` then installs exactly those archives into
   `src-tauri/binaries/` (the Vulkan build on Windows, the OpenCL build on Windows ARM64), and
   refuses one whose hash differs.
4. Release builds add `--config src-tauri/tauri.ffmpeg.conf.json` on a Mac and
   `--config src-tauri/tauri.ffmpeg.windows-x64.conf.json` or `windows-arm64` on Windows, which
   list the programs
   among the external binaries; the Mac App Store build signs its one to inherit the app's
   sandbox (`scripts/sign-app-store.sh`).

A new llama.cpp may read a page differently. The cache keys results by model, not by engine
build or device, so when an update changes what a model writes, give the model a new id in the
catalogue and in `crates/offline/src/vision.rs`, as for Tesseract.
