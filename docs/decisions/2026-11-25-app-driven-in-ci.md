# 2026-11-25 · The app driven in CI against a stand-in provider

## Context

No workflow started the app the way a researcher does. The interface's tests run against
`vite dev` and a mock of the commands written by hand (`src/lib/dev`), so no test crossed from
the interface to Rust; the only real launch was the sandboxed Mac App Store build
(`app-store.yml`), on the pull requests that touch its files. What stood in for the rest was the
pass by hand (`docs/windows-test-pass.md`, about 72 steps and ten hours), which 1.9.0 and 1.10.1
skipped. The review of 25 November filed this as #277.

Three things kept the built app out of CI: it reaches four companies' services at addresses
written as constants; it keeps its data in the researcher's own folder and its keys in the
system's keychain; and every file it reads is chosen in a native dialog no webview driver
reaches.

## Decision

- **A test build, never a released one.** A Cargo feature of the shell, `test-overrides`
  (`src-tauri/src/test_overrides.rs`), read once at launch:
  - `KALAMU_TEST_DATA`: the data folder (Tauri's root override, as each system's
    `platform::data_folder` gives it), so nothing of a researcher's is read or moved;
  - `KALAMU_TEST_PROVIDERS`: an address under which each cloud provider is reached
    (`<address>/<provider id>`);
  - the keychain is never asked or written: each cloud provider holds a fixed key in memory when
    the stand-in is used, and a key saved in Settings stays in memory;
  - the dialog plugin is replaced by one of the same name and commands (`plugin:dialog|open`,
    `save`, `message`) answering from a queue the driver writes in the data folder
    (`dialog-answers.json`), so the interface and the capability are unchanged;
  - the single-instance plugin is left out, so a test build runs beside an installed Kalamu
    rather than focusing it, and the model list is not refreshed from GitHub at launch.

  The module refuses to compile without debug assertions (`compile_error!`), so no release
  profile can carry it, and `src-tauri/tests/test_build.rs` fails if the feature is enabled by
  default, by another feature, by a crate, by a Tauri configuration, by a script or by any
  workflow but `app-driven.yml`, or if that workflow builds a release or names a secret.

- **Providers' addresses are values handed to the registry**, not test hooks in each provider:
  `kalamu_providers::Addresses` and `registry_at`/`embedders_at`, with `registry` and
  `embedders` the same calls with none. A provider at an address of the researcher's own
  (#283) is meant to be handed in the same way.

- **The stand-in is a Node script** (`scripts/app-driven/stand-in.mjs`), not a Rust binary:
  CI already sets up Node on every runner for the interface, it starts in a fraction of a
  second with nothing to compile, and the same file runs on Windows and macOS. It serves the
  recorded answers the providers' tests already hold (`crates/providers/tests/fixtures`:
  Gemini's page, OpenAI's, Claude's and Mistral OCR's), and Gemini's batch in the shape
  `crates/providers/tests/batches.rs` reads, held as pending until the driver releases it. A
  path it does not know is a 404 naming it. Controls under `/__` give the request log, a delay
  per page (to pause part way) and the batches' release. Nothing was recorded anew and no real
  service is called.

- **Windows is driven over the DevTools protocol with Playwright, not `tauri-driver`.**
  WebView2 opens the Chrome DevTools protocol when started with
  `--remote-debugging-port=<port>`, which is what Edge's WebDriver does under `tauri-driver`.
  The test build passes the argument itself, for the port `KALAMU_TEST_DEVTOOLS_PORT` names:
  WebView2's own variable (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`) worked on the maintainer's
  computer (WebView2 158) and was not read on GitHub's runners (157), whose browser process
  started without the argument. Playwright is already the interface's test runner, finds
  elements by accessible name as `e2e/` does, and `connectOverCDP` downloads no browser; the
  WebDriver route would add a Rust tool built on each runner and an Edge driver to download per
  run that must match the runner's WebView2 version, which updates on its own. Live
  Translation drives its app the same way. Should a WebDriver client be wanted later, only
  `e2e-app/app.ts` changes.

- **Five flows** (`e2e-app/flows.spec.ts`), on Windows x64 and ARM64: a job from New job to its
  exported files; a correction saved, written to the files as the app ends, and found after a
  restart; a project saved to a file and opened again as a new project; a paused job, the app
  quit and opened again, resumed without reading a page twice; a batch sent in the cheaper
  mode, released while the app is closed, collected at the next launch.

- **macOS starts and ends.** No WebDriver or DevTools protocol exists for WKWebView. The same
  test build is bundled as the app and as a disk image (`tauri build --debug --bundles
  app,dmg`), and each is started with the overrides, waited for until its job database is made
  and checked, quit with Apple's quit event, and checked to have ended with status 0 through its
  exit (`the app ends` in its log) and without a panic (`scripts/app-driven/launch-check.sh`).
  The store build's own launch stays `app-store.yml`'s `sandbox` job.

- **One workflow, `app-driven.yml`**, on the runners CI already uses (`windows-latest`,
  `windows-11-arm`, `macos-latest`), on pull requests that touch the shell, the engine, the
  providers, the store, the core or the flows' own files, on every release candidate's tag
  (`v*-rc*`), and by hand. It restores the Rust cache under ci.yml's prefix with a key of its
  own, saved from `main`. No job has a secret.

## Consequences

- What is driven is the **built executable**, a debug build with the interface embedded
  (`custom-protocol`), not the installed MSIX, the installer or the notarised disk image. Those
  are release builds, and a release build cannot carry the overrides: a packaged app could only
  be driven in its real data folder with the real keychain, on a runner. Left untested by these
  flows, and still the pass's: the installer and the MSIX installing, the Store package's
  folder virtualisation, the bundled FFmpeg and llama.cpp beside the executable, signing and
  notarisation. A further step would install the packaging dry run's installer on the runner
  (a throwaway machine, so its real data folder and keychain harm no one) and drive it there,
  with the stand-in's key pasted into Settings; it needs a packaged build that can be pointed
  at another address, which #283's provider at an address of the researcher's own may bring,
  and an answer for the native file pickers, which only the test build replaces.
- The flows locate controls by their English accessible names: renaming a label in `en.ts`
  breaks a flow, as it breaks `e2e/`. An interface-only change does not start the workflow; a
  release candidate always does.
- The batch is collected at launch, but the job is shown done only at the collection loop's
  next turn, up to a minute later (`EngineConfig::batch_poll`); the flow waits for it.
- A debug build of the whole shell takes some minutes per runner even with the cache; the
  five Windows flows took 1 minute 40 seconds on the maintainer's Snapdragon, the batch's
  minute of waiting the most of it.
- Two things the flows found while being written, neither a bug of the app: Tauri's IPC
  properties cannot be overridden from the page, which ruled out answering dialogs in the
  webview, and WebView2's browser process outlives its app by a few seconds, holding the
  DevTools port and the data folder's webview profile, so each launch takes a port of its own
  and waits for the last to let go.
- The pass by hand shrinks by the steps marked *CI* in `docs/windows-test-pass.md` and
  `docs/mac-test-day.md`, which keep their place and their wording.
- `macos.rs` is untouched; the Mac job, its bundling with `--ci` and its quit by Apple event
  have not yet run anywhere but in CI.
