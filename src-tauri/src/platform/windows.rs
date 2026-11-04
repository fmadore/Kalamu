//! Windows: WebView2 (Chromium) plays the review's recordings, Notepad opens JSON files, and the
//! window closes as Windows apps do, with the notification-area icon keeping running jobs.

use std::path::{Path, PathBuf};

use kalamu_media::Playback;
use tauri::utils::config::AppDirectoriesOverride;
use tauri::{App, AppHandle, RunEvent};

use super::Moved;
use crate::grants::Access;

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

/// The app's data folder in `%LOCALAPPDATA%`, named for the people who open it. Not `Kalamu`,
/// which the per-user installer takes for the program itself and its uninstaller removes.
const DATA_FOLDER: &str = "Kalamu Data";
/// Its earlier names, newest first: the bundle identifier, which Tauri names it after unless
/// told otherwise (1.0 to 1.1), and Scriptorium's (0.1 and 0.2).
const EARLIER_FOLDERS: [&str; 2] = ["io.github.fmadore.kalamu", "io.github.fmadore.scriptorium"];

/// Where Tauri keeps the app's data and WebView2's (the folder itself) and its logs (`logs`
/// inside it), after moving an earlier data folder there once.
///
/// Runs before Tauri builds the window, since WebView2 creates its data as soon as the window
/// exists.
pub fn data_folder() -> (AppDirectoriesOverride, Moved) {
    let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
        let folder = PathBuf::from("$LOCALDATA").join(DATA_FOLDER);
        return (AppDirectoriesOverride::Root(folder), Moved::Nothing);
    };
    let earlier = EARLIER_FOLDERS.map(|name| local.join(name));
    let (folder, moved) = super::settle(local.join(DATA_FOLDER), &earlier);
    (AppDirectoriesOverride::Root(folder), moved)
}

/// A folder of the app's as File Explorer finds it. The Store's package keeps what the app
/// writes to `%LOCALAPPDATA%` in its own folder, `Packages\<family>\LocalCache\Local`, so the
/// path the app uses is not where the researcher would look; the installer's app is.
pub fn shown_path(path: &Path) -> PathBuf {
    let family = std::env::current_exe()
        .ok()
        .and_then(|exe| package_family(&exe));
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    match (family, local) {
        (Some(family), Some(local)) => path.strip_prefix(&local).map_or_else(
            |_| path.to_path_buf(),
            |rest| {
                local
                    .join("Packages")
                    .join(family)
                    .join("LocalCache")
                    .join("Local")
                    .join(rest)
            },
        ),
        _ => path.to_path_buf(),
    }
}

/// The Store package the program runs from, if any: it is installed in
/// `WindowsApps\<name>_<version>_<architecture>_<resource>_<publisher>`, a package of the family
/// `<name>_<publisher>`.
fn package_family(exe: &Path) -> Option<String> {
    exe.ancestors().find_map(|dir| {
        let parent = dir.parent()?.file_name()?.to_str()?;
        if !parent.eq_ignore_ascii_case("WindowsApps") {
            return None;
        }
        let parts: Vec<&str> = dir.file_name()?.to_str()?.split('_').collect();
        match parts.as_slice() {
            [name, _, _, _, publisher] => Some(format!("{name}_{publisher}")),
            _ => None,
        }
    })
}

/// Nothing to add: Windows apps have no application menu.
pub fn init(_app: &mut App) -> tauri::Result<()> {
    Ok(())
}

/// No run event is Windows-only.
pub fn on_run_event(_app: &AppHandle, _event: RunEvent) {}

/// Nothing beyond the window itself to bring back.
pub fn show_app(_app: &AppHandle) {}

/// No sandbox: every path stays reachable, so nothing is kept. The Microsoft Store's package
/// is a full-trust app, which may read and write wherever the researcher can.
pub fn access() -> Access {
    Access {
        sandboxed: false,
        bookmark: |_| None,
        open: |_| None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::Path;

    use super::{EARLIER_FOLDERS, package_family};

    /// The folder the data moves from is the one earlier versions used: Tauri's, named after
    /// the bundle identifier.
    #[test]
    fn the_newest_earlier_folder_is_the_bundle_identifier() {
        let config = include_str!("../../tauri.conf.json");
        assert!(config.contains(&format!("\"identifier\": \"{}\"", EARLIER_FOLDERS[0])));
    }

    #[test]
    fn a_store_package_is_known_by_where_it_runs() {
        let store = Path::new(
            r"C:\Program Files\WindowsApps\49346FMadore.Kalamu_1.1.2.0_arm64__8wekyb3d8bbwe\Kalamu.exe",
        );
        assert_eq!(
            package_family(store).as_deref(),
            Some("49346FMadore.Kalamu_8wekyb3d8bbwe")
        );
        let installer = Path::new(r"C:\Users\you\AppData\Local\Kalamu\Kalamu.exe");
        assert_eq!(package_family(installer), None);
    }
}
