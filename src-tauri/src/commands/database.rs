//! The copies of the job database, for Settings: where they are, which there are, what this
//! launch did and found, and one opened in place of the database after the researcher agreed.

use kalamu_core::Timestamp;
use kalamu_store::protect::{Checked, CopyKind, Daily, Note};
use serde::Serialize;
use tauri::{AppHandle, State};
use ts_rs::TS;

use super::AppState;
use crate::database::Database;
use crate::errors::{AppError, CmdResult, id};
use crate::platform;

/// The copies of the job database and what this launch did to it.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DatabaseCopies {
    /// The folder they are kept in, as File Explorer or Finder shows it.
    pub folder: String,
    /// Newest first.
    pub copies: Vec<DatabaseCopy>,
    /// What the check after launch found wrong, once it has run.
    pub damaged: Option<String>,
    /// Whether the check after launch has run.
    pub checked: bool,
    /// A copy this launch did not make, the disk being too full: how much it needed and how
    /// much was free, in bytes.
    pub skipped: Option<SkippedCopy>,
    /// A copy opened in place of the database at this launch, by its file name.
    pub restored: Option<String>,
    /// A copy asked for that could not be opened at this launch, by its file name.
    pub restore_failed: Option<String>,
    /// A damaged database set aside at this launch for an empty one.
    pub started_empty: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DatabaseCopy {
    pub name: String,
    pub kind: DatabaseCopyKind,
    pub made: Timestamp,
    #[ts(type = "number")]
    pub bytes: u64,
    /// Its full path, as File Explorer or Finder shows it, to show it there.
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DatabaseCopyKind {
    /// Made after a launch whose check passed, at most one a day, the last three kept.
    Daily,
    /// Made before an update changed the tables.
    BeforeUpdate,
    /// A database that could not be read, or was replaced by a copy: kept, never deleted.
    SetAside,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct SkippedCopy {
    #[ts(type = "number")]
    pub needed: u64,
    #[ts(type = "number")]
    pub free: u64,
}

#[tauri::command]
pub fn database_copies(database: State<'_, Database>) -> DatabaseCopies {
    let protection = &database.protection;
    let folder = platform::shown_path(protection.folder());
    let copies = protection
        .copies()
        .into_iter()
        .map(|c| DatabaseCopy {
            path: folder.join(&c.name).display().to_string(),
            kind: match c.kind {
                CopyKind::Daily => DatabaseCopyKind::Daily,
                CopyKind::BeforeUpdate => DatabaseCopyKind::BeforeUpdate,
                CopyKind::SetAside => DatabaseCopyKind::SetAside,
            },
            name: c.name,
            made: c.made,
            bytes: c.bytes,
        })
        .collect();
    let launch = database.launch.lock().unwrap_or_else(|p| p.into_inner());
    let mut out = DatabaseCopies {
        folder: folder.display().to_string(),
        copies,
        damaged: None,
        checked: launch.checked.is_some(),
        skipped: None,
        restored: None,
        restore_failed: None,
        started_empty: false,
    };
    for note in &launch.notes {
        match note {
            Note::Skipped { needed, free } => {
                out.skipped = Some(SkippedCopy {
                    needed: *needed,
                    free: *free,
                });
            }
            Note::Restored { copy, .. } => out.restored = Some(copy.clone()),
            Note::RestoreFailed { copy, .. } => out.restore_failed = Some(copy.clone()),
            Note::StartedEmpty { .. } => out.started_empty = true,
            Note::CopiedBeforeUpdate(_) | Note::CopyFailed(_) => {}
        }
    }
    match &launch.checked {
        Some(Checked::Damaged(reason)) => out.damaged = Some(reason.clone()),
        Some(Checked::Sound(Daily::Skipped { needed, free })) => {
            out.skipped = Some(SkippedCopy {
                needed: *needed,
                free: *free,
            });
        }
        _ => {}
    }
    out
}

/// Opens the copy `name` in place of the job database: Kalamu restarts, and the database is
/// set aside in the copies folder before the copy takes its place, so nothing is deleted.
/// Refused while a job runs. The interface asks the researcher first.
#[tauri::command]
pub fn open_database_copy(
    app: AppHandle,
    state: State<'_, AppState>,
    database: State<'_, Database>,
    name: String,
) -> CmdResult<()> {
    if super::window::jobs_running(&state) {
        return Err(AppError::new(
            id::JOBS_RUNNING,
            "open a copy of the job database",
        ));
    }
    let copy = database
        .protection
        .open_at_next_launch(&name)
        .map_err(|e| AppError::new(id::FILE, e))?;
    tracing::warn!(copy = %copy.name, "a copy of the job database will open at the restart");
    // Files left out of date are written, and Whisper let go, as at any exit.
    app.request_restart();
    Ok(())
}
