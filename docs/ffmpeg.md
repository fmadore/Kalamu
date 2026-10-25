# FFmpeg in Scriptorium

How the app uses FFmpeg, how the bundled build is made, and what keeps it compatible with an
MIT-licensed app distributed through the Microsoft Store.

## What it does

FFmpeg and ffprobe run as separate programs, started by the `media` crate
(`crates/media/src/ffmpeg.rs`). FFmpeg is never linked into Scriptorium.

- **Length and streams** of a recording or video (`ffprobe`), when files are added.
- **Checks before a job**: no sound track, very long recordings, and very quiet ones (loudness
  sampled on up to three one-minute excerpts).
- **Pieces cut at pauses**: one decoding pass with `silencedetect` (−35 dB, at least 0.5 s) and
  `volumedetect`. A recording is cut into pieces of about 20 minutes (less when a model's
  catalogue limit is lower), at the longest pause in the 90 seconds before each mark. With no
  pause, the cut is made at the mark and the next piece repeats two seconds; segments that end
  inside the repeat are dropped when the transcript is assembled. The pieces are stored with
  the job, so resuming never decodes the file again or moves a cut.
- **Conversion**: each piece as mono 16 kHz audio in the best format the provider takes. That
  is Opus at 32 kbit/s for the cloud (in Ogg, or in WebM for OpenAI, which does not list Ogg or
  FLAC), FLAC or WAV as fallbacks, and raw 16-bit samples for Whisper. Video gives its first
  audio track. A file sent whole in a format the provider takes is not converted.
- **Evening out the volume** (`dynaudnorm`), on request, for quiet or uneven speakers. It is
  recorded in `provenance.json` and in the cache key.

Development builds also find an FFmpeg on `PATH`; release builds look only next to the app's
executable, or in `SCRIPTORIUM_FFMPEG_DIR`. Without FFmpeg, recordings in a format the provider
takes are sent whole, and video is unavailable.

## The bundled build

`.github/workflows/ffmpeg.yml` builds FFmpeg 9.0.2 with libopus 1.6.1 for Windows x64 and
ARM64, cross-compiled with llvm-mingw:

- **Audio only.** No video decoders or encoders, no network protocols, no external libraries
  besides libopus. The demuxers read the usual audio files and the audio track of MP4, MOV, MKV,
  WebM, AVI, FLV and MPEG-TS videos.
- **LGPL v2.1 or later only.** No `--enable-gpl`, `--enable-version3` or `--enable-nonfree`. The
  workflow fails unless FFmpeg's own `config.h` reports "LGPL version 2.1 or later", and a second
  job checks `ffmpeg -L` on each architecture.
- **About 10 MB** per architecture instead of the 120–170 MB of general-purpose builds.

Each run uploads the two programs with the LGPL text, Opus's BSD licence, the exact configure
line (`BUILD.txt`) and the source tarballs they were built from. After a run, the archives are
published as assets of a GitHub release (`ffmpeg-<version>-<n>`) and their SHA-256 written into
`scripts/ffmpeg.json`. `npm run fetch:ffmpeg` then installs the pinned archive into
`src-tauri/binaries/`, and release builds add `--config src-tauri/tauri.ffmpeg.conf.json`, which
bundles both programs as sidecars and their licences as resources. Development builds need
none of this.

## Licence obligations, and how each is met

From FFmpeg's [licence checklist](https://ffmpeg.org/legal.html):

| Obligation | How Scriptorium meets it |
| --- | --- |
| No GPL or non-free parts | Configure flags above; checked by the workflow |
| Separate, replaceable library code | Separate programs, not linked; names unchanged |
| Offer the exact source, with any changes and the build configuration | Source tarballs and `BUILD.txt` published with every bundled build; FFmpeg is not modified |
| Credit FFmpeg and the LGPL where the app is downloaded and in the About box | Store listing, GitHub releases and the About section of Settings |
| Mention FFmpeg and the LGPL in the EULA; claim no ownership of it | `docs/store-license-terms.md` |
| Do not forbid reverse engineering | Our own licence terms replace Microsoft's standard ones (below) |

## Microsoft Store

Two points matter for the Store:

- **Licence terms.** Without its own terms, a Store app is licensed under Microsoft's Standard
  Application License Terms, which forbid reverse engineering. FFmpeg's LGPL requires allowing
  it. Partner Center's optional **Additional license terms** field (under Store listings)
  replaces those standard terms, so the submission must point it to
  `docs/store-license-terms.md`, or paste that text.
- **Running the programs.** Desktop (full-trust) MSIX apps may start executables in their own
  package. The programs are started with `CREATE_NO_WINDOW`, so no console window appears, and
  they read only the files the user chose.

Checked against [Microsoft Store Policies 7.20](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)
(14 September 2026):

- **10.2.2, dynamic code**: an app must not change its described functionality by code it
  downloads. FFmpeg ships inside the package and is never downloaded; the Whisper models and the
  model catalogue that the app downloads are data, not code.
- **11.2, third-party content**: bundled content must be appropriately licensed, which the
  table above covers.
- **10.2.9, installers submitted by link**: only if the app were submitted as an `.exe` or
  `.msi` URL instead of MSIX, every executable in it, `ffmpeg.exe` and `ffprobe.exe` included,
  would need a code-signing certificate from a trusted authority. MSIX packages are signed by
  the Store, so the MSIX route avoids this.

Users never install FFmpeg themselves: it is part of the app on every route (Store, GitHub
installers). This is not legal advice; check the policies again before each submission.
