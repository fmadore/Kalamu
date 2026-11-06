# 2026-11-06 · Offline OCR models on a PC's graphics processor, through Vulkan

## Context

The offline OCR models ([decision](2026-11-06-ocr-vision-models.md)) run on the processor on
Windows: tens of seconds to over a minute a page, two to three minutes on GitHub's small
runners. On a Mac the same program already uses the graphics processor through Metal. Many
Windows PCs have a graphics processor too (NVIDIA, AMD, Intel, Qualcomm Adreno), and recent ones
an NPU. [#122](https://github.com/fmadore/Kalamu/issues/122) asks for the graphics processor
first, chosen at run time and never assumed, with the processor as the fallback, a Settings
choice to keep the models on the processor, the device in provenance, speeds per device, CI
builds for both Windows architectures, and an exploration of Adreno's OpenCL backend and of
NPUs.

Because llama.cpp runs as a separate program, none of this touches Whisper, which stays linked
into the app with its own ggml.

## Decisions

### A second program, not backends loaded at run time

llama.cpp's own Windows releases build one program with `GGML_BACKEND_DL`: the processor and
each graphics backend as libraries beside it, loaded when present. Kalamu ships instead a second
static program, `llama-mtmd-cli-vulkan.exe`, next to the processor one:

- each stays one executable with nothing beside it but what the system or its graphics driver
  provides, as `docs/llama.md` already required, which the MSIX package and the Tauri sidecar
  layout handle without change;
- a PC without a Vulkan loader cannot start the Vulkan program, and the processor program,
  which does not need one, is untouched: a static Vulkan build alone would not start there;
- the cost is size: the Vulkan program carries its own copy of llama.cpp, ggml and the shaders.

Vulkan rather than CUDA, ROCm or DirectML: one backend covers NVIDIA, AMD, Intel and Adreno
through the drivers people already have, its loader comes with them, and llama.cpp tests it on
all four. CUDA would mean shipping NVIDIA's runtime libraries, hundreds of megabytes, for one
vendor.

### Which device, and when

At start-up the app asks the Vulkan program, on a thread of its own, which devices it finds
(`--list-devices`, at most half a minute, so a hanging driver cannot hold anything up). It keeps
the first discrete card, else the first integrated graphics, else a device behind Microsoft's
Direct3D 12 layer (Dozen); it never keeps a software renderer (Basic Render, llvmpipe,
SwiftShader) or a device with less than 2 GB of memory. The listing does not say which devices
are integrated, so names decide: every NVIDIA card is discrete, AMD's are the Radeon RX and Pro,
Intel's the Arc A and B series. A wrong guess picks the integrated graphics of a laptop that
also has a card, which still reads faster than its processor.

The device kept is passed to the program (`--device Vulkan<n>`), which also sends the image
encoder there, so the device in provenance is the one that read the page. llama.cpp's own
choice, without it, would spread the model over every discrete card.

The app uses it unless Settings › Offline says not to (*Read pages on the graphics processor*,
naming the device, on by default and shown only when one was found). If the program fails on a
page (out of memory, a driver fault), the rest of the session reads on the processor: the page
returns a transient error, and the engine sends it again, so its record names the processor.
Each page also checks, once its turn comes, that it is read where its record said; if a failure
or Settings moved the models meanwhile, it is sent again in the same way.

### Provenance and the cache

Each page's request settings record `device` (`cpu`, `metal`, `vulkan`) and, for Vulkan,
`device_name` as the driver names it. Decoding is greedy, but a graphics processor sums in
another order than the processor and can, rarely, choose a different token. The device is not
part of the cache key, as Metal was not: the cache is per computer, provenance says where each
page was read, and keying by device would read every page again after a driver update or a
change in Settings, for differences that are rare and small. A researcher who wants a page read
on the processor turns the graphics processor off and asks for the page again.

### Speeds per device

`speeds.json` keeps the seconds a page took per device (`cpu`, `metal`, `vulkan:<name>`) and per
model. Settings and New job show the figures for the device the next page goes to, and say when
it is the graphics processor. A file from before holds one map, which is read as the built-in
device's (the processor, or Metal on a Mac).

### Builds

`.github/workflows/llama.yml` builds the Vulkan program for Windows x64 and ARM64 natively,
with LunarG's SDK for each architecture (1.4.363.1; `warm` is LunarG's name for Windows on Arm),
each installer checked against the SHA-256 LunarG publishes. The processor flags stay those of
the processor build, so whatever the graphics processor leaves to the processor runs the same.
The workflow fails if the program was built without Vulkan or needs a library other than the
Vulkan loader. GitHub's runners have no graphics processor, so the Vulkan program lists its
devices and reads the sample page once, on the processor, to show it works; its speed is for
real machines.

The `publish` job adds the two Vulkan archives to the release and to `scripts/llama.json`
(`vulkan`), and `fetch-llama.mjs` installs the program on Windows only. Windows release builds
bundle with `tauri.ffmpeg.windows.conf.json`, which lists it beside FFmpeg and the processor
program; the Mac keeps `tauri.ffmpeg.conf.json`.

### Licences

llama.cpp is MIT, as before. Vulkan's headers (Khronos, Apache 2.0 or MIT, taken under MIT) are
compiled into the program, so their licence ships in the licences folder and the third-party
notices name it. The loader is not shipped: it comes with the graphics driver.

### Adreno and NPUs

The workflow also builds, for Windows ARM64, a test program with llama.cpp's OpenCL backend and
Adreno kernels, with Khronos's loader beside it. It is kept as an artifact of the run, never
published or shipped, to compare with the Vulkan build on a Snapdragon laptop; whichever reads
faster and correctly is kept. NPUs (Hexagon on Snapdragon, OpenVINO on Intel) are left to
measurement: llama.cpp's support for vision encoders there is partial and moving, and nothing
is built for them yet.

### Whisper

Whisper could use the same graphics processor, but it is linked into the app with its own
ggml, where Vulkan would make the app itself need the Vulkan loader to start. That is a decision
of its own.

## Consequences

- On a Windows PC with a usable graphics processor, the OCR models should read several times
  faster, with nothing to set up; on one without, nothing changes.
- The Windows packages grow by the Vulkan program; the size is measured on its first build.
- `vision.rs` was already past 500 lines; the new logic lives in `device.rs` (device choice,
  fallback, speeds), and `llama.rs` gained the device listing. Splitting `vision.rs` (the
  recipes and the answer's reading apart from the provider) is left to the next checkpoint.
- Not yet tried: any run on a real graphics processor. The issue asks for at least one NVIDIA
  or AMD PC, one with Intel integrated graphics and one Snapdragon laptop, where Vulkan is
  compared with the OpenCL test build; the timings belong here once taken.
