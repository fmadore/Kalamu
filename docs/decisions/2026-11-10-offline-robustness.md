# 2026-11-10 · The offline engines made robust: a watchdog, a fairer fallback, downloads that resume

## Context

The [review of 9 November](2026-11-09-review.md) left the offline engines' part of
[#144](https://github.com/fmadore/Kalamu/issues/144), and the [second round](2026-11-10-second-round.md)
built FFmpeg's stoppable runner (`kalamu_media::program`) with a silence watchdog but left
`offline` alone, since it did not build where the round ran. What was found, each confirmed by
reading the code:

- **llama.cpp's runs had no watchdog.** `llama.rs` spawned the program, drained both pipes and
  polled the stop signal, and waited for ever: a graphics driver that hung held `LlamaHost`'s
  turn, and with it every later page and request, until the researcher cancelled. Its
  `--version`, asked from an export, had no limit at all. Whisper's helper already had a decode
  limit of ten minutes.
- **One failure gave the graphics processor up.** Any failure of a Vulkan or OpenCL run gave
  the device up until restart, also when the request was the cause (a schema llama.cpp cannot
  turn into a grammar, an image its decoder refuses, a command line past Windows' limit when
  page mode passes `-sys` and `-p`), and the request then failed the same way on the processor.
- **Model downloads started over from zero** after a 60-second stall, a cancel or closing the
  app, which deleted the `.part` file; Gemma 4 E4B is 5.15 GB with a 0.99 GB encoder. No free
  space was checked.
- **The Whisper helper's device listing** piped standard error and never read it: a talkative
  Vulkan loader or layer could fill the pipe before the answer, the listing timed out after
  30 s, and Whisper stayed on the processor all session.
- **Diagnostics were lost.** The helper's standard error went to the log at `debug`, which the
  app's default filter drops, though a comment said it was kept; llama.cpp's was cut to one
  line in the error.
- **Page images and request texts** were written to the system's temporary folder in a
  `TempDir`, deleted only when dropped: a crash or a kill left the researcher's pages there.

Unlike the second round, `offline` was built and tested here (Linux): its Tesseract build
script downloads Leptonica's and Tesseract's sources from GitHub, which this environment
refused, so the Debian archive's sources of the nearest versions (Leptonica 1.86.0, Tesseract
5.5.0) were placed where the script looks first (`target/tesseract/third_party`), nothing in the
repository changed. Every unit test passed but the one that pins Tesseract 5.5.2, as expected.

## Decisions

### llama.cpp on media's runner, watched for ten minutes of silence

`media`'s `program` gained `run_with` and `Limits`: the silence FFmpeg had, an optional overall
bound, and how much of standard error to keep (llama.cpp keeps its last 64 KB, FFmpeg all of
it, as before). A run ended without a result gives back its standard error for the log. FFmpeg's
calls are unchanged. `llama.rs` keeps no runner of its own: runs, the device listing and the
version all go through it, the listing and the version bounded to 30 s in all.

The limit on a run is a **silence of ten minutes**, with no overall bound. While it works the
program is rarely silent for long: it writes a line for each step of loading, a dot for each
percent of the model read, and the answer token by token on standard output. What it does in
silence is one step of computation: encoding the page image (42 to 73 s on the processors
measured in the [graphics decision](2026-11-06-ocr-graphics-processor.md), whose pages took 58
to 117 s in all; CI's small runners take two to three minutes a page), evaluating one batch of a
text's tokens (2,048 for the language models) or of the image's (one batch, up to 4,096), and
the first touch of a mapped model file on a slow disk. None is known to pass three or four
minutes on a computer the models run on at all; ten leave room for one two or three times
slower than the slowest measured. An overall bound was rejected: a page or a long text on a
weak processor has no length that can be called too long, the output limit (`-n`) already
bounds what is written, and a slow computer that keeps writing is working. If silence turns out
to cut slow runs, the constant (`llama::SILENCE`) is the one place to change.

### The graphics processor given up only when the processor does better

`device::verdict` is the rule, one pure function with its tests, applied by `LlamaHost::settle`
for both the OCR and the language models:

- a run on the graphics processor that answers or is stopped changes nothing;
- one that **hangs** (the watchdog) gives the device up at once: a driver that stopped
  answering is the device's fault, and the processor has never been seen to hang on a page;
- one that **fails** makes the device a suspect: the request comes back as transient, the
  engine sends it again, and it runs on the processor, as does everything else meanwhile. If it
  **answers** there, the device was the cause and is given up for the session, as before; if it
  **fails there too**, the request was, and the device is used again. A pause there leaves the
  suspicion for the next try.

The fallback exists because Vulkan crashed, ran out of memory or failed on some drivers; those
still give the device up after one page, read on the processor. A request is recognised again
by a fingerprint of what it reads (model, page or text, instructions, schema), kept in memory.
A text too long for the language models is refused as before, wherever it ran, and says
nothing of the device. If the suspect request is never sent again (its job cancelled), the
session stays on the processor, which is where the old rule left it. On Windows ARM64 a model
measured on both graphics builds (GLM-OCR) may be retried on the other graphics build rather
than the processor; the first then stays suspected, also as before.

### Downloads that resume, after checking the drive

The `.part` file is kept on a stall, a network error, a cancel and app close, what was buffered
written out first. The next download of the file asks for the rest with `Range: bytes=N-`,
having hashed the N bytes kept again, so one SHA-256 covers the whole file and the check is as
strict as before: final size and hash, exactly. A server that ignores the range (200) sends the
file from the start, which replaces the part; a refused range (416), a `Content-Range` that
does not start at N or names another length, a part longer than the file, or a whole file whose
hash fails after a resume deletes the part and downloads the file once more from the start; a
fresh download that fails the check is reported as damaged, as before. Parts of files no longer
in the engine's list are still deleted when a download starts; removing a model deletes its
part. The progress shown starts at the bytes kept.

Before a download, the free space on the models' drive is compared with what is still to come
for the model, its paired encoder and the engine's shared file, less what their parts hold,
plus 256 MB so the download never fills the drive to its last byte. Too little is refused with
`offline.no_space`, translated, its detail saying how much is needed and available in GB. A
drive the system cannot measure is not refused. Free space comes from **fs4 1.1.0** (MIT or
Apache-2.0, released 28 April 2026), whose two dependencies, rustix and windows-sys 0.61, were
already in the tree: one call instead of a `cfg` block per system.

### Diagnostics at `warn`, only what reads as a diagnostic

A failed or hung llama.cpp run, a failed device listing and a failed Whisper helper log the last
lines of standard error at `warn`, which the default filter keeps: at most 20 lines and 4 KB,
each line cut to 240 characters. Standard output, which holds the answer, is never logged. The
programs do not echo the request on standard error (no `--verbose-prompt`; the text program's
echo is off with `--no-display-prompt`; whisper.cpp's printing is off), but a later build or a
debug switch could, so only lines shaped like a program's own messages are kept: a function's or
program's name and a colon (`llama_model_loader:`, `ggml_vulkan:`, `main:`, a source path) and
the image steps, never prose. Nothing is logged when nothing qualifies, as when a graphics build
ends at once on a PC without the Vulkan loader. The helper's lines are no longer logged at
`debug` one by one. Its device listing now has no input and reads its standard error as it
comes.

### A scratch folder of the app's own

`LlamaHost::new` takes a scratch folder, `offline-scratch` in the app's data folder, passed by
the shell like the OpenCL kernels' folder, and empties it when made, at start-up. Each run's
files still go in a `TempDir`, now inside it, deleted when the run ends. Only folders named as
runs name them (`kalamu-page-*`, `kalamu-text-*`) are swept, so a wrong path deletes nothing
else. A host without a folder (tests, a copy of the app without the programs) uses the system's
temporary folder and sweeps nothing.

### Timeouts at launch and in exports

llama.cpp's `--version` is bounded to 30 s through the runner. FFmpeg's `-encoders` probe at
launch was already on the runner (two minutes of silence) and is left where it is: moving it off
`setup` changes how `EngineConfig` receives FFmpeg, more than a blind two-line change.

## Left

- **FFmpeg's decoders** (24-bit AIFF): `.github/workflows/ffmpeg.yml` lists `pcm_s24le` but not
  `pcm_s24be`, which AIFF's 24-bit samples need (and `pcm_s32be` for 32-bit); both builds' lists
  gain them, then a new pinned build is published and fetched.
- **Child programs that end with the app**: on Windows, a Job Object with
  `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, created once in `src-tauri/src/platform/windows.rs`, each
  child assigned to it after spawning (FFmpeg, llama.cpp, the Whisper helper). It needs the
  child's handle from the crates below, which today spawn by themselves, so either a hook passed
  down as a value (as the player's formats are) or a spawn function the platform provides. On a
  Mac a child outlives the app until it next writes to a pipe no one reads, so one computing
  in silence, or hung, keeps running: the same question holds there.
- **Settings could show a stopped download's progress**: the part's size is known, but the
  list shows bytes only while downloading.

## To check

- On a Windows PC with a graphics processor: a page and an analysis on Vulkan as before; a
  request that fails on the device (a schema llama.cpp refuses) keeps the device for the next
  page, and the log shows the suspicion and its outcome; a driver that hangs is ended after ten
  minutes and the session moves to the processor.
- The silence limit against the slowest computers: the log's timings of a page on CI's runners
  and on a weak laptop, and whether any single step comes near ten minutes.
- On a Mac and on Windows: a large download cut (the network turned off, the app quit) and
  continued, its progress starting at what was kept; the refusal on a nearly full drive, worded
  in both languages.
- That `offline-scratch` is created in each system's data folder (inside the container for the
  Mac App Store build) and emptied at start-up.
- The diagnostics logged on a real failure: that they say something useful and nothing of a
  page.

## Consequences

- `offline` now depends on `media`, as the architecture table allows; `media` does not depend on
  `offline`. AGENTS.md is unchanged.
- The privacy policy has a new effective date: download parts kept beside the models, the
  working folder, and the diagnostics a failed engine logs.
- The shell's only change is the scratch folder passed to `LlamaHost::new`; CI is its first
  build.
