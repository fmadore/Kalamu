use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

macro_rules! uuid_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
        #[serde(transparent)]
        #[ts(export)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}

uuid_id!(
    /// One run of a task over a set of sources with one or more set-ups.
    JobId
);
uuid_id!(
    /// A file added to a job.
    SourceId
);
uuid_id!(
    /// A provider, model and prompt combination within a job.
    SetupId
);
uuid_id!(
    /// A research workspace: its documents, collections and the jobs run on them.
    ProjectId
);
uuid_id!(
    /// A grouping of a project's documents chosen by the researcher.
    CollectionId
);
uuid_id!(
    /// An entry of a project's dictionary: one person, place or term under its spellings.
    EntryId
);
uuid_id!(
    /// A research item in a project: its source and every result read from it.
    DocumentId
);
uuid_id!(
    /// A workflow started on a set of sources: its steps, each a job.
    RunId
);
uuid_id!(
    /// An article, letter or item of a source, found by its table of contents: its pages.
    UnitId
);

/// A provider's stable identifier, as used in the model catalogue (`gemini`, `openai`, …).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(transparent)]
#[ts(export)]
pub struct ProviderId(pub String);

impl ProviderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Milliseconds since the Unix epoch, UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(transparent)]
#[ts(export, type = "number")]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or_default();
        Self(millis)
    }

    /// The day in UTC, `YYYY-MM-DD`.
    pub fn utc_date(self) -> String {
        let (year, month, day) = civil_from_days(self.0.div_euclid(86_400_000));
        format!("{year:04}-{month:02}-{day:02}")
    }

    /// The moment in UTC to the second, as an `xsd:dateTime` (`2026-11-04T09:30:00Z`).
    pub fn utc_date_time(self) -> String {
        let secs = self.0.div_euclid(1000);
        let rest = secs.rem_euclid(86_400);
        format!(
            "{}T{:02}:{:02}:{:02}Z",
            self.utc_date(),
            rest / 3600,
            rest / 60 % 60,
            rest % 60
        )
    }
}

/// Year, month and day of a count of days since 1970, from Howard Hinnant's `civil_from_days`.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn times_are_written_in_utc() {
        assert_eq!(Timestamp(0).utc_date(), "1970-01-01");
        assert_eq!(Timestamp(1_793_096_727_000).utc_date(), "2026-10-27");
        assert_eq!(Timestamp(0).utc_date_time(), "1970-01-01T00:00:00Z");
        assert_eq!(
            Timestamp(1_793_785_800_000).utc_date_time(),
            "2026-11-04T09:50:00Z"
        );
        assert_eq!(Timestamp(-1).utc_date_time(), "1969-12-31T23:59:59Z");
    }
}
