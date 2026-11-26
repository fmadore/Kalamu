# Development

How to build, check and preview Kalamu. The architecture, crate responsibilities and
coding rules are in [AGENTS.md](../AGENTS.md).

## Requirements

- Rust through rustup; the version in `rust-toolchain.toml` installs itself.
- Node.js 22 or 24.
- On Windows, the WebView2 runtime, which ships with Windows 11. On macOS (Apple Silicon), the
  Xcode command-line tools (`xcode-select --install`) and CMake; nothing else needs setting.
- On Windows, for offline transcription and text extraction, which compile whisper.cpp and
  Tesseract with Leptonica (through `kreuzberg-tesseract`; on ARM64 with
  `crates/offline/cmake/windows-arm64.cmake`, named in `.cargo/config.toml`): LLVM (`clang-cl`), CMake and Ninja
  (the Visual Studio Build Tools include the last two), with these variables set, as in CI:
  `CC=clang-cl`, `CXX=clang-cl`, `CXXFLAGS=-EHsc`, `CMAKE_GENERATOR=Ninja` and
  `LIBCLANG_PATH=C:\Program Files\LLVM\bin`.
- Only to build Whisper's Vulkan helper (`helpers/whisper-vulkan`, Windows releases): LunarG's
  [Vulkan SDK](https://vulkan.lunarg.com/sdk/home), which sets `VULKAN_SDK`, then
  `npm run build:whisper-vulkan -- <target>`, which builds in `\wv` at the root of the drive
  (`WHISPER_VULKAN_TARGET_DIR` chooses another): under `target\<triple>\release`, the folders
  ggml makes for its Vulkan shader generator pass Windows' 260-character path limit. Everything
  else builds without it; `--workspace`
  commands then take `--exclude whisper-vulkan`. On Linux the helper builds with the
  distribution's Vulkan headers and `glslc`. To run Whisper through it from a development build,
  copy `src-tauri/binaries/whisper-vulkan-<target>.exe` beside the app as `whisper-vulkan.exe`.
- To run Whisper on a real recording: `WHISPER_SMOKE_MODEL=<ggml-base-q5_1.bin>
  WHISPER_SMOKE_VAD=<ggml-silero-v5.1.2.bin> cargo test -p kalamu-offline whisper::tests::smoke
  -- --ignored --exact --nocapture`, as CI does; `whisper::tests::vulkan_smoke` does the same
  through the helper named by `WHISPER_VULKAN_HELPER`.
- For recordings and videos during development, an FFmpeg on `PATH` or in the folder
  `KALAMU_FFMPEG_DIR` names (releases bundle their own LGPL build and look nowhere else; see
  [ffmpeg.md](ffmpeg.md)); llama.cpp's programs likewise on `PATH` or in `KALAMU_LLAMA_DIR`
  ([llama.md](llama.md)).

## Run

```bash
npm install
```

```bash
npm run tauri dev
```

## Checks

Run before every commit; CI runs the same.

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

```bash
npm run format:check && npm run check && npm run lint && npm test
```

What differs between Windows and macOS is in `src-tauri/src/platform/` and `src/lib/platform.ts`
(see [AGENTS.md](../AGENTS.md)). The macOS file is not compiled on Windows, so CI's macOS lane is
where it is checked. The interface's main screens are also opened in WebKit, the engine of the
macOS webview, on a Mac in CI, and in Chromium, the engine of Windows' WebView2; on any
machine, after `npx playwright install webkit chromium`:

```bash
npm run test:webkit
npm run test:chromium
```

Both open every main screen in English and in French, and run a few interactions
(`e2e/interactions.spec.ts`): a correction saved and found again, a dialog's focus trap and
Escape, the window's shortcuts, a project search, New job kept across Settings; and what is done
from the keyboard alone (`e2e/keyboard.spec.ts`): a text page reviewed and corrected through its
readings, an analysis's name located, renamed, removed and put back, a failure dismissed, Back to
where a review was opened from, Ctrl F on each table, the remembered choices after a reload, a
window of 640 × 560, the release line hidden; and records (`e2e/records.spec.ts`): an analysis
turned into one row per entry, a page's rows corrected, added, removed and reviewed; and an
interview analysed (`e2e/interview-analysis.spec.ts`): its dialog's one box and one button, its
cards, a quotation played or shown in the text, removed and put back, a description of the
research kept as a prompt and chosen again, a transcript too long for the model chosen; the
first launch (`e2e/first-launch.spec.ts`): Settings and its welcome with neither a key nor a
model, New job with either, the guide's link in each language; an empty project
(`e2e/empty-project.spec.ts`): its orientation, gone once a document is added; and the default
prompts' fields in English and in French (`e2e/default-fields.spec.ts`). They
start the dev server on port 5173, or reuse one already there; `KALAMU_E2E_PORT=5291` picks
another, for two checkouts tested side by side.

`cargo test` also regenerates the TypeScript types in `src/lib/bindings/` from the Rust
definitions; commit them with the Rust change. Exporters are checked against golden files in
`crates/export/golden/`; after an intended change, rerun their tests with `UPDATE_GOLDEN=1`
and review the diff.

## The app driven

The built app itself, driven against a stand-in provider: what `e2e/` does against the dev
server's mock, but through Rust, the job database and the files written
([decision](decisions/2026-11-25-app-driven-in-ci.md)). CI runs it in the *App driven* workflow
on pull requests that touch the shell, the engine, the providers, the store or the core, and on
every release candidate's tag.

It drives a **test build**: a debug build with the `test-overrides` feature, which a release
build refuses to compile. Given `KALAMU_TEST_DATA`, it keeps its data there; given
`KALAMU_TEST_PROVIDERS`, it reaches every cloud provider under that address with a key of its
own; it never asks or writes the keychain, answers its native dialogs from
`dialog-answers.json` in its data folder, and runs beside an installed Kalamu. The stand-in,
`scripts/app-driven/stand-in.mjs`, answers with the providers' tests' recorded answers and
calls no real service.

On Windows (x64 or ARM64), with WebView2:

```bash
npm run build
cargo build -p kalamu --features custom-protocol,test-overrides
npm run test:app
```

`npm run test:app` starts the stand-in and the app for each flow (`e2e-app/flows.spec.ts`), each
in a temporary data folder with copies of the sample files, and drives the window over the
DevTools protocol the test build has WebView2 open (`KALAMU_TEST_DEVTOOLS_PORT`): a job from New job
to its files, a correction found after a restart, a project saved and opened, a paused job
resumed after a restart, a batch collected after a restart. The window shows while it runs;
leave it alone. `KALAMU_APP` names another executable. A failed flow keeps its trace in
`test-results/app` (`npx playwright show-trace <trace.zip>`). On a Mac there is no driver for
WKWebView: `scripts/app-driven/launch-check.sh <Kalamu.app>` starts a test build bundled with
`npx tauri build --debug --bundles app --features test-overrides`, checks that it opens its
database, and quits it.

To try the test build by hand, start the stand-in (`node scripts/app-driven/stand-in.mjs`, on
port 4310) and the app with both variables set. Its pickers open nothing: write what they
should answer into `dialog-answers.json` first (`[["C:\\path\\to\\page.jpg"]]` for one file
chosen), or drop files on the window. In PowerShell:

```powershell
$env:KALAMU_TEST_DATA = "$env:TEMP\kalamu-test"; $env:KALAMU_TEST_PROVIDERS = 'http://127.0.0.1:4310'
.\target\debug\kalamu.exe
```

## A server of one's own, by hand

A server of the researcher's own (Settings › *Other servers*,
[decision](decisions/2026-11-26-a-server-of-the-researchers-own.md)) is tested against answers
written from Ollama's documentation and source (Ollama 0.40.18, 26 November 2026); no real
server has answered it yet. To try it, on a Mac or a Windows PC, with [Ollama](https://ollama.com)
installed:

```bash
ollama serve                 # not needed when the Ollama app is running
ollama pull gemma3:4b        # reads pictures
ollama pull gemma3:270m      # text only, small
```

In Kalamu, add the server at `http://localhost:11434`, tick both models and try them: the first
should read pages and fill the form, the second fill the form at most. Across a network, start
Ollama on the other computer with `OLLAMA_HOST=0.0.0.0:11434` (for the Mac app,
`launchctl setenv OLLAMA_HOST 0.0.0.0:11434`, then quit and reopen Ollama) and add it here at
`http://<its address>:11434`: consent is asked, naming that host. Then run, writing down what
fails: a job of pages and an analysis, on this computer and across the network; a model too
small for its page (lower what the model reads, in Settings, until a page with its prompt no
longer fits, a few hundred tokens for `gemma3`, which counts a picture as about 256: the page
must fail with where to raise the limit, never come back short); a model updated between two
runs (`ollama cp` another model under the first one's name, then run again and try it again).

Write down from the first run, in the decision, the three points the answers were written
without:

- that a model without thinking (`gemma3`) accepts `think: false`: its trial's first request
  passes;
- that `/api/chat` reads `truncate` and `shift` at the top level of the request: the page too
  long fails with an error 400 instead of being answered. Outside Kalamu, the same check:
  `curl http://localhost:11434/api/chat -d '{"model":"gemma3:270m","stream":false,"truncate":false,"shift":false,"options":{"num_ctx":256},"messages":[{"role":"user","content":"<a text of a few thousand words>"}]}'`;
- the exact words of Ollama's error for a text too long, shown in brackets in the page's failure
  and in Ollama's log, so `overflows` in `crates/providers/src/own.rs` reads them and the
  fixture `crates/providers/tests/fixtures/own/ollama_overflow.json` is replaced by the real
  answer.

## Live checks

The live check (`src-tauri/examples/live/`) runs real jobs against the providers' services, through
the engine as the app runs them. It spends money and needs keys: CI's tests never run it, only
its own tests against local stand-ins. It runs once a week on GitHub, and by hand, there or on a
computer:

```bash
LIVE_SET=weekly LIVE_CEILING=0.05 cargo run -p kalamu --example live -- samples <out> [step name filter]
```

**Sets** (`LIVE_SET`):

- `weekly`: per provider, one page read (`bulletin-page.jpg`), one short recording
  (`interview.wav`, 20 seconds), one analysis (`bulletin.txt`, people, places and organisations),
  the same analysis sent as a batch and collected, and one embedding (the first 1,000 characters
  of `bulletin.txt`), where the provider offers each. Each step takes the cheapest model the
  catalogue offers for it (neither replaced, retired nor deprecated, reading the sample's type),
  by its estimate; a batch, the cheapest whose work the provider sends as one. A model without
  an estimate is never chosen, since the ceiling could not hold it; a step none of whose models
  has one is left out and says why. Without FFmpeg (the runner on GitHub has none), a recording
  is sent whole and priced by the length its WAV header gives.
- `roadmap`: the roadmap's *Live runs still to do* that a run can make, on `samples/`
  ([decision](decisions/2026-11-25-scheduled-live-check.md) for which). Judging the answers is
  yours: each job's outputs are in its own folder in `<out>`.
- `release` (the default): every task end to end before a release, on a folder holding
  `newspaper.jpg`, `letter.pdf`, `interview.wav` and `article.txt` (`article2.txt` and
  `article3.txt` for `analysis-batch`, `image.jpg`, `page-1.jpg`, `page-2.jpg`, `page-10.jpg`); a
  job whose files are missing is left out.

**What it spends.** At the catalogue's prices of 26 November 2026, the weekly set is estimated at
**$0.023 a run, $1.17 a year**, recordings included; `LIVE_PLAN=1` prints each step's model and
estimate and the total, reading no key and sending nothing, so run it again when the catalogue
changes. The weekly run's ceiling is **$0.05**, about twice the estimate, since a short analysis is
overestimated and a reasoning model can write more than its estimate. The roadmap set is estimated
at $0.55; give it $1.50.

**The ceiling** (`LIVE_CEILING`, in US dollars at the catalogue's prices). A step starts only if
its estimate (the economy price for a batch) fits in what is left, the steps still running
counted at their estimates; a step without a price does not start. Once what was billed passes
the ceiling, the jobs running are cancelled and nothing more starts; the calls already in flight
finish and are billed, so a run can end above its ceiling by those calls, never by a call started
after it. A step not started or stopped by the ceiling fails the run. Without a ceiling, a run at
a terminal asks before it starts; one with nobody at the terminal (no terminal, or `CI` set)
refuses to start.

**Keys.** `KALAMU_GEMINI_API_KEY`, `KALAMU_OPENAI_API_KEY`, `KALAMU_ANTHROPIC_API_KEY` and
`KALAMU_MISTRAL_API_KEY` when set and not blank; otherwise the key saved in the app's Settings
(Credential Manager, the Keychain), unless `LIVE_KEYCHAIN=off`. A provider without a key is left
out and said so. Keys are read once, at the start; no key is printed, and every message the tool
prints or writes has the keys it holds, and words shaped like keys or identifiers, taken out.

**What it leaves.** A table per step and provider (model, estimate, what the catalogue's prices
put on the usage reported, their ratio, the result) is printed, written to `<out>/summary.md`
and appended to `LIVE_SUMMARY` when set; it is the 3 November decision's way of noting cost
against estimate, ready to paste into a decision entry. A batch not answered within
`LIVE_BATCH_WAIT` minutes (60 in the weekly set, a day otherwise) is cancelled at the provider
and reads *late*, which does not fail the run. Each job's outputs and the job database are in
`<out>`: they hold the samples' text and never leave the computer or the runner.

Also: `LIVE_SKIP=gemini,openai` leaves out providers; `LIVE_PACE=economy` sends every job of the
`release` and `roadmap` sets as batches; `LIVE_TESSDATA=<folder>` adds the offline Tesseract jobs
(the `fra` and `eng` data), `LIVE_WHISPER=<folder>` offline Whisper (the folder of the app's
Whisper models, with FFmpeg found): both need no key and cost nothing. Use a fresh `<out>`: a
piece already answered in it is taken from its cache and reads *skipped*.

### The weekly run on GitHub

`.github/workflows/live.yml` runs the weekly set on Mondays at 06:17 UTC, and by hand from
**Actions → Live check → Run workflow** on `main`, with the set, the ceiling and the batch wait
to type in. `build` compiles the tool with a read-only token and no secret; `check`, the only
job with the keys, runs the compiled program on `samples/` and nothing else; `report` opens the
issue *Weekly live check failed* when a weekly run fails, comments on it while it stays open, and
closes it once a weekly run passes. The job summary holds the table.

Once, in the repository's settings on GitHub:

1. **Settings → Environments → New environment**, name it `live`, **Configure environment**.
2. Under *Deployment branches and tags*, choose **Selected branches and tags**, **Add deployment
   branch or tag rule**: *Ref type* **Branch**, *Name pattern* `main`. Only runs from `main` then
   get the secrets. No required reviewer (the weekly run would wait for one) and no wait timer.
3. Under *Environment secrets*, **Add environment secret** for each provider to check:
   `KALAMU_GEMINI_API_KEY`, `KALAMU_OPENAI_API_KEY`, `KALAMU_ANTHROPIC_API_KEY`,
   `KALAMU_MISTRAL_API_KEY`. Not repository secrets: those reach every workflow. A provider whose
   secret is missing is left out of the run.
4. Use keys made for this check alone, in a project of their own at each provider where it
   offers one, with a monthly spending limit (a dollar or two) set in the provider's console.

**Rotating a key**: make the new key in the provider's console, replace the environment secret
of the same name (**Settings → Environments → live → the secret → Update**), start the workflow
by hand to see it pass, then delete the old key in the console. A key that may have leaked is
deleted first and replaced after.

### Error answers kept as fixtures

With `LIVE_FIXTURES=<folder>` (as the workflow runs it), every request goes through a relay on a
local port that sends it on unchanged and keeps each answer with a status of 400 or more: its
body as `<provider>_<status>.json` (`_2`, `_3`… for more of the same), the shape of
`crates/providers/tests/fixtures/`, and its status, path and waiting headers (`retry-after`,
the rate-limit counts) in `index.json`, keys and identifiers taken out. The workflow uploads the
folder as the run's `live-fixtures` artifact, kept 90 days. To commit one: download the
artifact from the run's page (or `gh run download <run id> -n live-fixtures`), read the file
through, copy it into `crates/providers/tests/fixtures/` under a name that says what it is
(`gemini_429_per_minute.json`), and write the test that serves it with the status and headers
`index.json` gives, as the 429 tests in `crates/providers/tests/providers.rs` do.

## Preview in a browser

`npm run dev` serves the interface with sample data at `http://localhost:5173/?mock`. Query
parameters open a given state:

| Parameter | Effect |
| --- | --- |
| `view=new`, `jobs`, `prompts`, `settings`, `glossary` | Opens that screen (`jobs` is a job's page) |
| `view=all-jobs` | Opens the Jobs page, with 24 older finished jobs |
| `view=review`, `review-audio`, `review-analysis` | Opens the review of a sample extraction, recording or analysis |
| `view=review-youtube`, `review-contents`, `review-correction`, `review-records` | Opens the review of a YouTube video, a table of contents, a correction or records (a directory's rows, page by page) |
| `view=review-analysis-scan` | Opens the review of an analysis of a scan, where regions can be drawn on its pages |
| `view=review-interview` | Opens the review of an interview analysed for its themes and quotations (an invented conversation in French); a quotation plays in its transcript's review |
| `interview-gone`, `interview-two` | With `review-interview`: the transcript it came from is gone, so quotations are shown in the text; or a second set-up answers beside the first |
| `view=jobs-audio` | Opens the page of a transcription, whose Use this text offers Analyse this interview |
| `research` | With `jobs-audio`: two descriptions of research are already kept as prompts, so Analyse this interview offers them above its box |
| `interview-long` | With `jobs-audio`: the transcript is some 100,000 words long, so Analyse this interview says it is too long for a model on the computer chosen under More choices |
| `reading-slow` | A region read again answers after a minute rather than at once, to try Stop |
| `view=prompts-analysis` | Opens Prompts on an analysis, with its field editor |
| `view=projects`, `project`, `document` | Opens the list of projects, a sample project, or one of its documents |
| `view=workflows`, `workflow`, `run` | Opens the workflows and their runs, a sample workflow, or its run waiting for a review |
| `view=settings-general`, `settings-about` | Opens Settings on its General or About tab |
| `release` | The model list names a release newer than this copy (the sidebar's line, About) |
| `set-aside` | The settings file could not be read at launch and was moved aside |
| `catalog-refused` | The last model list downloaded failed its checks (Settings, General) |
| `theme=light` or `dark` | Forces the theme |
| `open=<word>` | Opens the readings of the highlighted words with that text in the review |
| `compare` | Opens the review with another set-up's text in the compare pane |

If the preview shows a stale model catalogue after `crates/catalog/catalog.json` changes,
restart the dev server.

The sample data and `src/lib/newJob.test.ts` take each task's rules (which files, models and
formats suit it) from `src/lib/dev/taskRules.json`, which `cargo test` writes from the engine's
`task_rules` for the bundled catalogue, as it writes the TypeScript bindings. Run `cargo test`
after changing the catalogue or the rules, and commit the file.

## The model list

`crates/catalog/catalog.json` is read by every installed copy from `main`, so a merge reaches
them all at once ([decision](decisions/2026-11-25-model-list-checked.md)). When changing it:

- **Raise `revision` by one** in every pull request that changes the file, however small the
  change; never lower or reset it. Copies keep the list with the higher revision, so a change
  without it reaches no one. Set `updated` to the day of the change as well: it is the date
  Settings shows, and copies up to 1.10.1 keep the list with the later date, so a change that
  leaves it as it was never reaches them (nor does a second change made the same day). The
  *Model list* workflow refuses a pull request that changes the file without raising the
  revision.
- **A new model is a new entry**, with a `successor` on the one it replaces, never a removal
  ([decision](decisions/2026-11-11-model-successors.md)).
- **The checks** (`crates/catalog/src/checks.rs`) run in `cargo test` on the bundled list and in
  every copy on a downloaded one: no model or provider, a negative price, a successor or alias
  naming no model, a task without a model offered, a key page on a host the bundled list does
  not name, and so on. A list that fails one is not used. A new rule binds every later list.
- **Older versions read it too.** `scripts/catalog-compat.sh` reads the list with the catalogue
  crate of every release from 1.0.0 on; the *Model list* workflow runs it when the file
  changes. A new value of `status`, `tier`, `language_hints` or a task makes every release
  before this check refuse the whole list (later ones hide that model): add one only with the
  release that knows it, and only once that release is the oldest the job tolerates, or not at
  all. Run it locally with `bash scripts/catalog-compat.sh` (it builds in
  `target/catalog-compat`; delete it afterwards).

## Releases

Releases are built by `.github/workflows/release.yml`, natively on Windows x64 and ARM64 and on
macOS (Apple Silicon):

1. Move the version in `package.json` (with `npm version <x.y.z> --no-git-tag-version`), the
   workspace `Cargo.toml` and `src-tauri/tauri.conf.json` together, and let Cargo update
   `Cargo.lock`. Merge.
2. Push a release candidate, `v<x.y.z>-rc1`, on that commit of `main`. It builds the draft
   described below and stays unpublished: no release gate, no approval.

```bash
git tag v1.0.0-rc1 && git push origin v1.0.0-rc1
```

3. Run the test pass on the candidate's draft: [windows-test-pass.md](windows-test-pass.md) and
   [mac-test-day.md](mac-test-day.md). A fix it calls for is merged and tried on a new
   candidate (`rc2`, and so on).
4. Write the release notes in `docs/release-<x.y.z>.md`: they become the release's body,
   followed by [checking-downloads.md](checking-downloads.md) with the version filled in. They
   may be merged after the candidate: what changes only documentation does not need a new one.
5. Tag the candidate's commit, or a later commit of `main` that changes only documentation,
   and push the tag:

```bash
git tag v1.0.0 && git push origin v1.0.0
```

6. When every asset is on the draft, the run waits at *Publish the release* for the `release`
   environment's approval: open the run (Actions → Release → the run of the tag), choose
   **Review deployments**, tick `release` and **Approve and deploy**. What the approval asserts
   is in the test pass's checklists ([Windows](windows-test-pass.md#the-sign-off),
   [Mac](mac-test-day.md#the-sign-off)). **Reject** leaves the draft unpublished.
7. Once the release is published, set `latest_version` at the top of
   `crates/catalog/catalog.json` to its version and `updated` to the day, raise `revision` by
   one, and merge. Copies installed from the releases page (not the Store apps) read it at
   their next refresh and say that the new version is out, with a link to its page here
   ([decision](decisions/2026-11-10-shell.md)). Only a plain version (`1.6.0`) is read; the
   app never names one before its release is public, since the link is made from the number.

Keep the candidate tags once the version is out: the gate reads them, and a rebuild of the
final tag finds them again. Their drafts can be deleted.

### The release gate

`prepare` refuses a final tag (`vX.Y.Z`, no hyphen) before any build time is spent unless a
candidate of the same version, `vX.Y.Z-rcN`, points at the tag's commit, or at an ancestor of it
whose later commits change only files under `docs/` and Markdown files at the repository's root
([decision](decisions/2026-11-25-release-gate.md)). The refusal names each candidate it
weighed, the commits since it and the files they changed. The rule is
`scripts/release-gate.sh`, run in a clone with every tag; `bash scripts/release-gate.test.sh`
tries it against throwaway repositories, as CI's `scripts` job does.

To publish a final version without a candidate, which the release notes then say at their top:
Actions → Release → **Run workflow**, choose the tag itself under *Use workflow from* (Tags →
`vX.Y.Z`; the `release` environment admits tags only), give the tag again in the first box, and
type in the last one, exactly, `publish vX.Y.Z without a candidate`. The run still waits for the
approval.

### Setting up the `release` environment, once

The publish job names the environment `release`; GitHub creates it on the first run that does,
with no protection, and **until the environment has a required reviewer the job does not wait**.
In the repository on GitHub:

1. **Settings → Environments → New environment**, name it `release`, **Configure environment**.
2. Under *Deployment protection rules*, tick **Required reviewers** and add yourself. Leave
   **Prevent self-review** unticked: the person who pushes the tag is the one who approves.
   Leave *Wait timer* off.
3. Under *Deployment branches and tags*, choose **Selected branches and tags**, then **Add
   deployment branch or tag rule**: *Ref type* **Tag**, *Name pattern* `v*`, **Add rule**. No
   branch rule: a manual run that is to publish starts from the tag.
4. **Save protection rules**. The environment needs no secret.

A run waits up to 30 days for the approval, then fails; the draft stays and the job can be
re-run.

### A dry run of the gate and the approval

The issue's acceptance: a test tag stops at the approval. It uses version `0.0.1`, which the
workflow would publish only as a pre-release, never as the latest, and spends one full build
(the Mac's sent to Apple's notary service like any other).

1. On a branch of its own: `git switch -c dry-run/release-gate main`, then
   `npm version 0.0.1 --no-git-tag-version`, `version = "0.0.1"` under `[workspace.package]` in
   `Cargo.toml`, `"version": "0.0.1"` in `src-tauri/tauri.conf.json`, `cargo update --workspace`
   for `Cargo.lock`, and a `docs/release-0.0.1.md` of one line ("Dry run of the release gate;
   never published"). Commit, `git push origin dry-run/release-gate`.
2. The refusal: `git tag v0.0.1 && git push origin v0.0.1`. *Version check and draft release*
   fails at *Release gate*, "v0.0.1 has no release candidate", within minutes, with nothing built
   and no draft made. Remove the tag: `git push origin :refs/tags/v0.0.1 && git tag -d v0.0.1`.
3. The candidate: `git tag v0.0.1-rc1 && git push origin v0.0.1-rc1`. Its run makes a draft and
   asks for no approval; once *Version check and draft release* has passed, its run can be
   cancelled (the gate reads the tag, not the build).
4. The final tag: `git tag v0.0.1 && git push origin v0.0.1`. *Release gate* says "v0.0.1 is on
   the commit of release candidate v0.0.1-rc1", the builds run, and *Publish the release* shows
   **Waiting** with **Review deployments**. That is the acceptance. Choose **Review
   deployments** → tick `release` → **Reject**.
5. Clean up:

```bash
gh release delete v0.0.1 --cleanup-tag --yes -R fmadore/Kalamu
gh release delete v0.0.1-rc1 --cleanup-tag --yes -R fmadore/Kalamu
git tag -d v0.0.1 v0.0.1-rc1
git push origin --delete dry-run/release-gate
git switch main && git branch -D dry-run/release-gate
```

The build attestations of the test assets and the environment's record of the rejected
deployment stay on GitHub; neither is public on the releases page.

### What the workflow builds

The workflow checks the tag against the version files and builds, into a draft release, both
installers, both Store packages and the `.msixbundle` for Partner Center
([packaging-msix.md](packaging-msix.md)), the macOS disk image `Kalamu_<x.y.z>_aarch64.dmg`
(signed with the Developer ID and notarised, with a check of its signatures, its stapled
ticket, Gatekeeper's verdict, FFmpeg and licences: [the decision](decisions/2026-11-04-macos-notarisation.md)
lists the six `APPLE_*` repository secrets it needs), and `kalamu-samples.zip`
from `samples/`; it
publishes the release once every asset is attached and the approval given, as a pre-release
while the version starts with 0. A hyphenated tag (`v1.0.0-rc1`) builds the same draft and
leaves it unpublished. To rebuild a tag, run the workflow by hand from the Actions tab: a
release already published gets its assets replaced, with no gate and no approval; a tag cut before the workflow took
its present shape (November 2026) lacks `.github/actions/windows-packages` and cannot be rebuilt
this way.

The jobs that compile (`npm ci`, build scripts, Cargo) hold a read-only token and no secret: each
Windows architecture is compiled once and packaged twice, the Mac app is compiled unsigned, and
a separate job signs and notarises it with `tauri bundle`, with the packages installed without
their install scripts and nothing compiled. Only the jobs that create, fill and publish the
release can write to the repository, and they run nothing but `gh` and the shell. The release
also carries `SHA256SUMS`, every asset's SHA-256, and a build attestation for each asset listed
there, kept by GitHub. The publish step adds [checking-downloads.md](checking-downloads.md) to
the release notes, so every release says how to check them; by hand:

```bash
sha256sum -c SHA256SUMS --ignore-missing
gh attestation verify Kalamu_1.6.0_x64-setup.exe --repo fmadore/Kalamu
```

A pull request that touches the packaging (the Tauri and MSIX configuration, the bundled
programs' pins, the scripts that fetch, build or package them, `release.yml`) runs
`.github/workflows/packaging.yml`: the Windows installer and Store package made as a tag makes
them, from a debug build and for x64, with nothing signed or uploaded. Run it by hand from the
Actions tab for both architectures before tagging a release whose packaging changed.

### Building an installer locally

To build an installer locally, fetch the bundled FFmpeg and llama.cpp first; they come from the
builds in `.github/workflows/ffmpeg.yml` and `.github/workflows/llama.yml`
([llama.md](llama.md)):

```bash
node scripts/fetch-ffmpeg.mjs
```

```bash
node scripts/fetch-llama.mjs
```

```bash
node scripts/third-party-notices.mjs
```

```bash
npm run tauri build -- --bundles nsis --config src-tauri/tauri.ffmpeg.windows-x64.conf.json
```

On a Mac the same two scripts fetch and describe the macOS build, and Tauri adds
`src-tauri/tauri.macos.conf.json` by itself:

```bash
npm run tauri build -- --bundles dmg --config src-tauri/tauri.ffmpeg.conf.json
```

`third-party-notices.mjs` writes `licenses/THIRD-PARTY-NOTICES.txt` beside FFmpeg's licences:
every Rust crate and npm package built into the app, with its licence texts. The Store package
is built with `npm run bundle:msix:x64` or `bundle:msix:arm64`; see
[packaging-msix.md](packaging-msix.md).
