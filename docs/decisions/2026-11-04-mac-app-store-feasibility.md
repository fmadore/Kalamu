# 2026-11-04 · Could Kalamu also ship in the Mac App Store?

**Status:** study. The maintainer chose to go ahead: see [the build](2026-11-04-mac-app-store-build.md).

## Context

The [macOS decision](2026-11-02-macos.md) ruled out the Mac App Store because "its sandbox and
running FFmpeg as a separate program do not mix", and because Kalamu writes its results next to
the researcher's files. From 1.4.0 the GitHub disk image is signed with a Developer ID and
notarised ([decision](2026-11-04-macos-notarisation.md)), so the maintainer now has an Apple Developer Program
membership (team `3HMSG8WB7K`). Given that, the maintainer asks whether the store is possible after all.

This record checks each obstacle against the code as it is on `main` (b756b05), against Tauri's
documentation and source, and against Apple's documentation and App Review Guidelines, all read
on 4 November 2026. The maintainer has no Mac, so nothing here was run in a sandbox. What could
not be verified is listed at the end.

**Short answer: yes, it is possible.** Neither obstacle in the earlier decision is a hard
limit. FFmpeg runs as a sandboxed helper that inherits the app's sandbox, which is Apple's
documented pattern. Writing next to the researcher's files needs a folder the researcher has
granted access to, kept across restarts as a security-scoped bookmark. The real cost is a
macOS-only layer in the shell that keeps those grants, a second signed build, and testing on a
real Mac.

## 1. Every path Kalamu stores and opens again later

Under the App Sandbox (`com.apple.security.app-sandbox`) the app reads and writes freely only
inside its container, `~/Library/Containers/io.github.fmadore.kalamu/Data`. Outside it, the app
can reach only what the researcher hands it:

- through the open or save panel (Powerbox), with `com.apple.security.files.user-selected.read-write`;
- through drag and drop.

Either way, access lasts until the app quits. To use a path again after a restart, the app must
have made a **security-scoped bookmark** from it while it still had access, stored it, and then
resolve it and call `startAccessingSecurityScopedResource` at the next launch.

| What | Where in the code | Picked how | Used again later | Under the sandbox |
| --- | --- | --- | --- | --- |
| Data folder: `jobs.sqlite`, `search.sqlite`, `settings.json`, `catalog.json`, `presets/`, `pipelines/`, `workflows/`, `export-profiles/`, `listening/`, `snapshots/` | `src-tauri/src/lib.rs:118-183`; `platform/macos.rs:38-50` builds it from `$HOME/Library/Application Support/Kalamu` | — | every launch | **Container, no change.** A sandboxed app's `HOME` is its container's `Data` folder, so the same code lands inside it. Paths inside the data folder are stored relative to it (`platform/mod.rs:33-36`). |
| Logs | `platform/macos.rs:47-48`, `~/Library/Logs/Kalamu` | — | — | Container, no change (same `HOME` rule). |
| Whisper models, Tesseract language data | `lib.rs:184-191`: `whisper-models/` and `tesseract-data/` in the data folder | downloaded | every offline job | Container. Needs `com.apple.security.network.client`. See point 4 for guideline 2.4.5(iv). |
| Temporary files (FFmpeg pieces, conversions) | `std::env::temp_dir()` (`crates/engine/src/lib.rs:95-97` defaults) | — | within a run | Container (`TMPDIR` points inside it). |
| **Job sources** | `SourceFile.path`, "absolute path on this PC" (`crates/core/src/source.rs:46`), kept in the `jobs.data` JSON (`crates/store/src/migrations.rs:8`) | open panel for files (`pickFiles`, `src/lib/tauri.ts:359`) or a folder (`pickFolder` → `expand_paths`, `src/lib/components/FileList.svelte:59`, `src-tauri/src/commands/jobs.rs:15`), or drag and drop (`onDrop`, `tauri.ts:448`) | resume and retry after a restart (`engine.recover()`, `lib.rs:228`); the review's page images and player (`crates/engine/src/listen.rs:65` plays the source itself through the asset protocol); conversion by FFmpeg; writing results again | **Bookmark per source, or per picked folder**, made when the job is created and resolved before `engine.recover()` at launch. Read-only would do for the sources themselves, but see the output row. |
| **Output folder** | `OutputDestination::Folder(String)` (`crates/core/src/job.rs:66`) in the job; `Settings.last_folder` in `settings.json` (`src-tauri/src/settings.rs:30`) | `pickFolder` (`OutputStep.svelte:45`) | every run and resume; the job's output list is read from disk (`crates/engine/src/output.rs:161-163`) and its files opened or shown in Finder (`commands/jobs.rs:82-106`) | **Read-write bookmark** for the job's folder and for `last_folder`. |
| **Results beside the sources** (the default, `OutputDestination::BesideSources`, `settings.rs:44`; `output.rs:249`) | the parent folder of each source, or of `derived_from.original_path` (`output.rs:243-246`) | not picked: implied | as above | **Not reachable from a file pick.** The open panel grants the picked files, not their folder. Apple's "related items" mechanism (`NSIsRelatedItemType` plus an `NSFilePresenter`) only covers a sibling with the *same base name and another extension*. Kalamu writes `letter.analysis.json`, `letter.contents.docx` and `provenance.json` (`output.rs:230-235`, `output.rs:116`), so it does not fit. In the store build, the folder must be granted once, through an open panel set to the source's folder (`open({ directory: true, defaultPath })`) with a line saying why, then kept as a bookmark. Sources added as a folder or dropped as a folder already carry that grant. |
| **Project documents** | `documents.source` (`crates/store/src/projects.rs:30-36`) | `pickDocuments`, a folder (`ProjectWorkspace.svelte:163`), drag and drop | starting a job or a workflow run from the project (`RunStart.svelte:140`), at any later launch | **Bookmark per document (or per folder)**, made when it is added. |
| Workflow runs | steps started later from stored sources (`engine.resume_runs()`, `lib.rs:265`) | as job sources | at launch | Covered by the source bookmarks, if they are resolved before `resume_runs`. |
| Exports and imports: presets, pipelines, workflows, export profiles, dictionary, descriptions | `tauri.ts:391-424`; `ExportDialog.svelte`, `PipelineList.svelte`, `PromptsView.svelte`, `DictionaryView.svelte` | save or open panel | no: used once | Powerbox access is enough, no bookmark. **Except twin files:** the dictionary export writes a CSV beside the JSON (`tauri.ts:398`), descriptions likewise (`tauri.ts:405`), and the pipeline record a Markdown file (`tauri.ts:412`). The save panel grants only the named file. These siblings do share the base name, so related items could cover them. Simpler: ask for a folder in the store build, or write both into one archive. |
| FFmpeg and ffprobe | `current_exe()`'s folder, `Contents/MacOS` (`lib.rs:192-198`) | bundled | every media job | Inside the bundle: readable. Needs its own signing, see point 3. |

How to build it so the rest of the code does not change: a macOS-only grant keeper in the
shell, in `src-tauri/src/platform/macos.rs` (with a no-op in `windows.rs`). It makes a bookmark
when a job, a document or the output setting is saved. It keeps the bookmarks as opaque bytes
keyed by path, in a file of its own in the data folder. At launch, before `recover()` and
`resume_runs()`, it resolves them and starts access, and makes a fresh bookmark when one comes
back stale. Paths stay strings. The engine, store and projects crates never learn about
bookmarks, as AGENTS.md requires. A source whose bookmark is missing (jobs made before the
sandbox, see point 5) is reported as a missing file, with a button that asks for the folder
again.

Libraries, with their latest versions on crates.io on 4 November 2026:

- **No official Tauri support.** `tauri-plugin-persisted-scope` 2.4.2 saves Tauri's own
  filesystem and asset scopes, not macOS bookmarks: its source has no bookmark code
  ([plugin docs](https://v2.tauri.app/plugin/persisted-scope/),
  [source](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/persisted-scope)).
  Kalamu does not use `tauri-plugin-fs` anyway: its Rust side reads files directly.
- **`objc2-foundation` 0.3.2** exposes the needed APIs directly:
  `NSURL::bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error` with
  `WithSecurityScope`, `URLByResolvingBookmarkData…`, and `startAccessingSecurityScopedResource`.
  It comes from the objc2 family, which Tauri itself already pulls in on macOS.
  **Recommended**, at about a hundred lines.
- `apple-scoped-bookmarks` 0.1.0 and `tauri-plugin-scoped-bookmarks` 0.1.0 do exactly this. They
  come from a single author and were published once, on 17 August 2026, with a few hundred
  downloads ([repository](https://github.com/CrispStrobe/apple-scoped-bookmarks)). Worth reading
  as a reference, but too young to depend on.

## 2. What Tauri 2 offers for App Store builds

Tauri's [App Store guide](https://v2.tauri.app/distribute/app-store/) covers the whole route:

- **Certificates and profile:** an *Apple Distribution* certificate signs the app. A *Mac
  Installer Distribution* certificate signs the `.pkg`. A *Mac App Store Connect* provisioning
  profile, tied to the App ID and the distribution certificate, is embedded through
  `bundle > macOS > files: { "embedded.provisionprofile": "<path>" }`.
- **Entitlements** through `bundle > macOS > entitlements`, at minimum
  `com.apple.security.app-sandbox`, `com.apple.application-identifier`
  (`3HMSG8WB7K.io.github.fmadore.kalamu`) and `com.apple.developer.team-identifier`
  (`3HMSG8WB7K`). Kalamu also needs `com.apple.security.network.client` (providers, catalogue,
  model downloads), `com.apple.security.files.user-selected.read-write`, and
  `com.apple.security.files.bookmarks.app-scope`.
- **Info.plist:** `ITSAppUsesNonExemptEncryption = false`. Kalamu uses only HTTPS and the
  system's own encryption.
- **A separate configuration**, `src-tauri/tauri.appstore.conf.json`, merged with `--config`,
  so the existing builds and CI need no profile. That fits the one-place-per-layer rule beside
  `tauri.macos.conf.json`.
- **Build and package:** `tauri build --bundles app --target …`, then
  `xcrun productbuild --sign "<Mac Installer Distribution identity>" --component Kalamu.app /Applications Kalamu.pkg`,
  then upload with `xcrun altool --upload-app --type macos --file Kalamu.pkg --apiKey … --apiIssuer …`,
  using an App Store Connect API key.
- **Universal binary is not required.** The guide builds `universal-apple-darwin` by default,
  but an Apple Silicon-only build needs only `minimumSystemVersion` of `12.0` or later. Kalamu
  already sets `13.0` (`tauri.macos.conf.json`), so the existing `aarch64` build and the
  arm64-only FFmpeg stay as they are.
- `bundle.macOS.bundleVersion` sets `CFBundleVersion` on its own, which must rise with every
  upload, including rejected ones.
- The guide says nothing about sidecars (`externalBin`). That is point 3.

## 3. FFmpeg and ffprobe under the sandbox

Apple's own article
[Embedding a command-line tool in a sandboxed app](https://developer.apple.com/documentation/xcode/embedding-a-helper-tool-in-a-sandboxed-app)
is exactly Kalamu's case: a tool built by an outside build system, placed in `Contents/MacOS`,
and started by the app. The tool must be signed with **only two entitlements**:
`com.apple.security.app-sandbox` and `com.apple.security.inherit`. It then runs inside the
app's sandbox rather than one of its own. The article warns that adding any other entitlement
"can cause problems" and makes the tool crash with a code-signing error. It also says
`get-task-allow` is incompatible with `inherit`.

**Tauri does not do this for you.** Its bundler signs every external binary with the *app's*
entitlements file (`crates/tauri-bundler/src/bundle/macos/app.rs` collects the sidecars into
`sign_paths`, and `sign.rs:53-70` passes the same `settings.macos().entitlements` to each).
Signed that way, FFmpeg would carry `application-identifier` and the file entitlements but no
`inherit`.

So the store build needs one more step after `tauri build`:

1. Re-sign `Contents/MacOS/ffmpeg` and `ffprobe` with the Apple Distribution identity, an
   identifier of their own (`-i io.github.fmadore.kalamu.ffmpeg`), and a two-key
   `Helper.entitlements` file.
2. Re-sign the app bundle over them with the app's entitlements. Signing goes from the inside
   out, and changing nested code invalidates the outer signature.
3. Run `productbuild`.

`codesign -d --entitlements - …` then shows the result in CI. This is a short shell step in
the workflow, not code in the app.

Two things this does not settle:

- **Can FFmpeg reach the source file?** `ffmpeg.rs` passes the source's path to `-i`. An
  inheriting child gets the parent's sandbox. Whether it also gets the security-scoped access
  the parent started *before* spawning it is widely reported but not stated in the article.
  This must be tried on a Mac. The fallback is to feed FFmpeg the file through standard input,
  which only the parent opens.
- **Guideline 2.4.5(iii)** forbids helper processes that outlive the app without consent.
  Kalamu's FFmpeg runs only for a job, and Quit already waits for pieces in flight. That should
  be confirmed in the review build.

Other parts of the shell under the sandbox:

- **Keychain** (`keyring` 4.2.0): sandboxed apps can use the login keychain for their own
  items. The store build is re-signed by Apple, so keys saved by the disk image are, for the
  Keychain, another app's. Expect one more prompt, or re-entering the keys.
- **`tauri-plugin-single-instance`** 2.5.4 opens a Unix socket at `/tmp/<id>_si.sock`
  ([source](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/single-instance/src/platform_impl/macos.rs)).
  `/tmp` is outside the container. On failure the plugin logs and starts normally, and macOS
  already brings the running app forward on a second launch. Harmless, but untested.
- **Opening a result in TextEdit and showing it in Finder** go through `tauri-plugin-opener`.
  Revealing in Finder is allowed for files the app may reach. Opening with a named app should be
  too, but untested.
- **Whisper and Tesseract** are compiled into the app (`whisper-rs`, `kreuzberg-tesseract`
  with `static-linking`, `crates/offline/Cargo.toml`) and run on the CPU. Nothing to sign apart.

## 4. App Review Guidelines that touch Kalamu

From the [App Review Guidelines](https://developer.apple.com/app-store/review/guidelines/):

- **2.4.5(i), sandboxing.** Covered by points 1 and 3.
- **2.4.5(ii) and (vii): installer and updates.** `productbuild` is Xcode's tool. Kalamu has no
  self-updater (`tauri-plugin-updater` is not a dependency), so nothing to remove. The model
  catalogue refresh (`crates/catalog/src/lib.rs:23`, a JSON file from the repository) is data
  that lists models and prices, not code. The review notes should say so.
- **2.4.5(iv) and 2.5.2: downloads.** Apps "may not download … additional code, or resources
  to add functionality or significantly change the app". Whisper models and Tesseract language
  data are weights and data files, not executable code, and the features exist in the
  reviewed build, with the download offered from Settings. Other dictation apps on the store
  download speech models the same way, but this is the guideline most open to a reviewer's
  reading. Two mitigations are possible:
  - bundle the smallest Whisper model and English Tesseract data in the store build, with
    larger ones as optional downloads;
  - explain the downloads plainly in the review notes, as the Microsoft Store notes already do
    (`docs/store-listing.md`, "Notes for certification").
- **Bring-your-own API keys.** Kalamu sells nothing and the provider bills its own customer, so
  in-app purchase rules do not apply. But reviewers must be able to try the app: the review
  notes should give the offline route (Tesseract or Whisper, with the sample files) and,
  optionally, a test key in App Store Connect's review fields, never in the listing.
- **5.1.2(i): third-party AI.** Apps must "clearly disclose where personal data will be shared
  with third parties, including with third-party AI, and obtain explicit permission before
  doing so". Choosing a provider and pasting one's own key is arguably that permission. A
  reviewer may still want an explicit line before the first job sent to each provider (which
  provider, what is sent, a link to the privacy policy). It is cheap, and reasonable on Windows
  too.
- **5.1.1(i): privacy policy** linked in App Store Connect and in the app. `docs/privacy.md`
  exists and Settings links to it. The App Privacy questionnaire can declare that the developer
  collects no data.
- **3.1.1: the Buy Me a Coffee link** (`src/lib/components/SupportLink.svelte`, in the sidebar
  and Settings). Tips must go through in-app purchase. Outside the United States storefront,
  apps "may not include buttons, external links, or other calls to action" toward other
  purchase methods. The Microsoft Store allows it under policy 10.8.2; Apple does not. **The
  store build must hide the link** (a build flag the shell reports through `app_info`), or the
  app must be offered on the US storefront only. Hiding it is the obvious choice.
- **Network use.** Only outgoing connections: providers, the catalogue, model downloads, the
  YouTube player. `network.client` is enough. No `network.server`, no camera or microphone.

## 5. Moving existing Mac users' data into the container

Apple's documented mechanism fits as is:
[container migration manifest](https://developer.apple.com/documentation/security/migrating-your-app-s-files-to-its-app-sandbox-container).
A `container-migration.plist` in the bundle's `Contents/Resources` with
`Move = ["${ApplicationSupport}/Kalamu", "${Library}/Logs/Kalamu"]` makes macOS copy those
folders to the same place inside the container the first time the sandboxed app starts.
Because `platform/macos.rs` already builds its paths from `HOME`, the copied folder is found
with no code change. The bundler places it through `bundle > macOS > files`.

What does not carry over:

- **Grants for files outside the container.** Jobs and documents made by the disk image hold
  absolute paths but no bookmarks. In the store build their sources and output folders are
  out of reach until the researcher grants the folder again. The missing-file button from
  point 1 is the answer. Results already in the database, corrections, projects and the
  dictionary are all inside the data folder, so nothing is lost. Only re-running and writing
  results again need the grant.
- **API keys:** see the Keychain note in point 3.
- **Two builds side by side.** Both use the same bundle identifier (they should, so the store
  build replaces the disk image). The migration copies, so a researcher who goes back to the
  disk image finds their data as it was at the switch. Deleting the container makes macOS
  migrate again, which is also how to test it.

## 6. What the maintainer must do at Apple

In the [Apple Developer portal](https://developer.apple.com/account/resources), under
Certificates, Identifiers & Profiles:

1. **Identifiers:** register an explicit App ID `io.github.fmadore.kalamu` for macOS. No extra
   capability is needed: the sandbox and file access are entitlements, not capabilities.
2. **Certificates:** create an *Apple Distribution* certificate and a *Mac Installer
   Distribution* certificate. Both can be made from a certificate request generated with
   OpenSSL on Windows, as the Developer ID one was. Pack each into a `.p12` with the legacy
   SHA-1/3DES encryption.
3. **Profiles:** create a *Mac App Store Connect* distribution profile for that App ID and the
   Apple Distribution certificate, and download it (it becomes `embedded.provisionprofile`).
   Profiles expire with their certificate.
4. **Optional, for testing on a Mac before review:** a *Mac Development* certificate and a
   development profile listing the tester's Mac, or simply TestFlight (below).

In [App Store Connect](https://appstoreconnect.apple.com):

5. **Users and Access → Integrations → App Store Connect API:** create a key with the App
   Manager role for `altool`. Keep the `.p8` file, its key ID and the issuer ID.
6. **Apps → New App:** platform macOS, the name "Kalamu" (if taken on the store, a subtitle-style
   name such as "Kalamu for researchers"; availability not checked), the bundle ID above, a SKU,
   and English as the primary language, with French added as a localisation.
7. **The listing:**
   - category Productivity;
   - free, all territories;
   - App Privacy answers ("data not collected", with providers described in the policy);
   - privacy policy URL;
   - age rating;
   - Mac screenshots (16:10, for example 2880×1800);
   - description and keywords adapted from `docs/store-listing.md`;
   - export compliance: no non-exempt encryption.
8. **App Review information:** contact details, the notes (sample files, the offline route, why
   models are downloaded, that the catalogue is data), and optionally a test API key.
9. **TestFlight:** add a Mac colleague as an internal tester, so each uploaded build is opened on
   a real Mac before review.

**Repository secrets** for a store job: the two `.p12` files and their passwords, the profile
(base64), `APPLE_API_KEY_ID`, `APPLE_API_ISSUER` and the `.p8` key. Free apps need no paid-apps
agreement, tax or banking forms.

## 7. Effort, a second build, and recommendation

**A second build is needed.** The store build differs from the notarised disk image in every
step that matters:

- **signing:** Apple Distribution, not Developer ID, and no notarisation (Apple re-signs it);
- **entitlements:** sandboxed, versus none;
- **package:** a `.pkg` with a profile, not a `.dmg`;
- **contents:** no Buy Me a Coffee link.

Both come from the same source and the same workflow: a second macOS job in `release.yml`
building with `--config tauri.appstore.conf.json` and a cargo feature `app-store`. That feature
changes only `platform/macos.rs` (grants) and `app_info` (no support link), and the bookmark
layer would also be active in the disk image if wanted. Keep the disk image: it serves
researchers who prefer GitHub, and it is the only build that writes beside sources without
asking.

Estimate, for someone who knows the codebase:

| Work | Days |
| --- | --- |
| Grant keeper in `platform/macos.rs` (objc2-foundation), bookmarks at job, document, output-folder and drop time, resolved before `recover()` and `resume_runs()` at launch, "grant this folder again" for missing sources | 3–5 |
| Results beside sources and twin-file exports: ask for the folder in the store build | 1–2 |
| `tauri.appstore.conf.json`, two entitlements files, migration manifest, helper re-signing, `productbuild` and `altool` in CI, the `app-store` feature hiding the support link | 2–3 |
| Explicit notice before the first job sent to each provider (5.1.2(i)) | 0.5–1 |
| Portal and App Store Connect set-up, listing, screenshots, review notes | 1 |
| TestFlight rounds on a real Mac, and App Review (often more than one round for a first sandboxed app) | 1–3 weeks elapsed |

That is about **two working weeks**, plus review time. The schedule depends mostly on how soon
a Mac tester can run TestFlight builds.

**Recommendation.**

- Correct the earlier reason: the Mac App Store is feasible, and FFmpeg is not the blocker.
- **Do not start it before the notarised disk image has been tried by Mac users.** Notarisation
  already removes the Gatekeeper detour that was the main argument for the store.
- Go ahead when Mac colleagues actually ask for the store (institutional machines that allow
  only store apps are the usual reason), and when one of them can test TestFlight builds.
- When it starts, first build the grant keeper and an ad-hoc-signed sandboxed build. The App
  Sandbox is enforced with an ad-hoc signature too, as long as no restricted entitlement such as
  `application-identifier` is present. CI on `macos-latest` could at least start that build and
  check that the data folder lands in the container and that FFmpeg runs. Only after that, the
  store paperwork.

## Not verified without a Mac

- That FFmpeg signed with `app-sandbox` + `inherit` can open a source the parent reached through
  a security-scoped bookmark (point 3).
- That the open panel with `defaultPath` set to a source's folder grants that folder, and that
  bookmarks made from Tauri's dialog paths, created later in the same session from the Rust
  side, resolve after a restart.
- The single-instance plugin's behaviour with `/tmp` refused, opening in TextEdit and revealing
  in Finder from the sandbox, and the asset protocol serving a bookmarked source to the player.
- The Keychain prompts when moving from the disk image to the store build.
- That `container-migration.plist` placed by `bundle > macOS > files` is honoured. The manifest
  must sit in `Contents/Resources`.
- Whether App Review accepts downloaded Whisper models and Tesseract data under 2.4.5(iv), and
  BYO keys without a test key. Only a submission answers this.
- Whether the name "Kalamu" is free on the Mac App Store.
- Whether App Store Connect requires a privacy manifest (`PrivacyInfo.xcprivacy`) for macOS
  uploads. Its validation at upload will say.
