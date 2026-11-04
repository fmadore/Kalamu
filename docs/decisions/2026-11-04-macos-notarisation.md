# 2026-11-04 · Signing and notarising the Mac app

## Context

Up to 1.3.0 the macOS disk image was signed ad hoc ([macOS decision](2026-11-02-macos.md)):
Apple Silicon runs nothing unsigned, but an ad-hoc signature names no developer, so Gatekeeper
refused the first opening and sent people to System Settings → Privacy & Security, and the
Keychain could ask again after an update, since each build's signature differed. The maintainer
now has an Apple Developer Program membership (team `3HMSG8WB7K`).

## Decisions

- **Developer ID, not the Mac App Store.** Kalamu is distributed outside the store, so it needs
  a *Developer ID Application* certificate and notarisation only: no App ID, provisioning
  profile or capability is registered. The Mac App Store stays out for the reasons the macOS
  decision gives.
- **The certificate was made without a Mac.** The private key and certificate request were
  generated with OpenSSL on Windows, the certificate issued under Apple's G2 Developer ID
  authority, and both packed into a `.p12` with SHA-1/3DES encryption, which macOS's
  `security import` accepts (OpenSSL 3's default it does not). The `.p12` and its password are
  kept in the maintainer's password manager. The certificate expires on 4 December 2027 and must
  be renewed before a release after that date.
- **Six repository secrets** carry it to the release workflow: `APPLE_CERTIFICATE` (the `.p12`,
  base64), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`
  (`Developer ID Application: Frédérick Madore (3HMSG8WB7K)`), and for notarisation `APPLE_ID`,
  `APPLE_PASSWORD` (an app-specific password, not the account's) and `APPLE_TEAM_ID`. Tauri's
  bundler imports the certificate into a temporary keychain, signs FFmpeg and ffprobe and then
  the app with the hardened runtime, has Apple's notary service check the app, and staples the
  ticket to it.
- **Local builds keep ad-hoc signing.** `tauri.macos.conf.json` keeps `"signingIdentity": "-"`;
  `APPLE_SIGNING_IDENTITY` overrides it in the release only, so CI's macOS lane and builds
  without the secrets still produce an app that runs.
- **A release cannot fall back silently.** Without the secrets the bundler would sign ad hoc and
  skip notarisation with only a warning, so the release's macOS job stops first if any secret
  is missing, and after the build checks, in the disk image as a user gets it, that the app,
  FFmpeg and ffprobe carry the Developer ID and the hardened runtime, that the ticket is
  stapled (`stapler validate`) and that Gatekeeper accepts the app (`spctl --assess`).
- **No entitlements.** Whisper runs on the CPU only, nothing records from the microphone, and the
  app is not sandboxed, so the hardened runtime needs no exception.

## Consequences

- From 1.4.0, macOS opens Kalamu after its usual question about an app
  downloaded from the internet, without the detour through System Settings.
- Keys saved by an ad-hoc build belong, for the Keychain, to a different app: on the first
  signed version macOS asks once more to let Kalamu use them. From then on the signature's
  requirement names the team, so updates no longer ask.
- A release's macOS job now takes longer, by the time Apple's notary service needs: usually
  minutes, but the first test submission was still queued after 80 minutes, so the job's limit
  is 240 minutes rather than 90.
- The README, security policy, specification, roadmap and development notes describe the signed
  image. The privacy policy is unchanged: notarisation sends Apple the app at build time, not
  anything of a user's, and Gatekeeper's own online check is macOS's, not Kalamu's.
- Still to try on a real Mac: the first opening of a downloaded image and the Keychain's one
  prompt after updating from an ad-hoc build.
