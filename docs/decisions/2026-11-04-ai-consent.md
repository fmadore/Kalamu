# 2026-11-04 · Asking before files are first sent to each AI provider

## Context

Apple's App Review Guideline 5.1.2(i), amended in November 2025, asks apps to disclose where
personal data is shared with third-party AI and to obtain explicit permission first. Reviewers
have rejected apps whose only disclosure was the privacy policy. Kalamu sends the researcher's
pages, recordings and texts, which often name people, to the provider the researcher chose. The
[feasibility study](2026-11-04-mac-app-store-feasibility.md) left the question for App Review
to raise; the maintainer chose to add it before the first Mac App Store submission (1.4.1),
on Windows as well.

## Decision

- **Once for each cloud provider**, before the first job, workflow run or dictionary proposal
  sent to it. A native dialog names the provider, says what is sent (the pages, recordings or
  text, with any personal information they hold), that it goes under the researcher's own
  account and API key and the provider's terms, that the developer receives nothing, and gives
  the privacy policy's address. *Send* records the consent; *Cancel* starts nothing. A
  workflow or a job with several set-ups asks about all its providers in one dialog.
- **Offline engines are never asked about**: a provider that needs no key sends nothing.
- **Kept in the settings** (`Settings.ai_consent`), and only ever added to: `save_settings`
  merges the list it holds with the one it receives, so a copy of the settings read before the
  answer cannot undo it. There is no way to withdraw it in the interface yet; removing the
  provider's key stops anything being sent.
- **Asked in the interface, enforced in the shell.** `askConsent` (`src/lib/tauri.ts`) is called
  where the folder question already is: New job (its *Then* analysis included), a workflow's
  start and the dictionary's proposals. `create_job`, `start_run` and `dictionary_propose`
  refuse work for a provider not agreed to (`provider.consent`), so no path skips the question.
  Starting, resuming or retrying a job already made sends nothing new to a new provider.
- **The same on both systems.** The dialog is the system's own, through Tauri's dialog plugin,
  in the interface's language.
- The development mock counts every provider as agreed to, so screens and their tests run
  without the dialog.

## Consequences

- The privacy policy, the specification, the store listings and the Mac App Store review notes
  describe the question.
- Existing installs are asked once at their next job to each provider.
- Not yet tried in the packaged app on either system: the dialog's wording and buttons as each
  system draws them.
