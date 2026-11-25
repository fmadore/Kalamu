//! What each step came to, and the table that says it: printed at the end of every run, written
//! to `<out>/summary.md`, and appended to `LIVE_SUMMARY` (the workflow's job summary). It holds
//! statuses, models and dollars, never a key or a document's text: an error is the provider's
//! message, redacted and cut.

use std::fmt::Write as _;

/// How a step ended, for one provider and model.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Passed,
    Failed(String),
    /// Not started, or stopped, by the ceiling: a failure of the run.
    Ceiling(String),
    /// A batch the provider had not answered within the wait: cancelled there, not a failure.
    Late(String),
    /// Nothing to check: no key, no model for the step, a sample missing, answered from the
    /// cache of an earlier run.
    Skipped(String),
}

impl Outcome {
    pub fn fails_the_run(&self) -> bool {
        matches!(self, Self::Failed(_) | Self::Ceiling(_))
    }

    fn words(&self) -> String {
        match self {
            Self::Passed => "passed".into(),
            Self::Failed(why) => format!("**failed**: {why}"),
            Self::Ceiling(why) => format!("**ceiling**: {why}"),
            Self::Late(why) => format!("late: {why}"),
            Self::Skipped(why) => format!("skipped: {why}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub step: String,
    pub provider: String,
    pub model: String,
    /// Through the provider's batch service, at its price.
    pub economy: bool,
    pub estimate: Option<f64>,
    /// What the catalogue's prices put on the usage the provider reported.
    pub billed: Option<f64>,
    pub outcome: Outcome,
}

impl Row {
    pub fn new(step: &str, provider: &str, model: &str, outcome: Outcome) -> Self {
        Self {
            step: step.into(),
            provider: provider.into(),
            model: model.into(),
            economy: false,
            estimate: None,
            billed: None,
            outcome,
        }
    }
}

/// What the run as a whole came to.
pub struct Totals {
    pub set: String,
    pub ceiling: Option<f64>,
    pub spent: f64,
    /// What was billed passed the ceiling: a failure of the run, whatever each step says.
    pub passed: bool,
    pub catalogue: String,
    pub date: String,
}

fn usd(value: Option<f64>) -> String {
    value.map_or("–".into(), |v| format!("${v:.4}"))
}

fn ratio(row: &Row) -> String {
    match (row.billed, row.estimate) {
        (Some(b), Some(e)) if e > 0.0 && b > 0.0 => format!("{:.2}×", b / e),
        _ => "–".into(),
    }
}

/// The run's table in Markdown, ready for a job summary or a decision entry, with the way the
/// 3 November entry notes costs: what was billed against the estimate New job would show.
pub fn markdown(rows: &[Row], totals: &Totals) -> String {
    let mut out = String::new();
    let failed = rows.iter().filter(|r| r.outcome.fails_the_run()).count();
    let _ = writeln!(out, "## Live check: {} set, {}\n", totals.set, totals.date);
    let _ = writeln!(
        out,
        "{}{} · spent **${:.4}** of a ceiling of {} · catalogue {}\n",
        if failed == 0 {
            "All steps passed or were skipped".to_string()
        } else {
            format!("**{failed} step(s) failed**")
        },
        if totals.passed {
            " · **the ceiling was passed**"
        } else {
            ""
        },
        totals.spent,
        totals.ceiling.map_or("none".into(), |c| format!("${c:.2}")),
        totals.catalogue,
    );
    out.push_str("| Step | Provider | Model | Estimate | Billed | Billed / estimate | Result |\n");
    out.push_str("| --- | --- | --- | ---: | ---: | ---: | --- |\n");
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {}{} | {} | {} | {} | {} |",
            cell(&row.step),
            cell(&row.provider),
            cell(&row.model),
            if row.economy { " (batch)" } else { "" },
            usd(row.estimate),
            usd(row.billed),
            ratio(row),
            cell(&row.outcome.words()),
        );
    }
    out.push_str(
        "\nBilled is what the catalogue's prices put on the usage each provider reported, as \
         the app counts it; the providers' own invoices are the reference.\n",
    );
    out
}

/// A table cell's text: no pipe or line break to break the row.
fn cell(text: &str) -> String {
    text.replace('|', "/").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_sets_what_was_billed_against_the_estimate() {
        let mut passed = Row::new(
            "page-gemini",
            "gemini",
            "gemini-3.5-flash-lite",
            Outcome::Passed,
        );
        passed.estimate = Some(0.0020);
        passed.billed = Some(0.0010);
        let mut batch = Row::new(
            "batch-openai",
            "openai",
            "gpt-6-luna",
            Outcome::Late("not back after 60 min".into()),
        );
        batch.economy = true;
        let failed = Row::new(
            "analysis-mistral",
            "mistral",
            "mistral-small-2603",
            Outcome::Failed("rate | limited".into()),
        );
        let table = markdown(
            &[passed, batch, failed],
            &Totals {
                set: "weekly".into(),
                ceiling: Some(0.05),
                spent: 0.001,
                passed: false,
                catalogue: "2026-11-24 (revision 1)".into(),
                date: "2026-11-25".into(),
            },
        );
        assert!(table.contains("| page-gemini | gemini | gemini-3.5-flash-lite | $0.0020 | $0.0010 | 0.50× | passed |"), "{table}");
        assert!(table.contains("gpt-6-luna (batch)"));
        assert!(table.contains("rate / limited"));
        assert!(table.contains("**1 step(s) failed**"));
        assert!(table.contains("ceiling of $0.05"));
    }

    #[test]
    fn late_and_skipped_steps_do_not_fail_the_run_but_the_ceiling_does() {
        assert!(!Outcome::Late(String::new()).fails_the_run());
        assert!(!Outcome::Skipped(String::new()).fails_the_run());
        assert!(Outcome::Ceiling(String::new()).fails_the_run());
        assert!(Outcome::Failed(String::new()).fails_the_run());
    }
}
