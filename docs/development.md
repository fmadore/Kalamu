# Development

How to build, check and preview Kalamu. The architecture, crate responsibilities and
coding rules are in [AGENTS.md](../AGENTS.md).

## Requirements

- Rust through rustup; the version in `rust-toolchain.toml` installs itself.
- Node.js 22 or 24.
- The WebView2 runtime, which ships with Windows 11.
- For offline transcription, which compiles whisper.cpp: LLVM (`clang-cl`), CMake and Ninja
  (the Visual Studio Build Tools include the last two), with these variables set, as in CI:
  `CC=clang-cl`, `CXX=clang-cl`, `CXXFLAGS=-EHsc`, `CMAKE_GENERATOR=Ninja` and
  `LIBCLANG_PATH=C:\Program Files\LLVM\bin`.
- For recordings and videos during development, an FFmpeg on `PATH` (releases bundle their own
  LGPL build; see [ffmpeg.md](ffmpeg.md)).

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

`cargo test` also regenerates the TypeScript types in `src/lib/bindings/` from the Rust
definitions; commit them with the Rust change. Exporters are checked against golden files in
`crates/export/golden/`; after an intended change, rerun their tests with `UPDATE_GOLDEN=1`
and review the diff.

## Live checks

Before a release, run every task against the live APIs with the keys saved in Settings. This
spends a few cents and never runs in CI:

```bash
cargo run -p kalamu --example live -- <samples> <out> [job name filter]
```

`<samples>` holds `newspaper.jpg`, `letter.pdf`, `interview.wav` and `article.txt`. The runner
extracts text with every provider, transcribes the interview with speakers, runs three analyses,
writes each job's outputs to its own folder in `<out>` and prints a line per piece with its
status and the cost. `LIVE_SKIP=gemini` leaves out a provider without credit.

## Preview in a browser

`npm run dev` serves the interface with sample data at `http://localhost:5173/?mock`. Query
parameters open a given state:

| Parameter | Effect |
| --- | --- |
| `view=new`, `jobs`, `prompts`, `settings` | Opens that screen |
| `view=review`, `review-audio`, `review-analysis` | Opens the review of a sample extraction, recording or analysis |
| `view=review-youtube`, `review-contents`, `review-correction` | Opens the review of a YouTube video, a table of contents or a correction |
| `view=prompts-analysis` | Opens Prompts on an analysis, with its field editor |
| `theme=light` or `dark` | Forces the theme |
| `open=<word>` | Opens the readings of the highlighted words with that text in the review |
| `compare` | Opens the review with another set-up's text in the compare pane |

If the preview shows a stale model catalogue after `crates/catalog/catalog.json` changes,
restart the dev server.

The sample data and `src/lib/newJob.test.ts` take each task's rules (which files, models and
formats suit it) from `src/lib/dev/taskRules.json`, which `cargo test` writes from the engine's
`task_rules` for the bundled catalogue, as it writes the TypeScript bindings. Run `cargo test`
after changing the catalogue or the rules, and commit the file.

## Releases

Releases are built by `.github/workflows/release.yml`, natively on x64 and ARM64:

1. Move the version in `package.json` (with `npm version <x.y.z> --no-git-tag-version`), the
   workspace `Cargo.toml` and `src-tauri/tauri.conf.json` together, and let Cargo update
   `Cargo.lock`.
2. Write the release notes in `docs/release-<x.y.z>.md`: they become the release's body.
3. Merge, then tag `main` and push the tag:

```bash
git tag v1.0.0 && git push origin v1.0.0
```

The workflow checks the tag against the version files and builds, into a draft release, both
installers, both Store packages and the `.msixbundle` for Partner Center
([packaging-msix.md](packaging-msix.md)), with `kalamu-samples.zip` from `samples/`; it
publishes the release once every asset is attached, as a pre-release while the version starts
with 0. A hyphenated tag (`v1.0.0-rc1`) builds the same draft and leaves it unpublished. To
rebuild a tag, run the workflow by hand from the Actions tab.

To build an installer locally, fetch the bundled FFmpeg first; it comes from the build in
`.github/workflows/ffmpeg.yml`:

```bash
node scripts/fetch-ffmpeg.mjs
```

```bash
node scripts/third-party-notices.mjs
```

```bash
npm run tauri build -- --bundles nsis --config src-tauri/tauri.ffmpeg.conf.json
```

`third-party-notices.mjs` writes `licenses/THIRD-PARTY-NOTICES.txt` beside FFmpeg's licences:
every Rust crate and npm package built into the app, with its licence texts. The Store package
is built with `npm run bundle:msix:x64` or `bundle:msix:arm64`; see
[packaging-msix.md](packaging-msix.md).
