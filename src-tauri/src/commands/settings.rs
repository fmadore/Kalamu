//! App information, settings and the model catalogue.

use kalamu_catalog::Catalog;
use tauri::State;

use super::{AppInfo, AppState, INTERRUPTED};
use crate::errors::{AppError, CmdResult, id};
use crate::settings::Settings;

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        data_dir: state.paths.data.display().to_string(),
        logs_dir: state.paths.logs.display().to_string(),
        interrupted: INTERRUPTED.lock().map(|v| v.clone()).unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().map(|s| s.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<()> {
    settings
        .save(&state.paths.settings)
        .map_err(|e| AppError::new(id::SETTINGS, e))?;
    *state.settings.lock().unwrap_or_else(|p| p.into_inner()) = settings;
    Ok(())
}

#[tauri::command]
pub fn get_catalog(state: State<'_, AppState>) -> Catalog {
    state.engine.catalog()
}

/// Downloads the catalogue from the repository and keeps it when it is newer than the one in use.
#[tauri::command]
pub async fn refresh_catalog(state: State<'_, AppState>) -> CmdResult<Catalog> {
    let response = state
        .http
        .get(kalamu_catalog::UPDATE_URL)
        .send()
        .await
        .map_err(|e| AppError::new(id::CATALOG_DOWNLOAD, e))?;
    if !response.status().is_success() {
        return Err(AppError::new(id::CATALOG_DOWNLOAD, response.status()));
    }
    let text = response
        .text()
        .await
        .map_err(|e| AppError::new(id::CATALOG_DOWNLOAD, e))?;
    Catalog::from_json(&text)?;
    let current = state.engine.catalog();
    let newest = Catalog::newest(Some(&text));
    if newest.updated > current.updated {
        let _ = std::fs::write(&state.paths.catalog_cache, &text);
        state.engine.set_catalog(newest.clone());
        return Ok(newest);
    }
    Ok(current)
}
