# Development

How to build, check and preview Scriptorium. The architecture, crate responsibilities and
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
cargo run -p scriptorium --example live -- <samples> <out> [job name filter]
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
| `view=prompts-analysis` | Opens Prompts on an analysis, with its field editor |
| `theme=light` or `dark` | Forces the theme |
| `open=<word>` | Opens the readings of the highlighted words with that text in the review |

If the preview shows a stale model catalogue after `crates/catalog/catalog.json` changes,
restart the dev server.

## Releases

`npm run tauri build` makes the installers. FFmpeg is fetched by `scripts/fetch-ffmpeg.mjs` from
the build in `.github/workflows/ffmpeg.yml` and bundled with `src-tauri/tauri.ffmpeg.conf.json`.
