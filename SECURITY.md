# Security policy

## Supported versions

Kalamu is in early development: only the latest tagged release receives fixes. Installers
are published per tag on the [releases page](https://github.com/fmadore/Kalamu/releases).

## Reporting a vulnerability

Report privately through
[GitHub's private vulnerability reporting](https://github.com/fmadore/Kalamu/security/advisories/new)
rather than opening a public issue. Expect an acknowledgement within a week.

Dependabot security alerts are on, and `cargo audit` and `cargo deny` run in CI; a security
alert takes priority over feature work.

## How the app handles credentials

- Provider API keys (Gemini, OpenAI, Anthropic, Mistral) are stored in the system's keychain
  (Windows Credential Manager, the macOS Keychain) and read only by the Rust core, into the one request that needs them. They are never
  written to the front end's storage, the settings, the job database, `provenance.json` or the
  logs.
- The interface runs under a restrictive content security policy (`connect-src 'self'`): every
  call to a provider is made by the Rust core, not the web view. The only frame allowed is
  YouTube's privacy-enhanced player, loaded when the user plays a video.

## What leaves the machine

Files the user adds to a job go directly to the provider the user chose, under that provider's
terms; offline Whisper and Tesseract keep recordings and pages on the computer. User-requested
downloads, Whisper models from Hugging Face and Tesseract's language data from GitHub
(`tessdata_best` at a pinned commit), are checked against their size and SHA-256 before use. The app also fetches
the public model catalogue from this repository, and video titles from YouTube's oEmbed
service. The developer receives no files, results or keys. See the
[privacy policy](docs/privacy.md).

## Installer signing

The installers on GitHub Releases are **unsigned**, so Windows SmartScreen may warn on first
launch ("Windows protected your PC": choose **More info**, then **Run anyway**). Download them
only from the releases page linked above. From version 1.0.0 Kalamu is also in the
[Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H), whose copy is signed by
Microsoft and updated automatically ([docs/microsoft-store.md](docs/microsoft-store.md)). The `.msix` and `.msixbundle` files on a
release are the unsigned Store submission, for testing; they do not install as they are.

From version 1.4.0 the macOS disk image is **signed with an Apple Developer ID** (team
3HMSG8WB7K) **and notarised by Apple**, so Gatekeeper checks it before it opens. Versions up to
1.3.0 were signed ad hoc only. Download it only from the releases page
([docs/decisions/2026-11-04-macos-notarisation.md](docs/decisions/2026-11-04-macos-notarisation.md)).
