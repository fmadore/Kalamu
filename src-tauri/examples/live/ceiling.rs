//! The most a run may spend, in US dollars at the catalogue's prices, and the ledger that
//! keeps to it.
//!
//! What a call costs is known only once it has answered, so the ceiling is kept twice:
//! before a step starts, its estimate (the one New job shows; the economy price for a batch)
//! must fit in what is left once the steps already running are counted at their estimates;
//! and as soon as what has been billed passes the ceiling, the step running is cancelled and
//! nothing more starts. Calls already in flight at that moment finish and are billed, so a run
//! can end above the ceiling by what those calls cost, never by a call started after it.

use std::collections::BTreeMap;

/// Why the ceiling given cannot be used.
#[derive(Debug, PartialEq)]
pub enum CeilingError {
    /// A run nobody watches must be given one.
    Missing,
    /// Not a positive number of dollars.
    Invalid(String),
}

impl std::fmt::Display for CeilingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str(
                "no ceiling: a run with nobody at the terminal needs LIVE_CEILING, \
                 the most it may spend in US dollars (for example LIVE_CEILING=0.10)",
            ),
            Self::Invalid(given) => write!(
                f,
                "LIVE_CEILING must be a positive number of US dollars, not {given:?}"
            ),
        }
    }
}

/// The ceiling given, if any: `None` only where someone at the terminal may be asked.
pub fn ceiling(given: Option<&str>, interactive: bool) -> Result<Option<f64>, CeilingError> {
    let Some(given) = given.map(str::trim).filter(|g| !g.is_empty()) else {
        return if interactive {
            Ok(None)
        } else {
            Err(CeilingError::Missing)
        };
    };
    let usd: f64 = given
        .trim_start_matches('$')
        .parse()
        .map_err(|_| CeilingError::Invalid(given.to_string()))?;
    if usd.is_finite() && usd > 0.0 {
        Ok(Some(usd))
    } else {
        Err(CeilingError::Invalid(given.to_string()))
    }
}

/// Whether a step may start.
#[derive(Debug, Clone, PartialEq)]
pub enum Admission {
    Go,
    /// Its estimate would pass the ceiling, with what is left.
    TooDear {
        estimate: f64,
        left: f64,
    },
    /// It has no estimate (a model without a price), so nothing says it fits.
    Unpriced,
    /// The ceiling was passed by what was billed: nothing more starts.
    Stopped,
}

/// What has been billed and what the steps running may still cost, step by step.
#[derive(Debug, Default)]
pub struct Ledger {
    ceiling: Option<f64>,
    /// Billed so far, by step.
    spent: BTreeMap<u64, f64>,
    /// The estimate of each step still running.
    running: BTreeMap<u64, f64>,
    passed: bool,
}

impl Ledger {
    pub fn new(ceiling: Option<f64>) -> Self {
        Self {
            ceiling,
            ..Self::default()
        }
    }

    /// Everything billed so far.
    pub fn spent(&self) -> f64 {
        self.spent.values().sum()
    }

    /// Whether what was billed passed the ceiling.
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// What the steps running may still cost: each one's estimate less what it was billed.
    fn outstanding(&self) -> f64 {
        self.running
            .iter()
            .map(|(step, estimate)| (estimate - self.spent.get(step).unwrap_or(&0.0)).max(0.0))
            .sum()
    }

    /// Whether step `step`, estimated at `estimate`, may start; when it may, its estimate is
    /// held for it until [`Ledger::finish`].
    pub fn admit(&mut self, step: u64, estimate: Option<f64>) -> Admission {
        if self.passed {
            return Admission::Stopped;
        }
        let Some(ceiling) = self.ceiling else {
            self.running.insert(step, estimate.unwrap_or(0.0));
            return Admission::Go;
        };
        let Some(estimate) = estimate else {
            return Admission::Unpriced;
        };
        let left = ceiling - self.spent() - self.outstanding();
        if estimate > left {
            return Admission::TooDear {
                estimate,
                left: left.max(0.0),
            };
        }
        self.running.insert(step, estimate);
        Admission::Go
    }

    /// Records what step `step` has been billed in all so far; true when that takes the run past
    /// its ceiling, the first time and every time after.
    pub fn bill(&mut self, step: u64, total_usd: f64) -> bool {
        self.spent.insert(step, total_usd.max(0.0));
        if let Some(ceiling) = self.ceiling
            && self.spent() > ceiling
        {
            self.passed = true;
        }
        self.passed
    }

    /// The step ended: its estimate is no longer held.
    pub fn finish(&mut self, step: u64) {
        self.running.remove(&step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_nobody_watches_refuses_to_start_without_a_ceiling() {
        assert_eq!(ceiling(None, false), Err(CeilingError::Missing));
        assert_eq!(ceiling(Some("  "), false), Err(CeilingError::Missing));
        assert_eq!(ceiling(None, true), Ok(None));
        assert_eq!(ceiling(Some("0.10"), false), Ok(Some(0.10)));
        assert_eq!(ceiling(Some("$2"), false), Ok(Some(2.0)));
        for bad in ["0", "-1", "abc", "NaN", "inf"] {
            assert!(
                matches!(ceiling(Some(bad), true), Err(CeilingError::Invalid(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_step_whose_estimate_would_pass_the_ceiling_does_not_start() {
        let mut ledger = Ledger::new(Some(0.10));
        assert_eq!(ledger.admit(1, Some(0.04)), Admission::Go);
        // The first step's estimate is held while it runs.
        assert_eq!(ledger.admit(2, Some(0.05)), Admission::Go);
        match ledger.admit(3, Some(0.02)) {
            Admission::TooDear { estimate, left } => {
                assert!((estimate - 0.02).abs() < 1e-9);
                assert!((left - 0.01).abs() < 1e-9);
            }
            other => panic!("{other:?}"),
        }
        // The first step cost less than its estimate: what it did not spend is free again.
        assert!(!ledger.bill(1, 0.01));
        ledger.finish(1);
        assert_eq!(ledger.admit(3, Some(0.02)), Admission::Go);
    }

    #[test]
    fn a_step_without_a_price_does_not_start_under_a_ceiling() {
        let mut ledger = Ledger::new(Some(1.0));
        assert_eq!(ledger.admit(1, None), Admission::Unpriced);
        let mut open = Ledger::new(None);
        assert_eq!(open.admit(1, None), Admission::Go);
    }

    #[test]
    fn once_what_was_billed_passes_the_ceiling_nothing_more_starts() {
        let mut ledger = Ledger::new(Some(0.05));
        assert_eq!(ledger.admit(1, Some(0.01)), Admission::Go);
        assert!(!ledger.bill(1, 0.03));
        assert!(ledger.bill(1, 0.06), "the real cost passed the ceiling");
        ledger.finish(1);
        assert!(ledger.passed());
        assert_eq!(ledger.admit(2, Some(0.0001)), Admission::Stopped);
        assert!((ledger.spent() - 0.06).abs() < 1e-9);
    }

    #[test]
    fn without_a_ceiling_everything_starts_and_is_counted() {
        let mut ledger = Ledger::new(None);
        assert_eq!(ledger.admit(1, Some(100.0)), Admission::Go);
        assert!(!ledger.bill(1, 150.0));
        assert!((ledger.spent() - 150.0).abs() < 1e-9);
    }
}
