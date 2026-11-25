# 2026-11-25 · Review of the repository, the roadmap and the issues: safety nets before features

## Context

1.10.1 was submitted to the Microsoft Store on 25 November, published on GitHub the day before
without a validation build or a test pass by hand, as 1.9.0 had been. A review was made that
day of the roadmap, the 22 open issues and the code, in four readings (the store and where
data lives; the engine, the providers and the catalogue; the interface; the shell, CI and the
release pipeline), asking what neither the roadmap nor the issues held. Nothing was built or
run: each finding was read in the code, and those inferred rather than read say so in their
issue.

It found the rules kept: keys in headers only and out of the logs, a strict content security
policy and an empty asset scope, no model output drawn as HTML, translation keys held equal
by the compiler, no matching on a provider's or a model's name in the engine, every action
pinned, secrets away from the jobs that compile.

It found six bugs, and behind them the same cause as the review of 20 November named: the
code is ahead of what has been tried, and nothing but the maintainer's hours stands between
a change and a researcher's data.

## Decision

- **No new feature before the bugs and the safety nets.** The order is in the roadmap's
  *Priorities since 25 November*.
- **Six bugs for 1.10.2**: the concurrency limit that holds within one file only (#269); a
  per-minute limit that may be read as an exhausted account, to confirm live first (#270);
  the window closed over an unsaved draft (#271); right-to-left text given no direction in
  the review (#272); the vector index deleted on any open error (#273); a project file's
  originals outliving the project and its working copy in the system's temporary folder
  (#274).
- **Six safety nets**: the job database protected (#275); a release gate (#276); the packaged
  app driven in CI against recorded answers (#277); a scheduled live check under a cost
  ceiling (#278); the refreshed model list checked, and read by installed versions in CI
  (#279); the interface's net for unexpected failures and its commands checked against the
  shell's (#280).
- **Eleven directions**, none in the roadmap before: spending (#281), an offline-only project
  (#282), a provider at the researcher's own address (#283), limits and a lost connection
  handled for the whole job (#284), an export for qualitative analysis software (#285), IIIF
  manifests (#286), text size and contrast themes (#287), storage (#288), a diagnostics file
  (#289), signed Windows installers with winget and Homebrew (#290), citing and contributing
  (#291). #283, #285 and #286 are designed in a decision of their own before any code.
- **The roadmap's Status is a table** of where each copy stands, written once, in place of
  paragraphs added release after release.

Weighed and left as they are: Tauri's updater (rejected on 11 November), an Intel or
universal Mac build (2 November), files over 500 lines already listed by the checkpoints.

## Consequences

- OpenRouter, in *Later*, is covered by #283 rather than a module of its own.
- The Store's signature cannot sign the installers on GitHub: Microsoft signs the Store's
  package for the Store. #290 weighs a certificate of the project's own; until then the
  Store copy is the one to recommend on Windows.
- #277 needs two overrides in the shell (the providers' addresses, the data folder) that must
  not exist in a released build; a test holds that.
- #274, #275, #282, #286, #288 and #289 each change `docs/privacy.md` when built; this entry
  and the roadmap change nothing a researcher sees.
- The *Done* section stays in the roadmap for now; moving it out would break the links that
  point into it.
