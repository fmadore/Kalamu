//! App information, settings and the model catalogue.

use kalamu_catalog::{Catalog, CatalogError, Release};
use kalamu_core::{ProviderId, PrunedResults, UnusedResults};
use kalamu_engine::{EngineError, TaskRules};
use tauri::State;

use super::{AppInfo, AppState, INTERRUPTED, SETTINGS_SET_ASIDE};
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
        settings_set_aside: SETTINGS_SET_ASIDE
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default()
            .map(|p| platform::shown_path(&p).display().to_string()),
    }
}

/// A release newer than this copy, as the catalogue in use names it. Never for a copy a store
/// keeps up to date (the Microsoft Store's, the Mac App Store's), which must not offer one.
#[tauri::command]
pub fn newer_release(state: State<'_, AppState>) -> Option<Release> {
    if platform::updated_by_store() {
        return None;
    }
    state
        .engine
        .catalog()
        .newer_release(env!("CARGO_PKG_VERSION"))
}

/// What [`clear_unused_results`] would remove: how many stored answers no job refers to any
/// more, and about how much space they take, so Settings can say "nothing to remove" without
/// asking, or say what it will remove before asking. Only reads, so allowed while jobs run.
#[tauri::command]
pub async fn unused_results(state: State<'_, AppState>) -> CmdResult<UnusedResults> {
    let cache = state.result_cache.clone();
    super::off_thread(move || Ok(cache.unused_results().map_err(EngineError::from)?)).await
}

/// Deletes the stored answers no job refers to any more, those of deleted jobs above all, and
/// gives their space back to the disk. Refused while a job runs: the database is held while it
/// is compacted, which on a large one takes a while.
#[tauri::command]
pub async fn clear_unused_results(state: State<'_, AppState>) -> CmdResult<PrunedResults> {
    if super::window::jobs_running(&state) {
        return Err(AppError::new(id::JOBS_RUNNING, "clear unused results"));
    }
    let engine = state.engine.clone();
    let pruned = super::off_thread(move || Ok(engine.prune_results()?)).await?;
    tracing::info!(
        results = pruned.results,
        bytes = pruned.bytes,
        "unused results cleared"
    );
    Ok(pruned)
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

/// Whether the last model list downloaded, or the one kept from before, failed its checks and
/// was set aside: Settings says so under its list's date until a list passes.
static CATALOG_REFUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn catalog_refused_now(refused: bool) {
    CATALOG_REFUSED.store(refused, std::sync::atomic::Ordering::Relaxed);
}

/// The list to start with: the one kept from the last download when it passes its checks and
/// is newer than the bundled one, else the bundled one. A kept list that fails its checks is
/// deleted, so it is not read again at every launch.
pub(crate) fn cached_catalog(cache: &std::path::Path) -> Catalog {
    let Ok(text) = std::fs::read_to_string(cache) else {
        return Catalog::bundled();
    };
    if let Err(e) = Catalog::from_json(&text) {
        tracing::warn!(error = %e, "the model list kept from before is set aside");
        let _ = std::fs::remove_file(cache);
        catalog_refused_now(!matches!(e, CatalogError::TooNew));
    }
    Catalog::newest(Some(&text))
}

/// Whether a model list was set aside for failing its checks (see [`CATALOG_REFUSED`]).
#[tauri::command]
pub fn catalog_refused() -> bool {
    CATALOG_REFUSED.load(std::sync::atomic::Ordering::Relaxed)
}

/// Downloads the catalogue from the repository and keeps it when its revision is higher than
/// the one in use. A list larger than [`kalamu_catalog::MAX_BYTES`] is not downloaded further,
/// and one that fails its checks is dropped: the list in use stays, and Settings says so.
#[tauri::command]
pub async fn refresh_catalog(state: State<'_, AppState>) -> CmdResult<Catalog> {
    let download = |e: &dyn std::fmt::Display| AppError::new(id::CATALOG_DOWNLOAD, e);
    let too_large = || {
        catalog_refused_now(true);
        let detail = format!("larger than {} bytes", kalamu_catalog::MAX_BYTES);
        tracing::warn!(detail, "the downloaded model list is set aside");
        AppError::new(id::CATALOG_REFUSED, detail)
    };
    let mut response = state
        .http
        .get(kalamu_catalog::UPDATE_URL)
        .send()
        .await
        .map_err(|e| download(&e))?;
    if !response.status().is_success() {
        return Err(download(&response.status()));
    }
    let limit = kalamu_catalog::MAX_BYTES;
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(too_large());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| download(&e))? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            return Err(too_large());
        }
    }
    let text = String::from_utf8(body)
        .map_err(|e| AppError::from(CatalogError::Invalid(e.to_string())))?;
    let downloaded = match Catalog::from_json(&text) {
        Ok(downloaded) => downloaded,
        // A list for a newer version of Kalamu is not a bad one: nothing to say in Settings.
        Err(e @ CatalogError::TooNew) => return Err(e.into()),
        Err(e) => {
            tracing::warn!(error = %e, "the downloaded model list is set aside");
            catalog_refused_now(true);
            return Err(e.into());
        }
    };
    catalog_refused_now(false);
    let current = state.engine.catalog();
    if downloaded.is_newer_than(&current) {
        let _ = std::fs::write(&state.paths.catalog_cache, &text);
        state.engine.set_catalog(downloaded.clone());
        return Ok(downloaded);
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
