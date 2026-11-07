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
change in Settings, for differences that are rare and small. So a page already read is taken
from the cache wherever it was read, even after the graphics processor is turned off, and its
provenance still names the device that read it (seen in the 1.5.0-rc1 test); Kalamu offers no
way to read a cached page again on another device. Turning the graphics processor off changes
where new pages are read.

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
bundle with `tauri.ffmpeg.windows-<arch>.conf.json`, which lists it beside FFmpeg and the
processor program; the Mac keeps `tauri.ffmpeg.conf.json`.

### Licences

llama.cpp is MIT, as before. Vulkan's headers (Khronos, Apache 2.0 or MIT, taken under MIT) are
compiled into the program, so their licence ships in the licences folder and the third-party
notices name it. The loader is not shipped: it comes with the graphics driver.

### Adreno and NPUs

The workflow also builds, for Windows ARM64, a test program with llama.cpp's OpenCL backend and
Adreno kernels, with Khronos's loader beside it. It is kept as an artifact of the run, never
published or shipped, to compare with the Vulkan build on a Snapdragon laptop; whichever reads
faster and correctly is kept. Measured below; it became GLM-OCR's image encoder on the Adreno
([decision](2026-11-06-ocr-opencl-adreno.md)). NPUs (Hexagon on Snapdragon, OpenVINO on Intel) are left to
measurement: llama.cpp's support for vision encoders there is partial and moving, and nothing
is built for them yet.

### Measured on a Snapdragon X Elite

A Snapdragon X Elite X1E80100 laptop (12 cores, Adreno X1-85, Qualcomm driver 31.0.152.1,
16 GB), the sample page, the app's arguments, the builds of run 42232655049 (`llama-b12110-2`),
except the i8mm one, built locally from the same source with the same flags plus `+i8mm`:

| Build and device | LightOnOCR-3 | GLM-OCR |
| --- | --- | --- |
| Processor, 8 threads (the app) | 68.7 s, image 42 s | 57.8 s, image 44 s |
| Processor, 11 threads | 62.8 s, image 32 s | 60.7 s, image 36 s |
| Processor with i8mm, 11 threads | 62.4 s, image 38 s | 50.2 s, image 37 s |
| Vulkan, Adreno | 612 s, nonsense | 464 s, nonsense |
| OpenCL, Adreno | out of memory (one 2.4 GB buffer); with `-ub 2048`, 862 s, nonsense | 45–46 s, image 17–18 s |
| OpenCL, encoder on the Adreno, model on the processor (`-ngl 0`) | 310–449 s, nonsense | 30–32 s, image 17 s |

Every GLM-OCR answer above is the same text, byte for byte; LightOnOCR's processor answers
differ only in a few block positions, by a thousandth of the page.

- **Vulkan on Adreno reads wrongly** and six times slower: a wrong reading ends normally, so the
  fallback would not catch it. `device.rs` never chooses a device named Adreno.
- **OpenCL reads GLM-OCR correctly and, with only its image encoder on the Adreno, in half the
  time**; text generation is faster on the processor. LightOnOCR-3's image encoder reads wrongly
  there, so it stays on the processor.
- **i8mm and more threads** gave no clear gain on one page; runs of the same build vary by
  about 5%. The ARM64 build keeps `armv8.2-a+dotprod+fp16`, which still runs on the 8cx.

`scripts/bench-ocr-devices.ps1` repeats the processor and Vulkan measurements on any Windows PC,
for the machines still to measure.

### Measured on Intel's integrated Arc graphics

An Intel Core Ultra 7 155H laptop (Meteor Lake, Arc Graphics with driver 32.0.101.8991), the
sample page, the app's arguments, the builds of `llama-b12110-2`, one run each, with
`scripts/bench-ocr-devices.ps1`:

| Build and device | LightOnOCR-3 | GLM-OCR |
| --- | --- | --- |
| Processor, 8 threads (the app before) | 116.8 s, image 49 s | 98.8 s, image 73 s |
| Vulkan, Arc Graphics | **51.5 s**, image 20 s | **43.4 s**, image 23 s |

- **Vulkan reads both models correctly there, in less than half the processor's time.**
  GLM-OCR's answers are the same byte for byte; LightOnOCR-3's text is the same, three block
  positions moved by a thousandth of the page, as between processor builds.
- **The app chooses that device.** The Vulkan build lists one, `Intel(R) Arc(TM) Graphics`
  (18,400 MiB of shared memory); `device.rs` ranks it as integrated graphics, the Arc name
  without an A or B model, and keeps it, being the only one.
- **Intel laptops gain the most so far**: their processor reads a page more slowly than the
  Snapdragon's (99–117 s against 58–69 s).

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
- Measured so far (above): on Intel's integrated Arc graphics both models read correctly in
  less than half the processor's time; on a Snapdragon the Adreno reads wrongly through Vulkan
  and the models stay on the processor. Still to measure: an NVIDIA or AMD card; the timings
  belong here once taken.
