//! Export profiles: the columns of a descriptions export saved for reuse, and shared as
//! templates.

use kalamu_core::{ExportProfile, ProfileColumn, ProfileError, ProfileStarter};
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn list_export_profiles(state: State<'_, AppState>) -> Vec<ExportProfile> {
    state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .list()
}

#[tauri::command]
pub fn save_export_profile(
    state: State<'_, AppState>,
    profile: ExportProfile,
) -> CmdResult<ExportProfile> {
    Ok(state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .save(profile)?)
}

#[tauri::command]
pub fn duplicate_export_profile(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CmdResult<ExportProfile> {
    Ok(state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .duplicate(&id, &name)?)
}

#[tauri::command]
pub fn delete_export_profile(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .delete(&id)?)
}

/// What keeps the profile from being saved, if anything: checked as it is edited.
#[tauri::command]
pub fn check_export_profile(profile: ExportProfile) -> Option<ProfileError> {
    profile.tidied().check().err()
}

/// The columns a new profile starts with.
#[tauri::command]
pub fn profile_starter(starter: ProfileStarter) -> Vec<ProfileColumn> {
    starter.columns()
}

/// Writes the profile as a template another Kalamu imports, its fixed texts only if asked.
#[tauri::command]
pub fn share_export_profile(
    state: State<'_, AppState>,
    id: String,
    fixed_texts: bool,
    path: String,
) -> CmdResult<()> {
    let json = state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .file(&id, fixed_texts, env!("CARGO_PKG_VERSION"))?;
    super::write_text(std::path::Path::new(&path).with_extension("json"), json)
}

#[tauri::command]
pub fn import_export_profile(state: State<'_, AppState>, path: String) -> CmdResult<ExportProfile> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::new(id::FILE, e))?;
    Ok(state
        .profiles
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .import(&json)?)
}
