//! Pipelines: a job's settings saved under a name.

use kalamu_prompts::Pipeline;
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

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

/// Writes the pipeline's record twice, side by side: the JSON file another Kalamu imports
/// at `path`, and the same as Markdown for a methods section, whose path it returns.
#[tauri::command]
pub fn export_pipeline(state: State<'_, AppState>, id: String, path: String) -> CmdResult<String> {
    let pipeline = state
        .pipelines
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(&id)?;
    let file = state.engine.pipeline_file(pipeline)?;
    super::write_record(
        std::path::Path::new(&path),
        file.to_json()?,
        file.to_markdown(),
    )
}

/// Adds a shared pipeline and the presets it brings.
#[tauri::command]
pub fn import_pipeline(state: State<'_, AppState>, path: String) -> CmdResult<Pipeline> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::new(id::FILE, e))?;
    let mut pipelines = state.pipelines.lock().unwrap_or_else(|p| p.into_inner());
    Ok(state
        .engine
        .with_prompts(|prompts| pipelines.import(&json, prompts))?)
}
