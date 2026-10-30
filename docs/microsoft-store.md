# Microsoft Store submission

Where Kalamu stands on the way to its first Microsoft Store release, what the Store will
check, and how to submit. The paste-ready listing text is in [store-listing.md](store-listing.md),
the privacy policy to publish in [privacy.md](privacy.md), the licence terms that replace
Microsoft's standard ones in [store-license-terms.md](store-license-terms.md), and how the
package is built in [packaging-msix.md](packaging-msix.md).

This follows what [Live Translation](https://github.com/fmadore/Live-translation) learned on
the same Partner Center account (its `docs/microsoft-store.md`, `partner-center-walkthrough.md`,
`store-updates.md` and `packaging-msix.md`); where Kalamu differs, the reason is given here.

## Status (30 October 2026)

**Version 1.0.0 is ready to build; not submitted.** Everything the app and the repository must
carry is done: version 1.0.0, Report a problem, the MSIX packaging and the Store bundle in the
release workflow, the third-party notices and the sample files. What is left needs Partner
Center, the GitHub repository's settings or real keys, and is listed in
[Before the first submission](#before-the-first-submission). The latest public build is the
[v0.4.0](https://github.com/fmadore/Kalamu/releases/tag/v0.4.0) test release.

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
  inappropriate output to the developer. The listing says it, and Report a problem (a flag in
  every review's header, a button in Settings › About) opens a GitHub issue form in the browser.

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
| 11.16 Live generative AI content | Disclose generative AI in the listing and in Partner Center; a way to report inappropriate output | The description and the Generative AI property say so; Report a problem opens the `output.yml` issue form |
| Licence terms | Microsoft's Standard Application License Terms forbid reverse engineering, which the LGPL (FFmpeg) must allow | Kalamu's own [licence terms](store-license-terms.md) in "Additional license terms" |

## What is done

- **Name**: Kalamu, reserved in Partner Center on 28 October 2026
  ([decision](decisions/2026-10-28-kalamu.md)).
- **MSIX packaging**: `@choochmeque/tauri-windows-bundle` with a committed
  `src-tauri/gen/windows/`, the taskbar icon variants and `resources.pri`, FFmpeg and its
  licences inside the package ([packaging-msix.md](packaging-msix.md)). The package asks for
  `internetClient` and `runFullTrust`, and no microphone.
- **The Store bundle in the release workflow**: a tag builds the x64 and ARM64 installers, the
  two `.msix` packages and one `Kalamu_<version>.msixbundle`, and attaches
  `kalamu-samples.zip`; the release is published only once all of them are there.
- **Version 1.0.0** in `package.json`, `src-tauri/tauri.conf.json` and the workspace
  `Cargo.toml`; the package is `1.0.0.0`.
- **Report a problem** (policy 11.16): `.github/ISSUE_TEMPLATE/output.yml`, opened in the
  default browser with the version filled in. Nothing is sent by the app.
- **Third-party notices**: `scripts/third-party-notices.mjs` writes
  `licenses/THIRD-PARTY-NOTICES.txt` (every Rust crate and npm package built into the app, with
  its licence texts) in each release build.
- **Sample files for reviewers**: `samples/` (a recording made with Windows' speech synthesis,
  a synthetic newspaper page and its text, all CC0), attached as `kalamu-samples.zip`.

## Before the first submission

In this order; the [roadmap](roadmap.md#toward-the-store-release) tracks them.

1. **Copy the package identity** from Partner Center → Kalamu → **Product management** →
   **Product identity** into [Store identity](#store-identity) and into
   `src-tauri/gen/windows/AppxManifest.xml.template`, replacing `PARTNER-CENTER-PACKAGE-NAME`.
   The MSIX build refuses to run until it is there.
2. **Publish the privacy policy.** Turn on GitHub Pages for the repository (Settings → Pages →
   Deploy from a branch → `main`, `/docs`), so [privacy.md](privacy.md) is served at
   `https://fmadore.github.io/Kalamu/privacy`, and check the page.
3. **Tag v1.0.0** once this is merged: the release workflow builds and attaches everything.
4. **Live runs** from the installed package: every provider, economy mode, offline Whisper on
   `interview.wav`, a table of contents and a correction, and the YouTube player under the
   packaged origin (see the [roadmap](roadmap.md#next)).
5. **Screenshots** from the packaged app (see [store-listing.md](store-listing.md#screenshots)).
6. **Decide on a test key** for the certification team ([below](#reviewer-access-to-cloud-features)).
7. Run the [acceptance checklist](#acceptance-checklist), then submit.

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

Kalamu is reserved; the name is still to be copied from Partner Center → product → **Product
management** → **Product identity**. The manifest template must carry these values exactly, or
ingestion fails.

| Manifest element | Value |
| --- | --- |
| `Package/Identity/Name` | *(to copy; the template holds `PARTNER-CENTER-PACKAGE-NAME` until then)* |
| `Package/Identity/Publisher` | `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86` (the account's, shared with Live Translation) |
| `Package/Properties/PublisherDisplayName` | `FMadore` |
| `Package/Properties/DisplayName` | `Kalamu` |

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
      transcribe `interview.wav` from `kalamu-samples.zip` with the network off, review it
      beside the player, find "Mopti" with Ctrl F, export `.txt`, `.srt` and `.docx`.
- [ ] Each provider, with a real key saved in Credential Manager: text extraction of the sample
      page, an analysis of the sample text, a recording where the provider takes audio; costs
      shown before starting and recorded after.
- [ ] Economy mode with at least one provider: the job waits, the app is closed, and the
      answers are collected when it reopens.
- [ ] A job interrupted by closing the app resumes without sending anything twice; closing
      during a job asks first, and the job can carry on from the notification area.
- [ ] Recordings and videos are converted by the bundled FFmpeg; Settings › About and the
      `licenses` folder list FFmpeg's licence and source, and `THIRD-PARTY-NOTICES.txt` is
      there.
- [ ] A public YouTube video is transcribed from its link, and the review plays it in the
      privacy-enhanced player under the packaged origin.
- [ ] Review: corrections are kept apart from the AI text, comparison highlights differences,
      Ctrl F finds words across pages, and the flag (Report a problem) opens the issue form in
      the browser, as does the button in Settings › About.
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
