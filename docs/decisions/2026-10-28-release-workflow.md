# 2026-10-28 · The release workflow

## Context

Scriptorium 0.1 and 0.2 were built by hand on an ARM64 laptop, so no x64 installer was ever
published, and nothing checked that the version files agreed with the tag. Kalamu 0.3.0 is the
first test build under the new name and the first meant for both architectures. Live
Translation's `release.yml` already solves the same problem on the same account.

## Decision

- **Port Live Translation's workflow**: a `prepare` job that checks the tag against the version
  files and finds or creates one draft release, a build matrix that uploads to it with
  tauri-action, and a last job that publishes it with `docs/release-<version>.md` from the
  tagged commit as its body. Nothing is public until every asset is attached; a hyphenated tag
  is a validation build left as a draft.
- **Where Kalamu differs**:
  - Both architectures get an installer, built natively (`windows-latest`, `windows-11-arm`).
    Live Translation ships x64 installers only and keeps ARM64 for the Store package; Kalamu's
    test builds have been ARM64 from the start, so dropping it was not an option.
  - The version lives in the workspace `Cargo.toml` (`[workspace.package]`), not in
    `src-tauri/Cargo.toml`.
  - Each build fetches the pinned FFmpeg (`scripts/fetch-ffmpeg.mjs`, checked against
    `scripts/ffmpeg.json`) and bundles it with `src-tauri/tauri.ffmpeg.conf.json`.
  - Only the NSIS installer is built (`--bundles nsis`), as for 0.1 and 0.2: one asset per
    architecture, which installs per user without administrator rights.
  - While the version starts with 0 the release is a pre-release and never marked latest.
  - No MSIX jobs yet: they come with the MSIX packaging and the Store identity
    ([microsoft-store.md](../microsoft-store.md#what-is-missing)).
- The Rust cache is restored from CI's key and never saved by a tag run, as Live Translation
  learned: a tag's cache can only be read by that tag, and filled the repository's quota there.

## Consequences

- A release is: bump the three version files, write the notes, merge, push the tag
  ([development.md](../development.md#releases)).
- Adding the Store bundle means adding the `msix` and `bundle` jobs before `publish`, and
  making `publish` wait for them.
