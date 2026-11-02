//! macOS: WebKit plays the review's recordings, TextEdit opens JSON files, and the app behaves
//! as Mac apps do: closing the window leaves it in the Dock, the Dock icon brings the window
//! back, and Quit asks first while jobs run.

use kalamu_media::Playback;
use tauri::menu::{Menu, MenuItem, MenuItemKind};
use tauri::{App, AppHandle, Emitter, Manager, RunEvent};

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

/// No older data folder to move: Kalamu was never built for macOS under its first name.
pub fn move_legacy_data() -> std::io::Result<bool> {
    Ok(false)
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
