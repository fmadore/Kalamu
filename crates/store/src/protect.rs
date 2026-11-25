//! The job database kept safe: copies of it, a check that it can be read, and a way back from
//! a damaged file ([decision](../../../docs/decisions/2026-11-25-job-database-protected.md)).
//!
//! Every correction, review mark, project, dictionary decision and paid answer lives in one
//! file, so it is guarded three ways, all in a folder of its own beside it (`backups` in the
//! app's data folder):
//!
//! - **Before an update changes its tables**, a copy of it as it was ([`CopyKind::BeforeUpdate`]),
//!   kept until three daily copies made after it show that launches went well.
//! - **Each day the app is opened**, once its check has passed, a copy ([`CopyKind::Daily`]),
//!   the last three kept.
//! - **A file that cannot be read** is moved into the folder, never deleted
//!   ([`CopyKind::SetAside`]), and the newest copy is offered in its place.
//!
//! A copy is made only when the disk keeps [`SPARE_BYTES`] free once it is made; one skipped is
//! said, never silent. The shell decides nothing here: it asks the researcher what this module
//! offers ([`Protection::open`]'s `ask`) and shows what it reports.

use std::path::{Path, PathBuf};

use kalamu_core::{StoreError, Timestamp};
use rusqlite::{Connection, ErrorCode, OpenFlags};

use crate::{SqliteStore, migrations};

mod copies;

pub use copies::{Copy, CopyKind};

/// The free space a copy leaves on the disk at least, beyond the copy itself: a database that
/// cannot grow, or a system without room, would cost more than the copy is worth.
pub const SPARE_BYTES: u64 = 1 << 30;

/// The copy named here is opened in place of the database at the next launch.
const WAITING: &str = "open-at-next-launch.txt";

/// How often, and how long apart, setting the database aside is tried again: the process that
/// asked for a copy to be opened may still hold the file for a moment as it ends.
const MOVE_TRIES: u32 = 40;
const MOVE_WAIT: std::time::Duration = std::time::Duration::from_millis(250);

/// The job database and the folder that keeps its copies.
pub struct Protection {
    database: PathBuf,
    folder: PathBuf,
    free_space: fn(&Path) -> std::io::Result<u64>,
}

/// What a launch found wrong with the database: it could not be read, and why; and the newest
/// copy that could take its place, if there is one.
#[derive(Debug, Clone, PartialEq)]
pub struct Damage {
    pub reason: String,
    pub latest: Option<Copy>,
}

/// What the researcher chose for a database that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// The file set aside, the newest copy opened in its place.
    UseCopy,
    /// The file set aside, an empty database started (when there is no copy).
    StartEmpty,
    /// Nothing changed; the app ends.
    Quit,
}

/// The database opened, and what was done to it on the way, for the logs and the interface.
pub struct Opened {
    pub store: SqliteStore,
    pub notes: Vec<Note>,
}

/// What opening the database did besides opening it.
#[derive(Debug, Clone, PartialEq)]
pub enum Note {
    /// A copy made before this version updated the tables.
    CopiedBeforeUpdate(Copy),
    /// A copy that was not made: the disk would have kept less than [`SPARE_BYTES`] free.
    Skipped { needed: u64, free: u64 },
    /// A copy that could not be made, and why.
    CopyFailed(String),
    /// A copy opened in place of the database, which was set aside under the name given.
    Restored {
        copy: String,
        set_aside: Option<String>,
    },
    /// The copy asked for could not be opened, and why; the database stays as it was.
    RestoreFailed { copy: String, reason: String },
    /// A database that could not be read set aside under the name given, an empty one begun.
    StartedEmpty { set_aside: String },
}

/// What the check made after launch found.
#[derive(Debug, Clone, PartialEq)]
pub enum Checked {
    /// The database reads whole; with what became of today's copy.
    Sound(Daily),
    /// SQLite's check found damage, as it says it.
    Damaged(String),
}

/// What became of the day's copy after a check that passed.
#[derive(Debug, Clone, PartialEq)]
pub enum Daily {
    Made(Copy),
    /// One was already made today.
    AlreadyMade,
    Skipped {
        needed: u64,
        free: u64,
    },
    Failed(String),
}

/// What a first look at the database found, before anything changes it.
enum Probe {
    Missing,
    /// It reads, with the migrations it has had.
    Readable(i64),
    /// SQLite says it is damaged or not a database.
    Damaged(String),
    /// It could not be read for another reason (no permission, a disk that is gone): not the
    /// file's fault, so nothing is moved; opening it says why.
    Unreachable,
}

impl Protection {
    /// The database at `database`, its copies kept in `folder` (made when first needed).
    pub fn new(database: PathBuf, folder: PathBuf) -> Self {
        Self {
            database,
            folder,
            free_space: |path| fs4::available_space(path),
        }
    }

    /// The same, with the free space on the disk asked of `free_space` (for tests).
    pub fn with_free_space(mut self, free_space: fn(&Path) -> std::io::Result<u64>) -> Self {
        self.free_space = free_space;
        self
    }

    /// The folder the copies are kept in.
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The copies kept, newest first.
    pub fn copies(&self) -> Vec<Copy> {
        copies::list(&self.folder)
    }

    /// The copy waiting to be opened at the next launch, by name.
    pub fn waiting(&self) -> Option<String> {
        let name = std::fs::read_to_string(self.folder.join(WAITING)).ok()?;
        Some(name.trim().to_owned()).filter(|n| !n.is_empty())
    }

    /// Asks for the copy `name` to be opened in place of the database at the next launch; the
    /// database is set aside then, not deleted. A file set aside may be asked for too: that is
    /// how opening a copy is undone.
    pub fn open_at_next_launch(&self, name: &str) -> Result<Copy, StoreError> {
        let copy = self
            .copies()
            .into_iter()
            .find(|c| c.name == name)
            .ok_or(StoreError::NotFound)?;
        std::fs::write(self.folder.join(WAITING), &copy.name).map_err(io)?;
        Ok(copy)
    }

    /// No copy is opened at the next launch after all.
    pub fn cancel_waiting(&self) -> Result<(), StoreError> {
        match std::fs::remove_file(self.folder.join(WAITING)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(io(e)),
            _ => Ok(()),
        }
    }

    /// Opens the database at launch: the copy asked for opened in its place first; a database
    /// that cannot be read offered to `ask`, with the newest copy; one an update will change
    /// copied first. Errors that are not the file's (another version's database, no
    /// permission) are returned as opening it gives them, and nothing is moved.
    pub fn open(&self, ask: impl FnOnce(&Damage) -> Choice) -> Result<Opened, StoreError> {
        self.open_at(Timestamp::now(), ask)
    }

    pub(crate) fn open_at(
        &self,
        now: Timestamp,
        ask: impl FnOnce(&Damage) -> Choice,
    ) -> Result<Opened, StoreError> {
        let mut notes = Vec::new();
        if let Some(name) = self.waiting() {
            notes.push(self.restore_waiting(now, &name));
        }
        let mut ask = Some(ask);
        match probe(&self.database) {
            Probe::Missing | Probe::Unreachable => {}
            Probe::Readable(applied) => {
                let behind = usize::try_from(applied)
                    .is_ok_and(|n| n > 0 && n < migrations::MIGRATIONS.len());
                if behind {
                    notes.push(self.copy_before_update(now));
                }
            }
            Probe::Damaged(reason) => {
                if let Some(ask) = ask.take() {
                    notes.push(self.recover(now, reason, ask)?);
                }
            }
        }
        match SqliteStore::open_telling(&self.database) {
            Ok(store) => return Ok(Opened { store, notes }),
            // Damage the probe did not reach, met by the pragmas or a migration: the same
            // question as for damage the probe found, asked once.
            Err(failure) => match (failure.damage(), ask.take()) {
                (Some(reason), Some(ask)) => notes.push(self.recover(now, reason, ask)?),
                _ => return Err(failure.into_store()),
            },
        }
        let store = SqliteStore::open(&self.database)?;
        Ok(Opened { store, notes })
    }

    /// A database that cannot be read: `ask` is offered the newest copy, and what it chooses
    /// is done; quitting changes nothing and is returned as the damage.
    fn recover(
        &self,
        now: Timestamp,
        reason: String,
        ask: impl FnOnce(&Damage) -> Choice,
    ) -> Result<Note, StoreError> {
        let latest = self.copies().into_iter().find(|c| c.kind.restorable());
        let damage = Damage { reason, latest };
        match (ask(&damage), damage.latest) {
            (Choice::UseCopy, Some(copy)) => {
                let set_aside = self.replace_with(&copy, now)?;
                Ok(Note::Restored {
                    copy: copy.name,
                    set_aside,
                })
            }
            (Choice::StartEmpty, _) => {
                let set_aside = self.set_aside(now)?.unwrap_or_default();
                Ok(Note::StartedEmpty { set_aside })
            }
            _ => Err(StoreError::Corrupt(damage.reason)),
        }
    }

    /// SQLite's own check of every page (`quick_check`) on a connection of its own, run off the
    /// window's thread after launch; when it passes, the day's copy, then the old ones pruned.
    pub fn check(&self) -> Checked {
        self.check_at(Timestamp::now())
    }

    pub(crate) fn check_at(&self, now: Timestamp) -> Checked {
        let conn = match connect(&self.database) {
            Ok(conn) => conn,
            Err(e) => return Checked::Damaged(e.to_string()),
        };
        if let Err(reason) = quick_check(&conn) {
            return Checked::Damaged(reason);
        }
        let today = now.utc_date();
        let made_today = self
            .copies()
            .iter()
            .any(|c| c.kind == CopyKind::Daily && c.made.utc_date() == today);
        let daily = if made_today {
            Daily::AlreadyMade
        } else {
            match self.copy_with(&conn, CopyKind::Daily, now) {
                Ok(Made::Copy(copy)) => Daily::Made(copy),
                Ok(Made::Skipped { needed, free }) => Daily::Skipped { needed, free },
                Err(e) => Daily::Failed(e.to_string()),
            }
        };
        copies::prune(&self.folder);
        Checked::Sound(daily)
    }

    /// A copy of the database as it is, before this version's migrations change it.
    fn copy_before_update(&self, now: Timestamp) -> Note {
        let conn = match connect(&self.database) {
            Ok(conn) => conn,
            Err(e) => return Note::CopyFailed(e.to_string()),
        };
        match self.copy_with(&conn, CopyKind::BeforeUpdate, now) {
            Ok(Made::Copy(copy)) => Note::CopiedBeforeUpdate(copy),
            Ok(Made::Skipped { needed, free }) => Note::Skipped { needed, free },
            Err(e) => Note::CopyFailed(e.to_string()),
        }
    }

    /// Writes a copy of the database `conn` reads (`VACUUM INTO`, which holds what its log
    /// holds too and leaves out free pages) into the folder, flushed to the disk, under its
    /// final name only once whole; or says why it was not made.
    fn copy_with(
        &self,
        conn: &Connection,
        kind: CopyKind,
        now: Timestamp,
    ) -> Result<Made, StoreError> {
        std::fs::create_dir_all(&self.folder).map_err(io)?;
        let needed = database_bytes(&self.database);
        let free = (self.free_space)(&self.folder).map_err(io)?;
        if free < needed.saturating_add(SPARE_BYTES) {
            return Ok(Made::Skipped { needed, free });
        }
        let name = copies::free_name(&self.folder, kind, now);
        let partial = self.folder.join(format!("{name}.partial"));
        let _ = std::fs::remove_file(&partial);
        let target = partial.to_string_lossy().into_owned();
        let written = conn
            .execute("VACUUM INTO ?1", [&target])
            .map_err(crate::db)
            .and_then(|_| sync(&partial));
        if let Err(e) = written {
            let _ = std::fs::remove_file(&partial);
            return Err(e);
        }
        let path = self.folder.join(&name);
        std::fs::rename(&partial, &path).map_err(io)?;
        Ok(Made::Copy(copies::read(&path).ok_or(StoreError::NotFound)?))
    }

    /// Opens the copy the researcher asked for at the last launch: the database set aside, the
    /// copy put in its place. The request is forgotten whatever happens, so a copy that cannot
    /// be opened is not tried at every launch; the note says so.
    fn restore_waiting(&self, now: Timestamp, name: &str) -> Note {
        let _ = std::fs::remove_file(self.folder.join(WAITING));
        let failed = |reason: String| Note::RestoreFailed {
            copy: name.to_owned(),
            reason,
        };
        let Some(copy) = self.copies().into_iter().find(|c| c.name == name) else {
            return failed("the copy is no longer there".into());
        };
        match self.replace_with(&copy, now) {
            Ok(set_aside) => Note::Restored {
                copy: copy.name,
                set_aside,
            },
            Err(e) => failed(e.to_string()),
        }
    }

    /// Puts `copy` in the database's place, never leaving it without one: the copy is staged
    /// beside the database and flushed first (with its write-ahead log when it has one, as a
    /// file set aside does), only then is the database set aside, and the staged copy renamed
    /// in. When any step fails, what was staged is removed and what was set aside put back,
    /// so the database is as it was. The name the database was set aside under.
    fn replace_with(&self, copy: &Copy, now: Timestamp) -> Result<Option<String>, StoreError> {
        let from = self.folder.join(&copy.name);
        let staged = sidecar(&self.database, ".restoring");
        let staged_log = sidecar(&staged, "-wal");
        let log = sidecar(&from, "-wal");
        let unstage = || {
            let _ = std::fs::remove_file(&staged);
            let _ = std::fs::remove_file(&staged_log);
        };
        let staging = stage(&from, &staged).and_then(|()| {
            if log.exists() {
                stage(&log, &staged_log)
            } else {
                Ok(())
            }
        });
        if let Err(e) = staging {
            unstage();
            return Err(e);
        }
        let set_aside = match self.set_aside(now) {
            Ok(set_aside) => set_aside,
            Err(e) => {
                unstage();
                return Err(e);
            }
        };
        let placed = std::fs::rename(&staged, &self.database).and_then(|()| {
            if staged_log.exists() {
                std::fs::rename(&staged_log, sidecar(&self.database, "-wal"))
            } else {
                Ok(())
            }
        });
        if let Err(e) = placed {
            // The copy, half in place, goes; the database comes back.
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(sidecar(&self.database, suffix));
            }
            unstage();
            if let Some(name) = &set_aside {
                self.put_back(name)?;
            }
            return Err(io(e));
        }
        Ok(set_aside)
    }

    /// Moves the database, with its log and shared memory, into the folder under a name of
    /// its own; never deletes it. The name it was given, or none when there was no database.
    /// When a part cannot be moved, what was moved is put back.
    fn set_aside(&self, now: Timestamp) -> Result<Option<String>, StoreError> {
        if !self.database.exists() {
            return Ok(None);
        }
        std::fs::create_dir_all(&self.folder).map_err(io)?;
        let name = copies::free_name(&self.folder, CopyKind::SetAside, now);
        let target = self.folder.join(&name);
        for suffix in ["", "-wal", "-shm"] {
            let from = sidecar(&self.database, suffix);
            if !from.exists() {
                continue;
            }
            if let Err(e) = move_patiently(&from, &sidecar(&target, suffix)) {
                self.put_back(&name)?;
                return Err(e);
            }
        }
        Ok(Some(name))
    }

    /// Moves the database set aside as `name` back into its place, with its log and shared
    /// memory.
    fn put_back(&self, name: &str) -> Result<(), StoreError> {
        let target = self.folder.join(name);
        for suffix in ["", "-wal", "-shm"] {
            let from = sidecar(&target, suffix);
            if from.exists() {
                move_patiently(&from, &sidecar(&self.database, suffix))?;
            }
        }
        Ok(())
    }
}

/// Copies `from` to `to` and waits until the disk holds it.
fn stage(from: &Path, to: &Path) -> Result<(), StoreError> {
    std::fs::copy(from, to).map_err(io)?;
    sync(to)
}

/// A copy made, or why not.
enum Made {
    Copy(Copy),
    Skipped { needed: u64, free: u64 },
}

/// Opens the database read-only and reads its header and table list: enough to tell a file
/// SQLite cannot read at all from one it can, without the cost of a whole check.
fn probe(path: &Path) -> Probe {
    if !path.exists() {
        return Probe::Missing;
    }
    let read = connect(path).and_then(|conn| {
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
        conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
    });
    match read {
        Ok(applied) => Probe::Readable(applied),
        Err(e) => damage_of(&e).map_or(Probe::Unreachable, Probe::Damaged),
    }
}

/// SQLite's words when `e` says the file is damaged or not a database.
pub(crate) fn damage_of(e: &rusqlite::Error) -> Option<String> {
    match e {
        rusqlite::Error::SqliteFailure(failure, message)
            if matches!(
                failure.code,
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase
            ) =>
        {
            Some(message.clone().unwrap_or_else(|| failure.to_string()))
        }
        _ => None,
    }
}

/// A connection of its own to an existing database, which it never creates. Not read-only: a
/// read-only connection cannot open a database in WAL mode whose shared memory file is
/// missing. Nothing here writes to it.
fn connect(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    // The app's own connection may be writing: wait for it rather than fail.
    conn.busy_timeout(std::time::Duration::from_secs(10))?;
    Ok(conn)
}

/// SQLite's `quick_check`: every page read and its structure checked, without matching each
/// index against its table (`integrity_check` does, at many times the cost). What it found
/// wrong, as it says it.
fn quick_check(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA quick_check")
        .map_err(|e| e.to_string())?;
    let found: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .and_then(Iterator::collect)
        .map_err(|e| e.to_string())?;
    if found.len() == 1 && found[0] == "ok" {
        Ok(())
    } else {
        Err(found.join("; "))
    }
}

/// The database's size with what its write-ahead log holds: about what a copy takes.
fn database_bytes(path: &Path) -> u64 {
    [path.to_path_buf(), sidecar(path, "-wal")]
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum()
}

/// `jobs.sqlite` with `suffix` after its whole name (`jobs.sqlite-wal`).
fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Waits until the disk holds what was written to `path`.
fn sync(path: &Path) -> Result<(), StoreError> {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|f| f.sync_all())
        .map_err(io)
}

/// Renames `from` to `to`, tried again for a few seconds while another process (the one that
/// just ended to open a copy) still holds it.
fn move_patiently(from: &Path, to: &Path) -> Result<(), StoreError> {
    let mut tries = 0;
    loop {
        match std::fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) if tries + 1 >= MOVE_TRIES || e.kind() == std::io::ErrorKind::NotFound => {
                return Err(io(e));
            }
            Err(_) => {
                tries += 1;
                std::thread::sleep(MOVE_WAIT);
            }
        }
    }
}

fn io(e: std::io::Error) -> StoreError {
    StoreError::Database(e.to_string())
}
