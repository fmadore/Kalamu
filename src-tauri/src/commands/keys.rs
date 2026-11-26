//! API keys: saved in the system keychain, never sent to the interface.

use kalamu_core::{ApiKey, ProviderId};
use kalamu_engine::KeySource;
use tauri::State;

use super::{AppState, KeyStatus};
use crate::errors::{AppError, CmdResult, id};

/// Which providers have a key. Off the window's thread: the first time, each provider's key is
/// read from the keychain, and on a Mac the Keychain may ask the researcher first.
#[tauri::command]
pub async fn key_status(state: State<'_, AppState>) -> CmdResult<Vec<KeyStatus>> {
    let providers: Vec<(ProviderId, bool)> = state
        .engine
        .catalog()
        .providers
        .iter()
        .map(|p| {
            // An offline engine needs no key, nor a server of the researcher's own that asks
            // for none.
            let keyless = p.local || p.own.as_ref().is_some_and(|own| !own.key);
            (p.id.clone(), keyless)
        })
        .collect();
    let keys = state.keys.clone();
    super::off_thread(move || {
        Ok(providers
            .into_iter()
            .map(|(provider, keyless)| KeyStatus {
                // So it is always usable.
                saved: keyless || keys.has(&provider),
                provider,
            })
            .collect())
    })
    .await
}

/// Refuses a key set or cleared here for a server of the researcher's own: its key is saved with
/// the server (`save_server`), so that the server's entry and the keychain always agree.
fn not_a_server(provider: &ProviderId) -> CmdResult<()> {
    if kalamu_catalog::is_own_id(provider.as_str()) {
        return Err(AppError::new(id::SERVER_KEY_ELSEWHERE, provider));
    }
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(
    state: State<'_, AppState>,
    provider: ProviderId,
    key: String,
) -> CmdResult<()> {
    not_a_server(&provider)?;
    let keys = state.keys.clone();
    tokio::task::spawn_blocking(move || keys.set(&provider, &key))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}

#[tauri::command]
pub async fn clear_api_key(state: State<'_, AppState>, provider: ProviderId) -> CmdResult<()> {
    not_a_server(&provider)?;
    let keys = state.keys.clone();
    tokio::task::spawn_blocking(move || keys.clear(&provider))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}

/// Lists the provider's models with the saved key: proves the key works, and shows models the
/// catalogue does not list yet. A server of the researcher's own that asks for no key is asked
/// without one.
#[tauri::command]
pub async fn test_api_key(
    state: State<'_, AppState>,
    provider: ProviderId,
) -> CmdResult<Vec<String>> {
    let p = state.engine.provider(&provider)?;
    let key = if !p.needs_key() && p.server().is_some() {
        ApiKey::new("")
    } else {
        state
            .keys
            .key(&provider)
            .ok_or_else(|| AppError::new("provider.missing_key", &provider))?
    };
    Ok(p.list_models(&key).await?)
}
