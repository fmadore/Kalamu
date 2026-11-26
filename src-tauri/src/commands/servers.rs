//! Servers of the researcher's own (Settings › *Other servers*): `servers.json` read and
//! written, a server found and each model tried before it is saved, and the one place the
//! engine is handed its providers and its catalogue, the servers merged in
//! ([decision](../../../docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
//!
//! What a model does comes from its trial, kept here until the server is saved
//! ([`kalamu_catalog::Trials`]); the checks and the merging are the catalogue's, the finding and
//! the trial the providers'. Only the downloaded model list is ever written to its cache: the
//! catalogue with the servers merged in is never stored as one.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use kalamu_catalog::{
    Catalog, DraftError, FoundServer, FoundServerModel, ModelTrial, OwnServer, OwnServers,
    OwnTrial, ServerDraft, TrialModel, TrialServer,
};
use kalamu_core::address::checked_address;
use kalamu_core::{ApiKey, Provider, ProviderId, Timestamp};
use kalamu_engine::KeySource;
use kalamu_providers::own::{self, ModelConfig, ServerConfig};
use tauri::{AppHandle, Emitter, State};

use super::AppState;
use crate::CATALOG_CHANGED_EVENT;
use crate::errors::{AppError, CmdResult, id};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|p| p.into_inner())
}

/// The servers `servers.json` holds, or none. A file that cannot be read or fails its checks
/// does not block the launch and is never overwritten: it is moved aside, as the settings are
/// (`servers.json.bad`), and the app starts with no servers.
pub(crate) fn load(path: &Path) -> OwnServers {
    let failure = match std::fs::read_to_string(path) {
        Ok(text) => match OwnServers::from_json(&text) {
            Ok(servers) => return servers,
            Err(e) => e.to_string(),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return OwnServers::default(),
        Err(e) => e.to_string(),
    };
    let aside = crate::settings::set_aside_name(path, |p| p.symlink_metadata().is_ok());
    match std::fs::rename(path, &aside) {
        Ok(()) => tracing::warn!(
            error = %failure,
            "your servers could not be read: none is in use, the file is kept as {}",
            aside.display()
        ),
        Err(e) => tracing::warn!(
            error = %failure,
            "your servers could not be read, nor moved aside ({e}): none is in use"
        ),
    }
    OwnServers::default()
}

/// The catalogue the engine is handed: `list`, the model list in use (bundled, kept or
/// downloaded), with the researcher's servers merged in.
pub(crate) fn catalogue(list: &Catalog, servers: &OwnServers) -> Catalog {
    list.with_servers(servers)
}

/// THE ONE PLACE the engine is handed a catalogue once it runs: `list` with the servers
/// merged in. Merging again replaces what an earlier merge added, so the catalogue in use can
/// be handed back. Returns what the engine now holds, for the interface.
pub(crate) fn use_catalog(state: &AppState, list: &Catalog) -> Catalog {
    let merged = catalogue(list, &lock(&state.servers));
    state.engine.set_catalog(merged.clone());
    merged
}

/// The providers the engine is handed: the fixed ones, then one per server.
pub(crate) fn providers(
    fixed: &[Arc<dyn Provider>],
    http: &reqwest::Client,
    servers: &OwnServers,
) -> Vec<Arc<dyn Provider>> {
    let mut all = fixed.to_vec();
    all.extend(own::providers(http.clone(), &configs(servers)));
    all
}

/// The researcher's stored entries as the providers take them.
fn configs(servers: &OwnServers) -> Vec<ServerConfig> {
    servers
        .servers
        .iter()
        .map(|server| ServerConfig {
            id: server.id.clone(),
            name: server.name.clone(),
            address: server.address.clone(),
            dialect: server.dialect,
            key: server.key,
            at_once: usize::try_from(server.at_once).unwrap_or(1).max(1),
            models: server
                .models
                .iter()
                .map(|model| ModelConfig {
                    id: model.id.clone(),
                    context: model.context,
                })
                .collect(),
        })
        .collect()
}

/// After any change to the servers: the engine is handed the providers and the catalogue
/// anew, and every open screen told, so a server added is offered at once and one removed is
/// gone.
fn hand_over(app: &AppHandle, state: &AppState) {
    let servers = lock(&state.servers).clone();
    state
        .engine
        .set_providers(providers(&state.fixed_providers, &state.http, &servers));
    use_catalog(state, &state.engine.catalog());
    let _ = app.emit(CATALOG_CHANGED_EVENT, ());
}

fn write(state: &AppState, servers: &OwnServers) -> CmdResult<()> {
    crate::settings::write_json(&state.paths.servers, servers)
        .map_err(|e| AppError::new(id::SERVERS_WRITE, e))
}

/// The key a request to a server carries: the one typed, else the one saved for the server
/// `id` names, when it has one.
async fn key_for(
    state: &AppState,
    typed: Option<String>,
    id: Option<&ProviderId>,
) -> CmdResult<Option<ApiKey>> {
    if let Some(typed) = typed
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
    {
        return Ok(Some(ApiKey::new(typed)));
    }
    let saved = id.filter(|id| lock(&state.servers).server(id).is_some_and(|s| s.key));
    let Some(id) = saved.cloned() else {
        return Ok(None);
    };
    let keys = state.keys.clone();
    super::off_thread(move || Ok(keys.key(&id))).await
}

/// Withdraws the consent given to send files to `provider`, a server whose address changed or
/// that was removed: it is asked again before anything is sent there.
fn forget_consent(state: &AppState, provider: &ProviderId) -> CmdResult<()> {
    let mut settings = lock(&state.settings);
    if settings.ai_consent.contains(provider) {
        settings.ai_consent.retain(|p| p != provider);
        settings
            .save(&state.paths.settings)
            .map_err(|e| AppError::new(id::SETTINGS, e))?;
    }
    Ok(())
}

/// The researcher's servers, in the order they were added.
#[tauri::command]
pub fn list_servers(state: State<'_, AppState>) -> Vec<OwnServer> {
    lock(&state.servers).servers.clone()
}

/// Asks the server at `address` what it is and what models it has, with the key typed, or the
/// one saved for the server `id` when it is being changed. Each model comes with the limits
/// Kalamu fills, and those already added with the researcher's.
#[tauri::command]
pub async fn find_server(
    state: State<'_, AppState>,
    address: String,
    key: Option<String>,
    id: Option<ProviderId>,
) -> CmdResult<FoundServer> {
    let address = checked_address(&address)?;
    let key = key_for(&state, key, id.as_ref()).await?;
    let found = own::discover(&state.http, &address, key.as_ref()).await?;
    let models = found
        .models
        .into_iter()
        .map(|model| {
            let (context, max_output) = own::suggested_limits(model.context);
            FoundServerModel {
                id: model.id,
                vision: model.vision,
                audio: model.audio,
                trained_context: model.context,
                elsewhere: model.elsewhere,
                size_bytes: model.size_bytes,
                digest: model.digest,
                context,
                max_output,
                added: false,
            }
        })
        .collect();
    let found = FoundServer::new(found.dialect, found.address, found.version, models);
    Ok(lock(&state.servers).found(id.as_ref(), found))
}

/// Tries `model` on `server` as the draft stands, saved or not (*Try and add*, *Try again*):
/// an answer at all, the bundled picture read, the form filled. May take minutes while the
/// server loads the model; [`cancel_server_trial`] with the same `trial` stops it. What it did
/// is kept until the server is saved, and decides what the model is offered for.
#[tauri::command]
pub async fn try_server_model(
    state: State<'_, AppState>,
    trial: String,
    server: TrialServer,
    model: TrialModel,
    key: Option<String>,
) -> CmdResult<ModelTrial> {
    let address = checked_address(&server.address)?;
    let key = if server.asks_key {
        key_for(&state, key, server.id.as_ref()).await?
    } else {
        None
    };
    let config = ServerConfig {
        // A draft not saved has no id yet; the provider tried is never handed to the engine.
        id: server
            .id
            .clone()
            .unwrap_or_else(|| ProviderId::new("own-trial")),
        name: server.name.trim().to_string(),
        address: address.clone(),
        dialect: server.dialect,
        key: key.is_some(),
        at_once: 1,
        models: Vec::new(),
    };
    let tried = ModelConfig {
        id: model.id.trim().to_string(),
        context: model.context,
    };
    let http = state.http.clone();
    let vision = model.vision;
    let running =
        tokio::spawn(
            async move { own::try_model(&http, &config, &tried, vision, key.as_ref()).await },
        );
    lock(&state.trials_running).insert(trial.clone(), running.abort_handle());
    let outcome = running.await;
    lock(&state.trials_running).remove(&trial);
    let result = match outcome {
        Ok(result) => result,
        Err(e) if e.is_cancelled() => {
            return Err(AppError::with(
                id::TRIAL_STOPPED,
                [("model", model.id.clone())],
            ));
        }
        Err(e) => return Err(AppError::new(id::INTERNAL, e)),
    };
    let tried_at = Timestamp::now().utc_date();
    let (failure, words) = match result.answers {
        Ok(()) => {
            // A model that did not answer leaves its earlier trial as it was: a server that
            // was down for a moment takes nothing from it.
            lock(&state.trials).record(
                &address,
                &model.id,
                OwnTrial {
                    reads_pages: result.reads_pages,
                    fills_forms: result.fills_forms,
                    digest: model.digest.clone(),
                    tried_at: tried_at.clone(),
                },
            );
            (None, None)
        }
        Err(failed) => (Some(failed.code), Some(failed.words)),
    };
    Ok(ModelTrial {
        reads_pages: result.reads_pages,
        fills_forms: result.fills_forms,
        failure,
        words,
        note: result.note,
        tried_at,
    })
}

/// Stops the trial `trial`; the trial's command then fails as stopped.
#[tauri::command]
pub fn cancel_server_trial(state: State<'_, AppState>, trial: String) {
    if let Some(running) = lock(&state.trials_running).remove(&trial) {
        running.abort();
    }
}

/// Saves a server: a new one under a new id, or the server of the draft's id changed. Each
/// model does what its trial showed. A key typed is saved in the system's keychain; *This
/// server asks for a key* left off removes a key saved before. A server whose address changed
/// is asked consent for again. The engine is handed the server at once, and every open screen
/// told.
#[tauri::command]
pub async fn save_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server: ServerDraft,
    key: Option<String>,
) -> CmdResult<OwnServer> {
    let _edit = state.server_edits.lock().await;
    let typed = key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
    let asks_key = server.asks_key;
    let mut next = lock(&state.servers).clone();
    let saved = {
        let trials = lock(&state.trials);
        next.apply(server, &trials)?
    };
    let provider = saved.server.id.clone();
    let had_key = saved.previous.as_ref().is_some_and(|p| p.key);
    let keys = state.keys.clone();
    match (asks_key, typed) {
        (true, Some(typed)) => {
            let provider = provider.clone();
            super::off_thread(move || keys.set(&provider, &typed)).await?;
        }
        (true, None) if had_key => {}
        (true, None) => {
            return Err(AppError::with(
                id::SERVER_KEY_MISSING,
                [("server", saved.server.name.clone())],
            ));
        }
        (false, _) if had_key => {
            let provider = provider.clone();
            super::off_thread(move || keys.clear(&provider)).await?;
        }
        (false, _) => {}
    }
    write(&state, &next)?;
    *lock(&state.servers) = next;
    if saved.address_changed() {
        forget_consent(&state, &provider)?;
    }
    hand_over(&app, &state);
    tracing::info!(server = %provider, "a server of the researcher's own saved");
    Ok(saved.server)
}

/// Removes a server, its key and the consent given to it. Its jobs and results stay; a job
/// that still names it stops at its next piece, as for any provider no longer known.
#[tauri::command]
pub async fn remove_server(
    app: AppHandle,
    state: State<'_, AppState>,
    id: ProviderId,
) -> CmdResult<()> {
    let _edit = state.server_edits.lock().await;
    let mut next = lock(&state.servers).clone();
    let Some(removed) = next.remove(&id) else {
        return Err(DraftError::NotFound { id: id.to_string() }.into());
    };
    write(&state, &next)?;
    *lock(&state.servers) = next;
    if removed.key {
        let keys = state.keys.clone();
        let provider = id.clone();
        if let Err(e) = super::off_thread(move || keys.clear(&provider)).await {
            tracing::warn!(server = %id, error = %e, "a removed server's key left in the keychain");
        }
    }
    forget_consent(&state, &id)?;
    hand_over(&app, &state);
    tracing::info!(server = %id, "a server of the researcher's own removed");
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn servers() -> OwnServers {
        OwnServers::from_json(
            r#"{"servers": [{
                "id": "own-0000abcd", "name": "Office Mac",
                "address": "http://192.168.1.20:11434", "dialect": "ollama", "at_once": 2,
                "models": [{"id": "gemma3:12b", "reads_pages": true, "context": 16384,
                            "max_output": 4096, "tried_at": "2026-11-26"}]
            }]}"#,
        )
        .unwrap()
    }

    #[test]
    fn the_stored_entries_become_what_the_providers_take() {
        assert_eq!(
            configs(&servers()),
            [ServerConfig {
                id: ProviderId::new("own-0000abcd"),
                name: "Office Mac".into(),
                address: "http://192.168.1.20:11434".into(),
                dialect: kalamu_core::Dialect::Ollama,
                key: false,
                at_once: 2,
                models: vec![ModelConfig {
                    id: "gemma3:12b".into(),
                    context: 16_384,
                }],
            }]
        );
        let all = providers(&[], &kalamu_providers::client(), &servers());
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id(), ProviderId::new("own-0000abcd"));
        assert_eq!(all[0].reach(), kalamu_core::Reach::Network);
    }

    #[test]
    fn the_catalogue_handed_over_holds_the_servers_and_merging_again_changes_nothing() {
        let merged = catalogue(&Catalog::bundled(), &servers());
        assert!(merged.provider(&ProviderId::new("own-0000abcd")).is_some());
        assert_eq!(catalogue(&merged, &servers()), merged);
        let none = catalogue(&merged, &OwnServers::default());
        assert_eq!(none, Catalog::bundled());
    }

    #[test]
    fn a_missing_file_is_no_servers_and_a_bad_one_is_kept_aside() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("servers.json");
        assert_eq!(load(&path), OwnServers::default());
        crate::settings::write_json(&path, &servers()).unwrap();
        assert_eq!(load(&path), servers());

        std::fs::write(&path, r#"{"servers": [{"id": "gemini"}]}"#).unwrap();
        assert_eq!(load(&path), OwnServers::default());
        assert!(!path.exists());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("servers.json.bad")).unwrap(),
            r#"{"servers": [{"id": "gemini"}]}"#
        );
    }
}
