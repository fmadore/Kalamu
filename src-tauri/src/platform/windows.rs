//! Windows: WebView2 (Chromium) plays the review's recordings, Notepad opens JSON files, and the
//! window closes as Windows apps do, with the notification-area icon keeping running jobs.

use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::OnceLock;

use kalamu_media::Playback;
use tauri::utils::config::AppDirectoriesOverride;
use tauri::{App, AppHandle, RunEvent};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};

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

/// Whether a store keeps this copy up to date: the Microsoft Store's package, known by where it
/// runs. It must not offer updates of its own; the installer's copy is told of a new release.
pub fn updated_by_store() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|exe| package_family(&exe))
        .is_some()
}

/// Nothing to add: Windows apps have no application menu.
pub fn init(_app: &mut App) -> tauri::Result<()> {
    Ok(())
}

/// No run event is Windows-only: the programs the app started end with it through their job
/// ([`keep_children`]), however it ends.
pub fn on_run_event(_app: &AppHandle, _event: RunEvent) {}

/// Every program the app starts (FFmpeg and ffprobe, llama.cpp, the Whisper helper) is put in
/// one Job Object that ends them all when it closes. The app holds the job's only handle until
/// its process ends, and Windows closes it then however the process ended: a second Quit, a
/// crash, Task Manager.
///
/// Each program is put in the job as it starts, rather than the app's own process at start-up
/// (which every child would then inherit): the opener starts the browser, File Explorer or
/// Notepad from inside the app's process (`ShellExecuteExW`), and those must outlive it; so
/// must WebView2's own processes go on as WebView2 decides. Nesting is allowed since Windows
/// 8, so the Microsoft Store's package, whose processes already run in a job, and `tauri dev`
/// work the same.
pub fn keep_children() {
    kalamu_media::program::keep_with(kalamu_media::program::Keeper {
        adopt: |child| {
            let Some(job) = job() else { return };
            if !job.assign(child) {
                tracing::warn!(
                    error = %std::io::Error::last_os_error(),
                    "a program the app started is not in its job: it may outlive the app"
                );
            }
        },
        // A job holds processes, not ids: nothing to forget.
        release: |_| {},
    });
}

/// The job of the programs the app starts, made with the first of them (when logging has
/// started, so a refusal is logged) and kept open until the process ends.
fn job() -> Option<&'static Job> {
    static JOB: OnceLock<Option<Job>> = OnceLock::new();
    JOB.get_or_init(|| {
        let job = Job::new();
        if job.is_none() {
            tracing::warn!(
                error = %std::io::Error::last_os_error(),
                "no job for the programs the app starts: they may outlive it"
            );
        }
        job
    })
    .as_ref()
}

/// A Job Object that ends its processes when its last handle closes.
struct Job(HANDLE);

// SAFETY: a kernel handle may be used and closed from any thread.
#[allow(unsafe_code)]
unsafe impl Send for Job {}
// SAFETY: as above; the job is only assigned to, which Windows serialises.
#[allow(unsafe_code)]
unsafe impl Sync for Job {}

// The crate denies unsafe code; these Job Object calls are the exception, since every call to
// the system's own API through windows-sys is unsafe.
#[allow(unsafe_code)]
impl Job {
    /// A new job with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, or `None` if Windows refused it.
    /// Its handle is not inheritable, so no program started from the app keeps it open.
    fn new() -> Option<Self> {
        // SAFETY: no security attributes (a handle that is not inherited) and no name.
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return None;
        }
        let job = Self(handle);
        // SAFETY: a plain structure of numbers, for which zero is a valid value.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let size = u32::try_from(std::mem::size_of_val(&limits)).ok()?;
        // SAFETY: `limits` lives for the whole call and `size` is its size.
        let set = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size,
            )
        };
        (set != 0).then_some(job)
    }

    /// Puts a running program in the job; `false` if Windows refused.
    fn assign(&self, child: &Child) -> bool {
        // SAFETY: both handles are open: the job's until it is dropped, the child's while
        // `child` is borrowed.
        unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle()) != 0 }
    }
}

#[allow(unsafe_code)]
impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle is open and owned by this value alone.
        unsafe { CloseHandle(self.0) };
    }
}

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
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    use super::{EARLIER_FOLDERS, Job, package_family};

    /// What Windows does when the app's process ends, however it ends: it closes the job's
    /// handle, and the programs in the job end with it.
    #[test]
    fn a_program_in_the_job_ends_when_the_job_closes() {
        let job = Job::new().unwrap();
        // Every Windows has `ping`; sixty echoes take a minute.
        let mut child = Command::new("ping")
            .args(["-n", "60", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        assert!(job.assign(&child));
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            child.try_wait().unwrap().is_none(),
            "ended before the job closed"
        );

        drop(job);
        let closed = Instant::now();
        // Ended by the job, long before its minute: Windows gives it the exit code 0.
        while child.try_wait().unwrap().is_none() {
            assert!(
                closed.elapsed() < Duration::from_secs(10),
                "still running after the job closed"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

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
