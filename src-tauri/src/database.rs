//! The job database kept safe, as the shell sees it: opened at launch through
//! [`kalamu_store::protect`], which copies it before an update and offers the newest copy when
//! it cannot be read (the question asked in a native message, the interface not existing yet);
//! checked after launch off the window's thread, with the day's copy made; and the copies
//! listed for Settings. Everything decided lives in the store; this only asks and tells.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kalamu_store::SqliteStore;
use kalamu_store::protect::{Checked, Daily, Note, Protection};
use tauri::{Emitter, Manager};

use crate::start_failure::{self, StartError};

/// Name of the event sent once the check after launch has run: the interface reads
/// [`crate::commands::database::database_copies`] again, and offers the newest copy when the
/// check found damage.
pub const DATABASE_CHECKED_EVENT: &str = "database-checked";

/// The copies folder, in the app's data folder beside the database.
const FOLDER: &str = "backups";

/// The protection of the job database, and what this launch did and found.
pub struct Database {
    pub protection: Protection,
    pub launch: Mutex<Launch>,
}

/// What this launch did to the database and what its check found, for Settings and for the
/// interface to say once.
#[derive(Default)]
pub struct Launch {
    pub notes: Vec<Note>,
    pub checked: Option<Checked>,
}

/// Opens the job database at launch: what the store's protection does, the researcher asked
/// in a native message when the file cannot be read. What was done is logged.
pub fn open(data: &Path, logs: &Path) -> Result<(SqliteStore, Database), StartError> {
    let protection = Protection::new(data.join("jobs.sqlite"), data.join(FOLDER));
    let opened = protection
        .open(|damage| {
            tracing::error!(reason = %damage.reason, "the job database cannot be read");
            start_failure::ask_damaged(damage, &protection_folder(data), Some(logs))
        })
        .map_err(StartError::database)?;
    for note in &opened.notes {
        log(note);
    }
    // One record this version cannot read costs its own row: the others are listed.
    for job in kalamu_core::JobStore::unreadable_jobs(&opened.store).unwrap_or_default() {
        tracing::warn!(job = %job.id, reason = %job.reason, "a job's record cannot be read");
    }
    let database = Database {
        protection,
        launch: Mutex::new(Launch {
            notes: opened.notes,
            checked: None,
        }),
    };
    Ok((opened.store, database))
}

/// The copies folder as File Explorer or Finder shows it.
pub fn protection_folder(data: &Path) -> PathBuf {
    crate::platform::shown_path(&data.join(FOLDER))
}

/// SQLite's check of the whole file and the day's copy, off the window's thread: a large
/// database takes seconds. The interface is told when it is done.
pub fn check_after_launch(app: &tauri::AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let database = handle.state::<Database>();
        let checked = database.protection.check();
        match &checked {
            Checked::Damaged(reason) => {
                tracing::error!(%reason, "the job database's check found damage");
            }
            Checked::Sound(Daily::Made(copy)) => {
                tracing::info!(copy = %copy.name, bytes = copy.bytes, "job database copied");
            }
            Checked::Sound(Daily::AlreadyMade) => {}
            Checked::Sound(Daily::Skipped { needed, free }) => tracing::warn!(
                needed,
                free,
                "job database not copied today: the disk would be too full"
            ),
            Checked::Sound(Daily::Failed(e)) => {
                tracing::warn!(error = %e, "job database not copied today");
            }
        }
        database
            .launch
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .checked = Some(checked);
        let _ = handle.emit(DATABASE_CHECKED_EVENT, ());
    });
}

fn log(note: &Note) {
    match note {
        Note::CopiedBeforeUpdate(copy) => {
            tracing::info!(copy = %copy.name, "job database copied before its update");
        }
        Note::Skipped { needed, free } => tracing::warn!(
            needed,
            free,
            "job database not copied before its update: the disk would be too full"
        ),
        Note::CopyFailed(e) => {
            tracing::warn!(error = %e, "job database not copied before its update");
        }
        Note::Restored { copy, set_aside } => tracing::warn!(
            copy,
            set_aside = set_aside.as_deref().unwrap_or(""),
            "a copy of the job database opened in its place"
        ),
        Note::RestoreFailed { copy, reason } => {
            tracing::warn!(copy, %reason, "the copy of the job database asked for was not opened");
        }
        Note::StartedEmpty { set_aside } => {
            tracing::warn!(
                set_aside,
                "an empty job database begun, the damaged one set aside"
            );
        }
    }
}
