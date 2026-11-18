# 2026-11-18 · Search by meaning and a map, a passage corrected by a model, items added to a list answer, a guide

## Context

A Mac can be rented for a few days, so the round after
[the morning's](2026-11-18-gemini-file-batches-set-aside-far-apart-names-resume.md) took what
gains most from being tried there: a model run through llama.cpp on Metal, and a menu on a
selection, which WebKit and WebView2 handle differently. The maintainer asked for a search by
meaning with local and paid models, a map of a project's documents, documents of very
different lengths handled fairly, a passage of the review corrected by a model, and a keyword
added to an analysis's answer. A description of photographs, also asked for, was already built
([#168](https://github.com/fmadore/Kalamu/issues/168)). Five agents wrote the pieces, each in a
worktree; they were reviewed and gathered on one branch where every check was run again.

Embeddings are a second kind of provider call, so this began with the architecture checkpoint
`AGENTS.md` asks for.

## Decision

### A trait of its own, not a task

- **`Embedder` in `core`**, beside `Provider`: some texts and a model in, vectors of length 1
  and the tokens billed out, with a role (a passage or a question) since most models embed
  them differently. One implementation for each cloud provider in `providers`, one in
  `offline`. The engine receives them as values and never matches on a name.
- **Embedding models are a list of their own in the catalogue**, never among the models New
  job offers; an older app reads a newer catalogue without it.
- **`VectorIndex` and `EmbeddingLog`** in `core`: the vectors in a file of their own, derived
  like the word search's index; the model chosen, the consent and each run's cost in the job
  database, so the cost survives the loss of the vectors. `engine` uses `search`, which the
  crate table already allowed.

### The models

- **Cloud**: Gemini Embedding 2, OpenAI's `text-embedding-3-large` and `-small`, Mistral
  Embed, from each provider's documentation read on 18 November 2026. Gemini takes the role as
  words before the text, as Google documents; OpenAI and Mistral take none. Every vector is
  brought to length 1 here, whatever the provider says.
- **A text is 4,000 characters at most** for the cloud models: two tokens to a character
  assumed in any script, against limits of about 8,000 tokens.
- **On this computer: EmbeddingGemma 2** (Google, Apache 2.0), its text model alone, in the
  Q8_0 build (310 MB): almost nothing lost against the full precision at half the size, and no
  16-bit floats, which its card forbids. Its image and sound encoders are not downloaded.
- **llama.cpp's `llama-embedding`**, shipped beside the processor builds and the Mac's; the
  pinned llama.cpp already knew the model. Texts go in one file, cut at a separator of
  Kalamu's own, since the default (a new line) would cut a text of several lines; the
  program's rewriting of backslashes is turned off. No Vulkan or OpenCL build: a model this
  small does not need one. 2,000 characters to a text, within a context of 4,096 tokens.
- Its provider is the one the offline language models already have in the catalogue; the two
  agents had chosen differently and the gathering settled it.

### Lengths

- **Stretches of about 1,200 characters**, a seventh shared with the next, cut at a
  paragraph, else a sentence, else a word, else a grapheme (UAX #29, from a crate already in
  the lock file), so Arabic and scripts without spaces are cut properly.
- **The search ranks stretches and lists documents** by their best one, three passages at
  most for each: a long book is neither diluted nor favoured.
- **The map has one point for a document**: the mean of its stretches weighted by length,
  from one reading. The screen says a long document's point is an average.
- **Two stamps to a stretch**: the word search's, and a hash of the text. Marking a page
  reviewed changes the first only, and then costs nothing; one stamp would have made every
  reviewed page out of date and billed it again.
- An exact comparison with every stretch: 24 ms for 100,000 stretches of 768 numbers on the
  development PC, so no approximate search and no dependency for it.

### Consent and cost

- Nothing is embedded until the researcher starts it, after an estimate. A cloud model needs
  consent for that project and model, naming the provider and what is sent: the text now, each
  question afterwards. An update after corrections is offered, never made by itself.
- A run goes through the engine: its limits, retries, pause and cancel, each call's vectors
  kept as it returns. A question's own call is recorded and counted too.

### The map

- **Principal components, written here.** The Rust UMAP and t-SNE crates looked at bring a
  dozen dependencies or are no longer kept up; a projection on the two directions of greatest
  difference needs none and gives the same map for the same input. It loses close groups,
  which can be drawn over each other, so the list of nearest documents beside the map is worked
  out from the full vectors.
- A click chooses a point; Enter or a double click opens the document. Opening at a click
  would have kept pointer users from the list of neighbours.

### A passage corrected by a model

- **No job.** A request is a row of its own tied to the result reviewed, so the sidebar gets
  no row for each snippet; it still goes through the engine, with the correction task's own
  request, recorded before and after, cached, its cost counted with the job's.
- **A proposal, never applied by itself**, written into the researcher's corrections only on
  accepting, and refused if the text changed meanwhile.
- **Context is sent and marked**: 300 characters each side from the same piece. 4,000
  characters at most, so the answer comes back whole.
- On macOS, WebKit opens a context menu when the button goes down, so the menu waits for it to
  come up.

### Items added to a list answer

- **To every set-up that read the document**: an addition is a fact about the document, not
  about one model's answer.
- **Nothing new is stored**: what the researcher wrote is the corrected list less the model's
  own, so a renamed name is marked as theirs as well, which is new.
- Agreement between models is unchanged: it compares their own answers.

### The guide

Nine chapters in `docs/guide`, in a layout of their own, published by the GitHub Pages that
already serves `docs` (the privacy policy's address). No new tool: Jekyll's layouts and
Markdown, built with GitHub's own image before committing.

## Consequences

- A project's text can leave the computer for a new purpose, to a provider the researcher
  chooses and agrees to; the privacy policy says so.
- Nothing here has been tried with a cloud model, in the packaged app, or on a Mac. In CI the
  real EmbeddingGemma 2 embedded two passages and a question on Windows x64, Windows ARM64 and
  macOS, and the question was nearer the passage that answers it.
- Left: a projection that keeps groups apart; the map following the search's filters;
  analyses' answers, images and sound; the dictionary's labels offered while adding; a pending
  proposal shown again when the review reopens.
- The llama.cpp workflow's pin step failed on the repository's formatter, which names a plugin
  the job does not install; the pin was written by hand from the release's checksums and the
  step now gives the formatter its settings.
