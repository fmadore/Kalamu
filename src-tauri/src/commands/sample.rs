//! After a sample (#215): the rest of its documents with one of its set-ups, and another model
//! on its pages. The engine makes both jobs from the sample ([`kalamu_engine::Engine`]'s
//! `rest_of_sample` and `compare_sample`); each joins the projects the sample is in, and the
//! interface starts it as it starts any job.

use kalamu_core::{Job, JobId, ProviderId, SetupId, SourceDocuments};
use kalamu_engine::RestEstimate;
use tauri::State;

use super::AppState;
use crate::errors::CmdResult;

/// What reading every page with the sample's set-up `setup` would cost, the pages it read
/// taken from the cache.
#[tauri::command]
pub fn rest_estimate(
    state: State<'_, AppState>,
    job: JobId,
    setup: SetupId,
) -> CmdResult<RestEstimate> {
    Ok(state.engine.rest_estimate(job, setup)?)
}

/// The job that reads every page of the sample `job`'s documents with its set-up `setup`,
/// named `name`; not started.
#[tauri::command]
pub fn rest_of_sample(
    state: State<'_, AppState>,
    job: JobId,
    setup: SetupId,
    name: String,
) -> CmdResult<Job> {
    let sample = state.engine.job(job)?;
    if let Some(chosen) = sample.setups.iter().find(|s| s.id == setup) {
        super::consented(&state, &chosen.provider)?;
        if let Some(merge) = &chosen.merge {
            super::consented(&state, &merge.provider)?;
        }
    }
    let made = state.engine.rest_of_sample(job, setup, name)?;
    join_projects(&state, &sample, made.id);
    Ok(made)
}

/// The job that reads the sample `job`'s pages again with its set-ups and with `model` of
/// `provider` beside them, named `name`; not started.
#[tauri::command]
pub fn compare_sample(
    state: State<'_, AppState>,
    job: JobId,
    provider: ProviderId,
    model: String,
    name: String,
) -> CmdResult<Job> {
    super::consented(&state, &provider)?;
    let sample = state.engine.job(job)?;
    let made = state.engine.compare_sample(job, provider, model, name)?;
    join_projects(&state, &sample, made.id);
    Ok(made)
}

/// Links `made` to every project the sample's files are documents of, as a job started from
/// the project is: its files are the same documents. A link that fails leaves the job as it
/// is, to be added from its menu.
fn join_projects(state: &AppState, sample: &Job, made: JobId) {
    let mut projects = Vec::new();
    for source in &sample.sources {
        let Ok(documents) = state.projects.documents(sample.id, source.id) else {
            continue;
        };
        for document in documents {
            if !projects.contains(&document.project_id) {
                projects.push(document.project_id);
            }
        }
    }
    for project in projects {
        if let Err(e) = state.projects.link_job(project, made, None) {
            tracing::warn!(job = %made, error = %e, "the job made from a sample joined no project");
        }
    }
}
