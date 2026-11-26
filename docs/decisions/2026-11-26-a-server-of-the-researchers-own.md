# 2026-11-26 · A server of the researcher's own: Ollama first, then any OpenAI-style address

## Context

Kalamu reaches four companies' services at addresses written as constants, and runs a few small
models itself. Universities and computing centres run models for their members, usually behind
an OpenAI-style address, and some researchers may send their material only there; others run
Ollama on a stronger machine in the office
([#283](https://github.com/fmadore/Kalamu/issues/283), first of the new directions of the
[review of 25 November](2026-11-25-review-safety-nets.md)). A colleague's Mac with Ollama is the
first real server to try.

What the code holds today, checked on 26 November:

- The engine's providers are fixed at launch (`Engine.providers`, no setter); only the model
  list can be replaced while the app runs (`Engine::set_catalog`).
- Every picker offers the models of the catalogue the engine holds, each for the tasks and the
  types of file its entry names (`task_rules`, `piece::accepted_inputs`). A model outside the
  catalogue is never offered; a refresh replaces the catalogue whole.
- "Offline or cloud" is one answer, `Provider::needs_key()`: it decides consent, the `engine`
  of provenance and whether a price is shown. The interface reads `ProviderInfo.local` for the
  same question.
- Provenance names a provider and a model, never an address; a result's cache key likewise.
- `openai.rs` speaks the Responses API. Chat Completions, which such servers speak, is spoken
  only by `mistral/chat.rs`.

What the servers do, read in their documentation and source on 26 November (Ollama 0.40.18):

- **Ollama cuts a request that is too long, and says nothing.** Its context is 4k, 32k or 256k
  tokens by the machine's memory. Past it, older messages are dropped, or the middle of a long
  text is removed, with a line in the server's log and nothing in the answer
  ([ollama#3839](https://github.com/ollama/ollama/issues/3839), still open). Its OpenAI-style
  address (`/v1/chat/completions`) cannot set the context nor refuse the cut. Its own address
  (`/api/chat`) can: `options.num_ctx` sets the context, and `truncate: false` with
  `shift: false` turns every overflow into an error 400.
- Ollama says what each model can do: `/api/show` lists `capabilities` (`vision`, `thinking`,
  `audio`, `embedding`…) and the model's trained context; `/api/tags` lists the models, and a
  `remote_host` for those that run at ollama.com rather than on the server.
- OpenAI-style servers differ. vLLM and llama.cpp refuse a request that is too long; OpenRouter
  compresses its middle on small models unless told not to; LM Studio's behaviour is not
  documented. Some say a model's context in their list (vLLM `max_model_len`, OpenRouter
  `context_length`, llama.cpp `meta.n_ctx_train`), none in the same field. Ollama's `/v1`
  ignores `max_completion_tokens`; `max_tokens` is read by all.
- Ollama listens on `127.0.0.1:11434`, without a key and in plain HTTP; on a network it is
  reached once its owner sets `OLLAMA_HOST`.

## Decision

### What the researcher does

One tab in Settings, **Other servers**, the only place the feature shows until a server is
added. Three steps, and nothing to describe by hand:

1. **Name and address.** A name of the researcher's choosing, an address
   (`http://localhost:11434` shown as the example), and a key only under *This server asks for
   a key*. *Connect*.
2. **Kalamu asks the server what it is and what it has.** `GET <address>/api/version` answered
   means Ollama; else `GET <address>/models`, then `<address>/v1/models`, means an OpenAI-style
   server; else the failure is said in plain words (nothing answers there; it answers but is
   not a server Kalamu knows; it refuses the key). No kind to choose. The models are listed,
   and the researcher ticks those to use.
3. **Kalamu tries each ticked model before it is listed** (*Try and add*), with three small
   requests: an answer at all; a small picture bundled with Kalamu that holds a number, which
   the answer must contain; a form of two fields, which must come back filled. Each model then
   shows what it does in two lines, *Reads pages* and *Fills an analysis's form*, each yes or
   no.

**A model is offered only for what it did in its trial.** Reading pages (text extraction, and
a region read again) needs the picture read; analyses, contents, corrections and translations
need the form filled; a model that did neither is not listed. Its catalogue entry is written
from the trial (`tasks`, `inputs`), so New job, workflows and every dialog obey it through the
rules the four providers' models already obey, and the engine converts a page to a picture or
refuses a type of file as it does for them. The researcher cannot tick an ability; *Try again*
repeats the trial, after a model was updated on the server. Where Ollama says a model has no
`vision`, the picture is not sent and the answer is no.

**Transcription is not in this increment**, and no model of such a server is offered for it.
The same rule is ready for it: a model would be offered for recordings only after a short
bundled clip came back transcribed. It waits for a real server to try, since Ollama's `audio`
models and the `/v1/audio/transcriptions` of other servers return neither times nor speakers
in one agreed form. Whisper, in Kalamu itself, covers the need meanwhile. Also left out:
batches (economy mode), embeddings for the search by meaning, and the models Ollama runs at
ollama.com (`remote_host` set), which are shown in the list as not on this server and cannot
be ticked: a local address must not send a page to a company unseen.

Under *More choices*, per model, each with a value Kalamu filled: *Reads up to* (tokens),
*Longest answer* (tokens), *Price* per million tokens in and out (none by default, and the
job's cost then reads as not priced); per server, *Requests at the same time* (1 by default).

### Never a reading in part

- **Ollama is spoken at its own address**, `/api/chat`, never `/v1`. Every request carries
  `options.num_ctx` (the model's *Reads up to*), `truncate: false`, `shift: false`,
  `stream: false`, `options.num_predict` (the *Longest answer*) and `think: false`. An overflow
  is then an error 400, said as: the page or text is longer than this model reads on this
  server, with where to raise it. `done_reason: "length"` keeps the text marked as cut off, as
  for every provider.
- *Reads up to* starts at the model's trained context, capped at 32 768 tokens, since a larger
  context takes memory the machine may not have; *Longest answer* at 8 192, or a quarter of the
  context if smaller. The catalogue entry's `max_input_tokens` is the first minus the second,
  so the engine's own checks refuse or split a long text before anything is sent.
- **An OpenAI-style server** is sent `max_tokens`, and no field that only some accept. Its
  *Reads up to* is taken from the server's list where it gives one, else 8 192 until the
  researcher changes it. After each answer of a request that carried a text, the tokens the
  server says it read (`usage.prompt_tokens`) are compared with the text sent: fewer than one
  token for eight characters is refused as a reading in part, since no language is written so
  compactly. An answer without `usage` is accepted, and the guide says that such a server is
  trusted on its word. Requests to `openrouter.ai` turn its compression off.
- Thinking is kept out of the answer: `message.thinking`, `reasoning` and `reasoning_content`
  are never read, and a leading `<think>…</think>` is removed. No reasoning level is offered
  for these models in this increment.
- A model reads one page a call.

### On this computer, or on the network

A third kind beside offline and cloud, in two halves, decided by the address alone:

- `localhost`, `127.0.0.0/8` and `::1`: **on this computer**. Nothing leaves it, no consent is
  asked, and an offline-only project ([#282](https://github.com/fmadore/Kalamu/issues/282))
  may use it.
- any other address: **on the network**. Consent is asked once, naming the server by the
  researcher's name for it and its host, in words of its own (the files go to a server someone
  runs, under that person's rules, not to a company under the researcher's account). Changing
  the address asks again.

Kalamu knows only the address: what a program at `localhost` does with a page is its owner's
affair, and the privacy policy says so. Plain `http://` is accepted, Ollama speaking nothing
else; for an address on the network Settings says that the files travel unencrypted, and with
a key, that the key does too. Only `http` and `https`, without a user name in the address.

Provenance records it: `engine` is `own`, and a new `server` holds the name and the address as
they were when the piece was read, so a result opened later, or on another computer, still
says where it was made.

### Where it lives

- **`core`**: `Reach` (`ThisComputer`, `Cloud`, `Network`), asked through
  `Provider::reach()`, whose default follows `needs_key()` so nothing existing changes; the
  engine and the shell ask it wherever they asked `needs_key()` to mean "sends something
  out". `Provider::server()` gives the name and address for provenance, `None` by default.
  `needs_key()` keeps its one meaning: a key must be there.
- **`providers`**: one module, `own`, with one file per way of speaking (`ollama.rs`,
  `chat.rs`), the finding of a server and its models (`discover`), and the trial (`trial`).
  One provider per server added, `own-<8 hex digits>` made once and kept through renames and a
  changed address, so results, the cache, consent and the key (`own-…-api-key` in the
  keychain) hang on it. Its failures name the server by the researcher's name, never "the
  provider", and a server that does not answer is not blamed on the internet connection. Its
  own limit on requests at the same time, since a waiting request would otherwise run out its
  time in the server's queue. `own::providers(client, &servers)` is the registration line.
- **`catalog`**: `OwnServer` and `OwnModel`, the researcher's entries, kept in `servers.json`
  beside the cached model list, never refreshed; and `Catalog::with_servers`, which adds each
  server as a provider (with `own`: its address, whether on this computer, whether it holds a
  key) and each tried model as a model. They are added after the list's checks, which hold a
  downloaded list and say nothing of what the researcher typed, and again after every refresh:
  the shell has one function that hands the engine a catalogue, and it merges.
- **`engine`**: `Engine::set_providers`, as `set_catalog`, so a server is usable the moment it
  is added. A job in flight keeps the provider it started with.
- **`app`**: `servers.json` read and written, the commands (find a server, try a model, save,
  remove), and the wiring. As built: `core` also holds `Dialect` and `checked_address` (with
  the `url` crate), so the address discovery keeps and the address `servers.json` may hold are
  one rule; `catalog` holds what Settings sends and receives (`FoundServer`, `TrialServer`,
  `ModelTrial`, `ServerDraft`) and `OwnServers::apply`, which takes each model's abilities from
  a trial made in the session at that address, or the one stored while the address stays, never
  from the draft. The shell maps the stored entries to the providers' configuration, one
  function for the catalogue (`use_catalog`) and one for the providers. Consent is withdrawn
  for a server whose address changed or that was removed, and `save_settings` no longer adds
  consent back from the copy it is sent. A saved pipeline or workflow that names a server since removed
  fails as an unknown provider does today, in plain words.
- **Interface**: the tab; in New job a server is a provider like the others, under its name.
  No expert mode.

### Sharing with the app driven in CI

The packaged app's tests ([#277](https://github.com/fmadore/Kalamu/issues/277)) need a
provider at an address that is a setting. This is it: the stand-in gains the two ways of
speaking, and the packaged app is driven by adding it as a server. That work follows this
increment; `Addresses` stays for the test build, which redirects the four providers.

## Kalamu differs deliberately

- From `iwac-ai-pipelines` and `zmo-ai-pipelines`, which name each service in code and trust a
  model to read what it is sent: here a model earns each task in a trial.
- From clients that speak `/v1` to everything, Ollama included: that address cannot refuse a
  cut.

## Consequences

- A change of the specification (providers, privacy, what each task accepts), the guide in
  both languages (a page of its own, *Other servers*), `docs/privacy.md` with a new effective
  date, the roadmap and `AGENTS.md`'s table.
- `Provenance` gains a field and `engine` a value; both are added, nothing stored is rewritten,
  so no fixture database is owed.
- Two servers' models under the same name are apart in the cache, their providers' ids
  differing. A model updated on the server under its old name keeps its cached answers, as an
  alias at a cloud provider does; provenance records the digest Ollama gives.
- To try by hand, and to write down with what failed: a job of pages and an analysis against
  Ollama on a Mac, on this computer and across a network; a model too small for its page; a
  model updated between two runs. The OpenAI-style way is written from documentation and
  recorded answers until a university's server is tried.
- Later, each with its own entry: transcription, embeddings, reasoning levels, Ollama's
  ollama.com models with consent, and the services that need their own sign-in (Azure OpenAI,
  Vertex AI, Bedrock).

Built differently from the decision above, as found when the documents were written on
26 November:

- **Provenance records the server as it stands when the files are written**, not as it was
  when each piece was read (`Engine::where_read`, `Engine::server_of`). A record per piece needs
  a stored field on each result and a migration of the job database, with a fixture for CI;
  it is left for later. Until then, a server renamed or moved between reading the pages and
  writing the files is named in `provenance.json` by its new name and address. A server no
  longer registered when the files are written (removed) leaves `engine` as `cloud` and no
  `server`, as for any provider the engine does not know.
- **The digest Ollama gives is not in provenance.** `/api/chat`'s answer carries none, so a
  page read by Ollama records no model version; the digest `/api/tags` gave at the trial is
  kept with the model in `servers.json` (`OwnModel::digest`), where it tells a model updated
  under the same name, not in the files.
- **A model's catalogue entry lists `TextExtraction` and `Analysis` only.** Contents,
  corrections and translations follow from `Analysis` through `Task::request_task`, as for
  every model; a table of contents' pages need pictures too, so its reading step is offered
  only to a model that did both.
- **A job in flight takes the providers as they are at each piece**, not as they were when it
  started: the call already sent keeps its provider, and the next piece looks the list up
  again. A server removed stops the job at its next page as interrupted, the pages read kept;
  added again in Settings, it is a new server under a new id, which the job does not name. A
  server whose address changed while its job runs sends the job's remaining pages to the new
  address.
- **Consent is asked when a job, a workflow run or a dictionary proposal is created**
  (`require_consent`), not when a job is started, resumed or retried. With consent now
  withdrawn when a server's address changes, a job made before the change, then resumed or
  retried, sends to the new address without the question. To close before release: ask again,
  or refuse, when a job that names a server not agreed to is started or retried.
- **`save_settings` keeps the consent the shell holds** and ignores the copy it is sent:
  consent is given by `consent_providers` alone and withdrawn only by a server's change of
  address or removal. This holds for the four providers too: saving settings can neither add
  nor remove an agreement any more.
