# Security policy

## Supported versions

Scriptorium is in early development: only the latest tagged release receives fixes. Installers
are published per tag on the [releases page](https://github.com/fmadore/Scriptorium/releases).

## Reporting a vulnerability

Report privately through
[GitHub's private vulnerability reporting](https://github.com/fmadore/Scriptorium/security/advisories/new)
rather than opening a public issue. Expect an acknowledgement within a week.

Dependabot security alerts are on, and `cargo audit` and `cargo deny` run in CI; a security
alert takes priority over feature work.

## How the app handles credentials

- Provider API keys (Gemini, OpenAI, Anthropic, Mistral) are stored in Windows Credential
  Manager and read only by the Rust core, into the one request that needs them. They are never
  written to the front end's storage, the settings, the job database, `provenance.json` or the
  logs.
- The interface runs under a restrictive content security policy (`connect-src 'self'`): every
  call to a provider is made by the Rust core, not the web view. The only frame allowed is
  YouTube's privacy-enhanced player, loaded when the user plays a video.

## What leaves the machine

Files the user adds to a job go directly to the provider the user chose, under that provider's
terms; offline Whisper keeps recordings on the PC. User-requested Whisper downloads come from
Hugging Face and are checked against their size and SHA-256 before use. The app also fetches
the public model catalogue from this repository, and video titles from YouTube's oEmbed
service. The developer receives no files, results or keys. See the
[privacy policy](docs/privacy.md).

## Installer signing

The installers on GitHub Releases are **unsigned**, so Windows SmartScreen may warn on first
launch ("Windows protected your PC": choose **More info**, then **Run anyway**). Download them
only from the releases page linked above. A Microsoft Store release, signed by Microsoft and
updated automatically, is in preparation ([docs/microsoft-store.md](docs/microsoft-store.md)).
