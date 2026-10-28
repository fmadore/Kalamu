//! Prompt presets.

use kalamu_core::Task;
use kalamu_prompts::Preset;
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn list_presets(state: State<'_, AppState>, task: Option<Task>) -> Vec<Preset> {
    state.engine.with_prompts(|p| p.list(task))
}

#[tauri::command]
pub fn save_preset(state: State<'_, AppState>, preset: Preset) -> CmdResult<Preset> {
    Ok(state.engine.with_prompts(|p| p.save(preset))?)
}

/// Deletes a user preset, or resets an edited default.
#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state.engine.with_prompts(|p| p.delete(&id))?)
}

#[tauri::command]
pub fn duplicate_preset(state: State<'_, AppState>, id: String, name: String) -> CmdResult<Preset> {
    Ok(state.engine.with_prompts(|p| p.duplicate(&id, &name))?)
}

#[tauri::command]
pub fn export_presets(state: State<'_, AppState>, ids: Vec<String>, path: String) -> CmdResult<()> {
    let json = state.engine.with_prompts(|p| p.export(&ids))?;
    std::fs::write(&path, json).map_err(|e| AppError::new(id::FILE, e))
}

#[tauri::command]
pub fn import_presets(state: State<'_, AppState>, path: String) -> CmdResult<Vec<Preset>> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::new(id::FILE, e))?;
    Ok(state.engine.with_prompts(|p| p.import(&json))?)
}
