//! Which work economy mode sends in a batch: one rule, read by the run when it sends a source's
//! pieces ([`crate::run`]) and by the estimate when it prices them ([`crate::estimate`]), so the
//! price said before Start is the price the run pays. A set-up's work on a source goes in a batch
//! when its provider batches the task with its model, unless its answers are checked against the
//! source in a way a batch's collection does not do; that work is sent at once, at the usual
//! price, in economy mode too.

use kalamu_core::{SourceFile, Task};
use serde::Serialize;
use ts_rs::TS;

/// Why economy mode sends a set-up's work at once, at the usual price, although its provider has
/// a batch service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AtOnce {
    /// An analysis whose fields cite the text: each quotation is looked for in the text as the
    /// answer comes in.
    Quotes,
    /// A transcript translated line by line: each answer is read back against the lines its
    /// passage sent.
    TimedLines,
}

/// Whether `task` reads `source` as timed lines: a translation of a transcript frozen with them.
pub(crate) fn reads_timed_lines(task: Task, source: &SourceFile) -> bool {
    task == Task::Translation && source.derived_from.as_ref().is_some_and(|d| d.timed)
}

/// Why a set-up's work on `source` is sent at once whatever the pace, `cites` saying whether its
/// fields cite the text; `None` when it can go in a batch.
pub(crate) fn at_once(task: Task, source: &SourceFile, cites: bool) -> Option<AtOnce> {
    if reads_timed_lines(task, source) {
        Some(AtOnce::TimedLines)
    } else if task == Task::Analysis && cites {
        Some(AtOnce::Quotes)
    } else {
        None
    }
}

/// Whether economy mode sends a set-up's work on `source` in a batch: `batches` says whether its
/// provider batches the task with its model, `cites` whether its fields cite the text.
pub(crate) fn in_batch(batches: bool, task: Task, source: &SourceFile, cites: bool) -> bool {
    batches && at_once(task, source, cites).is_none()
}
