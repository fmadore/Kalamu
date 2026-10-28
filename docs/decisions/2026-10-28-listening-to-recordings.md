# 2026-10-28 · Listening to recordings in the review

## Context

The review's player was given each piece as Ogg Opus bytes from `piece_audio`, which asked FFmpeg
to decode and re-encode the whole piece, up to twenty minutes, before anything played, and did
it again on every visit. On an ARM64 laptop a twenty-minute piece of an M4A took about nine
seconds with the bundled FFmpeg; researchers found the review slow to open on recordings.

## Decision

- **Play the recording itself** when the web view plays its format (`kalamu_media::plays_as_is`:
  MP3, M4A, AAC, WAV, Ogg, Opus, FLAC, WebM, MP4). `piece_audio` now returns a `PieceAudio`
  (a path and the piece's span in that file) instead of bytes. The front end streams the file
  through Tauri's asset protocol, with range requests, starts at the piece's start and stops at
  its end, so the transcript's times stay on the piece's clock as before.
- **The asset protocol's scope starts empty**; `piece_audio` opens each file it names, and
  only those, for the life of the app. The CSP allows media from the asset protocol.
- **Fallback**: a format the web view cannot play, or a file it fails on (the `<audio>` element's
  error asks again with `convert`), is converted by FFmpeg as before, at Opus's fastest
  setting (`-compression_level 0`: the same size in about a quarter of the time), and kept in
  the `listening` folder of the app's data, keyed by the recording's SHA-256 and the piece's
  span. A second visit is instant; the folder is kept under 256 MB by removing the copies
  heard longest ago.
- What is sent to providers is unchanged: `prepare` keeps its settings, and the fast setting
  is only reachable through `Ffmpeg::extract_to_listen`.

## Consequences

- A piece of a common recording starts as soon as the player has read the file's header.
- The player seeks within the original file. For a variable-bit-rate MP3 without an index,
  Chromium's seek can land slightly off the cut FFmpeg made; the highlighted line may then lead
  or lag by a fraction of a second.
- The app keeps a new kind of file on disk (privacy policy updated).
