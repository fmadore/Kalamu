# 2026-11-08 · Whisper on a PC's graphics processor, real speech detection, and Live Translation's fixes

## Context

Kalamu and Live Translation run the same whisper.cpp (whisper-rs 0.16.0, whisper-rs-sys 0.15.0,
whisper.cpp 1.8.3) with the same instruction sets and processor check. Comparing the two
showed what each could take from the other, and one bug both shared:

- **Speech detection did nothing.** Kalamu set whisper.cpp's `vad` parameter and its model
  whenever the voice-activity model was downloaded, and provenance said `vad: true`. But
  whisper.cpp applies `vad` only in `whisper_full`, and whisper-rs calls
  `whisper_full_with_state`, which ignores it: every window was decoded whole, silence and music
  included.
- **Pausing waited for a window.** Kalamu checked the job's stop signal between two-minute
  windows only, because whisper-rs offers whisper.cpp's abort callback through `unsafe`, which
  this workspace forbids. Live Translation used it, with `unsafe`.
- **The prompt was text, re-tokenized each window.** `set_initial_prompt` tokenizes again for
  every window and leaks a C string each time, and Kalamu guessed whisper.cpp's limit (223
  tokens) as 600 characters. Live Translation fitted the list to the model's own tokenizer and
  passed tokens (`set_tokens`), and dropped lines that only read the prompt back, which Whisper
  writes over noise or music.
- **No per-line check of invented text.** Live Translation applies whisper.cpp's own silence
  thresholds (no-speech probability above 0.6 and mean log-probability below −1) to each line,
  dropping the "Thank you." Whisper invents over room noise; Kalamu kept every line.
- **Whisper ran on the processor on Windows.** The [OCR models' decision](2026-11-06-ocr-graphics-processor.md)
  left Whisper on the processor "as a decision of its own": linking ggml's Vulkan backend into
  the app would make the app itself need the Vulkan loader (`vulkan-1.dll`) to start, and the
  backend initialises Vulkan even to run on the processor.
- **CI never ran Whisper** on a recording.

## Decisions

### Speech detection, done by Kalamu

`whisper/speech.rs` runs whisper-rs's standalone `WhisperVadContext` (Silero v5.1.2, the model
already downloaded with the first Whisper model) on each window. A window with no speech is not
decoded at all. Otherwise its stretches of speech, each widened by 0.2 s and merged where they
touch, are joined with 0.1 s of silence between them, as whisper.cpp's own `vad` mode does, and
every line's times are mapped back to the recording. Language detection then reads the loudest
thirty seconds of speech. If detection fails on a window, the whole window is decoded: it saves
work, it must never lose words. The VAD context is loaded once and kept.

Provenance's `vad` and `vad_model` keep their meaning, now true to what happened. Results made
before this change say `vad: true` though their windows were decoded whole; nothing rewrites
them, and the cache keeps them.

### The abort callback without `unsafe`

whisper-rs 0.16's `set_abort_callback_safe` hands its trampoline a pointer to a
`Box<dyn FnMut() -> bool>` but reads it as the closure's own type, which is undefined behaviour
for a bare closure (Live Translation's reason for its own `unsafe` callback) and exactly right
when the closure type *is* `Box<dyn FnMut() -> bool>`. `stop_when` always passes that type, with
a comment saying why. The box is never freed: a few bytes and one `StopSignal` handle per window.

whisper.cpp asks the callback after each encoder pass (thirty seconds of audio) and each decoding
step, not inside an encoder pass, and language detection is a pass of its own, so the stop signal
is also checked after detection. The smoke test stops the sample interview 0.3 s in; it ends after
two encoder passes (3.6 s on four cores with Base) rather than after the window (8 s).

### Live Translation's prompt and filters

`whisper/text.rs` takes Live Translation's `Prompt`: the first terms, in the researcher's order,
that fit half the model's text context less one token, tokenized by the model, given to every
window as tokens; and its echo check. Provenance still records `initial_prompt` before the model
is loaded, so the terms offered are cut to 600 characters first and written as before
(`"A, B."`); the tokens are those of the same text whenever it fits, which a list of this length
does in the scripts researchers' names are mostly written in, and the log says so when the tail
had to go. Each line also passes the per-line silence check.

### Whisper on Windows' graphics processor, in a helper program

`helpers/whisper-vulkan`, a workspace member outside `crates/` and outside `default-members`, is
whisper.cpp built with Vulkan as a program of its own, shipped beside the app as a sidecar
(`tauri.ffmpeg.windows-<arch>.conf.json`) like llama.cpp's programs. The app starts it when a
piece goes to the graphics processor, sends it the model to load and each window to detect and
decode over standard input and output, and keeps it with the model while it is loaded (the
janitor releases both after three idle minutes). The decoding parameters and the messages are
one file, `crates/offline/src/whisper/engine.rs`, which the helper includes by path, so a window
is decoded the same way wherever it runs.

- **Why a program, not the app linking Vulkan, or ggml's loadable backends.** A program that
  cannot start (no Vulkan loader) costs nothing but a log line; the app always starts.
  whisper-rs-sys links ggml statically and has no switch for `GGML_BACKEND_DL`. A crash on the
  graphics processor (out of memory, a driver fault) ends the helper, never the app. This is the
  design Live Translation adopted the same day.
- **Which device.** The helper takes the OCR models' rules (`device.rs`): a graphics card before
  integrated graphics, Microsoft's Direct3D 12 layer last, never a software renderer, a device
  under 2 GiB, or Snapdragon's Adreno, where Vulkan computed wrong answers that end normally.
  `whisper-vulkan --list-devices` answers once a session, at start-up, off the interface's
  thread, as the OCR models' builds do (`find_graphics_processor`).
- **Settings.** The existing choice, now *Run Whisper, the OCR and the language models on the
  graphics processor*, covers Whisper too; the setting keeps its name (`ocr_graphics_processor`).
- **Provenance and failures**, as for the OCR models: `device` is `vulkan` and `device_name`
  names it; each piece checks, once its turn comes, that it runs where its record said, and is
  sent again otherwise. A helper that cannot start, cannot load the model, fails or stops
  answering sends the piece back as a transient error, and the rest of the session runs on the
  processor. The device is not part of the cache key, as before.
- **Mac.** Unchanged: Metal in the app, now with flash attention, as whisper.cpp's own tools use
  on a graphics processor.

### Builds and CI

The helper needs LunarG's Vulkan SDK on Windows (`.github/actions/vulkan-sdk`, the version and
hashes `llama.yml` uses). CI's Windows x64 lane builds, lints and tests it; the ARM64 and macOS
lanes exclude it (`--exclude whisper-vulkan`); `release.yml` builds it for both Windows
architectures (`scripts/build-whisper-vulkan.mjs`) before the installer and the MSIX package.
Every Rust lane now runs `whisper::tests::smoke`: the sample interview with the pinned Base model
and voice-activity model (cached, checked against their hashes), on Metal on the Mac. The
helper's own smoke test (`whisper::tests::vulkan_smoke`) was run on Linux on Mesa's lavapipe,
which the helper accepts only with `WHISPER_VULKAN_ALLOW_SOFTWARE=1`; GitHub's runners have no
graphics processor, so speeds are still to be measured on real ones.

## Architecture checkpoint

A new kind of artifact (a helper program in the workspace) and a file past 500 lines called for
one.

- **Dependencies still point inwards.** The helper depends on no internal crate; it shares one
  source file with `offline`, by path, rather than depending on it, which would compile
  Tesseract and the rest into it and turn Vulkan on for the app's own whisper-rs.
- **One job each.** `whisper.rs` (872 lines) became `whisper/`: the provider and placement
  (`mod.rs`), the shared decoding and messages (`engine.rs`), the helper's client (`helper.rs`),
  speech detection (`speech.rs`) and the text (`text.rs`). `mod.rs` is still about 650 lines
  before its tests; the window loop and the provider are the candidates to split next.
- **Duplication.** The device ranking now exists in `device.rs` and in the helper's `device.rs`
  (and in Live Translation's helper). Folding it into a crate both could use was set aside: the
  helper would then depend on an internal crate for forty lines.
- **Resume, cache, progress, provenance** still cover every path: the helper is reached only
  from `Whisper::process`, inside the engine's call; provenance names the device.

## Consequences

- On a Windows PC with a usable graphics processor, Whisper should transcribe several times
  faster with nothing to set up; on one without, nothing changes but a probe at start-up.
- Recordings with silence or music take less time, and lines invented over them disappear.
- The Windows packages grow by the helper (its own copy of whisper.cpp, ggml and the shaders).
- Not yet tried on a real Mac (Metal with flash attention) or on a real Windows graphics
  processor; the CI lanes and the lavapipe run cover the code paths.
