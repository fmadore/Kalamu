//! Workflow runs (#49's first increment): steps run one after another over the same sources,
//! each as an ordinary job, so caching, resuming, batches, progress and provenance work as they
//! do for any job.
//!
//! A run freezes its definition, its sources and every step's prompts when it starts. A step
//! that reads the files starts at once; a step that reads an earlier one starts when that step's
//! job has finished and, where a review was asked or pieces could not be read, the researcher
//! has said to go on. It then reads that step's corrected text, frozen as a snapshot (#47) at
//! that moment. Running a workflow again reuses every unchanged answer from the cache: only the
//! steps whose model, prompt or input changed are paid for again.

use std::collections::HashMap;
use std::sync::Arc;

use kalamu_core::provenance::ProvenanceWorkflow;
use kalamu_core::workflow::{check_steps, may_read, run_status, step_state};
use kalamu_core::{
    Approval, CollectionId, DerivedFrom, InputKind, Job, JobId, JobStatus, JobSummary,
    ProgressEvent, ProjectId, RunId, RunStatus, RunStep, SetupId, SourceFile, SourceId, StepInput,
    StepProgress, StepState, StoreError, Timestamp, WorkflowRun,
};
use kalamu_prompts::Workflow;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::control::lock;
use crate::{Engine, EngineError, NewJob};

/// A workflow to start on a selection of files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StartRun {
    pub workflow: Workflow,
    /// Names the run, and each step's job after it.
    pub name: String,
    pub sources: Vec<SourceFile>,
    /// The project it is started from, whose documents its jobs are tied to.
    #[serde(default)]
    pub project: Option<ProjectId>,
    #[serde(default)]
    pub collection: Option<CollectionId>,
}

/// A run as the interface shows it: each step's state and job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RunView {
    pub run: WorkflowRun,
    pub status: RunStatus,
    pub steps: Vec<StepView>,
}

/// The run a job belongs to and which step it is, so the job's page and review can say what
/// comes next and offer to continue there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobStep {
    pub view: RunView,
    /// Numbered from 0.
    pub step: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StepView {
    pub state: StepState,
    /// It waits for the researcher, as the run's status counts it.
    pub needs_you: bool,
    pub job: Option<JobSummary>,
}

/// A stand-in for the text a later step will read from `source`, to check the step before the
/// run starts: a text, from that original.
pub(crate) fn as_text(source: &SourceFile) -> SourceFile {
    SourceFile {
        id: SourceId::new(),
        path: String::new(),
        file_name: source.file_name.clone(),
        sha256: String::new(),
        size_bytes: 0,
        modified_ms: None,
        kind: InputKind::Text,
        page_count: None,
        duration_ms: None,
        split_spreads: false,
        even_out_volume: false,
        title: None,
        derived_from: Some(DerivedFrom {
            snapshot_id: String::new(),
            job_id: JobId::new(),
            setup_id: SetupId::new(),
            source_id: source.id,
            original_path: source.path.clone(),
            unit: None,
        }),
        pages: Vec::new(),
    }
}

fn in_step(step: usize, error: EngineError) -> EngineError {
    EngineError::Step {
        step: step as u32 + 1,
        source: Box::new(error),
    }
}

impl Engine {
    /// Checks a workflow against the files it is to run on, freezes it, and starts the steps
    /// that read the files. Call it from within the Tokio runtime.
    pub fn start_run(self: &Arc<Self>, start: StartRun) -> Result<WorkflowRun, EngineError> {
        let StartRun {
            workflow,
            name,
            sources,
            project,
            collection,
        } = start;
        if sources.is_empty() {
            return Err(EngineError::EmptyJob);
        }
        check_steps(&workflow.shape()).map_err(EngineError::Workflow)?;
        let catalog = self.current_catalog();
        let texts: Vec<SourceFile> = sources.iter().map(as_text).collect();
        let mut steps = Vec::new();
        for (i, step) in workflow.steps.iter().enumerate() {
            let p = &step.pipeline;
            let setups = p.setups.clone();
            let new = NewJob {
                name: p.name.clone(),
                task: p.task,
                sources: match step.input {
                    StepInput::Sources => sources.clone(),
                    StepInput::Step { .. } => texts.clone(),
                },
                setups,
                settings: p.settings.clone(),
                output: kalamu_core::OutputSettings {
                    formats: p.formats.clone(),
                    destination: p.destination.clone(),
                    ..Default::default()
                },
            };
            self.check_new(&catalog, &new).map_err(|e| in_step(i, e))?;
            let mut new = new;
            crate::rules::writable_formats(
                &catalog,
                new.task,
                &new.setups,
                &mut new.output.formats,
            );
            let setups = self
                .freeze_setups(&catalog, new.task, &new.settings, &new.setups)
                .map_err(|e| in_step(i, e))?;
            steps.push(RunStep {
                name: p.name.clone(),
                task: p.task,
                input: step.input,
                review: step.review,
                setups,
                settings: new.settings,
                output: new.output,
                progress: StepProgress::default(),
            });
        }
        let name = match name.trim() {
            "" => workflow.name.clone(),
            n => n.to_string(),
        };
        let run = WorkflowRun {
            id: RunId::new(),
            name,
            workflow_id: workflow.id,
            workflow_name: workflow.name,
            version: workflow.version,
            created_at: Timestamp::now(),
            sources,
            steps,
            project,
            collection,
            cancelled: false,
        };
        self.store.create_run(&run)?;
        self.advance_run(run.id, true)?;
        Ok(self.store.run(run.id)?)
    }

    /// Starts each step whose input is ready: the files, or an earlier step finished and
    /// approved where that is needed. With `start` false the step's job is made but left for
    /// the researcher to start, as at launch, when nothing is sent without them.
    pub(crate) fn advance_run(self: &Arc<Self>, id: RunId, start: bool) -> Result<(), EngineError> {
        // One advance at a time: two steps finishing together must not start a step twice.
        let _turn = lock(&self.workflows);
        let run = self.store.run(id)?;
        if run.cancelled {
            return Ok(());
        }
        let mut changed = false;
        for (i, step) in run.steps.iter().enumerate() {
            // A step that could not start is tried again: its input may have been read since.
            if step.progress.job_id.is_some() {
                continue;
            }
            let sources = match step.input {
                StepInput::Sources => run.sources.clone(),
                StepInput::Step { step: read } => {
                    let Some(upstream) = run.steps.get(read as usize) else {
                        continue;
                    };
                    let Some(job) = upstream
                        .progress
                        .job_id
                        .and_then(|j| self.store.job(j).ok())
                    else {
                        continue;
                    };
                    if !may_read(upstream, job.status) {
                        continue;
                    }
                    let Some(setup) = job.setups.first() else {
                        continue;
                    };
                    match self.snapshot(job.id, setup.id) {
                        Ok(sources) => sources,
                        Err(e) => {
                            changed |= self.step_failed(&run, i, &e)?;
                            continue;
                        }
                    }
                }
            };
            let job = self.step_job(&run, step, sources);
            let progress = StepProgress {
                job_id: Some(job.id),
                error: None,
                ..step.progress.clone()
            };
            if let Err(e) = self
                .store
                .create_step_job(&job, run.id, i as u32, &progress)
            {
                changed |= self.step_failed(&run, i, &e.into())?;
                continue;
            }
            self.emit(ProgressEvent::StepStarted {
                run_id: run.id,
                step: i as u32,
                job_id: job.id,
            });
            if start && let Err(e) = self.start(job.id) {
                tracing::warn!(run = %run.id, step = i, error = %e, "step not started");
            }
        }
        if changed {
            self.emit(ProgressEvent::RunChanged { run_id: run.id });
        }
        Ok(())
    }

    /// The job that runs a step, with its set-ups as frozen when the run started; recorded with
    /// the step's progress.
    fn step_job(&self, run: &WorkflowRun, step: &RunStep, sources: Vec<SourceFile>) -> Job {
        let setups = step
            .setups
            .iter()
            .cloned()
            .map(|mut s| {
                s.id = SetupId::new();
                s
            })
            .collect();
        crate::setup::build_job(
            format!("{} · {}", run.name, step.name),
            step.task,
            sources,
            setups,
            step.settings.clone(),
            step.output.clone(),
        )
    }

    /// Records why a step could not start; whether that is news, to say the run changed.
    fn step_failed(
        &self,
        run: &WorkflowRun,
        index: usize,
        error: &EngineError,
    ) -> Result<bool, EngineError> {
        let code = error.code().to_string();
        let progress = &run.steps[index].progress;
        if progress.error.as_deref() == Some(code.as_str()) {
            return Ok(false);
        }
        tracing::warn!(run = %run.id, step = index, error = %error, "step could not start");
        let progress = StepProgress {
            error: Some(code),
            ..progress.clone()
        };
        self.store
            .set_step_progress(run.id, index as u32, &progress)?;
        Ok(true)
    }

    /// Lets the steps that read a finished step go on: after its review, or with the pieces
    /// that were read when some could not be. Recorded with the time, and in provenance.
    pub fn approve_step(self: &Arc<Self>, id: RunId, index: u32) -> Result<(), EngineError> {
        let run = self.store.run(id)?;
        let step = run
            .steps
            .get(index as usize)
            .ok_or(EngineError::Store(StoreError::NotFound))?;
        let status = step
            .progress
            .job_id
            .and_then(|j| self.store.job(j).ok())
            .map(|j| j.status);
        let partial = match status {
            Some(JobStatus::Completed) => false,
            Some(JobStatus::CompletedWithErrors) => true,
            _ => return Err(EngineError::StepNotFinished),
        };
        let progress = StepProgress {
            approval: Some(Approval {
                at: Timestamp::now(),
                partial,
            }),
            ..step.progress.clone()
        };
        self.store.set_step_progress(id, index, &progress)?;
        self.emit(ProgressEvent::RunChanged { run_id: id });
        self.advance_run(id, true)
    }

    /// Stops a run: its running steps are cancelled, and no further step starts. What was read
    /// is kept.
    pub fn cancel_run(&self, id: RunId) -> Result<(), EngineError> {
        // Taken as an advance takes it: a step being started is started before the run is read
        // here, and cancelled with the others; none starts after.
        let run = {
            let _turn = lock(&self.workflows);
            self.store.cancel_run(id)?;
            self.store.run(id)?
        };
        for step in &run.steps {
            if let Some(job) = step.progress.job_id {
                self.cancel(job);
            }
        }
        self.emit(ProgressEvent::RunChanged { run_id: id });
        Ok(())
    }

    /// Forgets a run; its jobs and their results stay.
    pub fn delete_run(&self, id: RunId) -> Result<(), EngineError> {
        Ok(self.store.delete_run(id)?)
    }

    /// At launch: makes the jobs of steps whose input became ready while the app was closing,
    /// without starting them.
    pub fn resume_runs(self: &Arc<Self>) -> Result<(), EngineError> {
        for run in self.store.runs()? {
            if !run.cancelled {
                self.advance_run(run.id, false)?;
            }
        }
        Ok(())
    }

    /// After a job's work ends: the run it belongs to, if any, goes on.
    pub(crate) fn job_ended(self: &Arc<Self>, job: JobId, start: bool) {
        match self.store.run_of_job(job) {
            Ok(Some((run, _))) => {
                if let Err(e) = self.advance_run(run, start) {
                    tracing::warn!(run = %run, error = %e, "workflow not advanced");
                }
            }
            Ok(None) => {}
            Err(e) => tracing::warn!(job = %job, error = %e, "workflow not found"),
        }
    }

    pub fn run_views(&self) -> Result<Vec<RunView>, EngineError> {
        let jobs = self.summaries()?;
        self.store
            .runs()?
            .into_iter()
            .map(|run| Ok(view(run, &jobs)))
            .collect()
    }

    pub fn run_view(&self, id: RunId) -> Result<RunView, EngineError> {
        Ok(view(self.store.run(id)?, &self.summaries()?))
    }

    /// The run and step a job was started by, if a workflow started it.
    pub fn job_step(&self, job: JobId) -> Result<Option<JobStep>, EngineError> {
        let Some((run, step)) = self.store.run_of_job(job)? else {
            return Ok(None);
        };
        Ok(Some(JobStep {
            view: self.run_view(run)?,
            step,
        }))
    }

    /// Every job by id; one the engine has started is running, though its loop may not yet
    /// have recorded so.
    fn summaries(&self) -> Result<HashMap<JobId, JobSummary>, EngineError> {
        Ok(self
            .store
            .list_jobs()?
            .into_iter()
            .map(|mut j| {
                if self.is_running(j.id) {
                    j.status = JobStatus::Running;
                }
                (j.id, j)
            })
            .collect())
    }

    /// The workflow step a job ran, for its results' provenance.
    pub(crate) fn workflow_provenance(
        &self,
        job: JobId,
    ) -> Result<Option<ProvenanceWorkflow>, EngineError> {
        let Some((id, index)) = self.store.run_of_job(job)? else {
            return Ok(None);
        };
        let run = self.store.run(id)?;
        let Some(step) = run.steps.get(index as usize) else {
            return Ok(None);
        };
        let read = match step.input {
            StepInput::Sources => None,
            StepInput::Step { step } => Some(step),
        };
        Ok(Some(ProvenanceWorkflow {
            run_id: run.id,
            workflow_id: run.workflow_id.clone(),
            name: run.workflow_name.clone(),
            version: run.version,
            step: index + 1,
            steps: run.steps.len() as u32,
            step_name: step.name.clone(),
            reads: read.map(|r| r + 1),
            approval: read
                .and_then(|r| run.steps.get(r as usize))
                .and_then(|s| s.progress.approval),
        }))
    }
}

fn view(run: WorkflowRun, jobs: &HashMap<JobId, JobSummary>) -> RunView {
    let mut states: Vec<StepState> = Vec::new();
    let mut steps = Vec::new();
    for (i, step) in run.steps.iter().enumerate() {
        let job = step.progress.job_id.map(|id| jobs.get(&id).cloned());
        let upstream = match step.input {
            StepInput::Sources => None,
            StepInput::Step { step } => states.get(step as usize).copied(),
        };
        let state = step_state(
            step,
            job.as_ref().map(|j| j.as_ref().map(|j| j.status)),
            run.is_read(i),
            upstream,
            run.cancelled,
        );
        states.push(state);
        steps.push(StepView {
            state,
            needs_you: state.needs_you(),
            job: job.flatten(),
        });
    }
    RunView {
        status: run_status(run.cancelled, &states),
        run,
        steps,
    }
}
