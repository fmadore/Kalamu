Kalamu 1.8.1 fixes the scan in the review, which in 1.8.0 could keep resizing, flickering
between two widths and staying blurry, when the window was not full screen. Nothing else changes
for you. For what 1.8.0 brought (records, text regions over the scan, a batch's files deleted,
Gemini 4 Hydrogen), see the [1.8.0 release](https://github.com/fmadore/Kalamu/releases/tag/v1.8.0).

## Fixed

- **The scan in the review stays still**
  ([#192](https://github.com/fmadore/Kalamu/pull/192)). The page is drawn as wide as its pane.
  At some window sizes it was just tall enough for a scrollbar to appear. The scrollbar made the
  pane narrower, so the page was drawn smaller and fitted. The scrollbar then went, the page was
  drawn larger again, and so on, many times a second. The pane now keeps the scrollbar's room
  whether it shows or not. Seen on Windows; on a Mac only with scrollbars set to always show.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.8.1_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.8.1_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.8.1_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.8.1.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.8.1 over an earlier version keeps your jobs, settings, presets and
keys. 1.8.1 uses the same job database as 1.8.0; a version before 1.8.0 refuses it.

## Known limitations

Those of [1.8.0](https://github.com/fmadore/Kalamu/releases/tag/v1.8.0) still hold. The fix
relies on the webview keeping a scrollbar's room, which macOS's WebKit does from Safari 18.2: a
Mac with an older system WebKit and scrollbars set to always show may still see the page resize.
The fix has not yet been tried on a real Mac.

## Included software

The same as 1.8.0: FFmpeg 9.0.2 (audio only) under the LGPL v2.1 or later, with libopus (BSD),
whose source, build configuration and licences are in the
[ffmpeg-9.0.2-2 release](https://github.com/fmadore/Kalamu/releases/tag/ffmpeg-9.0.2-2);
llama.cpp b12110 (MIT), described in the
[llama-b12110-1 release](https://github.com/fmadore/Kalamu/releases/tag/llama-b12110-1);
whisper.cpp (MIT), Tesseract 5.5.2 (Apache 2.0) with Leptonica (BSD), and hayro (MIT). Every
component is listed with its licence in `licenses/THIRD-PARTY-NOTICES.txt`.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
