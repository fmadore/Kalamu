//! The copies in the folder: their names, which say what each is and when it was made
//! (`jobs-daily-2026-11-25-143210.sqlite`), and which are kept.

use std::path::Path;

use kalamu_core::Timestamp;

/// How many daily copies are kept: the newest.
pub const DAILY_KEPT: usize = 3;
/// How many daily copies made after a copy taken before an update show that it is no longer
/// needed: that many launches, on as many days, went well with the updated database.
pub const GOOD_DAYS_AFTER_UPDATE: usize = 3;
/// How many copies taken before an update are kept at most, the newest.
pub const BEFORE_UPDATE_KEPT: usize = 2;

/// What a copy in the folder is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    /// Made after a launch whose check passed, at most one a day.
    Daily,
    /// Made before this version's migrations changed the tables.
    BeforeUpdate,
    /// A database that could not be read, or was replaced by a copy: moved here, never deleted.
    SetAside,
}

impl CopyKind {
    fn word(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::BeforeUpdate => "before-update",
            Self::SetAside => "set-aside",
        }
    }

    /// Whether it may be opened in place of the database: a copy made of a sound one.
    pub fn restorable(self) -> bool {
        self != Self::SetAside
    }
}

/// A copy in the folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Copy {
    /// Its file name in the folder.
    pub name: String,
    pub kind: CopyKind,
    /// When it was made (set aside), to the second, as its name says.
    pub made: Timestamp,
    pub bytes: u64,
}

/// `jobs-<kind>-YYYY-MM-DD-HHMMSS.sqlite`, with `-2`, `-3`… after the time when that name is
/// taken (two in the same second).
pub(super) fn free_name(folder: &Path, kind: CopyKind, now: Timestamp) -> String {
    let stamp = stamp(now);
    let base = format!("jobs-{}-{stamp}", kind.word());
    let mut name = format!("{base}.sqlite");
    let mut n = 2;
    while folder.join(&name).exists() || folder.join(format!("{name}.partial")).exists() {
        name = format!("{base}-{n}.sqlite");
        n += 1;
    }
    name
}

/// The copy at `path`, if its name is one of ours.
pub(super) fn read(path: &Path) -> Option<Copy> {
    let name = path.file_name()?.to_str()?;
    let (kind, made) = parse(name)?;
    let bytes = std::fs::metadata(path).ok()?.len();
    Some(Copy {
        name: name.to_owned(),
        kind,
        made,
        bytes,
    })
}

/// The copies in `folder`, newest first. Files whose names are not ours are left out.
pub(super) fn list(folder: &Path) -> Vec<Copy> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut copies: Vec<Copy> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| read(&entry.path()))
        .collect();
    copies.sort_by(|a, b| b.made.cmp(&a.made).then_with(|| b.name.cmp(&a.name)));
    copies
}

/// Deletes the copies no longer kept: daily ones past the newest [`DAILY_KEPT`]; those made
/// before an update once [`GOOD_DAYS_AFTER_UPDATE`] daily copies are newer, and past the newest
/// [`BEFORE_UPDATE_KEPT`]; and what a copy cut short left (`.partial`). Never one set aside.
pub(super) fn prune(folder: &Path) {
    let copies = list(folder);
    let daily: Vec<&Copy> = copies
        .iter()
        .filter(|c| c.kind == CopyKind::Daily)
        .collect();
    let mut gone: Vec<&Copy> = daily.iter().skip(DAILY_KEPT).copied().collect();
    for (i, copy) in copies
        .iter()
        .filter(|c| c.kind == CopyKind::BeforeUpdate)
        .enumerate()
    {
        let newer_days = daily.iter().filter(|d| d.made > copy.made).count();
        if newer_days >= GOOD_DAYS_AFTER_UPDATE || i >= BEFORE_UPDATE_KEPT {
            gone.push(copy);
        }
    }
    for copy in gone {
        let _ = std::fs::remove_file(folder.join(&copy.name));
    }
    if let Ok(entries) = std::fs::read_dir(folder) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name();
            let partial = name
                .to_str()
                .is_some_and(|n| n.starts_with("jobs-") && n.ends_with(".sqlite.partial"));
            if partial {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// `YYYY-MM-DD-HHMMSS` in UTC.
fn stamp(now: Timestamp) -> String {
    now.utc_date_time()
        .replace('T', "-")
        .replace([':', 'Z'], "")
}

/// What a name says: `jobs-<kind>-YYYY-MM-DD-HHMMSS[-n].sqlite`.
fn parse(name: &str) -> Option<(CopyKind, Timestamp)> {
    let rest = name.strip_prefix("jobs-")?.strip_suffix(".sqlite")?;
    let (kind, rest) = [CopyKind::Daily, CopyKind::BeforeUpdate, CopyKind::SetAside]
        .into_iter()
        .find_map(|kind| Some((kind, rest.strip_prefix(kind.word())?.strip_prefix('-')?)))?;
    let stamp = rest.get(..17)?;
    let after = &rest[17..];
    if !(after.is_empty() || after.strip_prefix('-')?.parse::<u32>().is_ok()) {
        return None;
    }
    let number = |range: std::ops::Range<usize>| stamp.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(13..15)?, number(15..17)?);
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    Some((kind, Timestamp(secs * 1000)))
}

/// Days since 1970 of a date, from Howard Hinnant's `days_from_civil` (the inverse of the one
/// `Timestamp::utc_date` uses).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_says_what_the_copy_is_and_when_it_was_made() {
        let now = Timestamp(1_795_617_130_000);
        let name = format!("jobs-daily-{}.sqlite", stamp(now));
        assert_eq!(name, "jobs-daily-2026-11-25-143210.sqlite");
        assert_eq!(parse(&name), Some((CopyKind::Daily, now)));
        assert_eq!(
            parse("jobs-before-update-2026-11-25-143210-2.sqlite"),
            Some((CopyKind::BeforeUpdate, now))
        );
        assert_eq!(
            parse("jobs-set-aside-1999-03-01-000000.sqlite").map(|(_, t)| t.utc_date()),
            Some("1999-03-01".into())
        );
        for other in [
            "jobs.sqlite",
            "jobs-daily-2026-11-25.sqlite",
            "jobs-daily-2026-11-25-143210.sqlite-wal",
            "jobs-weekly-2026-11-25-143210.sqlite",
            "jobs-daily-2026-11-25-143210-x.sqlite",
        ] {
            assert_eq!(parse(other), None, "{other}");
        }
    }
}
