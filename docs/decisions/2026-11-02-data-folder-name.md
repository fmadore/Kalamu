# 2026-11-02 · The data folder is named Kalamu Data

## Context

Tauri names an app's data folder after its bundle identifier, so Kalamu's was
`%LOCALAPPDATA%\io.github.fmadore.kalamu` on Windows and
`~/Library/Application Support/io.github.fmadore.kalamu` on macOS. The renaming of 28 October
kept that name rather than `%LOCALAPPDATA%\Kalamu`, where the per-user installer puts the
program and from which its uninstaller removes everything. Settings shows the folder, and a
reverse-domain name means nothing to the researchers who open it.

In the Store version, Windows keeps what a packaged app writes to `%LOCALAPPDATA%` inside the
package's own folder, `%LOCALAPPDATA%\Packages\49346FMadore.Kalamu_…\LocalCache\Local`, so the
path Settings showed was not where File Explorer finds the files.

## Decisions

- **`%LOCALAPPDATA%\Kalamu Data` on Windows; `~/Library/Application Support/Kalamu` and
  `~/Library/Logs/Kalamu` on macOS**, where the program does not live, so the usual name fits.
  The bundle identifier is unchanged.
- **Tauri's `app.appDirectoriesOverride`** (Tauri 2.12) points the app directories there, set
  at start by `platform::data_folder` rather than in `tauri.conf.json`, so each operating system
  has its own name. Tauri resolves WebView2's data from the same setting, so nothing is left in
  the old folder.
- **The earlier folder moves once**, before Tauri builds the window, as Scriptorium's did:
  the newest earlier folder found (`io.github.fmadore.kalamu`, then
  `io.github.fmadore.scriptorium`) is renamed while the new folder does not exist. If the
  rename fails (a file held open, the Store's file system refusing it), the earlier folder is
  left as it was and **stays in use**, so a researcher never opens an empty Kalamu; the log
  says so.
- **Settings shows the folder where File Explorer finds it**: in the Store version, the path
  inside the package's folder, its family name read from where the program runs
  (`WindowsApps\<name>_<version>_<architecture>_<resource>_<publisher>`).
- **Credential Manager keeps the entry name `io.github.fmadore.kalamu`.** Nobody sees it, and
  renaming it would mean moving every key again.

## Consequences

- The privacy policy names the new folders, with a new effective date.
- An older Kalamu started after a newer one finds no data in its folder and starts empty; the
  data is still in `Kalamu Data`.
- The move has been tested on Windows, on a copy of an earlier folder. A whole start in the new
  folder (logs and WebView2's data there), the Store package and macOS are first tried with the
  next release's build.
