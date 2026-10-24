//! The window: taskbar progress, the notification-area icon, hiding and closing.

use scriptorium_core::JobId;
use tauri::{AppHandle, Manager, State};

use super::AppState;
use crate::errors::CmdResult;
use crate::tray::{TrayMenu, TrayState};

/// Progress on the taskbar button: 0–100, or `None` to clear it.
#[tauri::command]
pub fn set_taskbar_progress(app: AppHandle, percent: Option<u32>) {
    use tauri::window::{ProgressBarState, ProgressBarStatus};
    if let Some(window) = app.get_webview_window("main") {
        let state = match percent {
            Some(p) => ProgressBarState {
                status: Some(ProgressBarStatus::Normal),
                progress: Some(u64::from(p.min(100))),
            },
            None => ProgressBarState {
                status: Some(ProgressBarStatus::None),
                progress: None,
            },
        };
        let _ = window.set_progress_bar(state);
    }
}

/// The job status and translated labels for the tray menu.
///
/// Synchronous on purpose: Tauri runs sync commands on the main thread, where menu items live.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive their arguments by value"
)]
pub fn set_tray_state(app: AppHandle, tray: State<'_, TrayMenu>, state: TrayState) {
    tray.apply(&app, &state);
}

/// Hides the window; jobs keep running and the tray icon brings it back.
#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Closes the app after the user confirmed. Running jobs are paused first so they resume
/// cleanly; even without that, the next launch marks them as interrupted.
#[tauri::command]
pub async fn quit_app(app: AppHandle) -> CmdResult<()> {
    pause_and_quit(app).await;
    Ok(())
}

/// Brings the window back from the tray, the taskbar or behind other windows.
pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Pauses every running job and returns them.
pub fn pause_running(state: &AppState) -> Vec<JobId> {
    let running: Vec<JobId> = match state.engine.jobs() {
        Ok(jobs) => jobs
            .into_iter()
            .filter(|j| state.engine.is_running(j.id))
            .map(|j| j.id)
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, "could not list jobs to pause");
            Vec::new()
        }
    };
    for id in &running {
        state.engine.pause(*id);
    }
    running
}

/// Pauses running jobs, waits a moment for pages in flight to land in the cache, and exits.
pub async fn pause_and_quit(app: AppHandle) {
    {
        let state = app.state::<AppState>();
        let running = pause_running(&state);
        for _ in 0..50 {
            if running.iter().all(|id| !state.engine.is_running(*id)) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    app.exit(0);
}
