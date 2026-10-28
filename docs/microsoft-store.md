# Microsoft Store submission

Where Kalamu stands on the way to its first Microsoft Store release, what the Store will
check, and how to submit. The paste-ready listing text is in [store-listing.md](store-listing.md),
the privacy policy to publish in [privacy.md](privacy.md), and the licence terms that replace
Microsoft's standard ones in [store-license-terms.md](store-license-terms.md).

This follows what [Live Translation](https://github.com/fmadore/Live-translation) learned on
the same Partner Center account (its `docs/microsoft-store.md`, `partner-center-walkthrough.md`,
`store-updates.md` and `packaging-msix.md`); where Kalamu differs, the reason is given here.

## Status (28 October 2026)

**Not submitted. The name Kalamu is reserved; the package identity is still to be copied
below.** The latest build is the
[v0.3.0](https://github.com/fmadore/Kalamu/releases/tag/v0.3.0) test release, unsigned
installers for Windows on x64 and ARM64. Before a first submission, the work in
[What is missing](#what-is-missing) has to be done; the [roadmap](roadmap.md#toward-the-store-release)
tracks it.

## Product and certification approach

Kalamu reads scanned and handwritten documents, transcribes recordings and analyses texts
with the AI provider the researcher chooses, under the researcher's own API key, or offline with
Whisper. Three things shape the submission:

- **Most features need the user's own provider account.** Text extraction and analyses run only
  with a Gemini, OpenAI, Claude or Mistral key, and usage is billed by that provider. The
  listing says so in its first paragraph, and the app sells no credits, subscription or API
  access. Live Translation's 1.0.3 failed certification because the reviewer pressed Start and
  nothing happened (policy 10.1.2.20), so reviewers need a route that works.
- **The credential-free route is offline transcription.** After a one-time model download from
  Hugging Face, Whisper transcribes on the PC with no account, key or network. The certification
  notes walk through it with a short sample recording. Text extraction has no offline engine yet
  (Tesseract is on the roadmap), so reviewers cannot test it without a key; see
  [Reviewer access to cloud features](#reviewer-access-to-cloud-features).
- **The output is AI-generated text.** Store policy 11.16 asks apps that generate content with
  AI to say so in the listing and in Partner Center, and to give users a way to report
  inappropriate output to the developer. The listing says it; the app still needs its
  [Report a problem](#what-is-missing) link.

Review the current [Microsoft Store policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
when submitting; the numbers below are those cited in Live Translation's submissions and in
this repository's notes, and Microsoft renumbers them from time to time.

## Policies that apply

| Policy | What it asks | How Kalamu meets it |
| --- | --- | --- |
| 10.1 Distinct function, accurate representation | The listing describes what the app does, its dependencies and costs; advertised features work | The description opens with the need for a provider key and its charges; the certification notes give a working route |
| 10.5 Personal information | A privacy policy when the app sends personal information | Documents and recordings go to the provider the user chooses: [privacy.md](privacy.md), published at a stable URL |
| 10.8.2 Payments outside Microsoft commerce | A voluntary donation through a secure third-party provider is allowed, and must be declared in Partner Center | The Buy Me a Coffee link unlocks nothing; tick the Product declaration and explain the link in the certification notes |
| 11.2 and 10.1 Trademarks | Other companies' names only descriptively, no logos, no implied endorsement | Providers, YouTube and Word are named to say what the app works with; a trademark line ends the description |
| 11.16 Live generative AI content | Disclose generative AI in the listing and in Partner Center; a way to report inappropriate output | The description and the Generative AI property say so; the Report a problem link is still to add |
| Licence terms | Microsoft's Standard Application License Terms forbid reverse engineering, which the LGPL (FFmpeg) must allow | Kalamu's own [licence terms](store-license-terms.md) in "Additional license terms" |

## What is missing

Before the first submission, in rough order; each is also on the roadmap.

1. **A name.** Done: Kalamu is reserved ([decision](decisions/2026-10-28-kalamu.md)). Copy its
   package identity into [Store identity](#store-identity) before packaging.
2. **MSIX packaging.** Port Live Translation's set-up: Tauri has no MSIX target, so it uses the
   `@choochmeque/tauri-windows-bundle` devDependency with a committed
   `src-tauri/gen/windows/` (`bundle.config.json`, `AppxManifest.xml.template` with the reserved
   identity written out as literals, `Assets/` with the logo sizes and taskbar variants,
   `resources.pri` enabled), `bundle:msix:{x64,arm64}` npm scripts, and a `.gitignore` exception
   for `gen/windows/`. Kalamu differs in two ways: it needs `internetClient` and
   `runFullTrust` but no `microphone`; and the package must carry the FFmpeg sidecars
   (`ffmpeg.exe`, `ffprobe.exe`) and the `licenses/` folder from `tauri.ffmpeg.conf.json`,
   which the bundler's handling of `externalBin` and `resources` has to be checked for.
3. **The Store bundle in the release workflow.** `.github/workflows/release.yml` already builds
   both architectures natively (x64 on `windows-latest`, ARM64 on `windows-11-arm`), with
   FFmpeg and whisper.cpp, into a draft release published once every asset is attached. It
   still needs Live Translation's `msix` and `bundle` jobs, which add one unsigned
   multi-architecture `.msixbundle` beside the installers.
4. **Version 1.0.0.** The Store refuses a package version whose first segment is 0, and every
   later package must be higher than the one already in the Store. `package.json`,
   `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and the workspace `Cargo.toml` move
   together; MSIX appends a fourth segment (`1.0.0.0`).
5. **Report a problem** (policy 11.16): a link in the review view and in Settings that opens a
   new GitHub issue, or an email to the support address, in the default browser, so a
   researcher can report inappropriate or harmful output. Nothing is sent by the app itself.
6. **The privacy policy published.** Turn on GitHub Pages for this repository (Settings →
   Pages → Deploy from a branch → `main`, `/docs`), so [privacy.md](privacy.md) is served at
   `https://fmadore.github.io/Kalamu/privacy`, as Live Translation's is. Check the page
   before submitting.
7. **Third-party notices.** The About section names FFmpeg, Opus, whisper.cpp and the Whisper
   models; the bundle also carries other Rust crates and npm packages, PDF.js (Apache 2.0) among
   them. Generate a complete notices file at build time (`cargo about` for Rust, a licence
   report for the front end) and install it in `licenses/`.
8. **Live runs** of every provider through the packaged app, economy mode included, and the
   YouTube player under the packaged origin (see the [roadmap](roadmap.md#next)).
9. **Screenshots** from the packaged app (see [store-listing.md](store-listing.md#screenshots)).
10. **A sample bundle for reviewers**: a short public-domain recording, a scanned page and a
    text file, attached to the release (`kalamu-samples.zip`), so the certification notes
    can point to files that work.

## Reviewer access to cloud features

The certification team cannot be expected to hold a Gemini, OpenAI, Claude or Mistral account.
Two ways to let them test text extraction and analyses; decide before the first submission:

- **A test key in the certification notes**, which only the certification team reads: a
  Gemini key on a project with a small budget cap, revoked once certification is done. This
  tests the main features as a researcher uses them.
- **No key**: the notes say that text extraction and analyses need the user's own key, as the
  listing does, and give the offline transcription route only. Live Translation passed with its
  cloud features tested this way after 1.0.5, but its main feature worked offline.

Either way, never put a key in the listing, the screenshots or this repository.

## Store identity

Kalamu is reserved; to be copied from Partner Center → product → **Product management** →
**Product identity**). The manifest template must carry these values exactly, or ingestion
fails.

| Manifest element | Value |
| --- | --- |
| `Package/Identity/Name` | *(assigned on reservation)* |
| `Package/Identity/Publisher` | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` (the account's, shared with Live Translation) |
| `Package/Properties/PublisherDisplayName` | `FMadore` |

| Store value | Value |
| --- | --- |
| Package Family Name | *(assigned)* |
| Product ID | *(assigned)* |
| Store URL | *(assigned)* |

## Acceptance checklist

Run against the signed per-architecture `.msix` installed locally (Live Translation's
`scripts/install-local-msix.ps1` and its signing steps carry over), not a development build:
package identity changes where data is kept and how the window is drawn.

- [ ] Both packages report version `1.0.0.0` and the reserved identity; the bundle holds x64
      and ARM64.
- [ ] The taskbar, Start and Alt+Tab show the icon without a plate, in the dark and the light
      theme.
- [ ] First launch with no key: the app opens on New job and says how to add a key; nothing is
      downloaded or sent until the user asks.
- [ ] Offline: download Whisper Base in Settings (and cancel a download part-way once),
      transcribe the sample recording with the network off, review it beside the player,
      export `.txt`, `.srt` and `.docx`.
- [ ] Each provider, with a real key saved in Credential Manager: text extraction of the sample
      page, an analysis of the sample text, a recording where the provider takes audio; costs
      shown before starting and recorded after.
- [ ] Economy mode with at least one provider: the job waits, the app is closed, and the
      answers are collected when it reopens.
- [ ] A job interrupted by closing the app resumes without sending anything twice; closing
      during a job asks first, and the job can carry on from the notification area.
- [ ] Recordings and videos are converted by the bundled FFmpeg; Settings › About and the
      `licenses` folder list FFmpeg's licence and source.
- [ ] A public YouTube video is transcribed from its link, and the review plays it in the
      privacy-enhanced player under the packaged origin.
- [ ] Review: corrections are kept apart from the AI text, comparison highlights differences,
      Report a problem opens the browser.
- [ ] The Buy Me a Coffee link opens `https://buymeacoffee.com/fmadore` in the default browser,
      not in a window of the app.
- [ ] Light, dark and system themes; Windows text size; keyboard use of New job and the review.
- [ ] Windows App Certification Kit passes on the final packages.
- [ ] The listing text, screenshots and the published privacy policy match this version.

## Partner Center, step by step

1. **Apps and games** → the reserved product → **Start your submission**.
2. **Pricing and availability**: free; all markets, unless a provider is unavailable in one;
   visibility public. No add-ons, no in-app purchases.
3. **Properties**:

   | Field | Value |
   | --- | --- |
   | Category | Productivity (subcategory none); Education is the alternative |
   | Privacy policy URL | `https://fmadore.github.io/Kalamu/privacy` |
   | Website | `https://github.com/fmadore/Kalamu` |
   | Support contact | `https://github.com/fmadore/Kalamu/issues` |
   | Product declarations | Tick **This product allows users to make purchases, but does not use the Microsoft Store commerce system** (the Buy Me a Coffee link, policy 10.8.2) |
   | Generative AI | Yes: every result is text generated by the AI model the user chooses, in the cloud or offline with Whisper |
   | Personal information | Yes: files the user chooses go to the AI provider the user chooses, under the user's account; offline Whisper sends nothing |
   | System requirements | Memory 8 GB recommended (Whisper's larger models); see the additional requirements in [store-listing.md](store-listing.md#additional-system-requirements) |

4. **Age ratings**: complete the questionnaire. The app connects to third-party services chosen
   by the user (AI providers, Hugging Face for model downloads, YouTube for videos), shares no
   user content with other users, and has no chat or user-to-user communication.
5. **Packages**: upload the single unsigned `.msixbundle` from the GitHub release; nothing else.
   The Store signs accepted packages. For later submissions, delete every package already
   listed before uploading, so no architecture ships stale.
6. **Store listings** → English (United States): paste each block from
   [store-listing.md](store-listing.md), upload the screenshots with their captions, and under
   **Additional license terms** paste [store-license-terms.md](store-license-terms.md).
7. **Submission options** → **Notes for certification**: paste the notes from
   [store-listing.md](store-listing.md#notes-for-certification), with the test key if one is
   given. **Restricted capabilities**: justify `runFullTrust`:

   > Win32 desktop app packaged as MSIX (Tauri/WebView2). The full-trust process reads the documents and recordings the user selects, converts audio with the bundled FFmpeg executables, runs offline Whisper transcription on the CPU, keeps API keys in Windows Credential Manager, writes results next to the user's files or in a folder they choose, and keeps a job running from the notification area while its window is closed. No driver, service, auto-start task, telemetry or developer server.

8. **Submit**, then watch certification: an accepted upload is not a pass. Record the date and
   the result in this file's Status.

## Constraints worth knowing

- **Submission is by hand.** The account is an Individual account, and the submission API
  needs a Microsoft Entra tenant, which only Company accounts have (Live Translation's
  `store-updates.md`, *Why by hand*). Converting the account is not possible without a new
  publisher identity.
- **One pending submission per product.** Finish or delete the pending one before starting
  another.
- **The listing is not in this repository.** `store-listing.md` is source text to paste;
  nothing in `docs/` reaches the Store until someone types or uploads it.
- **What's new is rewritten for every release**, in every listing language.
- **Unsigned GitHub installers stay**, for those who cannot use the Store; the Store copy is the
  one that installs without a warning and updates itself.
