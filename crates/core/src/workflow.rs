//! Workflows: steps run one after another over the same documents, each a job of its own (#49).
//!
//! A workflow's definition is saved by the researcher (in `prompts`); a **run** is that
//! definition frozen with the sources it was started on and every step's prompts, so editing
//! the definition or a preset afterwards never changes a run already started. Each step becomes
//! an ordinary job when its input is ready, and reads either the run's sources or the corrected
//! text of an earlier step, frozen as a snapshot (#47) when the step starts.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    CollectionId, InputKind, JobId, JobSettings, JobStatus, OutputSettings, ProjectId, RunId,
    Setup, SourceFile, Task, Timestamp,
};

/// What a step reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum StepInput {
    /// The files the run was started on.
    #[default]
    Sources,
    /// The corrected text of an earlier step (numbered from 0), under its first set-up.
    Step { step: u32 },
}

/// Why a list of steps cannot run.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowError {
    #[error("A workflow needs at least one step")]
    NoSteps,
    /// The first step has nothing earlier to read.
    #[error("Step {step} reads a step that does not come before it")]
    NotEarlier { step: u32 },
    /// The step read is a table of contents or an analysis, which has no text to read.
    #[error("Step {step} reads a step whose results are not a text")]
    NotText { step: u32 },
    /// The step's task does not read text (an extraction reads scans).
    #[error("Step {step} cannot read text")]
    CannotReadText { step: u32 },
}

impl WorkflowError {
    /// A stable id the interface translates.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoSteps => "workflow.no_steps",
            Self::NotEarlier { .. } => "workflow.not_earlier",
            Self::NotText { .. } => "workflow.not_text",
            Self::CannotReadText { .. } => "workflow.cannot_read_text",
        }
    }

    /// The step at fault, numbered from 1 as the interface shows it.
    pub fn step(&self) -> Option<u32> {
        match self {
            Self::NoSteps => None,
            Self::NotEarlier { step } | Self::NotText { step } | Self::CannotReadText { step } => {
                Some(*step)
            }
        }
    }
}

/// Checks that every step reads something it can: a step before it, never itself or one after
/// (so no cycle can form), whose results are a text, by a task that reads text. Steps are given
/// as their task and input, in order.
pub fn check_steps(steps: &[(Task, StepInput)]) -> Result<(), WorkflowError> {
    if steps.is_empty() {
        return Err(WorkflowError::NoSteps);
    }
    for (i, (task, input)) in steps.iter().enumerate() {
        let number = i as u32 + 1;
        let StepInput::Step { step } = input else {
            continue;
        };
        let Some((read, _)) = steps.get(*step as usize).filter(|_| (*step as usize) < i) else {
            return Err(WorkflowError::NotEarlier { step: number });
        };
        if !read.yields_text() {
            return Err(WorkflowError::NotText { step: number });
        }
        if !task.reads_source(InputKind::Text) {
            return Err(WorkflowError::CannotReadText { step: number });
        }
    }
    Ok(())
}

/// One step of a run, frozen when the run started: its prompts are those of that moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RunStep {
    pub name: String,
    pub task: Task,
    pub input: StepInput,
    /// Stop after this step until the researcher has reviewed it and says to go on.
    pub review: bool,
    /// The set-ups with their prompts frozen; their ids are given anew to each job.
    pub setups: Vec<Setup>,
    pub settings: JobSettings,
    pub output: OutputSettings,
    /// How far it got: kept by the store apart from the frozen definition.
    #[serde(default)]
    pub progress: StepProgress,
}

/// What has happened to a step of a run.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StepProgress {
    /// The job that runs it, once its input was ready.
    pub job_id: Option<JobId>,
    /// The researcher's go-ahead for the steps that read this one.
    pub approval: Option<Approval>,
    /// Why it could not start (an error code the interface translates).
    pub error: Option<String>,
}

/// The researcher's decision to let later steps read a step's results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Approval {
    pub at: Timestamp,
    /// Given although some pieces could not be read: the later steps read what was.
    pub partial: bool,
}

/// A workflow started on a set of sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorkflowRun {
    pub id: RunId,
    pub name: String,
    /// The definition it was started from, and its version then.
    pub workflow_id: String,
    pub workflow_name: String,
    pub version: u32,
    pub created_at: Timestamp,
    /// The selection it was started on: later changes to a collection do not reach it.
    pub sources: Vec<SourceFile>,
    pub steps: Vec<RunStep>,
    /// The project it was started from, whose documents its jobs are tied to.
    #[serde(default)]
    pub project: Option<ProjectId>,
    #[serde(default)]
    pub collection: Option<CollectionId>,
    /// Stopped by the researcher: no further step starts.
    #[serde(default)]
    pub cancelled: bool,
}

impl WorkflowRun {
    /// Whether any step reads this one.
    pub fn is_read(&self, index: usize) -> bool {
        self.steps
            .iter()
            .any(|s| s.input == StepInput::Step { step: index as u32 })
    }
}

/// Where a step of a run stands, from its job's status and the steps it depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum StepState {
    /// Its input is not ready yet.
    Waiting,
    Running,
    /// Its job is paused, interrupted or not started: the researcher resumes it.
    Paused,
    /// Finished, and waiting for the researcher's review before the steps that read it.
    Review,
    /// Finished with pieces that could not be read: retry them, or go on with what was read.
    Attention,
    Done,
    /// Finished with pieces not read, and nothing waits on it (or the researcher went on).
    DoneWithErrors,
    Cancelled,
    /// The step it reads was cancelled or deleted, so it cannot start.
    Blocked,
    /// It could not start: `StepProgress::error` says why.
    Failed,
}

impl StepState {
    /// A step whose results later steps may read, once approved where that is asked.
    pub fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Review | Self::Attention | Self::Done | Self::DoneWithErrors
        )
    }

    /// A step that waits for the researcher: to check it, to retry or go on, or to start it.
    pub fn needs_you(self) -> bool {
        matches!(self, Self::Review | Self::Attention | Self::Paused)
    }
}

/// A step's state from its job's status (`None` before it has a job, `Some(None)` for a job
/// since deleted), whether later steps read it, and the state of the step it reads.
pub fn step_state(
    step: &RunStep,
    job: Option<Option<JobStatus>>,
    read_by_others: bool,
    upstream: Option<StepState>,
    cancelled: bool,
) -> StepState {
    let approved = step.progress.approval.is_some();
    match job {
        Some(None) => StepState::Blocked,
        Some(Some(status)) => match status {
            JobStatus::Running => StepState::Running,
            JobStatus::Pending | JobStatus::Paused | JobStatus::Interrupted => {
                if cancelled {
                    StepState::Cancelled
                } else {
                    StepState::Paused
                }
            }
            JobStatus::Cancelled => StepState::Cancelled,
            JobStatus::Completed if step.review && read_by_others && !approved => StepState::Review,
            JobStatus::Completed => StepState::Done,
            JobStatus::CompletedWithErrors if read_by_others && !approved => StepState::Attention,
            JobStatus::CompletedWithErrors => StepState::DoneWithErrors,
        },
        None if step.progress.error.is_some() => StepState::Failed,
        None if cancelled => StepState::Cancelled,
        None => match upstream {
            Some(StepState::Cancelled | StepState::Blocked | StepState::Failed) => {
                StepState::Blocked
            }
            _ => StepState::Waiting,
        },
    }
}

/// Whether a step reading `upstream` may start: its job finished, and approved where a review
/// was asked or pieces were not read. A pending decision never counts as approval.
pub fn may_read(upstream: &RunStep, status: JobStatus) -> bool {
    let approved = upstream.progress.approval.is_some();
    match status {
        JobStatus::Completed => !upstream.review || approved,
        JobStatus::CompletedWithErrors => approved,
        _ => false,
    }
}

/// The run as a whole, for lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RunStatus {
    Running,
    /// A step waits for the researcher: a review, failed pieces, or a paused job.
    NeedsYou,
    Finished,
    /// Cancelled, or a step could not start.
    Stopped,
}

pub fn run_status(cancelled: bool, states: &[StepState]) -> RunStatus {
    use StepState::*;
    if cancelled {
        return RunStatus::Stopped;
    }
    if states.contains(&Running) {
        return RunStatus::Running;
    }
    if states.iter().any(|s| s.needs_you()) {
        return RunStatus::NeedsYou;
    }
    if states.iter().all(|s| matches!(s, Done | DoneWithErrors)) {
        return RunStatus::Finished;
    }
    if states
        .iter()
        .any(|s| matches!(s, Failed | Blocked | Cancelled))
    {
        return RunStatus::Stopped;
    }
    // A step about to start, between its input finishing and its job being created.
    RunStatus::Running
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(step: u32) -> StepInput {
        StepInput::Step { step }
    }

    #[test]
    fn steps_read_earlier_text_only() {
        use Task::*;
        assert_eq!(check_steps(&[]), Err(WorkflowError::NoSteps));
        assert!(check_steps(&[(TextExtraction, StepInput::Sources), (Analysis, read(0))]).is_ok());
        // Transcribe, correct the transcript, analyse the correction.
        assert!(
            check_steps(&[
                (AudioTranscription, StepInput::Sources),
                (TextCorrection, read(0)),
                (Analysis, read(1)),
            ])
            .is_ok()
        );
        assert_eq!(
            check_steps(&[(Analysis, read(0))]),
            Err(WorkflowError::NotEarlier { step: 1 })
        );
        assert_eq!(
            check_steps(&[(TextExtraction, StepInput::Sources), (Analysis, read(2))]),
            Err(WorkflowError::NotEarlier { step: 2 })
        );
        assert_eq!(
            check_steps(&[(Analysis, StepInput::Sources), (Analysis, read(0))]),
            Err(WorkflowError::NotText { step: 2 })
        );
        assert_eq!(
            check_steps(&[(TextExtraction, StepInput::Sources), (Contents, read(0))]),
            Err(WorkflowError::CannotReadText { step: 2 })
        );
    }

    fn step(review: bool, approval: Option<Approval>) -> RunStep {
        RunStep {
            name: "Extract".into(),
            task: Task::TextExtraction,
            input: StepInput::Sources,
            review,
            setups: Vec::new(),
            settings: JobSettings::default(),
            output: OutputSettings::default(),
            progress: StepProgress {
                approval,
                ..StepProgress::default()
            },
        }
    }

    #[test]
    fn a_review_or_failed_pieces_hold_the_next_step_until_approved() {
        let approval = Some(Approval {
            at: Timestamp(1),
            partial: false,
        });
        let state = |s: &RunStep, status| step_state(s, Some(Some(status)), true, None, false);
        assert!(may_read(&step(false, None), JobStatus::Completed));
        assert!(!may_read(&step(true, None), JobStatus::Completed));
        assert_eq!(
            state(&step(true, None), JobStatus::Completed),
            StepState::Review
        );
        assert!(may_read(&step(true, approval), JobStatus::Completed));
        assert_eq!(
            state(&step(true, approval), JobStatus::Completed),
            StepState::Done
        );
        assert!(!may_read(
            &step(false, None),
            JobStatus::CompletedWithErrors
        ));
        assert_eq!(
            state(&step(false, None), JobStatus::CompletedWithErrors),
            StepState::Attention
        );
        assert!(may_read(
            &step(false, approval),
            JobStatus::CompletedWithErrors
        ));
        assert!(!may_read(&step(false, None), JobStatus::Paused));
        // Nothing reads the last step: its review mark holds nothing.
        assert_eq!(
            step_state(
                &step(true, None),
                Some(Some(JobStatus::Completed)),
                false,
                None,
                false
            ),
            StepState::Done
        );
    }

    #[test]
    fn a_step_without_a_job_waits_or_is_blocked() {
        let s = step(false, None);
        assert_eq!(step_state(&s, None, false, None, false), StepState::Waiting);
        assert_eq!(
            step_state(&s, None, false, Some(StepState::Running), false),
            StepState::Waiting
        );
        assert_eq!(
            step_state(&s, None, false, Some(StepState::Cancelled), false),
            StepState::Blocked
        );
        assert_eq!(
            step_state(&s, Some(None), false, None, false),
            StepState::Blocked
        );
        assert_eq!(
            step_state(&s, None, false, None, true),
            StepState::Cancelled
        );
        assert_eq!(
            run_status(false, &[StepState::Done, StepState::Review]),
            RunStatus::NeedsYou
        );
        assert_eq!(
            run_status(false, &[StepState::Done, StepState::DoneWithErrors]),
            RunStatus::Finished
        );
        assert_eq!(
            run_status(false, &[StepState::Cancelled, StepState::Blocked]),
            RunStatus::Stopped
        );
    }
}
