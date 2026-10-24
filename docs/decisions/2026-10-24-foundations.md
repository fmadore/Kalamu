# 2026-10-24 · Foundations: layout, PDFs, types and reuse

## Context

Before writing code, the three reference projects were reviewed: Live Translation (Tauri 2,
Rust, SvelteKit) for the app shell, and iwac-ai-pipelines and zmo-ai-pipelines (Python) for the
AI side.

## Decisions

- **Workspace layout.** A Cargo workspace (edition 2024) with library crates under `crates/`
  and the Tauri app in `src-tauri/`, with the SvelteKit front end at the repository root, as in
  Live Translation. Live Translation is a single crate on edition 2021; Scriptorium departs from
  it because AGENTS.md asks for small crates with one job each.
- **Toolchain pinned** in `rust-toolchain.toml` (1.99.1), as in Live Translation, so new Clippy
  lints arrive in a Dependabot pull request rather than on `main`.
- **PDFs are split, not rendered.** Both Python projects cut PDFs into single-page PDFs (pypdf)
  and never render them; Gemini, Claude, OpenAI and Mistral all accept PDF input. Scriptorium
  does the same with `lopdf`. PDFium, listed in the README for PDF-to-image conversion, is
  deferred until a provider or Tesseract needs images; the review view renders pages in the
  front end.
- **Generated TypeScript types.** Live Translation mirrors Rust types by hand and checks names
  with regex tests; a shape mismatch slipped through. Scriptorium generates bindings with
  `ts-rs` (stable; `tauri-specta` 2 is still a release candidate) into `src/lib/bindings/`.
- **`sha2` stays on 0.10.** wry still depends on 0.10; moving ahead would ship two copies, as
  Live Translation found.
- **Errors reach the front end as ids**, translated there, following Live Translation's
  `AppError { id, detail }`, so interface text stays in the translation files.
- **Bundle identifier `io.github.fmadore.scriptorium` from day one.** Live Translation had to
  migrate keychain entries and data folders after renaming its identifier.
- **Settings live in Rust** (a JSON file in the app data folder), not in the WebView's
  `localStorage` as in Live Translation, so the engine and the interface read the same values.
- **Provenance** follows zmo-ai-pipelines' `provenance.json`: source hash, the full prompt text
  and its hash, model version and usage.
- **Provider defaults ported from iwac-ai-pipelines:** no temperature for Gemini 3 (lower
  values make it loop) or Claude; Gemini PDF parts at high media resolution and the lowest
  thinking level the model accepts; Claude with the lowest effort; Mistral OCR pinned to a dated
  model rather than `-latest`.

## Consequences

- Image-only models cannot read PDFs until PDFium is added; the catalogue marks which models
  accept PDFs.
- OpenAI text extraction has no Python code to port and is written from the API documentation.
