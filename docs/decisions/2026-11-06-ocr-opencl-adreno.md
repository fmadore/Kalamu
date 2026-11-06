# 2026-11-06 · GLM-OCR's image encoder on Snapdragon's Adreno, through OpenCL

## Context

The [Vulkan decision](2026-11-06-ocr-graphics-processor.md) runs the offline OCR models on a
PC's graphics processor through Vulkan, except on Snapdragon's Adreno, where Vulkan read the
sample page wrongly and six times slower. Measured on a Snapdragon X Elite (Adreno X1-85,
Qualcomm driver 31.0.152.1), llama.cpp's OpenCL backend, with Qualcomm's own kernels, did
better for one model and one part of it:

- GLM-OCR read the page correctly through OpenCL, and fastest with only its image encoder on the
  Adreno and the language model on the processor: 30–32 s against 58 s, the same text byte for
  byte in five runs;
- LightOnOCR-3's image encoder reads wrongly there, whether the language model runs on the
  Adreno or on the processor.

This decision ships that one case.

## Decisions

### A third program, on Windows ARM64 only

`llama-mtmd-cli-opencl.exe` is built in `.github/workflows/llama.yml` like the Vulkan program:
static, the C runtime inside, the same processor flags as the processor build, with Khronos's
OpenCL headers (`OPENCL_TAG`) and Qualcomm's Adreno kernels built in. It imports `OpenCL.dll`,
which Qualcomm's driver installs; no loader is shipped. Without one the program does not start,
and the processor reads, as today. It is published with the other archives, pinned in
`scripts/llama.json` (`opencl`), installed by `fetch-llama.mjs` for Windows ARM64 only, and
bundled through `tauri.ffmpeg.windows-arm64.conf.json`; Windows x64 has its own
`tauri.ffmpeg.windows-x64.conf.json`, since a program listed for bundling must exist for every
target that lists it. About 14 MB.

### Which device, for which model

The OpenCL program is asked for its devices as the Vulkan one is, and keeps an Adreno with at
least 2 GB of memory; on any other device OpenCL is unmeasured, and Vulkan covers it. Each
model lists the graphics builds it was measured correct with (`Recipe::graphics` in
`vision.rs`, beside the instruction and the image size, which are equally fixed per model and
per engine build): LightOnOCR-3 Vulkan only, GLM-OCR Vulkan then OpenCL. A model's page goes to
the first of its builds that found a device, else to the processor. In the code rather than the
catalogue: which engine build reads a model correctly changes with the engine build, which ships
with the app, not with the catalogue.

Through OpenCL the program is given `--device GPUOpenCL -ngl 0`: the image encoder follows the
device, the language model's layers stay on the processor, where GLM-OCR writes its answer
faster than on the Adreno.

### Provenance, speeds, Settings

Provenance records `device: opencl`, the Adreno's name, and `device_runs: image encoder`. Speeds
are kept under `opencl:<name>`; Settings and New job show each model's own figure and say when
it used the graphics processor. The Settings choice is the same one: off, every model reads on
the processor. Its hint now says that only the models known to read correctly on this graphics
processor use it.

### Compiled kernels

The OpenCL backend compiles its kernels for the device on the first run and caches them, by
default in `%LOCALAPPDATA%\llama.cpp\cl-cache`. Kalamu points it at its own folder
(`ocr-models/opencl-kernels`, about 8 MB, through `GGML_OPENCL_KERNEL_CACHE_DIR`), so nothing is
written outside the app's data and the privacy policy's list stays whole. The first page takes
about 37 s more; later pages take about 37 s in all, against about 58 s on the processor, in the
app's own code path (`vision::tests::smoke`).

### What a wrong reading would do

A wrong reading ends normally, so the fallback after a failure cannot catch it: that is why
LightOnOCR-3 never gets the Adreno, and why the rule is per model rather than per device. What
protects GLM-OCR is the measurement: one laptop, one driver, one page, five identical answers.
That is thin, and was weighed: shipping it as part of the Vulkan pull request was set aside for
this reason, and it is kept apart so it can be reverted alone. Before it is relied on widely it
needs a second Snapdragon (another model or driver) and more pages, read against the
processor's answers; `scripts/bench-ocr-devices.ps1` measures processor and Vulkan, and takes
the OpenCL program with `-OpenClDir`.

## Consequences

- On a Snapdragon X laptop with Qualcomm's driver, GLM-OCR reads a page in about two thirds of
  the processor's time; LightOnOCR-3 is unchanged.
- The Windows ARM64 packages grow by about 14 MB.
- A new Qualcomm driver or llama.cpp may read differently. The rule is in code, so a release
  that finds it wrong removes OpenCL from GLM-OCR's list.
