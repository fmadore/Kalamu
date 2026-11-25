# 2026-11-25 · A scheduled live check of each provider, under a cost ceiling

## Context

Everything Kalamu sends is tested against recorded answers, and the providers' interfaces move
(Claude's version header, Gemini's `v1beta`, OpenAI's Responses). The only check against the real
services was `cargo run --example live`, run by hand with the keys in the keychain and no limit on
what it spent, and the roadmap's *Live runs still to do* had grown with every release since 3
November. The review of 25 November ([decision](2026-11-25-review-safety-nets.md)) made a
scheduled check one of the six safety nets (#278). Its first task is to catch real 429 answers,
which #270 waits for.

## Decision

- **Keys from the environment first.** `KALAMU_<PROVIDER>_API_KEY` (`KALAMU_GEMINI_API_KEY`,
  `KALAMU_OPENAI_API_KEY`, `KALAMU_ANTHROPIC_API_KEY`, `KALAMU_MISTRAL_API_KEY`) when set and not
  blank, else the app's keychain entry, unless `LIVE_KEYCHAIN=off`, which the workflow sets.
  Keys are read once at the start. The tool never prints one, and every text it prints or writes
  (console, summary, fixtures) goes through one redactor. It replaces the keys it holds with
  `[key]`, and words shaped like a key (`sk-…`, `AIza…`) or an identifier (`req_…`, `batch_…`,
  UUIDs, numbers of ten digits or more such as a Google project's, long runs that mix letters and
  digits) with `[key]` or `[id]`. Model names stay.
- **The ceiling is kept twice, because a call's cost is known only afterwards.** Before a step
  starts, its estimate (New job's, or its economy price for a batch) must fit in what is left,
  with the steps still running counted at their estimates. A step with no price does not start
  under a ceiling. As soon as what was billed (the catalogue's prices on the usage reported, as
  the engine counts it) passes the ceiling, every job running is cancelled and nothing more
  starts. Calls already in flight finish and are billed, so a run can end above its ceiling by
  what those calls cost (at most four pieces, the runner's concurrency), but never by a call
  started after the ceiling was passed. A step stopped by the ceiling fails the run, and so does a
  run that ends past it. Without a ceiling, a run at a terminal asks first, and one with nobody at
  the terminal refuses to start.
- **The weekly set, chosen by fields.** For each provider there are five steps: one page read
  (`bulletin-page.jpg`), one 20-second recording (`interview.wav`), one analysis of people, places
  and organisations (`bulletin.txt`), the same analysis sent as a batch and collected, and one
  embedding of 1,000 characters. Each step uses the model with the lowest estimate among those the
  catalogue offers for new work (no successor, not retired, not deprecated) that do the task and
  read the sample's type. For the batch, the candidates are the models whose work the provider
  sends as a batch. Embeddings use the cloud model with the lowest price per token. No model is
  named in the code that chooses. With the catalogue of 24 November (revision 1):

  | Provider | Page | Recording | Analysis | Batch | Embedding |
  | --- | --- | --- | --- | --- | --- |
  | Gemini | 3.5 Flash-Lite $0.0035 | 4 Hydrogen $0.0015 | 3.5 Flash-Lite $0.0040 | 3.5 Flash-Lite $0.0020 | Embedding 2 $0.00005 |
  | OpenAI | GPT-6 Luna $0.0006 | GPT Transcribe $0.0016 | GPT-6 Luna $0.0008 | GPT-6 Luna $0.0004 | 3-small $0.000005 |
  | Anthropic | Haiku 5.5 $0.0009 | none | Haiku 5.5 $0.0008 | Haiku 5.5 $0.0004 | none |
  | Mistral | OCR 4.1 $0.0040 | Voxtral Mini $0.0013 | Small 2603 $0.0005 | Small 2603 $0.0003 | Embed $0.00003 |

  That comes to **$0.023 a run, $1.17 a year**. The weekly ceiling is **$0.05**, about twice the
  estimate, because short analyses are overestimated and reasoning models have been billed up to
  1.7 times theirs (3 November). `LIVE_PLAN=1` prints this table again from whatever catalogue the
  tool is built with, reading no key and sending nothing.
- **A batch is waited for, up to a bound, and then cancelled.** Batches are sent first and run
  alongside the other steps. One not answered within 60 minutes (`LIVE_BATCH_WAIT`) is cancelled
  at the provider, through the engine's own cancel, which bills nothing for what is unanswered.
  It reads *late* and does not fail the run, because the providers promise 24 hours (OpenAI
  answered none in 35 minutes on 3 November). Collecting last week's batch instead was weighed and
  left aside. It would need the job database to live from one run to the next, in a cache or an
  artifact, holding the samples' text and the batches' identifiers. A provider whose batch is late
  week after week shows it in the summary.
- **The embedding is one direct call to the provider's embedder.** The search by meaning's own
  path through the engine needs a project, its search index and the researcher's consent, which is
  the app's to test. The call checks the request and answer shape and is costed and kept under the
  ceiling like the jobs. The jobs themselves all go through the engine.
- **Secrets stay away from build code (#145).** This workflow keeps the rule rather than being an
  exception to it. `build` compiles the tool on `main` with a read-only token, no secret and no
  environment, and uploads the program as an artifact that lasts one day. `check` is the only job
  that names the `live` environment. It checks out `samples/` alone, downloads that program and
  runs it: no `npm ci`, no cargo, no build script, and a read-only token. The keys reach only the
  step that runs the program, as variables GitHub masks. The workflow inputs reach it as variables
  too, never written into the script. `report` has `issues: write`, no secret, and runs `gh`
  alone.
- **Never from a pull request.** There is no `pull_request` trigger. Every job checks
  `github.ref == 'refs/heads/main'`, and the `live` environment admits only `main` (in the
  repository's settings), so a branch that rewrote the workflow would still get no secret. Fork
  pull requests never receive environment secrets. The secrets belong to the environment, not
  the repository, so no other workflow sees them.
- **One issue.** A weekly run that fails (build or check) opens the issue titled *Weekly live
  check failed*, found by that exact title among the open issues, or comments on it if it is
  already open. Each comment holds the run's link and its summary. The next weekly run that passes
  comments and closes it. A `roadmap` run never reports there: some of its failures are what it
  was run to find.
- **Error answers kept as fixtures, through a relay.** With `LIVE_FIXTURES`, each provider is
  given a local relay's address (the `with_base_url` the crates' tests use). The relay sends each
  request on unchanged with the app's own HTTPS client and hands the answer back unchanged. Any
  answer with a status of 400 or more is also written as `<provider>_<status>.json` (the shape of
  `crates/providers/tests/fixtures/`), with its status, path and waiting headers in `index.json`,
  all redacted. The relay speaks HTTP/1.1 over Tokio's own sockets, so no dependency was added. Two
  things differ from the app: an error read through the relay names "The provider" rather than
  "Google" or "Mistral" (`answered_by`), and the relay keeps the routes in its own table. A test
  fails when the registry lists a provider that table lacks. Without `LIVE_FIXTURES`, the tool uses
  the app's registry as it is. The workflow uploads the folder as `live-fixtures` for 90 days, and
  development.md says how one is committed. Success answers are not kept, because they hold the
  samples' text.
- **The larger set**, `LIVE_SET=roadmap`, maps the roadmap's *Live runs still to do* onto runs:

  | Item | Run |
  | --- | --- |
  | Offline Whisper | `roadmap-offline-whisper`, with `LIVE_WHISPER` and FFmpeg |
  | A full Tesseract job with networking off | `roadmap-offline-tesseract`, with `LIVE_TESSDATA`; turning the network off is the person's part |
  | GLM 5.3 through a whole job | `roadmap-glm-whole-job`: three analyses of one text in one job |
  | GPT-6.1 Terra on handwriting and older print | `roadmap-terra-handwriting-print`: the Dodington report of 1755, the Daily Nebraskan of 1911 |
  | GPT-6.2 Sol and Astra on real pages | `roadmap-sol-astra-pages` |
  | Gemini 4 Neon against Hydrogen on handwriting, older print and refusals | `roadmap-neon-hydrogen` (Gemini has refused the Dodington page) |
  | A table of contents mixing providers (Gemini reading, Mistral combining) | `roadmap-contents-mixed` |
  | A correction mixing providers | `roadmap-correction-mixed`: Gemini and Mistral side by side on the 1911 page's text layer, as the item is read here |
  | A table of contents in economy mode, Claude Sonnet 5.5 at `low` and higher | `roadmap-contents-economy`, at `low` and `medium` |

  The run makes the calls, keeps the outputs and sets each cost against its estimate. Whether a
  hand was read well, every article was found, or a refusal was right is a person's judgement,
  read from the outputs. Not automated: Claude's one-hour cache, which needs a batch of a few
  hundred pages and a judgement of whether it is needed. Nor are the test pass's items that need
  the interface: the search by meaning's preparation and its bill, a region read again, a passage
  corrected, an hour's interview analysed, a batch's files deleted at each provider as the
  interface shows it, and Gemini's batch sent as a file, which needs more than 20 MB. The roadmap
  set is estimated at $0.55 (Sol and Astra $0.20, Neon and Hydrogen $0.13, the economy table of
  contents $0.12); start it with a ceiling of $1.50.
- **The `release` set** keeps the plans run since 3 November unchanged. A job whose files are
  missing, a set-up whose model the catalogue no longer lists, or a provider without a key is now
  left out and said so, where before the whole run stopped.

## Consequences

- The first weekly run is the maintainer's: until it has passed on `main`, nothing here has met
  a real service. It needs the `live` environment and its secrets (development.md). Keys made for
  the check alone, with a monthly limit at each provider, cap what a mistake can spend beyond the
  ceiling.
- The real 429s #270 waits for arrive as `live-fixtures`. A test per provider with the real
  wording follows once one is committed.
- What the app sends, to whom, and what it keeps are unchanged. This is a developer's tool run
  with the maintainer's keys on the repository's sample files, so `docs/privacy.md` needs no
  change.
- When the catalogue's prices change, the weekly estimate moves with them. `LIVE_PLAN=1` gives
  the new figure, and the default ceiling in `live.yml` should follow if the estimate passes half
  of it.
- Once #277 gives the registry an override for the providers' addresses, the relay's table can
  use it instead of building the four providers itself.
