//! The window: the words of the tray icon and the notifications, hiding and closing.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use kalamu_core::JobId;
use tauri::{AppHandle, Manager};

use super::AppState;
use crate::activity::ShellLabels;
use crate::errors::CmdResult;

/// The words the tray menu and the notifications use, translated by the interface: sent at
/// start-up and when the language changes. What they say of the jobs is worked out by the shell
/// from the engine's events (`activity`), so it stays right while the window is hidden.
///
/// Synchronous on purpose: Tauri runs sync commands on the main thread, where menu items live.
#[tauri::command]
pub fn set_shell_labels(app: AppHandle, labels: ShellLabels) {
    crate::activity::set_labels(&app, labels);
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

/// Brings the window back from the tray, the taskbar, the Dock or behind other windows.
pub fn show_main_window(app: &AppHandle) {
    crate::platform::show_app(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    // Windows drops a hidden window's taskbar progress; it is written again.
    crate::activity::show(app);
}

/// Whether any job is running, so closing or quitting asks first.
pub fn jobs_running(state: &AppState) -> bool {
    state
        .engine
        .jobs()
        .map(|jobs| jobs.iter().any(|j| state.engine.is_running(j.id)))
        .unwrap_or(false)
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

/// How long quitting waits for pieces already sent: a call ends within the HTTP client's ten
/// minutes, so past this something is stuck and waiting longer would not help.
const QUIT_WAIT: Duration = Duration::from_secs(11 * 60);

/// Pauses running jobs, waits for the pieces already sent to come back and be stored (they are
/// paid for, and dropping them would send and pay for them again), and exits. Asked again while
/// it waits, it exits at once.
pub async fn pause_and_quit(app: AppHandle) {
    {
        let state = app.state::<AppState>();
        if state.quitting.swap(true, Ordering::SeqCst) {
            app.exit(0);
            return;
        }
        let running = pause_running(&state);
        let deadline = Instant::now() + QUIT_WAIT;
        while running.iter().any(|id| state.engine.is_running(*id)) && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    app.exit(0);
}

/// How long exiting waits for the files corrections left out of date: past this, a folder on a
/// share that stopped answering, they are written at the next launch.
const WRITE_WAIT: Duration = Duration::from_secs(20);

/// Writes the files corrections left out of date as the app exits, whichever way it was quit.
/// What cannot be written keeps its mark and is written at the next launch.
pub fn write_before_exit(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    match state.engine.finish_writing(WRITE_WAIT) {
        Ok(0) => {}
        Ok(count) => tracing::warn!(count, "files out of date left for the next launch"),
        Err(e) => tracing::warn!(error = %e, "files out of date left for the next launch"),
    }
}

/// How long exiting waits for a transcription still running to let go of Whisper's model: a
/// stopped one does within one pass of the encoder.
const RELEASE_WAIT: Duration = Duration::from_secs(5);

/// Releases Whisper's model as the app exits, whichever way it was quit, so the process's own
/// clean-up does not find it still loaded: on a Mac, ggml's Metal device then aborts the app
/// with "quit unexpectedly".
pub fn release_before_exit(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if !state.whisper.release(RELEASE_WAIT) {
        tracing::warn!("Whisper was still transcribing as the app exited");
    }
}
