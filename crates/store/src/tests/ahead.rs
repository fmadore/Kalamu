//! What work started now would send (`kalamu_core::ahead`), read from the job database: a
//! job's models and its combining step's, the steps of its run not yet started, and the models
//! of a server that running work still uses.

use kalamu_core::ahead::{Sends, job_sends, models_in_use, providers, run_sends};
use kalamu_core::{MergeStep, RunId, RunStep, StepInput, StepProgress, WorkflowRun};

use super::*;

fn sends(provider: &str, model: &str) -> Sends {
    Sends {
        provider: ProviderId::new(provider),
        model: model.into(),
    }
}

/// A job whose set-up is `provider`'s `model`.
fn job_on(provider: &str, model: &str) -> Job {
    let mut job = job();
    job.id = JobId::new();
    job.setups[0].provider = ProviderId::new(provider);
    job.setups[0].model = model.into();
    job
}

/// A run of two steps: the first read by `first`, the second, which reads it, by `second`.
fn run(first: &Job, second: &Job) -> WorkflowRun {
    let step = |job: &Job, input| RunStep {
        name: job.name.clone(),
        task: job.task,
        input,
        review: false,
        setups: job.setups.clone(),
        settings: JobSettings::default(),
        output: OutputSettings::default(),
        progress: StepProgress::default(),
    };
    WorkflowRun {
        id: RunId::new(),
        name: "Letters".into(),
        workflow_id: "workflow-1".into(),
        workflow_name: "Extract and analyse".into(),
        version: 1,
        created_at: Timestamp::now(),
        sources: first.sources.clone(),
        steps: vec![
            step(first, StepInput::Sources),
            step(second, StepInput::Step { step: 0 }),
        ],
        project: None,
        collection: None,
        cancelled: false,
    }
}

#[test]
fn a_job_sends_to_its_set_ups_and_the_step_that_combines_them() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let mut job = job_on("own-0000abcd", "gemma3:12b");
    let mut second = job.setups[0].clone();
    second.id = SetupId::new();
    second.provider = ProviderId::new("gemini");
    second.model = "gemini-3.8-flash".into();
    second.merge = Some(MergeStep {
        provider: ProviderId::new("own-0000abcd"),
        model: "llama3.1:8b".into(),
        prompt: second.prompt.clone(),
    });
    job.setups.push(second);
    store.create_job(&job)?;

    let ahead = job_sends(&store, job.id)?;
    assert_eq!(
        ahead,
        [
            sends("own-0000abcd", "gemma3:12b"),
            sends("gemini", "gemini-3.8-flash"),
            sends("own-0000abcd", "llama3.1:8b"),
        ]
    );
    assert_eq!(
        providers(&ahead),
        [ProviderId::new("own-0000abcd"), ProviderId::new("gemini")]
    );
    Ok(())
}

#[test]
fn a_step_s_job_also_sends_to_the_steps_its_run_has_not_started() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let first = job_on("mistral", "mistral-ocr-latest");
    let analysis = job_on("own-0000abcd", "llama3.1:8b");
    let run = run(&first, &analysis);
    store.create_job(&first)?;
    store.create_run(&run)?;
    store.set_step_progress(
        run.id,
        0,
        &StepProgress {
            job_id: Some(first.id),
            ..StepProgress::default()
        },
    )?;

    // The extraction, resumed, starts the analysis when it ends.
    assert_eq!(
        job_sends(&store, first.id)?,
        [
            sends("mistral", "mistral-ocr-latest"),
            sends("own-0000abcd", "llama3.1:8b"),
        ]
    );
    assert_eq!(
        run_sends(&store, run.id)?,
        [sends("own-0000abcd", "llama3.1:8b")]
    );

    // Once the analysis has its own job, it is that job's to start.
    let second = run.steps[1].setups.clone();
    let mut analysis_job = job_on("own-0000abcd", "llama3.1:8b");
    analysis_job.setups = second;
    store.create_step_job(
        &analysis_job,
        run.id,
        1,
        &StepProgress {
            job_id: Some(analysis_job.id),
            ..StepProgress::default()
        },
    )?;
    assert_eq!(
        job_sends(&store, first.id)?,
        [sends("mistral", "mistral-ocr-latest")]
    );
    assert_eq!(run_sends(&store, run.id)?, []);

    // A cancelled run starts nothing more.
    let other = self::run(&first, &analysis);
    store.create_run(&other)?;
    store.cancel_run(other.id)?;
    assert_eq!(run_sends(&store, other.id)?, []);
    Ok(())
}

#[test]
fn a_server_is_in_use_while_running_work_sends_to_it() -> Result<(), StoreError> {
    let store = SqliteStore::in_memory()?;
    let server = ProviderId::new("own-0000abcd");
    let reading = job_on("own-0000abcd", "gemma3:12b");
    let elsewhere = job_on("gemini", "gemini-3.8-flash");
    let paused = job_on("own-0000abcd", "qwen2.5vl:7b");
    for job in [&reading, &elsewhere, &paused] {
        store.create_job(job)?;
    }
    // A run whose first step runs on Gemini, and whose analysis, not started, on the server.
    let first = job_on("gemini", "gemini-3.8-flash");
    let analysis = job_on("own-0000abcd", "llama3.1:8b");
    let run = run(&first, &analysis);
    store.create_job(&first)?;
    store.create_run(&run)?;
    store.set_step_progress(
        run.id,
        0,
        &StepProgress {
            job_id: Some(first.id),
            ..StepProgress::default()
        },
    )?;

    // Only running jobs count: the paused one is checked again when it is resumed.
    assert_eq!(
        models_in_use(&store, [reading.id, elsewhere.id], &server)?,
        ["gemma3:12b"]
    );
    assert_eq!(
        models_in_use(&store, [elsewhere.id], &server)?,
        Vec::<String>::new()
    );
    assert_eq!(
        models_in_use(&store, [first.id, reading.id], &server)?,
        ["llama3.1:8b", "gemma3:12b"]
    );
    // A job deleted as it stopped sends nothing.
    assert_eq!(
        models_in_use(&store, [JobId::new()], &server)?,
        Vec::<String>::new()
    );
    Ok(())
}
