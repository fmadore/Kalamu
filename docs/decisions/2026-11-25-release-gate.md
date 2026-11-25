# 2026-11-25 · The release gate: a candidate, then an approval

## Context

Any `v*` tag without a hyphen published itself ([#276](https://github.com/fmadore/Kalamu/issues/276)):
`release.yml` checked the version files, the tag and the notes file, and nothing else. 1.9.0 and
1.10.1 went public without a validation build or a test pass, and nothing in the pipeline said
so ([the review of 25 November](2026-11-25-review-safety-nets.md)). 1.8.0 and 1.10.0 had their
candidates (`v1.8.0-rc1`, `v1.10.0-rc1` and `rc2`), but those tags are no longer on GitHub:
deleted once the version was out, so nothing now shows which commit was tried.

The same review found `rust-version` in `Cargo.toml` at 1.99.1 and the toolchain pinned at
1.100.0. The toolchain was pinned at 1.99.1 with `rust-version` beside it on 24 October
([foundations](2026-10-24-foundations.md)); Dependabot's pull request of 14 November moved the
toolchain alone (378eeca), although both files and `dependabot.yml` say to move them together.

## Decision

- **Publishing waits for an approval.** `publish` runs in a GitHub environment, `release`, with
  the maintainer as required reviewer and only tags `v*` admitted. The run builds everything,
  attaches it to the draft, then waits; the click is the sign-off of the test pass, and
  [windows-test-pass.md](../windows-test-pass.md#the-sign-off) and
  [mac-test-day.md](../mac-test-day.md#the-sign-off) say what it asserts. A candidate never
  reaches `publish` (a hyphenated tag is skipped there, as before), so it is not slowed; nor is
  a rebuild of a release already published, which now skips `publish` instead of entering it
  to do nothing, so no approval is asked for a no-op.
- **A final tag needs a candidate.** `prepare` runs `scripts/release-gate.sh` before the draft
  is made and before any build: a `vX.Y.Z` tag passes when a `vX.Y.Z-rcN` tag points at the same
  commit, or at an ancestor of it. Another hyphenated tag (`v1.2.0-test`) or a candidate of
  another version does not count; a candidate on a commit that is not an ancestor (another
  branch, or cut after the final tag) does not either.
- **The ancestor is bounded by what changed, not by how far back it is.** A count of commits
  proves nothing either way: one commit can change the engine, fifty can be release notes. A
  candidate on an ancestor counts only when every file changed since it is under `docs/` or a
  Markdown file at the repository's root (README, AGENTS): what the test pass writes, the
  release notes, the guide. Nothing there is built into the app. Markdown elsewhere is not
  allowed, because the default prompts (`crates/prompts/defaults/*.md`) are compiled in. A fix
  found by the pass needs a new candidate; the nearest candidate that qualifies is the one
  named. The refusal lists, for each candidate weighed, why it did not count: not an ancestor,
  or the commits since it (hash and subject) and the files outside the documentation.
- **Without a candidate, in so many words.** A manual run may let a final version through when
  its last box reads exactly `publish vX.Y.Z without a candidate` for that tag: a sentence typed,
  not a box ticked, and one that names the version, so an old one pasted again does not fit. The
  release notes then open with "Published without a release candidate". The approval is still
  asked. A candidate that qualifies wins over the sentence, so it is never recorded needlessly.
- **The rule lives in a script with its tests.** `scripts/release-gate.test.sh` builds a
  throwaway repository per case (same commit, ancestor with documentation only, ancestor with
  code, Markdown compiled into the app, nearest of two, none, another version only, not an
  ancestor, cut after, the sentence right, wrong and for another version, the workflow's
  outputs, a hyphenated tag, a missing tag). CI's new `scripts` job runs it on every pull
  request, on Ubuntu; it runs in Git Bash on Windows too.
- **`rust-version` follows the toolchain, and CI holds it.** `scripts/check-rust-version.sh`
  fails when they differ, in the same `scripts` job, so Dependabot's next toolchain pull request
  fails until `Cargo.toml` follows. `rust-version` is set to 1.100.0: every build since
  14 November compiled with that toolchain, and nothing tests 1.99.1, so 1.100.0 is the oldest
  Rust the code is known to build with. Lowering the channel instead would change what every
  build compiles with.

## Consequences

- **Candidate tags are kept** once the version is out; their drafts can be deleted.
  [development.md](../development.md#releases) says so. The tags deleted before this cannot be
  restored from history, so 1.10.1's line has no candidate to show.
- A release costs two full builds: the candidate's and the final tag's, which is built again
  rather than promoted. The final build is from the same commit, or one that differs only in
  documentation, so the pass still describes it; promoting the candidate's assets would save
  the build but rename every asset and its attestation, and was left aside.
- The approval is only as strong as the environment: **until the maintainer creates `release`
  with a required reviewer, `publish` does not wait** (GitHub creates the environment on the
  first run that names it, with no protection). The steps, and a dry run on version 0.0.1 that
  stops at the approval, are in [development.md](../development.md#releases); the dry run is
  the issue's acceptance and has not been made.
- A manual run that is to publish starts from the tag (*Use workflow from*), since the
  environment admits tags only.
- The gate reads tags, not builds: it does not check that the candidate's run succeeded or
  that a pass was made on it. The approval covers that.
- `rust-version` at 1.100.0 lets Clippy apply lints for what 1.100.0 stabilised; the pull
  request's CI shows whether any fires.
