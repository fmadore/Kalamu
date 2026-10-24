# Scriptorium: instructions for coding agents

Read by Claude Code (through `CLAUDE.md`) and Codex. [README.md](README.md) says what the app
must do; this file says how to build it.

## Before writing code

- Read the README in full. It is the specification; keep it in sync when a decision changes it.
- Review the three reference projects it lists (`iwac-ai-pipelines`, `zmo-ai-pipelines`,
  `Live-translation`) for the part being built, and port what they already solve rather than
  reinventing it. Note in the decision log where Scriptorium deliberately differs.
- Check the latest version of every dependency on its registry before adding it.

## Dependencies and security

- Dependabot security alerts are enabled on the GitHub repository. When the first manifests
  exist, add `.github/dependabot.yml` covering `cargo`, the front end's package manager and
  `github-actions`, with grouped weekly update pull requests.
- Treat a security alert as a priority over feature work: update, or record why the
  vulnerable path is not reachable.
- Run `cargo audit` (or `cargo deny`, which also checks licences against the README's
  licence rules) in CI.

## Architecture

A Cargo workspace (Rust 2024 edition) of small crates with one job each, plus a thin Tauri app.
Dependencies point inwards: the domain knows nothing about Tauri, SQLite, HTTP or FFmpeg.

| Crate | Responsibility | May depend on |
| --- | --- | --- |
| `core` | Domain types (job, source file, piece, result, provenance, task, language), the traits below, errors. No I/O. | nothing internal |
| `catalog` | Model catalogue: loading, refreshing, capability queries (tasks, inputs, languages, limits, prices). | `core` |
| `prompts` | Default and user presets, variables, import/export. | `core` |
| `media` | Conversion and splitting: PDFium, images, FFmpeg, reading text inputs; input checks (spreads, resolution, skew, audio levels) returning warnings with suggested fixes. Never modifies originals. | `core` |
| `providers` | One module per cloud provider, each implementing the provider trait. | `core` |
| `offline` | Whisper and Tesseract engines, behind the same trait as cloud providers. | `core`, `media` |
| `store` | SQLite job database and result cache. | `core` |
| `export` | One module per output format, each implementing the exporter trait. | `core` |
| `engine` | Job orchestration: queue, splitting, calls, retries, caching, resume, progress events, cost. | all of the above through their traits |
| `app` | Tauri shell: commands, events, settings, Credential Manager, packaging. | `engine`, and others only to wire them up |

Key traits, defined in `core`:

- **`Provider`**: declares its capabilities (tasks, accepted inputs, languages, limits) and
  processes one prepared piece. The engine never matches on a provider's name.
- **`Exporter`**: turns stored results into one format. Adding a format touches nothing else.
- **`JobStore`**: persistence used by the engine, so the engine can be tested with an in-memory
  store.

Rules that keep this modular:

- **Adding a provider, model, export format or prompt preset is local.** A new provider is a
  new module plus one registration line; a new model is a catalogue entry; a new format is a new
  exporter. If it needs edits in several crates, the design is wrong: fix the design first.
- **No hard-coded interface text.** The interface ships in English first, but every visible
  string, including error messages and tips, goes through translation files from the first
  screen, so French and German are added without touching code.
- **The front end only talks to `app` through typed commands and events**, with types generated
  from Rust so both sides cannot drift. No business logic in the front end.
- **A job runs one or more set-ups** (provider, model, prompt, settings) over the same sources,
  from the first version of `core` and `store`. A normal job is simply a job with one set-up, so
  comparison mode is not bolted on later. Results are keyed by source piece and set-up.
- **AI output is immutable.** User corrections and review status are stored as a separate layer
  on top of it; exporters read the corrected view, provenance records both.
- **Every piece of work is resumable.** Anything that calls a provider goes through the engine,
  which records it in the store before and after. No side channel that bypasses the cache.
- **Async with Tokio**; cancellation and pause are explicit, not by dropping tasks mid-write.
- **Errors**: `thiserror` types in library crates, with messages a researcher can understand at
  the edge; `anyhow` only in `app`. No `unwrap()` or `expect()` outside tests and startup.
- **Secrets** never leave Credential Manager except into the request that needs them, and never
  reach logs, the store or `provenance.json`.

## Testing

- Unit tests in each crate; providers tested against recorded responses, never live APIs in CI.
- Golden-file tests for every exporter.
- Resume tests that stop a job mid-run and check that it restarts without repeating or losing
  work.
- `cargo fmt`, `cargo clippy -- -D warnings` and `cargo test --workspace` pass before every
  commit, plus the front end's lint and type checks.

## Architecture checkpoints

Stop feature work and review the whole architecture, not just the current change:

- at the end of each milestone (e.g. first end-to-end job, second provider, offline mode,
  first release);
- before adding the second implementation of any trait, when the abstraction is first tested;
- when a crate wants a new dependency on another internal crate, or a file passes ~500 lines;
- when a change needed edits in more places than expected.

At each checkpoint:

1. Re-read the README and this file against the code as it is now.
2. Check that dependencies still point inwards and each crate still has one job.
3. Look for duplication across providers, exporters or media steps that belongs in a shared
   helper, and for abstractions that turned out to be unnecessary.
4. Check that resuming, caching, progress and provenance still cover every path.
5. Write down what changed and why as a short entry in `docs/decisions/` (date, context,
   decision, consequences), update this file and the README if needed, and refactor before
   continuing.

Summarise the checkpoint findings for the user before resuming feature work.
