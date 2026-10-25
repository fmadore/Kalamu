//! Pipelines: a job's settings saved under a name.

use scriptorium_prompts::Pipeline;
use tauri::State;

use super::AppState;
use crate::errors::CmdResult;

#[tauri::command]
pub fn list_pipelines(state: State<'_, AppState>) -> Vec<Pipeline> {
    state
        .pipelines
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .list()
}

#[tauri::command]
pub fn save_pipeline(state: State<'_, AppState>, pipeline: Pipeline) -> CmdResult<Pipeline> {
    Ok(state
        .pipelines
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .save(pipeline)?)
}

#[tauri::command]
pub fn delete_pipeline(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state
        .pipelines
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .delete(&id)?)
}
