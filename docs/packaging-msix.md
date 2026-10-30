# MSIX packaging

How the Microsoft Store package is built, on CI and on your own machine, and how to install it
locally to check what only a packaged app shows. [microsoft-store.md](microsoft-store.md) has
the certification approach, the identity values and the acceptance checklist. Ported from Live
Translation's `docs/packaging-msix.md`; where Kalamu differs, the reason is given.

## What is committed

Tauri has no MSIX target, so packaging goes through
[`@choochmeque/tauri-windows-bundle`](https://github.com/Choochmeque/tauri-windows-bundle), a
devDependency. It runs `tauri build --no-bundle`, stages the payload and hands it to the
Windows SDK's `MakeAppx`.

| Path | What it is |
| --- | --- |
| `src-tauri/gen/windows/bundle.config.json` | Publisher, capabilities (`internetClient`; the bundler adds `runFullTrust`), resource index |
| `src-tauri/gen/windows/AppxManifest.xml.template` | The manifest, with the reserved identity written out and `{{PLACEHOLDER}}`s the bundler fills per architecture |
| `src-tauri/gen/windows/Assets/*.png` | Store logo, tiles and the 42 taskbar sizes, rendered by `scripts/msix-assets.py` |
| `scripts/bundle-msix.mjs` | `npm run bundle:msix:{x64,arm64}`: checks the identity and FFmpeg, then runs the bundler with FFmpeg's configuration |
| `scripts/third-party-notices.mjs` | Writes `licenses/THIRD-PARTY-NOTICES.txt`, which every installer and package carries |
| `.github/workflows/release.yml` → `msix`, `bundle` | Native x64 and ARM64 packages, then one `.msixbundle` for Partner Center |

`src-tauri/gen/` is otherwise generated and ignored; `.gitignore` keeps `gen/windows/`.

Things that are not what they look like:

- **`"displayName": "kalamu"` in `bundle.config.json` is a file name.** The bundler looks for
  `kalamu.exe` (the Cargo binary) in the target folder. Every name a user sees is a literal in
  the manifest template.
- **`Identity/Name` and both display names are literals.** The bundler would derive them from
  `tauri.conf.json`'s `identifier` and `productName`, which are not what Partner Center
  assigned. Until the reserved name is copied in, the template carries
  `PARTNER-CENTER-PACKAGE-NAME` and `scripts/bundle-msix.mjs` refuses to build, so a package
  the Store would reject at ingestion is never made.
- **FFmpeg is laid in for the build only.** The bundler reads `tauri.conf.json` and
  `tauri.windows.conf.json` and nothing else; Kalamu keeps FFmpeg in
  `tauri.ffmpeg.conf.json` so that a plain build or `tauri dev` does not need it fetched. The
  script copies it to `src-tauri/tauri.windows.conf.json` (ignored) for the length of the
  build, so the bundler stages `ffmpeg.exe` and `ffprobe.exe` beside `kalamu.exe` and the
  `licenses/` folder with them, then removes it.
- **No microphone.** Live Translation declares it; Kalamu reads recordings from files and never
  records, so its package asks for nothing beyond the network.

`Version` is stamped from `tauri.conf.json` with a fourth component: `1.0.0.0`.

## Icons and `resources.pri`

The manifest names one image per slot (`Square44x44Logo.png`, `Square150x150Logo.png`,
`Wide310x150Logo.png`, `StoreLogo.png`). Beside them, `Assets/` holds
`Square44x44Logo.targetsize-N.png` and its `_altform-unplated` and `_altform-lightunplated`
forms for N = 16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96 and 256: what the taskbar,
Start, Alt+Tab and jump lists draw, the unplated forms with no plate behind them. Windows finds
them only through the package resource index, which `"resourceIndex": { "enabled": true }`
generates; without it the taskbar shows the base logo on an accent-coloured plate.

`python scripts/msix-assets.py` renders them all, as [DESIGN.md](../DESIGN.md) sets out: up to
48 px from the simplified `src-tauri/icons/small.svg` (rendered by Edge at 1024 px on a
transparent background), from 64 px from the master `source.png`, each a Lanczos downscale. Run
it again whenever either changes. Do not pass `--regenerate-assets` to the bundler: its wide
tile comes out black.

## Building a package locally

Prerequisites: Rust through rustup, Node 22.13+ or 24 and `npm ci`, the Windows SDK (for
`MakeAppx` and `SignTool`, found through the registry), and LLVM, CMake and Ninja for
whisper.cpp ([development.md](development.md)). Then, for this PC's architecture:

```powershell
npm run fetch:ffmpeg
node scripts/third-party-notices.mjs
npm run bundle:msix:arm64
```

The package lands in `target/msix/`, named after the manifest's display name, with the payload
staged in `target/appx/arm64/` (the workspace's target folder: the script points the bundler
there with `CARGO_TARGET_DIR`). Check that it holds `kalamu.exe`,
`ffmpeg.exe`, `ffprobe.exe`, `licenses\` (FFmpeg, Opus and the third-party notices),
`AppxManifest.xml`, `resources.pri` and `Assets\`.

## Installing it locally

Windows installs no unsigned MSIX. Sign the per-architecture `.msix` (never the bundle, whose
inner packages stay unsigned) with a throwaway certificate whose subject is exactly the
manifest's publisher, `CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86`; Live Translation's
`scripts/install-local-msix.ps1` and its signing steps carry over unchanged, with Kalamu's
package name. The Store re-signs the submitted package with its own certificate.

Uninstall an earlier Kalamu package first if its version is the same or higher. A packaged app
keeps its data in its own folder under `%LOCALAPPDATA%\Packages\`, apart from an installer's.

## What CI does

The `msix` job runs on `windows-latest` (x64) and `windows-11-arm` (ARM64), fetches FFmpeg,
writes the notices and runs `npm run bundle:msix:<arch>`; it renames the package
`Kalamu_<version>_<arch>.msix`, attaches it to the draft release and hands it to `bundle`,
which combines both into `Kalamu_<version>.msixbundle` with `MakeAppx bundle` and attaches it.
`publish` checks every asset is there before the release goes public. The packages are
unsigned: Partner Center signs what it accepts.
