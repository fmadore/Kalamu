//! The job database kept safe: a copy before an update, a copy a day, and a damaged file set
//! aside with the newest copy offered in its place.

use std::path::{Path, PathBuf};

use crate::protect::{Checked, Choice, CopyKind, Daily, Damage, Note, Protection};

use super::*;

/// 25 November 2026, noon, UTC; and each day after.
fn day(n: i64) -> Timestamp {
    Timestamp(1_795_608_000_000 + n * 86_400_000)
}

struct Place {
    _dir: tempfile::TempDir,
    database: PathBuf,
    folder: PathBuf,
}

impl Place {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
        Self {
            database: dir.path().join("jobs.sqlite"),
            folder: dir.path().join("backups"),
            _dir: dir,
        }
    }

    fn roomy(&self) -> Protection {
        Protection::new(self.database.clone(), self.folder.clone()).with_free_space(|_| Ok(1 << 50))
    }

    fn cramped(&self) -> Protection {
        Protection::new(self.database.clone(), self.folder.clone()).with_free_space(|_| Ok(4096))
    }

    /// A database with one job named `name`, closed.
    fn with_job(&self, name: &str) -> Result<JobId, StoreError> {
        let store = SqliteStore::open(&self.database)?;
        let mut job = job();
        job.name = name.into();
        store.create_job(&job)?;
        Ok(job.id)
    }
}

fn never_asked(damage: &Damage) -> Choice {
    panic!("asked about {damage:?}")
}

fn names(jobs: &[JobSummary]) -> Vec<&str> {
    jobs.iter().map(|j| j.name.as_str()).collect()
}

fn user_version(path: &Path) -> i64 {
    rusqlite::Connection::open(path)
        .and_then(|c| c.pragma_query_value(None, "user_version", |row| row.get(0)))
        .unwrap_or(-1)
}

/// Cuts a file short, as a power cut or a failing disk might.
fn truncate(path: &Path, len: u64) {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|f| f.set_len(len))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_database_an_update_will_change_is_copied_first() -> Result<(), StoreError> {
    let place = Place::new();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/jobs-1.0.0.sqlite"),
        &place.database,
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    let before = user_version(&place.database);
    let opened = place.roomy().open_at(day(0), never_asked)?;
    let [Note::CopiedBeforeUpdate(copy)] = opened.notes.as_slice() else {
        panic!("{:?}", opened.notes);
    };
    assert_eq!(copy.kind, CopyKind::BeforeUpdate);
    assert_eq!(copy.made, day(0));
    // The copy holds the database as the earlier version left it; the database is updated.
    assert_eq!(user_version(&place.folder.join(&copy.name)), before);
    assert_eq!(
        user_version(&place.database) as usize,
        migrations::MIGRATIONS.len()
    );
    assert_eq!(names(&opened.store.list_jobs()?), ["Letters"]);
    drop(opened);

    // Up to date, it is opened without a copy; a new database, too.
    let again = place.roomy().open_at(day(0), never_asked)?;
    assert!(again.notes.is_empty(), "{:?}", again.notes);
    let fresh = Place::new();
    assert!(fresh.roomy().open_at(day(0), never_asked)?.notes.is_empty());
    assert!(fresh.roomy().copies().is_empty());
    Ok(())
}

#[test]
fn a_copy_that_would_leave_the_disk_too_full_is_skipped_and_said() -> Result<(), StoreError> {
    let place = Place::new();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/jobs-1.0.0.sqlite"),
        &place.database,
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    let opened = place.cramped().open_at(day(0), never_asked)?;
    assert!(
        matches!(opened.notes.as_slice(), [Note::Skipped { free: 4096, needed }] if *needed > 0),
        "{:?}",
        opened.notes
    );
    drop(opened);
    assert!(matches!(
        place.cramped().check_at(day(0)),
        Checked::Sound(Daily::Skipped { free: 4096, .. })
    ));
    assert!(place.cramped().copies().is_empty());
    Ok(())
}

/// One copy a day, the last three kept; a copy taken before an update kept until three daily
/// copies are newer. A copy set aside is never deleted.
#[test]
fn daily_copies_are_kept_three_days_and_the_update_s_copy_until_three_good_days()
-> Result<(), StoreError> {
    let place = Place::new();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/jobs-1.0.0.sqlite"),
        &place.database,
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    drop(place.roomy().open_at(day(0), never_asked)?);
    let kinds = |p: &Protection| -> Vec<(CopyKind, String)> {
        p.copies()
            .into_iter()
            .map(|c| (c.kind, c.made.utc_date()))
            .collect()
    };
    let protection = place.roomy();
    assert!(matches!(
        protection.check_at(day(0)),
        Checked::Sound(Daily::Made(_))
    ));
    // The same day again: none more.
    assert_eq!(
        protection.check_at(Timestamp(day(0).0 + 3_600_000)),
        Checked::Sound(Daily::AlreadyMade)
    );
    for n in 1..=2 {
        assert!(matches!(
            protection.check_at(day(n)),
            Checked::Sound(Daily::Made(_))
        ));
    }
    // Three daily copies, two of them after the update's day: its copy stays.
    assert_eq!(
        kinds(&protection),
        [
            (CopyKind::Daily, day(2).utc_date()),
            (CopyKind::Daily, day(1).utc_date()),
            (CopyKind::Daily, day(0).utc_date()),
            (CopyKind::BeforeUpdate, day(0).utc_date()),
        ]
    );
    // A third good day: the update's copy goes, and the oldest daily one.
    protection.check_at(day(3));
    assert_eq!(
        kinds(&protection),
        [
            (CopyKind::Daily, day(3).utc_date()),
            (CopyKind::Daily, day(2).utc_date()),
            (CopyKind::Daily, day(1).utc_date()),
        ]
    );
    // What a copy cut short left is cleared; a set-aside file and others' files stay.
    std::fs::write(
        place
            .folder
            .join("jobs-daily-2026-01-01-000000.sqlite.partial"),
        b"x",
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    std::fs::write(
        place.folder.join("jobs-set-aside-2020-01-01-000000.sqlite"),
        b"x",
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    std::fs::write(place.folder.join("notes.txt"), b"x")
        .map_err(|e| StoreError::Database(e.to_string()))?;
    protection.check_at(day(4));
    let left: Vec<String> = protection.copies().into_iter().map(|c| c.name).collect();
    assert_eq!(left.len(), 4, "{left:?}");
    assert!(left.contains(&"jobs-set-aside-2020-01-01-000000.sqlite".to_string()));
    assert!(place.folder.join("notes.txt").exists());
    assert!(
        !place
            .folder
            .join("jobs-daily-2026-01-01-000000.sqlite.partial")
            .exists()
    );
    Ok(())
}

/// The acceptance test of #275: a database cut short cannot be read; at the next launch it is
/// set aside, never deleted, the newest copy is offered and, accepted, opened in its place.
#[test]
fn a_truncated_database_is_set_aside_and_the_newest_copy_offered() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("Kept in the copy")?;
    let protection = place.roomy();
    let Checked::Sound(Daily::Made(copy)) = protection.check_at(day(0)) else {
        panic!("no daily copy");
    };
    place.with_job("Added after the copy")?;
    truncate(&place.database, 2000);

    let mut offered = None;
    let opened = protection.open_at(day(1), |damage| {
        offered = Some(damage.clone());
        Choice::UseCopy
    })?;
    let damage = offered.unwrap_or_else(|| panic!("nothing offered"));
    assert_eq!(damage.latest.as_ref(), Some(&copy));
    assert!(!damage.reason.is_empty());
    let [
        Note::Restored {
            copy: used,
            set_aside: Some(set_aside),
        },
    ] = opened.notes.as_slice()
    else {
        panic!("{:?}", opened.notes);
    };
    assert_eq!(used, &copy.name);
    // The damaged file is kept, as it was, beside the copies.
    let aside = place.folder.join(set_aside);
    assert_eq!(std::fs::metadata(&aside).map(|m| m.len()).ok(), Some(2000));
    assert!(
        protection
            .copies()
            .iter()
            .any(|c| c.name == *set_aside && c.kind == CopyKind::SetAside)
    );
    assert_eq!(names(&opened.store.list_jobs()?), ["Kept in the copy"]);
    Ok(())
}

/// Damage only SQLite's whole check finds (the file opens, a page is gone): said by the check
/// after launch, and the copy the researcher then chooses is opened at the next launch.
#[test]
fn damage_the_check_finds_leads_to_the_copy_opened_at_the_next_launch() -> Result<(), StoreError> {
    let place = Place::new();
    {
        let store = SqliteStore::open(&place.database)?;
        for n in 0..40 {
            let mut job = job();
            job.name = format!("Job {n}");
            store.create_job(&job)?;
        }
    }
    let protection = place.roomy();
    let Checked::Sound(Daily::Made(copy)) = protection.check_at(day(0)) else {
        panic!("no daily copy");
    };
    let len = std::fs::metadata(&place.database)
        .map(|m| m.len())
        .unwrap_or(0);
    truncate(&place.database, len - 4096);
    assert!(
        matches!(protection.check_at(day(1)), Checked::Damaged(ref r) if !r.is_empty()),
        "the check finds the missing page"
    );

    // Asked for, then taken back: nothing happens at the next launch.
    protection.open_at_next_launch(&copy.name)?;
    assert_eq!(protection.waiting(), Some(copy.name.clone()));
    protection.cancel_waiting()?;
    assert_eq!(protection.waiting(), None);
    // A copy that is not one may not be asked for.
    assert!(protection.open_at_next_launch("jobs.sqlite").is_err());

    protection.open_at_next_launch(&copy.name)?;
    let opened = protection.open_at(day(1), never_asked)?;
    assert!(
        matches!(opened.notes.as_slice(), [Note::Restored { copy: c, set_aside: Some(_) }] if *c == copy.name),
        "{:?}",
        opened.notes
    );
    assert_eq!(protection.waiting(), None);
    assert_eq!(opened.store.list_jobs()?.len(), 40);
    drop(opened);
    // The copy in place reads whole, and is copied again that day.
    assert!(matches!(
        protection.check_at(day(1)),
        Checked::Sound(Daily::Made(_))
    ));
    Ok(())
}

/// Without a copy, a damaged database can be set aside for an empty one, or left as it is.
#[test]
fn without_a_copy_an_empty_start_is_offered_and_quitting_changes_nothing() -> Result<(), StoreError>
{
    let place = Place::new();
    place.with_job("Lost")?;
    std::fs::write(
        &place.database,
        b"not a database at all, not even close to one",
    )
    .map_err(|e| StoreError::Database(e.to_string()))?;
    let protection = place.roomy();

    let mut offered = None;
    let quit = protection.open_at(day(0), |damage| {
        offered = Some(damage.clone());
        Choice::Quit
    });
    assert!(matches!(quit, Err(StoreError::Corrupt(_))));
    assert_eq!(offered.and_then(|d| d.latest), None);
    assert!(place.database.exists());
    assert!(protection.copies().is_empty());

    let opened = protection.open_at(day(0), |_| Choice::StartEmpty)?;
    let [Note::StartedEmpty { set_aside }] = opened.notes.as_slice() else {
        panic!("{:?}", opened.notes);
    };
    assert!(place.folder.join(set_aside).exists());
    assert!(opened.store.list_jobs()?.is_empty());
    Ok(())
}

/// A database a newer version wrote is refused as before, and never moved.
#[test]
fn a_newer_database_is_refused_and_left_where_it_is() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("Newer")?;
    rusqlite::Connection::open(&place.database)
        .and_then(|c| c.pragma_update(None, "user_version", 9_999))
        .map_err(db)?;
    assert!(matches!(
        place.roomy().open_at(day(0), never_asked),
        Err(StoreError::Newer { .. })
    ));
    assert!(place.database.exists());
    assert!(place.roomy().copies().is_empty());
    Ok(())
}

/// A copy asked for that is gone by the next launch is said, and the database stays.
#[test]
fn a_copy_gone_before_the_next_launch_leaves_the_database_as_it_was() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("Stays")?;
    let protection = place.roomy();
    let Checked::Sound(Daily::Made(copy)) = protection.check_at(day(0)) else {
        panic!("no daily copy");
    };
    protection.open_at_next_launch(&copy.name)?;
    std::fs::remove_file(place.folder.join(&copy.name))
        .map_err(|e| StoreError::Database(e.to_string()))?;
    let opened = protection.open_at(day(1), never_asked)?;
    assert!(
        matches!(opened.notes.as_slice(), [Note::RestoreFailed { .. }]),
        "{:?}",
        opened.notes
    );
    assert_eq!(protection.waiting(), None);
    assert_eq!(names(&opened.store.list_jobs()?), ["Stays"]);
    Ok(())
}

fn fail(e: std::io::Error) -> StoreError {
    StoreError::Database(e.to_string())
}

/// Replaces a copy, once listed, by a folder of the same name: listed still, never readable.
fn unreadable(place: &Place, name: &str) -> Result<(), StoreError> {
    std::fs::remove_file(place.folder.join(name)).map_err(fail)?;
    std::fs::create_dir(place.folder.join(name)).map_err(fail)
}

/// Nothing set aside and nothing staged left beside the database.
fn nothing_moved(place: &Place, protection: &Protection) {
    assert!(
        protection
            .copies()
            .iter()
            .all(|c| c.kind != CopyKind::SetAside),
        "{:?}",
        protection.copies()
    );
    let staged = place.database.with_extension("sqlite.restoring");
    assert!(!staged.exists());
}

/// A copy asked for that cannot be read at the next launch leaves the database, and every job
/// in it, as it was: the copy is staged before the database is moved.
#[test]
fn a_copy_that_cannot_be_read_leaves_the_database_as_it_was() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("Stays")?;
    let protection = place.roomy();
    let Checked::Sound(Daily::Made(copy)) = protection.check_at(day(0)) else {
        panic!("no daily copy");
    };
    place.with_job("Added after the copy")?;
    protection.open_at_next_launch(&copy.name)?;
    unreadable(&place, &copy.name)?;

    let opened = protection.open_at(day(1), never_asked)?;
    assert!(
        matches!(opened.notes.as_slice(), [Note::RestoreFailed { .. }]),
        "{:?}",
        opened.notes
    );
    let mut listed = names(&opened.store.list_jobs()?)
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    listed.sort();
    assert_eq!(listed, ["Added after the copy", "Stays"]);
    nothing_moved(&place, &protection);
    Ok(())
}

/// The same at launch for a damaged database: the copy offered cannot be read, so the damaged
/// file stays exactly where and as it was, and the launch ends as before.
#[test]
fn a_damaged_database_stays_where_it_is_when_its_copy_cannot_be_read() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("In the copy")?;
    let protection = place.roomy();
    let Checked::Sound(Daily::Made(copy)) = protection.check_at(day(0)) else {
        panic!("no daily copy");
    };
    unreadable(&place, &copy.name)?;
    let garbage = b"not a database at all, not even close to one".to_vec();
    std::fs::write(&place.database, &garbage).map_err(fail)?;

    let opened = protection.open_at(day(1), |_| Choice::UseCopy);
    assert!(opened.is_err());
    assert_eq!(std::fs::read(&place.database).map_err(fail)?, garbage);
    nothing_moved(&place, &protection);
    Ok(())
}

/// Damage the probe does not reach (the header and the table list whole, a page further on
/// not) met by a migration as the database is opened: the same question as damage the probe
/// finds, and the copy made before the update opened in its place.
#[test]
fn damage_a_migration_meets_offers_the_copy_too() -> Result<(), StoreError> {
    let place = Place::new();
    {
        let store = SqliteStore::open(&place.database)?;
        let mut kept = job();
        kept.name = "Survivor".into();
        store.create_job(&kept)?;
        let mut gone = Vec::new();
        for n in 0..60 {
            let mut job = job();
            job.name = format!("Gone {n}");
            store.create_job(&job)?;
            gone.push(job.id);
        }
        store.delete_jobs(&gone)?;
        // One step behind: the last migration's table is made again at the next open, from a
        // page taken off the list of free pages.
        let conn = store.conn();
        conn.execute_batch(&format!(
            "DROP TABLE region_readings; PRAGMA user_version = {};",
            migrations::MIGRATIONS.len() - 1
        ))
        .map_err(db)?;
    }
    // The first page of free pages, from the header, overwritten.
    let mut bytes = std::fs::read(&place.database).map_err(fail)?;
    let read_u32 =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let page_size = usize::from(u16::from_be_bytes([bytes[16], bytes[17]]));
    let trunk = read_u32(32) as usize;
    assert!(trunk > 1 && read_u32(36) > 0, "no free pages");
    let at = (trunk - 1) * page_size;
    bytes[at..at + page_size].fill(0xFF);
    std::fs::write(&place.database, &bytes).map_err(fail)?;

    let protection = place.roomy();
    let mut offered = None;
    let opened = protection.open_at(day(0), |damage| {
        offered = Some(damage.clone());
        Choice::UseCopy
    })?;
    let damage = offered.unwrap_or_else(|| panic!("nothing offered"));
    assert_eq!(
        damage.latest.as_ref().map(|c| c.kind),
        Some(CopyKind::BeforeUpdate)
    );
    assert!(
        matches!(
            opened.notes.as_slice(),
            [
                Note::CopiedBeforeUpdate(_),
                Note::Restored {
                    set_aside: Some(_),
                    ..
                }
            ]
        ),
        "{:?}",
        opened.notes
    );
    assert_eq!(names(&opened.store.list_jobs()?), ["Survivor"]);
    assert_eq!(
        user_version(&place.database) as usize,
        migrations::MIGRATIONS.len()
    );
    Ok(())
}

/// A failure that is not the file's when it is opened (a newer version's schema) is reported
/// as before: nothing asked, nothing moved.
#[test]
fn a_failure_that_is_not_the_file_s_asks_nothing() -> Result<(), StoreError> {
    let place = Place::new();
    place.with_job("Newer")?;
    rusqlite::Connection::open(&place.database)
        .and_then(|c| c.pragma_update(None, "user_version", 9_999))
        .map_err(db)?;
    let protection = place.roomy();
    assert!(matches!(
        protection.open_at(day(0), never_asked),
        Err(StoreError::Newer { .. })
    ));
    nothing_moved(&place, &protection);
    Ok(())
}
