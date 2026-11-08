//! App information, settings and the model catalogue.

use kalamu_catalog::Catalog;
use kalamu_core::ProviderId;
use kalamu_engine::TaskRules;
use tauri::State;

use super::{AppInfo, AppState, INTERRUPTED};
use crate::errors::{AppError, CmdResult, id};
use crate::platform;
use crate::settings::Settings;

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        data_dir: platform::shown_path(&state.paths.data)
            .display()
            .to_string(),
        logs_dir: platform::shown_path(&state.paths.logs)
            .display()
            .to_string(),
        interrupted: INTERRUPTED.lock().map(|v| v.clone()).unwrap_or_default(),
        // Asked of the system rather than the webview: WebKit gives an app only the languages
        // its bundle declares.
        system_languages: sys_locale::get_locales().collect(),
        app_store: platform::access().sandboxed,
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().map(|s| s.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<()> {
    let mut settings = settings;
    // Agreeing to a provider is never undone by saving a copy of the settings read before it.
    {
        let kept = state.settings.lock().unwrap_or_else(|p| p.into_inner());
        for provider in &kept.ai_consent {
            if !settings.ai_consent.contains(provider) {
                settings.ai_consent.push(provider.clone());
            }
        }
    }
    // The results folder, just picked, is needed again at later launches.
    let folders = [
        settings.last_folder.as_deref(),
        destination_folder(&settings.destination),
    ];
    state.grants.keep(folders.into_iter().flatten());
    settings
        .save(&state.paths.settings)
        .map_err(|e| AppError::new(id::SETTINGS, e))?;
    state
        .llama
        .use_graphics_processor(settings.ocr_graphics_processor);
    state
        .whisper
        .use_graphics_processor(settings.ocr_graphics_processor);
    *state.settings.lock().unwrap_or_else(|p| p.into_inner()) = settings;
    Ok(())
}

#[tauri::command]
pub fn get_catalog(state: State<'_, AppState>) -> Catalog {
    state.engine.catalog()
}

/// Which sources, models and formats suit each task, against the catalogue in use; fetched
/// again after the catalogue is refreshed.
#[tauri::command]
pub fn task_rules(state: State<'_, AppState>) -> Vec<TaskRules> {
    state.engine.task_rules()
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

/// The folder a destination names, if it names one.
pub(crate) fn destination_folder(destination: &kalamu_core::OutputDestination) -> Option<&str> {
    match destination {
        kalamu_core::OutputDestination::Folder(folder) => Some(folder),
        kalamu_core::OutputDestination::BesideSources => None,
    }
}

/// The cloud providers among these the researcher has not yet agreed to send their files to.
/// Offline engines are never asked about: nothing leaves the computer.
#[tauri::command]
pub fn providers_to_consent(
    state: State<'_, AppState>,
    providers: Vec<ProviderId>,
) -> Vec<ProviderId> {
    super::unconsented(&state, providers)
}

/// Records that the researcher agreed to send their files to these providers.
#[tauri::command]
pub fn consent_providers(state: State<'_, AppState>, providers: Vec<ProviderId>) -> CmdResult<()> {
    let mut settings = state.settings.lock().unwrap_or_else(|p| p.into_inner());
    let mut changed = false;
    for provider in providers {
        if !settings.ai_consent.contains(&provider) {
            settings.ai_consent.push(provider);
            changed = true;
        }
    }
    if changed {
        settings
            .save(&state.paths.settings)
            .map_err(|e| AppError::new(id::SETTINGS, e))?;
    }
    Ok(())
}
