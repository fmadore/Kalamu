//! macOS: WebKit plays the review's recordings, TextEdit opens JSON files, and the app behaves
//! as Mac apps do: closing the window leaves it in the Dock, the Dock icon brings the window
//! back, and Quit asks first while jobs run. In the Mac App Store's build, the App Sandbox's
//! grants are kept as security-scoped bookmarks.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use kalamu_media::Playback;
use objc2::runtime::Bool;
use objc2_foundation::{
    NSData, NSString, NSURL, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions,
};
use tauri::image::Image;
use tauri::menu::{HELP_SUBMENU_ID, Menu, MenuItem, MenuItemKind, PredefinedMenuItem};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{App, AppHandle, Emitter, Manager, RunEvent};
use tauri_plugin_opener::OpenerExt;

use super::Moved;
use crate::commands::AppState;
use crate::commands::window::{jobs_running, show_main_window};
use crate::grants::Access;

/// The app a job's JSON files open in; every Mac has TextEdit.
pub const JSON_APP: Option<&str> = Some("TextEdit");

/// The red button hides the window and the app stays in the Dock, as Mac apps do.
pub const CLOSE_HIDES_WINDOW: bool = true;

/// Menu-bar icons open their menu on a click.
pub const TRAY_MENU_ON_LEFT_CLICK: bool = true;

/// The menu-bar icon: a template image, black shapes on clear, which macOS tints to suit a
/// light, dark or tinted menu bar, as every menu-bar icon does. Drawn from `icons/small.svg`'s
/// two sheets (`icons/tray-template.svg`).
pub const TRAY_TEMPLATE: Option<Image<'static>> =
    Some(tauri::include_image!("icons/tray-template.png"));

/// What the review view's player (WebKit) plays.
pub const PLAYBACK: Playback = Playback::WEBKIT;

const ID_QUIT: &str = "app-quit";
const ID_SETTINGS: &str = "app-settings";
const ID_REPORT: &str = "help-report";
const ID_WEBSITE: &str = "help-website";

/// Where Help › Report a Problem… goes: GitHub's choice of forms, as Settings' link
/// (`src/lib/support.ts`).
const REPORT_URL: &str = "https://github.com/fmadore/Kalamu/issues/new/choose";
/// Where Help › Kalamu Website goes: the project's page, with its README.
const WEBSITE_URL: &str = "https://github.com/fmadore/Kalamu";

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
/// shortcuts, with what a Mac app's menus hold: Settings… (Cmd+,) under About, a Help menu
/// that is not empty, and Quit replaced, since the system's ends the process at once, without
/// the question asked while jobs run or the wait for pieces already sent. In English, as the
/// rest of Tauri's default menu is.
pub fn init(app: &mut App) -> tauri::Result<()> {
    let handle = app.handle().clone();
    let menu = Menu::default(&handle)?;
    // The application menu comes first: About, then Settings…, and Quit last.
    if let Some(MenuItemKind::Submenu(app_menu)) = menu.items()?.into_iter().next() {
        if let Some(quit) = app_menu.items()?.into_iter().last() {
            app_menu.remove(&quit)?;
        }
        let name = handle.package_info().name.clone();
        app_menu.insert(
            &MenuItem::with_id(&handle, ID_SETTINGS, "Settings…", true, Some("CmdOrCtrl+,"))?,
            1,
        )?;
        app_menu.insert(&PredefinedMenuItem::separator(&handle)?, 1)?;
        app_menu.append(&MenuItem::with_id(
            &handle,
            ID_QUIT,
            format!("Quit {name}"),
            true,
            Some("CmdOrCtrl+Q"),
        )?)?;
    }
    // Tauri's Help menu, empty on macOS, which adds its search field.
    if let Some(MenuItemKind::Submenu(help)) = menu.get(HELP_SUBMENU_ID) {
        let name = handle.package_info().name.clone();
        help.append(&MenuItem::with_id(
            &handle,
            ID_WEBSITE,
            format!("{name} Website"),
            true,
            None::<&str>,
        )?)?;
        help.append(&MenuItem::with_id(
            &handle,
            ID_REPORT,
            "Report a Problem…",
            true,
            None::<&str>,
        )?)?;
    }
    app.set_menu(menu)?;
    app.on_menu_event(|app, event| match event.id().as_ref() {
        ID_QUIT => request_quit(app),
        ID_SETTINGS => {
            show_main_window(app);
            let _ = app.emit(crate::OPEN_SETTINGS_EVENT, ());
        }
        ID_REPORT => open_url(app, REPORT_URL),
        ID_WEBSITE => open_url(app, WEBSITE_URL),
        // The menu-bar icon's own items, handled in `tray.rs`.
        _ => {}
    });
    Ok(())
}

/// Opens `url` in the researcher's browser.
fn open_url(app: &AppHandle, url: &str) {
    if let Err(error) = app.opener().open_url(url, None::<&str>) {
        tracing::warn!(url, %error, "could not open the page");
    }
}

/// A click on the Dock icon, or opening the app again from Finder, brings back the window the
/// red button hid: it does not start a second process. When the app quits, the programs it
/// started end with it ([`keep_children`]).
pub fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::Reopen { .. } => show_main_window(app),
        RunEvent::Exit => end_children(),
        _ => {}
    }
}

/// The programs the app started (FFmpeg and ffprobe, llama.cpp, the Whisper helper) are kept
/// by process id while they run, and ended when the app quits by any of its own ways: Quit,
/// a second Quit, the window closed while idle, logging out.
///
/// macOS has no Job Object, and nothing ends a child when its parent dies: after a crash, a
/// Force Quit or a `kill`, a program goes on until it next writes to a pipe of the app's, which
/// then ends it; one that is computing in silence, or hung, runs to its end. A program's id is
/// forgotten once it was reaped, so the app never ends a later process that was given it.
pub fn keep_children() {
    kalamu_media::program::keep_with(kalamu_media::program::Keeper {
        adopt: |child| {
            running().insert(child.id());
        },
        release: |id| {
            running().remove(&id);
        },
    });
}

/// The process ids of the programs the app started and has not reaped yet.
static RUNNING: Mutex<BTreeSet<u32>> = Mutex::new(BTreeSet::new());

fn running() -> MutexGuard<'static, BTreeSet<u32>> {
    RUNNING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Ends every program the app started that still runs, as `Child::kill` does.
// The crate denies unsafe code; `kill` is the exception, since every call to the C library is
// unsafe.
#[allow(unsafe_code)]
fn end_children() {
    for id in std::mem::take(&mut *running()) {
        if let Ok(pid) = libc::pid_t::try_from(id) {
            // SAFETY: `kill` takes plain numbers; the id is a child of this process not yet
            // reaped, so it names no other process.
            unsafe { libc::kill(pid, libc::SIGKILL) };
        }
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

/// The App Sandbox's grants, kept as security-scoped bookmarks. Only the Mac App Store's build
/// is sandboxed: macOS then sets `APP_SANDBOX_CONTAINER_ID` for the app.
pub fn access() -> Access {
    Access {
        sandboxed: std::env::var_os("APP_SANDBOX_CONTAINER_ID").is_some(),
        bookmark,
        open: open_bookmark,
    }
}

/// Whether a store keeps this copy up to date: the Mac App Store's build, the sandboxed one. It
/// must not offer updates of its own; the disk image's copy is told of a new release.
pub fn updated_by_store() -> bool {
    access().sandboxed
}

/// A security-scoped bookmark of a path the app can reach now.
fn bookmark(path: &Path) -> Option<Vec<u8>> {
    let url = NSURL::fileURLWithPath(&NSString::from_str(path.to_str()?));
    url.bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
        NSURLBookmarkCreationOptions::WithSecurityScope,
        None,
        None,
    )
    .ok()
    .map(|data| data.to_vec())
}

/// Where a bookmark leads now, with access opened for the rest of the session, and whether
/// the system asks for it to be made again. Access is never closed: the app may need any kept
/// path until it quits, when the system closes it.
// The crate denies unsafe code; these two Foundation calls are the exception, since objc2
// marks every call taking a raw pointer, or not yet audited, as unsafe.
#[allow(unsafe_code)]
fn open_bookmark(data: &[u8]) -> Option<(PathBuf, bool)> {
    let data = NSData::with_bytes(data);
    let mut stale = Bool::NO;
    // SAFETY: `stale` is a live local for the whole call, as the pointer must be.
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data,
            NSURLBookmarkResolutionOptions::WithSecurityScope,
            None,
            &mut stale,
        )
    }
    .ok()?;
    // SAFETY: a plain method call on the URL resolved above, with no arguments.
    if !unsafe { url.startAccessingSecurityScopedResource() } {
        return None;
    }
    let path = PathBuf::from(url.path()?.to_string());
    Some((path, stale.as_bool()))
}
