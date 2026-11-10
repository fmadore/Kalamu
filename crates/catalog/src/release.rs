//! The release a copy of Kalamu installed from the releases page is told about: the catalogue
//! names the version last released, and a newer one than the copy running is offered with a
//! link to its page. Nothing is fetched for it beyond the catalogue itself, and the link is made
//! here from a version checked to be only numbers, so nothing downloaded becomes an address.

use std::cmp::Ordering;
use std::fmt;

use serde::Serialize;
use ts_rs::TS;

use crate::Catalog;

/// Where each release is published, followed by its tag (`v1.5.0`).
const RELEASES: &str = "https://github.com/fmadore/Kalamu/releases/tag/v";

/// A version number, `major.minor.patch`, as semantic versioning orders them. A pre-release
/// (`1.6.0-beta.1`) comes before the release of the same number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    pre: bool,
}

impl Version {
    /// A release's number: three numbers and nothing else, none written with a leading zero.
    pub fn release(text: &str) -> Option<Self> {
        let mut parts = text.split('.');
        let mut number = || -> Option<u64> {
            let part = parts.next()?;
            let digits =
                !part.is_empty() && part.len() <= 9 && part.bytes().all(|b| b.is_ascii_digit());
            if !digits || (part.len() > 1 && part.starts_with('0')) {
                return None;
            }
            part.parse().ok()
        };
        let version = Self {
            major: number()?,
            minor: number()?,
            patch: number()?,
            pre: false,
        };
        parts.next().is_none().then_some(version)
    }

    /// The running copy's number, which may be a pre-release or carry build metadata
    /// (`1.6.0-beta.1`, `1.6.0+local`).
    pub fn running(text: &str) -> Option<Self> {
        let (number, _build) = text.split_once('+').unwrap_or((text, ""));
        match number.split_once('-') {
            Some((core, pre)) if !pre.is_empty() => Some(Self {
                pre: true,
                ..Self::release(core)?
            }),
            Some(_) => None,
            None => Self::release(number),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch, !self.pre).cmp(&(
            other.major,
            other.minor,
            other.patch,
            !other.pre,
        ))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A release newer than the copy running, and the page it is published on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Release {
    /// `1.6.0`.
    pub version: String,
    /// Its page on the repository's releases, always `https://github.com/...`.
    pub url: String,
}

impl Catalog {
    /// The release the catalogue names, when it is newer than `running` (this copy's version);
    /// `None` when it names none, names one that is not a plain version number, or names this
    /// one or an older one.
    pub fn newer_release(&self, running: &str) -> Option<Release> {
        let latest = Version::release(self.latest_version.as_deref()?.trim())?;
        let running = Version::running(running)?;
        (latest > running).then(|| Release {
            version: latest.to_string(),
            url: format!("{RELEASES}{latest}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::running(text).unwrap_or(Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: true,
        })
    }

    #[test]
    fn versions_are_ordered_by_number_not_by_text() {
        assert!(v("1.10.0") > v("1.9.0"));
        assert!(v("2.0.0") > v("1.99.99"));
        assert!(v("1.5.1") > v("1.5.0"));
        assert_eq!(v("1.5.0").cmp(&v("1.5.0")), Ordering::Equal);
        // A pre-release comes before its release, after the one before.
        assert!(v("1.6.0-beta.1") < v("1.6.0"));
        assert!(v("1.6.0-beta.1") > v("1.5.9"));
        assert_eq!(v("1.6.0+local"), v("1.6.0"));
    }

    #[test]
    fn only_plain_numbers_are_a_release() {
        assert!(Version::release("1.5.0").is_some());
        assert!(Version::release("0.10.2").is_some());
        for text in [
            "",
            "1.5",
            "1.5.0.1",
            "v1.5.0",
            "1.5.0-beta",
            "01.5.0",
            "1.5.x",
            "1.5.0/../../evil",
            "1.5.0 ",
            "1..0",
            "9999999999.0.0",
            "1.5.0?x=https://example.com",
        ] {
            assert_eq!(Version::release(text), None, "{text}");
        }
    }

    #[test]
    fn a_newer_release_is_offered_with_its_page() {
        let mut catalog = Catalog::empty();
        catalog.latest_version = Some("1.6.0".into());
        assert_eq!(
            catalog.newer_release("1.5.0"),
            Some(Release {
                version: "1.6.0".into(),
                url: "https://github.com/fmadore/Kalamu/releases/tag/v1.6.0".into(),
            })
        );
        assert!(catalog.newer_release("1.6.0-rc.1").is_some());
        assert_eq!(catalog.newer_release("1.6.0"), None);
        assert_eq!(catalog.newer_release("1.10.0"), None);
        assert_eq!(catalog.newer_release("not a version"), None);
    }

    #[test]
    fn nothing_is_offered_from_a_missing_or_odd_version() {
        let mut catalog = Catalog::empty();
        assert_eq!(catalog.newer_release("1.5.0"), None);
        for odd in [
            "latest",
            "2.0",
            "v2.0.0",
            "2.0.0-beta",
            "2.0.0 https://example.com",
        ] {
            catalog.latest_version = Some(odd.into());
            assert_eq!(catalog.newer_release("1.5.0"), None, "{odd}");
        }
    }

    /// The bundled catalogue names a version no newer than this build, so a fresh copy never
    /// tells itself about itself.
    #[test]
    fn the_bundled_catalogue_names_a_release_no_newer_than_this_build() {
        let bundled = Catalog::bundled();
        let latest = bundled.latest_version.as_deref().unwrap_or_default();
        assert!(Version::release(latest).is_some(), "{latest}");
        assert_eq!(bundled.newer_release(env!("CARGO_PKG_VERSION")), None);
    }
}
