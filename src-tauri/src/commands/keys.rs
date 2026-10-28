//! API keys: saved in Credential Manager, never sent to the interface.

use kalamu_core::ProviderId;
use kalamu_engine::KeySource;
use tauri::State;

use super::{AppState, KeyStatus};
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn key_status(state: State<'_, AppState>) -> Vec<KeyStatus> {
    state
        .engine
        .catalog()
        .providers
        .iter()
        .map(|p| KeyStatus {
            provider: p.id.clone(),
            // An offline engine needs no key, so it is always usable.
            saved: p.local || state.keys.has(&p.id),
        })
        .collect()
}

#[tauri::command]
pub async fn set_api_key(
    state: State<'_, AppState>,
    provider: ProviderId,
    key: String,
) -> CmdResult<()> {
    let keys = state.keys.clone();
    tokio::task::spawn_blocking(move || keys.set(&provider, &key))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}

#[tauri::command]
pub async fn clear_api_key(state: State<'_, AppState>, provider: ProviderId) -> CmdResult<()> {
    let keys = state.keys.clone();
    tokio::task::spawn_blocking(move || keys.clear(&provider))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}

/// Lists the provider's models with the saved key: proves the key works, and shows models the
/// catalogue does not list yet.
#[tauri::command]
pub async fn test_api_key(
    state: State<'_, AppState>,
    provider: ProviderId,
) -> CmdResult<Vec<String>> {
    let key = state
        .keys
        .key(&provider)
        .ok_or_else(|| AppError::new("provider.missing_key", &provider))?;
    let p = state.engine.provider(&provider)?;
    Ok(p.list_models(&key).await?)
}
