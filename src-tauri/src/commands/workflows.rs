//! Workflows: saved definitions, and the runs started from them. The engine runs each step as
//! a job; these commands only pass requests through.

use kalamu_core::JobId;
use kalamu_core::RunId;
use kalamu_engine::{JobStep, RunView, StartRun};
use kalamu_prompts::Workflow;
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn list_workflows(state: State<'_, AppState>) -> Vec<Workflow> {
    state
        .workflows
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .list()
}

#[tauri::command]
pub fn save_workflow(state: State<'_, AppState>, workflow: Workflow) -> CmdResult<Workflow> {
    Ok(state
        .workflows
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .save(workflow)?)
}

#[tauri::command]
pub fn duplicate_workflow(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CmdResult<Workflow> {
    Ok(state
        .workflows
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .duplicate(&id, &name)?)
}

#[tauri::command]
pub fn delete_workflow(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state
        .workflows
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .delete(&id)?)
}

/// Writes the workflow's record twice, side by side: the JSON file another Kalamu imports at
/// `path`, and the same with a `.md` extension as Markdown for a methods section.
#[tauri::command]
pub fn export_workflow(state: State<'_, AppState>, id: String, path: String) -> CmdResult<()> {
    let workflow = state
        .workflows
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&id)?;
    let file = state.engine.workflow_file(workflow)?;
    let path = std::path::Path::new(&path);
    super::write_text(path.with_extension("json"), file.to_json()?)?;
    super::write_text(path.with_extension("md"), file.to_markdown())
}

/// Adds a shared workflow and the presets it brings.
#[tauri::command]
pub fn import_workflow(state: State<'_, AppState>, path: String) -> CmdResult<Workflow> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::new(id::FILE, e))?;
    let mut workflows = state.workflows.lock().unwrap_or_else(|p| p.into_inner());
    Ok(state
        .engine
        .with_prompts(|prompts| workflows.import(&json, prompts))?)
}

/// Freezes the workflow with the files chosen and starts its first steps. Async so that it
/// runs on the Tokio runtime its jobs are spawned on.
#[tauri::command]
pub async fn start_run(state: State<'_, AppState>, start: StartRun) -> CmdResult<RunView> {
    super::require_consent(
        &state,
        start.workflow.steps.iter().flat_map(|s| &s.pipeline.setups),
    )?;
    let folders = start
        .workflow
        .steps
        .iter()
        .filter_map(|step| super::settings::destination_folder(&step.pipeline.destination));
    state.grants.keep(folders);
    let run = state.engine.start_run(start)?;
    Ok(state.engine.run_view(run.id)?)
}

#[tauri::command]
pub fn list_runs(state: State<'_, AppState>) -> CmdResult<Vec<RunView>> {
    Ok(state.engine.run_views()?)
}

#[tauri::command]
pub fn get_run(state: State<'_, AppState>, id: RunId) -> CmdResult<RunView> {
    Ok(state.engine.run_view(id)?)
}

/// The run and step a job belongs to, if a workflow started it.
#[tauri::command]
pub fn job_step(state: State<'_, AppState>, id: JobId) -> CmdResult<Option<JobStep>> {
    Ok(state.engine.job_step(id)?)
}

/// The researcher's go-ahead after a step's review, or despite pieces not read.
#[tauri::command]
pub async fn approve_step(state: State<'_, AppState>, id: RunId, step: u32) -> CmdResult<()> {
    Ok(state.engine.approve_step(id, step)?)
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, id: RunId) -> CmdResult<()> {
    Ok(state.engine.cancel_run(id)?)
}

/// Forgets the run; its jobs and their results stay.
#[tauri::command]
pub fn delete_run(state: State<'_, AppState>, id: RunId) -> CmdResult<()> {
    Ok(state.engine.delete_run(id)?)
}
