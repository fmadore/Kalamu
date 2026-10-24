//! The window: taskbar progress and closing.

use scriptorium_core::JobId;
use tauri::{AppHandle, Manager, State};

use super::AppState;
use crate::errors::CmdResult;

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

/// Closes the app after the user confirmed. Running jobs are paused first so they resume
/// cleanly; even without that, the next launch marks them as interrupted.
#[tauri::command]
pub async fn quit_app(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    let running: Vec<JobId> = state
        .engine
        .jobs()?
        .into_iter()
        .filter(|j| state.engine.is_running(j.id))
        .map(|j| j.id)
        .collect();
    for id in &running {
        state.engine.pause(*id);
    }
    // Give pages in flight a moment to land in the cache.
    for _ in 0..50 {
        if running.iter().all(|id| !state.engine.is_running(*id)) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    app.exit(0);
    Ok(())
}
