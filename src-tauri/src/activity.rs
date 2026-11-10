//! What the shell shows of the jobs outside the window: the tray menu's status line (in the
//! notification area on Windows, the menu bar on macOS), progress on the taskbar button or the
//! Dock icon, and the notification when a job finishes.
//!
//! Derived here from the engine's own events and job list, not in the webview: the system slows
//! a webview's timers exactly when its window is hidden in the notification area or the menu
//! bar, which is when these matter most. The interface sends only the words, translated, once at
//! start-up and again when the language changes ([`ShellLabels`]); the numbers are filled in
//! here.

use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::time::Duration;

use kalamu_core::{JobId, JobStatus, JobSummary, ProgressEvent, StepState};
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::broadcast::{self, error::RecvError};
use ts_rs::TS;

use crate::commands::AppState;
use crate::tray::{TrayLabels, TrayMenu, TrayState};

/// How long a burst of progress events is gathered before the tray and the taskbar are written
/// again: the half second the interface's job list refreshes at.
const SETTLE: Duration = Duration::from_millis(500);

/// The words the shell shows outside the window, translated by the interface. Placeholders in
/// braces are filled in here: `{jobs}`, `{percent}`, `{status}`, `{job}` and `{state}`.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct ShellLabels {
    pub tray: TrayWords,
    pub notice: NoticeWords,
}

/// The tray menu's words.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TrayWords {
    pub open: String,
    pub pause: String,
    pub quit: String,
    /// *Quit* while jobs run, which pauses them first.
    pub quit_pausing: String,
    /// The status line with nothing running.
    pub idle: String,
    /// The status line with one job running: "1 job running · {percent}%".
    pub running_one: String,
    /// With several: "{jobs} jobs running · {percent}%".
    pub running_many: String,
    /// The icon's tooltip around the status line: "Kalamu: {status}".
    pub tooltip: String,
}

/// The words of the notification a finished job sends.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct NoticeWords {
    pub title: String,
    /// "{job}: {state}", the job's name and what became of it.
    pub body: String,
    pub statuses: StatusWords,
    /// A workflow step that now waits to be checked.
    pub review: String,
    /// A workflow step that now waits because pages were not read.
    pub attention: String,
}

/// A job's status as the interface names it.
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct StatusWords {
    pub pending: String,
    pub running: String,
    pub paused: String,
    pub interrupted: String,
    pub completed: String,
    pub completed_with_errors: String,
    pub cancelled: String,
}

impl StatusWords {
    fn of(&self, status: JobStatus) -> &str {
        match status {
            JobStatus::Pending => &self.pending,
            JobStatus::Running => &self.running,
            JobStatus::Paused => &self.paused,
            JobStatus::Interrupted => &self.interrupted,
            JobStatus::Completed => &self.completed,
            JobStatus::CompletedWithErrors => &self.completed_with_errors,
            JobStatus::Cancelled => &self.cancelled,
        }
    }
}

/// `template` with each `{name}` among `values` replaced, in one pass, so a value that itself
/// holds braces (a job named "{state}") is never read as a placeholder. Unknown placeholders are
/// left as they are.
fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let found = after.find('}').and_then(|close| {
            let name = &after[..close];
            values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value, close))
        });
        match found {
            Some((value, close)) => {
                out.push_str(value);
                rest = &after[close + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The running jobs, as the tray and the taskbar sum them up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overview {
    pub running: u32,
    /// Pieces finished over the pieces of every running job, 0–100; `None` with nothing to
    /// count, which clears the taskbar's progress.
    pub percent: Option<u32>,
}

impl Overview {
    /// From the job list: a job counts as running when its stored status says so or the engine
    /// is running it (it records the status a moment after starting).
    pub fn of<'a>(
        jobs: impl IntoIterator<Item = &'a JobSummary>,
        is_running: impl Fn(JobId) -> bool,
    ) -> Self {
        let (mut running, mut total, mut done) = (0u32, 0u64, 0u64);
        for job in jobs {
            if job.status == JobStatus::Running || is_running(job.id) {
                running += 1;
                total += u64::from(job.pieces_total);
                done += u64::from(job.pieces_finished.min(job.pieces_total));
            }
        }
        let percent = (total > 0).then(|| {
            u32::try_from((100 * done + total / 2) / total)
                .unwrap_or(100)
                .min(100)
        });
        Self { running, percent }
    }
}

/// What the tray menu says for these running jobs.
pub fn tray_state(overview: Overview, words: &TrayWords) -> TrayState {
    let percent = overview.percent.unwrap_or(0).to_string();
    let status = match overview.running {
        0 => words.idle.clone(),
        1 => fill(&words.running_one, &[("percent", &percent)]),
        n => fill(
            &words.running_many,
            &[("jobs", &n.to_string()), ("percent", &percent)],
        ),
    };
    let running = overview.running > 0;
    TrayState {
        running,
        labels: TrayLabels {
            tooltip: fill(&words.tooltip, &[("status", &status)]),
            status,
            open: words.open.clone(),
            pause: words.pause.clone(),
            quit: if running {
                words.quit_pausing.clone()
            } else {
                words.quit.clone()
            },
        },
    }
}

/// The notification's text for a job that finished with `status`; `step` is the state of the
/// workflow step it is, if it is one, which says what the next step waits for.
pub fn notice_body(
    words: &NoticeWords,
    job: &str,
    status: JobStatus,
    step: Option<StepState>,
) -> String {
    let state = match step {
        Some(StepState::Review) => words.review.as_str(),
        Some(StepState::Attention) => words.attention.as_str(),
        _ => words.statuses.of(status),
    };
    fill(&words.body, &[("job", job), ("state", state)])
}

/// The words the interface sent, and the finished jobs not yet told of because they had not
/// arrived (a batch collected in the first second after launch).
#[derive(Default)]
pub struct Activity {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    labels: Option<ShellLabels>,
    untold: Vec<(JobId, JobStatus)>,
}

impl Activity {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// Takes the interface's words, writes the tray with them at once, and sends the notifications
/// that waited for them.
pub fn set_labels(app: &AppHandle, labels: ShellLabels) {
    let Some(activity) = app.try_state::<Activity>() else {
        return;
    };
    let untold = {
        let mut inner = activity.lock();
        inner.labels = Some(labels);
        std::mem::take(&mut inner.untold)
    };
    show(app);
    for (job, status) in untold {
        tell(app, job, status);
    }
}

/// Writes the tray menu and the taskbar's or the Dock's progress from the engine's job list.
/// Also called when the window comes back, since Windows forgets a hidden window's taskbar
/// progress.
pub fn show(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let jobs = match state.engine.jobs() {
        Ok(jobs) => jobs,
        Err(e) => {
            tracing::warn!(error = %e, "could not list jobs for the tray");
            return;
        }
    };
    let overview = Overview::of(&jobs, |id| state.engine.is_running(id));
    set_progress(app, overview.percent);
    let tray = app.try_state::<Activity>().and_then(|a| {
        a.lock()
            .labels
            .as_ref()
            .map(|l| tray_state(overview, &l.tray))
    });
    match (tray, app.try_state::<TrayMenu>()) {
        (Some(tray), Some(menu)) => menu.apply(app, &tray),
        // Before the words arrive the menu keeps its placeholders, but *Pause all* follows the
        // jobs already.
        (None, Some(menu)) => menu.set_running(overview.running > 0),
        _ => {}
    }
}

/// Progress on the taskbar button (Windows) or the Dock icon (macOS), through Tauri's one call
/// for both: 0–100, or `None` to clear it.
fn set_progress(app: &AppHandle, percent: Option<u32>) {
    use tauri::window::{ProgressBarState, ProgressBarStatus};
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
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

/// Sends the notification for a finished job, unless the window is in front (the job's page
/// says it there) or the app is quitting (it paused the jobs itself). Kept until the words
/// arrive when they have not yet.
fn tell(app: &AppHandle, job: JobId, status: JobStatus) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if state.quitting.load(Ordering::SeqCst) || in_front(app) {
        return;
    }
    let Some(activity) = app.try_state::<Activity>() else {
        return;
    };
    let notice = {
        let mut inner = activity.lock();
        match inner.labels.as_ref() {
            Some(labels) => labels.notice.clone(),
            None => {
                inner.untold.push((job, status));
                return;
            }
        }
    };
    let name = state
        .engine
        .jobs()
        .ok()
        .and_then(|jobs| jobs.into_iter().find(|j| j.id == job).map(|j| j.name))
        .unwrap_or_default();
    // The step's state is read after the job's status was stored, so it already says whether
    // the step waits for the researcher.
    let step = match state.engine.job_step(job) {
        Ok(Some(found)) => found.view.steps.get(found.step as usize).map(|s| s.state),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!(job = %job, error = %e, "could not read the job's workflow step");
            None
        }
    };
    let body = notice_body(&notice, &name, status, step);
    if let Err(e) = app
        .notification()
        .builder()
        .title(notice.title)
        .body(body)
        .show()
    {
        tracing::warn!(error = %e, "notification not shown");
    }
}

/// Whether the window is on screen and has the focus.
fn in_front(app: &AppHandle) -> bool {
    app.get_webview_window("main")
        .is_some_and(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false))
}

/// Follows the engine's events for as long as the app runs: a finished job writes the tray at
/// once and is told of; any other burst of events is gathered for [`SETTLE`] first. Subscribed
/// before the engine resumes anything, so no job is missed.
pub fn watch(app: &AppHandle, mut events: broadcast::Receiver<ProgressEvent>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut due: Option<tokio::time::Instant> = None;
        loop {
            let received = match due {
                Some(at) => tokio::select! {
                    received = events.recv() => Some(received),
                    () = tokio::time::sleep_until(at) => None,
                },
                None => Some(events.recv().await),
            };
            match received {
                None => {
                    due = None;
                    show(&app);
                }
                Some(Ok(ProgressEvent::JobFinished { job_id, status })) => {
                    due = None;
                    show(&app);
                    tell(&app, job_id, status);
                }
                // A change to a workflow run as a whole, or to what a job costs, moves no count.
                Some(Ok(ProgressEvent::RunChanged { .. } | ProgressEvent::CostUpdated { .. })) => {}
                Some(Ok(_) | Err(RecvError::Lagged(_))) => {
                    due.get_or_insert_with(|| tokio::time::Instant::now() + SETTLE);
                }
                Some(Err(RecvError::Closed)) => break,
            }
        }
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use kalamu_core::{Task, Timestamp};

    use super::*;

    fn words() -> ShellLabels {
        ShellLabels {
            tray: TrayWords {
                open: "Open Kalamu".into(),
                pause: "Pause all jobs".into(),
                quit: "Quit".into(),
                quit_pausing: "Pause jobs and quit".into(),
                idle: "No jobs running".into(),
                running_one: "1 job running · {percent}%".into(),
                running_many: "{jobs} jobs running · {percent}%".into(),
                tooltip: "Kalamu: {status}".into(),
            },
            notice: NoticeWords {
                title: "Kalamu".into(),
                body: "{job}: {state}".into(),
                statuses: StatusWords {
                    pending: "Waiting".into(),
                    running: "Running".into(),
                    paused: "Paused".into(),
                    interrupted: "Interrupted".into(),
                    completed: "Done".into(),
                    completed_with_errors: "Done with errors".into(),
                    cancelled: "Cancelled".into(),
                },
                review: "ready for you to check".into(),
                attention: "some pages were not read".into(),
            },
        }
    }

    fn job(status: JobStatus, total: u32, finished: u32) -> JobSummary {
        JobSummary {
            id: JobId::new(),
            name: "Letters".into(),
            task: Task::TextExtraction,
            status,
            created_at: Timestamp(0),
            updated_at: Timestamp(0),
            source_count: 1,
            setup_count: 1,
            pieces_total: total,
            pieces_finished: finished,
            pieces_failed: 0,
            pieces_truncated: 0,
            cost_usd: None,
            models: Vec::new(),
            workflow: None,
        }
    }

    #[test]
    fn placeholders_are_filled_once() {
        assert_eq!(fill("{a} and {b}", &[("a", "1"), ("b", "2")]), "1 and 2");
        // A value holding a placeholder's name stays as it is.
        assert_eq!(
            fill("{job}: {state}", &[("job", "{state}"), ("state", "Done")]),
            "{state}: Done"
        );
        assert_eq!(fill("{other} {", &[("a", "1")]), "{other} {");
        assert_eq!(fill("{percent} %", &[("percent", "45")]), "45 %");
    }

    #[test]
    fn nothing_running_says_so_and_clears_the_progress() {
        let jobs = [
            job(JobStatus::Completed, 10, 10),
            job(JobStatus::Paused, 4, 1),
        ];
        let overview = Overview::of(&jobs, |_| false);
        assert_eq!(
            overview,
            Overview {
                running: 0,
                percent: None
            }
        );
        let tray = tray_state(overview, &words().tray);
        assert!(!tray.running);
        assert_eq!(tray.labels.status, "No jobs running");
        assert_eq!(tray.labels.tooltip, "Kalamu: No jobs running");
        assert_eq!(tray.labels.quit, "Quit");
    }

    #[test]
    fn running_jobs_are_counted_and_their_pieces_summed() {
        let jobs = [
            job(JobStatus::Running, 10, 3),
            job(JobStatus::Completed, 50, 50),
            // Started by the engine, its status not yet recorded.
            job(JobStatus::Pending, 10, 6),
        ];
        let started = jobs[2].id;
        let overview = Overview::of(&jobs, |id| id == started);
        assert_eq!(
            overview,
            Overview {
                running: 2,
                percent: Some(45)
            }
        );
        let tray = tray_state(overview, &words().tray);
        assert!(tray.running);
        assert_eq!(tray.labels.status, "2 jobs running · 45%");
        assert_eq!(tray.labels.tooltip, "Kalamu: 2 jobs running · 45%");
        assert_eq!(tray.labels.quit, "Pause jobs and quit");
    }

    #[test]
    fn one_running_job_reads_in_the_singular() {
        let jobs = [job(JobStatus::Running, 3, 2)];
        let tray = tray_state(Overview::of(&jobs, |_| false), &words().tray);
        assert_eq!(tray.labels.status, "1 job running · 67%");
    }

    #[test]
    fn a_job_not_yet_split_shows_no_percent() {
        let jobs = [job(JobStatus::Running, 0, 0)];
        let overview = Overview::of(&jobs, |_| false);
        assert_eq!(
            overview,
            Overview {
                running: 1,
                percent: None
            }
        );
        let tray = tray_state(overview, &words().tray);
        assert_eq!(tray.labels.status, "1 job running · 0%");
    }

    #[test]
    fn the_notice_names_the_job_and_what_became_of_it() {
        let notice = words().notice;
        assert_eq!(
            notice_body(&notice, "Letters", JobStatus::CompletedWithErrors, None),
            "Letters: Done with errors"
        );
        assert_eq!(
            notice_body(
                &notice,
                "Letters",
                JobStatus::Completed,
                Some(StepState::Review)
            ),
            "Letters: ready for you to check"
        );
        assert_eq!(
            notice_body(
                &notice,
                "Letters",
                JobStatus::CompletedWithErrors,
                Some(StepState::Attention)
            ),
            "Letters: some pages were not read"
        );
        // A step paused by the researcher is simply paused.
        assert_eq!(
            notice_body(
                &notice,
                "Letters",
                JobStatus::Paused,
                Some(StepState::Paused)
            ),
            "Letters: Paused"
        );
    }
}
