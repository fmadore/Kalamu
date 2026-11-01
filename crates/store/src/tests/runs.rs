//! Workflow runs: frozen as started, their steps' progress kept apart, and a deleted job left
//! pointing at, so the step is not started again.

use kalamu_core::{Approval, RunId, RunStep, StepInput, StepProgress, WorkflowRun};

use super::*;

fn run(job: &Job) -> WorkflowRun {
    let step = |name: &str, task, input| RunStep {
        name: name.into(),
        task,
        input,
        review: false,
        setups: job.setups.clone(),
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        progress: StepProgress::default(),
    };
    WorkflowRun {
        id: RunId::new(),
        name: "Letters, extract and analyse".into(),
        workflow_id: "workflow-1".into(),
        workflow_name: "Extract and analyse".into(),
        version: 3,
        created_at: Timestamp::now(),
        sources: job.sources.clone(),
        steps: vec![
            step("Extract", Task::TextExtraction, StepInput::Sources),
            step("People", Task::Analysis, StepInput::Step { step: 0 }),
        ],
        project: None,
        collection: None,
        cancelled: false,
    }
}

#[test]
fn a_run_keeps_its_frozen_steps_and_each_steps_progress() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let job = job();
    store.create_job(&job)?;
    let run = run(&job);
    store.create_run(&run)?;
    assert_eq!(store.run(run.id)?, run);
    assert_eq!(store.run_of_job(job.id)?, None);

    let started = StepProgress {
        job_id: Some(job.id),
        ..StepProgress::default()
    };
    store.set_step_progress(run.id, 0, &started)?;
    let approved = StepProgress {
        approval: Some(Approval {
            at: Timestamp(42),
            partial: true,
        }),
        ..started.clone()
    };
    store.set_step_progress(run.id, 0, &approved)?;
    store.set_step_progress(
        run.id,
        1,
        &StepProgress {
            error: Some("job.nothing_read".into()),
            ..StepProgress::default()
        },
    )?;
    let read = store.run(run.id)?;
    assert_eq!(read.steps[0].progress, approved);
    assert_eq!(
        read.steps[1].progress.error.as_deref(),
        Some("job.nothing_read")
    );
    assert_eq!(store.run_of_job(job.id)?, Some((run.id, 0)));
    assert!(matches!(
        store.set_step_progress(run.id, 5, &started),
        Err(StoreError::NotFound)
    ));

    // A job deleted leaves the step pointing at it: the run knows it ran.
    store.delete_job(job.id)?;
    assert_eq!(store.run(run.id)?.steps[0].progress.job_id, Some(job.id));

    store.cancel_run(run.id)?;
    assert!(store.run(run.id)?.cancelled);
    let second = run_named(&job, "Second");
    store.create_run(&second)?;
    let listed: Vec<RunId> = store.runs()?.iter().map(|r| r.id).collect();
    assert_eq!(listed, vec![second.id, run.id], "newest first");

    store.delete_run(run.id)?;
    assert!(matches!(store.run(run.id), Err(StoreError::NotFound)));
    assert_eq!(store.run_of_job(job.id)?, None);
    Ok(())
}

fn run_named(job: &Job, name: &str) -> WorkflowRun {
    let mut run = run(job);
    run.name = name.into();
    run.created_at = Timestamp(Timestamp::now().0 + 1);
    run
}
