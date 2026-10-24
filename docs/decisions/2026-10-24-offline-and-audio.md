# 2026-10-24 · Tray, offline Whisper, FFmpeg and audio

## Context

The next round after the stage 2 checkpoint: keep jobs running with the window closed, add
offline transcription downloadable from the app as in Live Translation, bundle FFmpeg for
audio and video, then summaries and keywords. Provider documentation was re-read on this date.

## Decisions

- **Tray icon.** Closing the window while jobs run offers to keep them running in the
  background; the notification-area menu shows progress and can open the window, pause all
  jobs, or pause them and quit. Ported from Live Translation's `tray.rs`; the interface pushes
  translated labels.
- **Offline Whisper is a provider.** A new `offline` crate implements the `Provider` trait with
  whisper.cpp through whisper-rs 0.16 (bundling whisper.cpp 1.8.3), CPU only. Providers now say
  whether they need a key (`needs_key`) and whether a model can run (`check_ready`), so an
  undownloaded model or an unsupported processor is reported before the job starts. The
  catalogue marks the provider `local` for the interface.
- **Models are pinned.** Large v3 Turbo, Large v3, Small and Base (quantised), from a fixed
  Hugging Face revision with SHA-256 checks, plus Silero VAD 5.1.2 (the version whisper.cpp
  1.8.3 shipped with). whisper.cpp opens the VAD model by path, which the C runtime cannot do
  for non-ASCII Windows user folders; those users go without VAD rather than failing.
- **No unsafe code for cancellation.** whisper-rs 0.16's safe abort callback casts its user
  data wrongly, and Live Translation uses the unsafe one; this workspace forbids unsafe code.
  Instead a piece is transcribed in two-minute windows cut at the quietest moment, and the job's
  new `StopSignal` is checked between windows. A piece stopped this way returns
  `ProviderError::Interrupted` and goes back to pending.
- **zip advisories ignored, with reason.** RUSTSEC-2026-0367, -0368 and -0369 affect extracting
  zip archives to disk; docx-rs 0.4.22 (the latest) still needs zip 8, and Scriptorium only
  writes Word files and reads .docx entries into memory. Remove the ignores in
  `.cargo/audit.toml` when docx-rs moves to zip 9.
