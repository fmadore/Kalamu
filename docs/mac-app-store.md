# Mac App Store submission

How Kalamu reaches the Mac App Store: what to set up once at Apple and on GitHub, what to answer
in App Store Connect, and the steps for each version. The paste-ready listing text and review
notes are in [store-listing.md](store-listing.md#mac-app-store-app-store-connect), the privacy
policy in [privacy.md](privacy.md). Why the store build is made this way is in the
[feasibility study](decisions/2026-11-04-mac-app-store-feasibility.md) and the
[build decision](decisions/2026-11-04-mac-app-store-build.md); the Microsoft Store's own steps
are in [microsoft-store.md](microsoft-store.md).

## Status (13 November 2026)

**Version 1.6.0 is in App Review**, the first submission: the *Mac App Store* workflow signed and
uploaded it from the `v1.6.0` tag on 11 November 2026, after the app was tried on a Mac with
Apple Silicon (macOS 26), and it was submitted with the listing in
[store-listing.md](store-listing.md#mac-app-store-app-store-connect). The next version, 1.7.0,
follows the same steps once Apple has answered, with a *What's New* text. What the repository
provides:

- the sandboxed build, its entitlements, the container migration and FFmpeg signed to inherit
  the sandbox, started and checked in CI by the *Mac App Store* workflow;
- the upload job, which signs the app and its installer package and sends them to App Store
  Connect with `altool --upload-package`;
- a notice before the first job sent to each AI provider (Guideline 5.1.2(i));
- the listing, review notes and the answers below;
- the privacy policy's lines on the store build (`grants.json`, the container).

What only the maintainer can do: the Apple portal and App Store Connect steps, the GitHub
secrets, the screenshots (taken on a Mac, from the material in
[store-listing.md](store-listing.md#screenshots)), and each submission. Nothing from the Windows
set is reused: its 1.4.0 screenshots were refused by the Microsoft Store (see
[What App Review may question](#what-app-review-may-question)).

## Once: at Apple

In [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources), with
the team `3HMSG8WB7K`:

1. **Identifier**: an explicit App ID for macOS, bundle ID `io.github.fmadore.kalamu`. No
   capability to tick: the sandbox and file access are entitlements, not capabilities.
2. **Two certificates**, each from a certificate request made with OpenSSL on Windows, as the
   Developer ID one was ([decision](decisions/2026-11-04-macos-notarisation.md)):
   - *Apple Distribution*, which signs the app;
   - *Mac Installer Distribution*, which signs the `.pkg`.

   For each, in Git Bash (keep the `.key` files and passwords in the password manager):

   ```bash
   openssl req -new -newkey rsa:2048 -nodes -keyout distribution.key -out distribution.csr -subj "/CN=Frédérick Madore/C=DE"
   ```

   Upload the `.csr`, download the `.cer`, then pack the key and certificate into a `.p12` that
   macOS's `security import` accepts (SHA-1/3DES, not OpenSSL 3's default):

   ```bash
   openssl x509 -inform der -in distribution.cer -out distribution.pem
   ```

   ```bash
   openssl pkcs12 -export -inkey distribution.key -in distribution.pem -out distribution.p12 -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1
   ```

   Repeat with `installer.key`, `installer.csr`, `installer.cer` and `installer.p12`. Each
   certificate's name, needed below, is in its subject:

   ```bash
   openssl x509 -noout -subject -in distribution.pem
   ```

   Expect `Apple Distribution: Frédérick Madore (3HMSG8WB7K)` and, for the installer,
   `3rd Party Mac Developer Installer: Frédérick Madore (3HMSG8WB7K)`.
3. **Profile**: a *Mac App Store Connect* distribution profile for that App ID and the Apple
   Distribution certificate. Download it; it expires with the certificate.

In [App Store Connect](https://appstoreconnect.apple.com):

4. **Users and Access → Integrations → App Store Connect API**: a team key with the *App
   Manager* role. Keep the `.p8` file (downloadable once), its key ID and the issuer ID.
5. **Apps → New App**: platform macOS, name `Kalamu` (or `Kalamu for Researchers` if taken),
   primary language English (U.S.), bundle ID `io.github.fmadore.kalamu`, SKU `kalamu`, full
   access. Then **App Information → General → Apple ID**: the number the upload needs.

Free apps need no paid-apps agreement, tax or banking forms.

## Once: on GitHub

**Repository secrets** (Settings → Secrets and variables → Actions):

| Secret | Value |
| --- | --- |
| `APPLE_DISTRIBUTION_CERTIFICATE` | `distribution.p12`, base64 (`base64 -w0 distribution.p12`) |
| `APPLE_DISTRIBUTION_CERTIFICATE_PASSWORD` | its password |
| `APPLE_DISTRIBUTION_IDENTITY` | `Apple Distribution: Frédérick Madore (3HMSG8WB7K)` |
| `APPLE_INSTALLER_CERTIFICATE` | `installer.p12`, base64 |
| `APPLE_INSTALLER_CERTIFICATE_PASSWORD` | its password |
| `APPLE_INSTALLER_IDENTITY` | `3rd Party Mac Developer Installer: Frédérick Madore (3HMSG8WB7K)` |
| `APPLE_PROVISIONING_PROFILE` | the `.provisionprofile`, base64 |
| `APPLE_API_KEY_ID` | the API key's ID |
| `APPLE_API_ISSUER` | the issuer ID |
| `APPLE_API_KEY` | the `.p8` file's text |

**Repository variable** (same page, *Variables*): `APPLE_APP_ID`, the app's Apple ID from step
5. The upload job stops at its first step, naming what is missing, if any is absent.

## Once: in App Store Connect

**Pricing and Availability**: free, all countries and regions.

**App Privacy → Data Types**: *No, we do not collect data from this app*. Apple counts data as
collected when it leaves the device in a way that lets the developer or its partners access it
beyond serving the request. Kalamu's developer receives nothing; files go from the Mac to the
provider the user chose, under the user's own account and API key and that provider's terms, and
the app asks before the first job sent to each. If App Review asks for the providers to be
declared anyway, the cautious answer is *User Content → Other User Content*, not linked to the
user, not used for tracking, purpose *App Functionality*. **Privacy Policy URL**:
`https://fmadore.github.io/Kalamu/privacy`.

**Age Rating**: answer *None* or *No* to every question: no violence, sexual content, profanity,
drugs, gambling, horror or medical content of the app's own; no unrestricted web access (the one
web view is YouTube's embedded player for a video the user pastes); no user-generated content
shared with others, no messaging or chat; no advertising; no location sharing; no dating or
relationships (the two questions App Store Connect adds from 16 November 2026, to be answered by
12 January 2027). Kalamu writes AI text only from the user's own documents and has no
conversational assistant; say so where a question touches generated content. The expected
rating is 4+.

**Export compliance**: answered by the build. `Info.plist` declares
`ITSAppUsesNonExemptEncryption` false (HTTPS and the system's own encryption only), so App Store
Connect asks nothing.

**Content rights**: *Yes*, it accesses third-party content: a public YouTube video whose link
the user pastes is played in YouTube's own embedded player, under YouTube's terms, and read by
Gemini from YouTube; Kalamu downloads and stores none of it.

## Each version

1. **Release it as usual**: merge the release pull request (version files and
   `docs/release-<version>.md`), tag it, and let the release workflow publish the installers
   and the disk image. The store build reads the same version from `tauri.conf.json`.
2. **Upload**: Actions → *Mac App Store* → *Run workflow*, on the tag (for example `v1.4.1`),
   with *upload* ticked. It builds the sandboxed app, checks it starts, signs it and the
   package, validates and uploads. The build number is the workflow's run number, which always
   rises, as App Store Connect requires.
3. **Wait for processing**: the build appears under TestFlight within about an hour, or App
   Store Connect emails the reason it was refused.
4. **TestFlight on a real Mac** (the first time at least): add a Mac colleague as an internal
   tester, or rent a Mac for a day with [mac-test-day.md](mac-test-day.md), and try what CI
   cannot:
   - granting a folder from the dialog, and a job resumed after quitting and reopening;
   - FFmpeg converting a recording reached through a granted folder, and the player playing it;
   - the consent dialog before a first cloud job;
   - opening results in TextEdit and showing them in Finder;
   - moving from the disk image: data copied into the container, and the Keychain's question;
   - the menu-bar icon, ⌘Q, the Dock and YouTube's player.
5. **The version page**: choose the build, paste the listing and the review notes from
   [store-listing.md](store-listing.md#mac-app-store-app-store-connect), add the screenshots, and
   choose *Manually release this version* for the first one. Without a Mac at hand, the
   workflow's *screenshots* input (Actions → *Mac App Store* → *Run workflow*) captures the
   sandboxed app on the runner at 1440 × 900, the start screen and Settings › Offline models, kept as
   the run's `store-screenshots` artifact: enough to submit, though the ten scenes of
   [store-listing.md](store-listing.md#screenshots) need a Mac and real material. The runner's
   screen offers no size App Store Connect takes, so it is set to 1600 × 1200 and the window
   alone is captured; controls are pressed by their accessible name.
6. **Submit for review**, then release once approved, and note the store link in the README,
   the roadmap and the release notes.

## What App Review may question

- **Downloaded models (2.4.5(iv), 2.5.2).** Whisper models, the OCR models and Tesseract
  language data are downloaded from Settings. They are weights and data, not code, for features already in the
  reviewed build; the review notes say so. If refused, the fallback is to bundle the smallest
  Whisper model and English language data in the store build.
- **Bring-your-own keys.** The review notes give an offline route through the sample files, and
  optionally a test key with a spending cap in the review fields, never in the listing.
- **Third-party AI (5.1.2(i)).** The notice before the first job to each provider names it,
  says what is sent and links the policy; the review notes point to it.
- **Screenshots (2.3.3, 2.3.8)** must show the Mac app, so they are taken on a Mac, and suit all
  audiences, since App Review reads the text in them as metadata. The Microsoft Store refused
  Kalamu 1.4.0 under its policy 11.5 for a 1915 newspaper masthead in two screenshots; the Mac
  set uses the material screened after that, *The Daily Nebraskan* of 1911, never the old
  captures ([store-listing.md](store-listing.md#screenshots)). The sample files the review notes
  point to were screened the same way.
- **Privacy manifest** (`PrivacyInfo.xcprivacy`): not checked whether App Store Connect wants
  one for a macOS app; the upload's validation names it if so.

## Not handled yet

- Jobs made by the disk image hold no bookmarks: in the store app, running one again needs its
  folder granted once, for example by adding the folder to a new job. A *grant this folder
  again* action on the job, and a list of the folders out of reach at launch, come next if
  testers meet it.
- French listing and screenshots, once the English one is through review.
