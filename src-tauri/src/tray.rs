//! The notification-area icon, so jobs keep running with the window closed.
//!
//! Ported from Live Translation's tray. The menu holds no state of its own: the interface
//! pushes the job status and the translated labels through `set_tray_state`, and they are
//! written onto the live items. Open, Pause all and Quit need nothing from the interface, so
//! they keep working while the window is hidden or the interface is busy.

use std::sync::Mutex;

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use ts_rs::TS;

use crate::commands::AppState;
use crate::commands::window::{pause_and_quit, pause_running, show_main_window};

const TRAY_ID: &str = "main";
const ID_OPEN: &str = "tray-open";
const ID_PAUSE: &str = "tray-pause";
const ID_QUIT: &str = "tray-quit";

/// What the interface knows and the menu shows.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TrayState {
    /// Whether any job is running, which enables *Pause all*.
    pub running: bool,
    pub labels: TrayLabels,
}

/// The menu's text, translated by the interface.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TrayLabels {
    /// The first, disabled line: "2 jobs running · 45%" or "No jobs running".
    pub status: String,
    pub tooltip: String,
    pub open: String,
    pub pause: String,
    pub quit: String,
}

/// The two writes a live menu item takes. A trait so the write-only-what-changed bookkeeping
/// can be tested against a fake item, without a running app to build a menu in.
trait MenuEntry {
    fn set_text(&self, text: &str) -> tauri::Result<()>;
    fn set_enabled(&self, enabled: bool) -> tauri::Result<()>;
}

impl MenuEntry for MenuItem<Wry> {
    fn set_text(&self, text: &str) -> tauri::Result<()> {
        MenuItem::set_text(self, text)
    }

    fn set_enabled(&self, enabled: bool) -> tauri::Result<()> {
        MenuItem::set_enabled(self, enabled)
    }
}

struct Items<I = MenuItem<Wry>> {
    status: I,
    open: I,
    pause: I,
    quit: I,
    written: Written,
}

/// What was last written onto each item. The interface pushes on every progress refresh, but
/// usually only the status line has changed; each write is a native menu call.
#[derive(Default)]
struct Written {
    status: String,
    tooltip: String,
    open: String,
    pause: String,
    quit: String,
    pause_enabled: Option<bool>,
}

/// Write `text` unless it is what the item already shows. A write that fails is not
/// remembered, so the next push tries it again.
fn write_text(item: &impl MenuEntry, written: &mut String, text: &str) {
    if written != text && item.set_text(text).is_ok() {
        *written = text.to_owned();
    }
}

impl<I: MenuEntry> Items<I> {
    /// Write the interface's view onto the menu. Returns the tooltip when it changed, since
    /// the tooltip belongs to the icon rather than to a menu item.
    fn apply(&mut self, state: &TrayState) -> Option<String> {
        let labels = &state.labels;
        let written = &mut self.written;
        write_text(&self.status, &mut written.status, &labels.status);
        write_text(&self.open, &mut written.open, &labels.open);
        write_text(&self.pause, &mut written.pause, &labels.pause);
        write_text(&self.quit, &mut written.quit, &labels.quit);
        // Disabled rather than hidden, so "nothing to pause" reads differently from "no such
        // entry".
        if written.pause_enabled != Some(state.running)
            && self.pause.set_enabled(state.running).is_ok()
        {
            written.pause_enabled = Some(state.running);
        }
        (written.tooltip != labels.tooltip).then(|| {
            written.tooltip.clone_from(&labels.tooltip);
            labels.tooltip.clone()
        })
    }
}

/// The live menu items, once the tray has been built.
#[derive(Default)]
pub struct TrayMenu {
    items: Mutex<Option<Items>>,
}

impl TrayMenu {
    pub fn apply(&self, app: &AppHandle, state: &TrayState) {
        let mut guard = self.items.lock().unwrap_or_else(|e| e.into_inner());
        let Some(items) = guard.as_mut() else {
            return;
        };
        if let Some(tooltip) = items.apply(state)
            && let Some(tray) = app.tray_by_id(TRAY_ID)
        {
            let _ = tray.set_tooltip(Some(tooltip));
        }
    }
}

/// Build the icon and its menu. Called once from `setup`. The English labels are placeholders
/// until the interface pushes its translated ones, a moment after launch.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "tray-status", "Kalamu", false, None::<&str>)?;
    let open = MenuItem::with_id(app, ID_OPEN, "Open Kalamu", true, None::<&str>)?;
    // Starts disabled: nothing runs a moment after launch.
    let pause = MenuItem::with_id(app, ID_PAUSE, "Pause all jobs", false, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&status, &open, &pause, &separator, &quit])?;

    app.state::<TrayMenu>()
        .items
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .replace(Items {
            status,
            open,
            pause,
            quit,
            written: Written::default(),
        });

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        // Left click opens the window, as Windows users expect; the menu is on right click.
        .show_menu_on_left_click(false)
        .tooltip("Kalamu")
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            // Single and double click both open: showing a window twice costs nothing.
            let opens = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );
            if opens {
                show_main_window(tray.app_handle());
            }
        });
    // The bundle icon, so the tray matches the taskbar.
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        ID_OPEN => show_main_window(app),
        ID_PAUSE => {
            pause_running(&app.state::<AppState>());
        }
        ID_QUIT => {
            let app = app.clone();
            tauri::async_runtime::spawn(pause_and_quit(app));
        }
        other => tracing::warn!("unknown tray menu id: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    /// A menu item that records what was written to it, and can be told to refuse writes.
    #[derive(Default)]
    struct Fake {
        texts: RefCell<Vec<String>>,
        enabled: RefCell<Vec<bool>>,
        refuse: Cell<bool>,
    }

    impl MenuEntry for Fake {
        fn set_text(&self, text: &str) -> tauri::Result<()> {
            if self.refuse.get() {
                return Err(tauri::Error::FailedToReceiveMessage);
            }
            self.texts.borrow_mut().push(text.to_owned());
            Ok(())
        }

        fn set_enabled(&self, enabled: bool) -> tauri::Result<()> {
            if self.refuse.get() {
                return Err(tauri::Error::FailedToReceiveMessage);
            }
            self.enabled.borrow_mut().push(enabled);
            Ok(())
        }
    }

    fn items() -> Items<Fake> {
        Items {
            status: Fake::default(),
            open: Fake::default(),
            pause: Fake::default(),
            quit: Fake::default(),
            written: Written::default(),
        }
    }

    fn state(running: bool, status: &str) -> TrayState {
        TrayState {
            running,
            labels: TrayLabels {
                status: status.into(),
                tooltip: format!("Kalamu: {status}"),
                open: "Open Kalamu".into(),
                pause: "Pause all jobs".into(),
                quit: "Quit".into(),
            },
        }
    }

    #[test]
    fn repeated_pushes_rewrite_only_what_changed() {
        let mut items = items();
        assert!(items.apply(&state(true, "1 job running · 10%")).is_some());
        assert!(items.apply(&state(true, "1 job running · 20%")).is_some());
        assert!(items.apply(&state(true, "1 job running · 20%")).is_none());
        assert_eq!(items.status.texts.borrow().len(), 2);
        assert_eq!(items.open.texts.borrow().len(), 1);
        assert_eq!(*items.pause.enabled.borrow(), [true]);
        items.apply(&state(false, "No jobs running"));
        assert_eq!(*items.pause.enabled.borrow(), [true, false]);
    }

    #[test]
    fn a_refused_write_is_tried_again() {
        let mut items = items();
        items.status.refuse.set(true);
        items.apply(&state(true, "Running"));
        assert!(items.status.texts.borrow().is_empty());
        items.status.refuse.set(false);
        items.apply(&state(true, "Running"));
        assert_eq!(*items.status.texts.borrow(), ["Running"]);
        assert_eq!(items.open.texts.borrow().len(), 1);
    }
}
