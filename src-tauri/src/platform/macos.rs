//! macOS: WebKit plays the review's recordings, TextEdit opens JSON files, and the app behaves
//! as Mac apps do: closing the window leaves it in the Dock, the Dock icon brings the window
//! back, and Quit asks first while jobs run.

use std::path::{Path, PathBuf};

use kalamu_media::Playback;
use tauri::menu::{Menu, MenuItem, MenuItemKind};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{App, AppHandle, Emitter, Manager, RunEvent};

use super::Moved;
use crate::commands::AppState;
use crate::commands::window::{jobs_running, show_main_window};

/// The app a job's JSON files open in; every Mac has TextEdit.
pub const JSON_APP: Option<&str> = Some("TextEdit");

/// The red button hides the window and the app stays in the Dock, as Mac apps do.
pub const CLOSE_HIDES_WINDOW: bool = true;

/// Menu-bar icons open their menu on a click.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = true;

/// What the review view's player (WebKit) plays.
pub const PLAYBACK: Playback = Playback::WEBKIT;

const ID_QUIT: &str = "app-quit";

/// The data folder in `~/Library/Application Support` and the logs' in `~/Library/Logs`, named
/// as Mac apps name theirs.
const FOLDER: &str = "Kalamu";
/// Their earlier name, the bundle identifier, which Tauri names them after unless told
/// otherwise (1.1.2). Kalamu was never built for macOS under its first name.
const EARLIER_FOLDER: &str = "io.github.fmadore.kalamu";

/// Where Tauri keeps the app's data and its logs, after moving earlier folders there once.
pub fn data_folder() -> (AppDirectoriesOverride, Moved) {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        let dirs = folders(
            PathBuf::from("$DATA").join(FOLDER),
            PathBuf::from("$HOME/Library/Logs").join(FOLDER),
        );
        return (dirs, Moved::Nothing);
    };
    let support = home.join("Library/Application Support");
    let logs = home.join("Library/Logs");
    let (data, moved) = super::settle(support.join(FOLDER), &[support.join(EARLIER_FOLDER)]);
    let (log, _) = super::settle(logs.join(FOLDER), &[logs.join(EARLIER_FOLDER)]);
    (folders(data, log), moved)
}

/// The data, settings and logs folders; the cache keeps Tauri's place, which the app never uses.
fn folders(data: PathBuf, log: PathBuf) -> AppDirectoriesOverride {
    AppDirectoriesOverride::Directories(AppDirectoryOverrides {
        config: Some(data.clone()),
        data: Some(data.clone()),
        local_data: Some(data),
        cache: None,
        log: Some(log),
    })
}

/// A folder of the app's as Finder finds it: where the app keeps it.
pub fn shown_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

/// Tauri's default application menu, which gives the Edit commands (Cmd+C, Cmd+V…) their
/// shortcuts, with its Quit replaced: the system's ends the process at once, without the
/// question asked while jobs run or the wait for pieces already sent.
pub fn init(app: &mut App) -> tauri::Result<()> {
    let handle = app.handle().clone();
    let menu = Menu::default(&handle)?;
    // The application menu comes first and ends with Quit.
    if let Some(MenuItemKind::Submenu(app_menu)) = menu.items()?.into_iter().next() {
        if let Some(quit) = app_menu.items()?.into_iter().last() {
            app_menu.remove(&quit)?;
        }
        let name = handle.package_info().name.clone();
        app_menu.append(&MenuItem::with_id(
            &handle,
            ID_QUIT,
            format!("Quit {name}"),
            true,
            Some("CmdOrCtrl+Q"),
        )?)?;
    }
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| {
        if event.id() == ID_QUIT {
            request_quit(app);
        }
    });
    Ok(())
}

/// A click on the Dock icon, or opening the app again from Finder, brings back the window the
/// red button hid: it does not start a second process.
pub fn on_run_event(app: &AppHandle, event: RunEvent) {
    if let RunEvent::Reopen { .. } = event {
        show_main_window(app);
    }
}

/// Quit from the application menu: at once when idle; otherwise the window comes back and the
/// interface asks whether to pause the jobs or keep them running from the menu bar.
fn request_quit(app: &AppHandle) {
    if jobs_running(&app.state::<AppState>()) {
        show_main_window(app);
        let _ = app.emit(crate::CLOSE_REQUESTED_EVENT, ());
    } else {
        app.exit(0);
    }
}

/// Brings the app back after Cmd+H hid it.
pub fn show_app(app: &AppHandle) {
    let _ = app.show();
}
