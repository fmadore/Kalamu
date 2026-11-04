# 2026-11-04 · The Mac App Store build

## Context

The [feasibility study](2026-11-04-mac-app-store-feasibility.md) found that Kalamu can ship in
the Mac App Store beside the notarised disk image. The App Sandbox needs three things:

- a way to keep the researcher's grants across launches;
- FFmpeg and ffprobe signed to inherit the app's sandbox;
- a second, store-signed build.

The maintainer chose to build it for the release after 1.4.0. This record says what was built,
and what is left before a first submission.

## Decisions

- **One build of the code, two signatures.** No cargo feature or second front end. The app
  knows it is the store build because macOS sets `APP_SANDBOX_CONTAINER_ID` in a sandboxed app
  (`platform::access()`), and `AppInfo.app_store` tells the interface. The disk image and
  Windows behave exactly as before: nothing is kept and nothing is asked.
- **Grants live in the shell, as plain paths elsewhere.** `src-tauri/src/grants.rs` keeps a
  security-scoped bookmark of every path the interface hands the shell while the app may still
  reach it:
  - files and folders through `expand_paths` and `inspect_files` (picked or dropped);
  - results folders through `save_settings`, `create_job` and `start_run`;
  - folders granted on request through `keep_access`.

  It writes them to `grants.json` in the data folder and opens them all again at launch,
  before the engine recovers jobs or resumes workflow runs. A folder kept replaces the paths
  inside it, which keeps the list short. Engine, store and projects are unchanged: paths stay
  strings. The bookmarks themselves are two functions in `platform/macos.rs` (objc2-foundation
  0.3.2). Windows' `access()` says there is no sandbox. The module's logic is tested on any
  system with fake bookmarks.
- **Unsafe code is denied in the shell, not forbidden.** objc2 marks the bookmark resolution
  and `startAccessingSecurityScopedResource` unsafe. The workspace still forbids unsafe code
  in every library crate. `src-tauri` now denies it, and the one function that calls those two
  methods allows it, with a `SAFETY` comment on each call.
- **Results beside sources: ask for the folder.** Picking a file grants the file, not its
  folder. Before a job or workflow run that writes beside its sources, `askForFolders`
  (`src/lib/tauri.ts`) asks `folders_to_ask` which folders are not granted yet, says why in a
  native dialog, and opens the folder picker on each. Cancelling leaves the job unstarted.
  Choosing another folder says which one was needed. Exports that write a second file beside
  the one saved (dictionary CSV, descriptions CSV, pipeline Markdown) ask the same way. The
  dialog's words are in the translation files, in English and French. Outside the sandbox
  `folders_to_ask` is always empty, so nothing changes.
- **No support link in the store build.** App Review Guideline 3.1.1 allows tips only through
  in-app purchase outside the US storefront. Settings' support section and the sidebar's link
  are hidden when `app_store` is set.
- **Signing outside Tauri.** Tauri signs sidecars with the app's own entitlements. The store
  build is made with `--no-sign`; then `scripts/sign-app-store.sh` signs FFmpeg and ffprobe with
  only `app-sandbox` and `inherit` (`src-tauri/appstore/Helper.plist`), and the app over them.
  The entitlements are in `src-tauri/appstore/`:
  - `Entitlements.plist` for the store: sandbox, outgoing network, files the researcher picks,
    bookmarks, App ID and team;
  - `Sandbox.plist`: the same without the App ID, for ad-hoc signing in CI.
- **Configuration.**
  - `src-tauri/tauri.appstore.conf.json` adds the entitlements and the container migration
    manifest. It is merged after `tauri.ffmpeg.conf.json`.
  - `src-tauri/appstore/container-migration.plist` has macOS copy the disk image's data folder
    and logs into the container at the first sandboxed launch.
  - `src-tauri/Info.plist` declares no non-exempt encryption, for every Mac build.
  - The provisioning profile and `CFBundleVersion` are added by the workflow only, so local
    builds and CI need no profile.
- **A workflow of its own**, `.github/workflows/app-store.yml`, apart from the release:
  - `sandbox` runs on pull requests that touch these files. It builds the app with the store's
    configuration, signs it ad hoc with the sandbox's entitlements, starts it on the runner,
    and fails unless the database is created in the container, the disk image's data was
    copied in, a log was written there, and FFmpeg was found and ran inside the sandbox.
  - `upload` runs on demand. It imports the Apple Distribution and Mac Installer Distribution
    certificates, embeds the profile, signs, makes the package with `productbuild`, then
    validates and uploads it with `altool` and an App Store Connect API key. It stops first if
    a secret is missing.

## Consequences

- **What the maintainer does next.** The portal and App Store Connect steps in point 6 of the
  feasibility study, then ten repository secrets:
  - `APPLE_DISTRIBUTION_CERTIFICATE`, `APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD` and
    `APPLE_DISTRIBUTION_IDENTITY`;
  - `APPLE_INSTALLER_CERTIFICATE`, `APPLE_INSTALLER_CERTIFICATE_PASSWORD` and
    `APPLE_INSTALLER_IDENTITY`;
  - `APPLE_PROVISIONING_PROFILE` (base64);
  - `APPLE_API_KEY_ID`, `APPLE_API_ISSUER` and `APPLE_API_KEY` (the `.p8` file's text).

  Then run *Mac App Store* with `upload` ticked on the release tag, and add a Mac colleague in
  TestFlight.
- **Before submitting:**
  - the App Store listing, review notes and privacy answers in `docs/store-listing.md`;
  - the privacy policy's line on `grants.json`, with a new effective date, when the store
    build first ships;
  - an explicit notice before the first job sent to each provider (Guideline 5.1.2(i)) if
    App Review asks for one.
- **Still to try on a real Mac** (CI cannot click through a folder picker):
  - granting a folder from the dialog;
  - a job resumed after a restart from bookmarks;
  - FFmpeg converting a source reached through a bookmark;
  - the player streaming one;
  - opening results in TextEdit and showing them in Finder;
  - the Keychain's prompt after the disk image.
- **Not handled yet:**
  - Jobs and documents made by the disk image hold no bookmarks. In the store build, re-running
    them fails on unreachable files until their folder is granted again, for example by adding
    the folder to a new job. A "grant this folder again" action on the job is the next step if
    testers hit it.
  - The list of grants is never pruned except of bookmarks that no longer resolve.
