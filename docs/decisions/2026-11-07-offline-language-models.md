# 2026-11-07 · Offline language models for analyses: Gemma 4 E4B and E2B

## Context

Analyses, tables of contents and corrections needed a cloud model: the specification said so
outright. Researchers who cannot send their material anywhere (interviews, restricted archives,
personal papers) could transcribe and extract it offline, then had nothing to analyse it with.
Google's Gemma 4 (Apache 2.0) has two small models built for laptops, E4B and E2B, that read
texts and images in many languages; Google publishes them as GGUF files at the 4-bit
quantisation they were trained for (QAT), which llama.cpp, already shipped for the OCR models,
runs. Gemma 4 for text extraction was set aside: the OCR models of about a billion parameters
are faster, smaller to download and less inclined to write text that is not on the page; a
comparison on real pages may change that.

## Checkpoint findings

The checkpoint due when a crate takes on a new task (`offline` answering in fields, not only
reading pages and recordings) was held against the specification, the roadmap, AGENTS.md and
the `offline`, `core`, `providers`, `media` and `engine` crates.

- **The provider trait holds.** The models are one more `Provider` in `offline` (`llm.rs`),
  registered in one line. Tables of contents and corrections reach providers as analyses
  (`Task::request_task`), so one provider that answers in fields does all three, and New job
  offers them where the model's inputs allow (a table of contents reads page images, a
  correction text). Nothing in `engine` names it.
- **llama.cpp's programs became shared.** The OCR models owned the programs, their graphics
  builds, the turn that keeps one run at a time and the session's checked file hashes. Both
  engines now share a `LlamaHost` (`host.rs`): one turn, so an analysis and an OCR page never
  compete for the processor or the graphics processor's memory; one search for the graphics
  processor; one Settings choice. The checked hashes moved with it (`Checked`), and the page
  image's resizing to `llama.rs`. Settings and the app's commands ask the host, not the OCR
  engine, for the graphics processor.
- **A shared helper moved to `core`.** The JSON schema built from the researcher's fields and
  the reading of the answer back were private to `providers`; `offline` depends on `core` only.
  They are `kalamu_core::structured` now, used by every cloud provider and the offline models,
  so a field asks the same of every model and every answer is checked the same way.
- **Dependencies still point inwards.** `offline` depends on `core` only; `app` wires the host
  and both engines.
- **Resume, cache, progress and provenance cover the new path.** A document or page is a piece
  like any other, stored before and after. Decoding is greedy (temperature 0), so a cached answer
  is what a new run would write. Provenance records the engine's build, both files and their
  SHA-256, the quantisation, that thinking is off, the output limit, the context, the image's
  size for a page, and the device. Pausing ends the program at once.

## Decisions

### Models: Gemma 4 E4B, and E2B for smaller computers

From Google's own repositories (`google/gemma-4-E4B-it-qat-q4_0-gguf`,
`google/gemma-4-E2B-it-qat-q4_0-gguf`), pinned by commit and SHA-256: the language model and its
encoder (images and audio), downloaded together, 6.1 GB for E4B and 4.3 GB for E2B. Not gated,
Apache 2.0. The 12B and larger models were left out: 7 GB and more to download and about 10 GB of
free memory, too much for most researchers' laptops.

### Two programs: the text program for texts

`llama-mtmd-cli`, the OCR models' program, answers once only when it is given an image, and
takes the request on its command line, which Windows limits to about 32,000 characters, far less
than a document. llama.cpp's text program, `llama-completion`, reads the instructions
(`-sysf`), the text (`-f`) and the answer's schema (`-jf`) from files, applies the model's own
chat template for one turn (`-cnv -st`) with thinking off (`--reasoning off`; Gemma 4 E4B and E2B
then write no thoughts) and prints only the answer (`--no-display-prompt`). A page image (a
photographed letter, a page of an issue for a table of contents) goes to `llama-mtmd-cli` as for
the OCR models, the image first as Gemma's makers advise, with the largest of Gemma 4's image
budgets (1,120 tokens), the one they advise for reading text.

The text program is built beside each processor and Vulkan build in `.github/workflows/llama.yml`
from the same pinned source (b12110), shipped as one more `externalBin`, and signed to inherit
the sandbox in the Mac App Store build. The OpenCL build has none (below). `scripts/llama.json`
records the programs its archives hold, so the workflow builds and publishes them again; until
it has, `fetch-llama.mjs` stops the release builds.

### The answer held to the schema as it is written

llama.cpp turns the analysis's JSON schema into a grammar, so a small model cannot break the
answer's shape, only fill it badly; the answer is then checked against the fields as every
provider's is. An answer that reaches the output limit is refused without a retry, since greedy
decoding would write it again. The limit is the catalogue's `max_output_tokens` (8,192), which
the engine passes like any model's and provenance records; the provider keeps it between 512 and
8,192.

### No thinking to choose

Gemma 4 can think before answering, but llama.cpp's text program applies the schema's grammar
from the first token, which leaves no room for a thought block: only llama.cpp's server knows
where thinking ends and starts the grammar there. So the models have no reasoning levels in the
catalogue, New job offers none, and thinking is off. Small models gain little from it, and on a
processor it would add minutes to every document. Temperature (0), the context (32K) and the
image budget are fixed, since the cache assumes the same request answers the same way.

### What they read at once

A context of 32,768 tokens: the models take 128K, but reading that many on a processor takes far
too long. A text over 66,000 characters (about 11,000 words) is refused before anything runs,
with a message saying to split it or choose a cloud model; llama.cpp's own refusal, for a script
that needs more tokens per character, gets the same message. A whole PDF is not read: the
media step says to extract its text first, then analyse the result.

### The graphics processor wherever llama.cpp can use it

On a Mac the models run on Metal. On Windows they follow the OCR models' Vulkan build and Settings
choice, on any device it chooses, with `-ngl auto --fit on`: llama.cpp puts as many layers on the
device as its memory holds beside the fixed context, the rest on the processor. A card with
little memory of its own still speeds them up rather than being passed over or running out of
memory; the OCR models, a fifth of the size, keep every layer there (`-ngl 99`). A failure on the
graphics processor sends both engines back to the processor for the session.

Snapdragon's Adreno is left to the processor for now: Vulkan read wrongly there for both OCR
models (a wrong answer ends normally, so no fallback catches it), and the OpenCL build was
measured only for GLM-OCR's image encoder. llama.cpp's OpenCL backend, with Qualcomm's kernels,
is written for 4-bit models like these, so it is the next thing to measure on a Snapdragon:
building `llama-completion-opencl` and adding OpenCL to the models' recipes is the whole change
once it reads correctly.

## Consequences

- Analyses, tables of contents and corrections can stay on the computer, with no key, account
  or charge. The specification's line that analyses need a cloud model is gone.
- The answers are far weaker than the cloud models': the catalogue, Settings and the README say
  so, and say to check them.
- The privacy policy names Google's two repositories and the new folder (`language-models`),
  with a new effective date; the third-party notices and Settings › About credit Gemma 4.
- The catalogue refreshed from the repository reaches 1.5.0 and earlier, which do not know the
  `local-llm` provider: their New job offers its models, and a job with one stops with the
  message for an unknown provider before anything is sent, as with the OCR models' arrival.
- Not yet tried: any run inside the app on a real document, a real Mac, Windows x64, and the
  speed on any graphics processor. The llama.cpp workflow asks E2B for a short analysis through
  the text program on the three runners and fails if the answer is not the schema's JSON.
