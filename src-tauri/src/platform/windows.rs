//! Windows: WebView2 (Chromium) plays the review's recordings, Notepad opens JSON files, and the
//! window closes as Windows apps do, with the notification-area icon keeping running jobs.

use kalamu_media::Playback;
use tauri::{App, AppHandle, RunEvent};

/// The app a job's JSON files open in: Windows often has none for them, and every PC has
/// Notepad.
pub const JSON_APP: Option<&str> = Some("notepad");

/// Closing the window ends the app when no job runs, as on any Windows app.
pub const CLOSE_HIDES_WINDOW: bool = false;

/// A left click on the notification-area icon opens the window, as Windows users expect; the
/// menu is on right click.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = false;

/// What the review view's player (Chromium, in WebView2) plays.
pub const PLAYBACK: Playback = Playback::CHROMIUM;

/// The data folder's name before the app was renamed from Scriptorium (0.1 and 0.2).
const LEGACY_IDENTIFIER: &str = "io.github.fmadore.scriptorium";

/// Moves the jobs, settings, presets and Whisper models of a Scriptorium install in
/// `%LOCALAPPDATA%` to the renamed data folder, once, before anything is created there.
///
/// Runs before Tauri builds the window, since WebView2 creates its data in the new folder as
/// soon as the window exists.
pub fn move_legacy_data() -> std::io::Result<bool> {
    std::env::var_os("LOCALAPPDATA").map_or(Ok(false), |local| {
        move_legacy_into(&std::path::Path::new(&local).join(crate::DATA_FOLDER))
    })
}

/// Every stored path is relative to the folder, so it moves whole. If the move fails the old
/// folder is left untouched.
fn move_legacy_into(data: &std::path::Path) -> std::io::Result<bool> {
    let Some(legacy) = data.parent().map(|p| p.join(LEGACY_IDENTIFIER)) else {
        return Ok(false);
    };
    if data.exists() || !legacy.is_dir() {
        return Ok(false);
    }
    std::fs::rename(&legacy, data)?;
    Ok(true)
}

/// Nothing to add: Windows apps have no application menu.
pub fn init(_app: &mut App) -> tauri::Result<()> {
    Ok(())
}

/// No run event is Windows-only.
pub fn on_run_event(_app: &AppHandle, _event: RunEvent) {}

/// Nothing beyond the window itself to bring back.
pub fn show_app(_app: &AppHandle) {}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{LEGACY_IDENTIFIER, move_legacy_into};
    use crate::DATA_FOLDER;

    #[test]
    fn a_scriptorium_data_folder_moves_once() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(LEGACY_IDENTIFIER);
        let data = root.path().join(DATA_FOLDER);
        std::fs::create_dir_all(legacy.join("presets")).unwrap();
        std::fs::write(legacy.join("jobs.sqlite"), b"jobs").unwrap();

        assert!(move_legacy_into(&data).unwrap());
        assert_eq!(std::fs::read(data.join("jobs.sqlite")).unwrap(), b"jobs");
        assert!(data.join("presets").is_dir());
        assert!(!legacy.exists());

        // A later Scriptorium folder never replaces the Kalamu one.
        std::fs::create_dir_all(&legacy).unwrap();
        assert!(!move_legacy_into(&data).unwrap());
        assert!(legacy.exists());
    }
}
