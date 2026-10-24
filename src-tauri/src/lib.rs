//! The Tauri shell: wires the engine to Windows (Credential Manager, the app data folder, the
//! window and taskbar) and exposes it to the interface through typed commands and events.

mod command_names;
mod commands;
mod errors;
mod secrets;
mod settings;
mod tray;

use std::sync::{Arc, Mutex};

use scriptorium_catalog::Catalog;
use scriptorium_engine::{Engine, EngineConfig};
use scriptorium_prompts::PromptLibrary;
use scriptorium_store::SqliteStore;
use tauri::{Emitter, Manager};
use tracing_subscriber::prelude::*;

use crate::commands::{AppState, Paths};
use crate::secrets::CredentialKeys;
use crate::settings::Settings;

/// Name of the event carrying [`scriptorium_core::ProgressEvent`]s.
pub const PROGRESS_EVENT: &str = "progress";
/// Sent when the window is closed while jobs run, so the interface can ask first.
pub const CLOSE_REQUESTED_EVENT: &str = "close-requested";

fn init_logging(dir: &std::path::Path) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "scriptorium=info,scriptorium_lib=info,scriptorium_engine=info,warn".into()
    });
    // Release builds have no console, and long batch jobs need a record: log to a daily file.
    let appender = tracing_appender::rolling::daily(dir, "scriptorium.log");
    let (file, guard) = tracing_appender::non_blocking(appender);
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init();
    Some(guard)
}

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Also when the window was hidden in the tray.
            commands::window::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data = app.path().app_local_data_dir()?;
            let logs = app.path().app_log_dir()?;
            std::fs::create_dir_all(&data)?;
            std::fs::create_dir_all(&logs)?;
            if let Some(guard) = init_logging(&logs) {
                // Kept for the life of the process so buffered log lines are flushed.
                app.manage(Mutex::new(Some(guard)));
            }

            let paths = Paths {
                settings: data.join("settings.json"),
                catalog_cache: data.join("catalog.json"),
                logs,
                data: data.clone(),
            };
            let settings = Settings::load(&paths.settings);
            let store = SqliteStore::open(&data.join("jobs.sqlite"))?;
            let prompts = PromptLibrary::open(&data.join("presets"))?;
            let cached = std::fs::read_to_string(&paths.catalog_cache).ok();
            let catalog = Catalog::newest(cached.as_deref());
            let http = scriptorium_providers::client();
            let keys = Arc::new(CredentialKeys::default());
            let models = scriptorium_offline::ModelManager::new(data.join("whisper-models"));
            let mut providers = scriptorium_providers::registry(http.clone());
            providers.push(Arc::new(scriptorium_offline::Whisper::new(models.clone())));
            let engine = Engine::new(
                Arc::new(store),
                providers,
                catalog,
                prompts,
                keys.clone(),
                EngineConfig {
                    concurrency: settings.concurrency.clamp(1, 8) as usize,
                    app_version: env!("CARGO_PKG_VERSION").into(),
                    ..EngineConfig::default()
                },
            );
            let interrupted = engine.recover().unwrap_or_default();
            if !interrupted.is_empty() {
                tracing::info!(count = interrupted.len(), "jobs interrupted at last close");
            }
            commands::remember_interrupted(interrupted);

            // Progress to the interface.
            let mut events = engine.subscribe();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match events.recv().await {
                        Ok(event) => {
                            let _ = handle.emit(PROGRESS_EVENT, &event);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(n, "progress events dropped");
                        }
                        Err(_) => break,
                    }
                }
            });

            let refresh = settings.refresh_catalog;
            app.manage(AppState {
                engine,
                models,
                keys,
                settings: Mutex::new(settings),
                paths,
                http,
                preview: Mutex::new(None),
            });
            app.manage(tray::TrayMenu::default());
            // Without a tray the window can still be used; closing it just pauses jobs.
            if let Err(e) = tray::init(app.handle()) {
                tracing::warn!(error = %e, "no tray icon");
            }
            if refresh {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = handle.state::<AppState>();
                    if let Err(e) = commands::settings::refresh_catalog(state).await {
                        tracing::info!(error = %e, "catalogue not refreshed");
                    }
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let busy = state
                    .engine
                    .jobs()
                    .map(|jobs| jobs.iter().any(|j| state.engine.is_running(j.id)))
                    .unwrap_or(false);
                if busy {
                    api.prevent_close();
                    let _ = window.emit(CLOSE_REQUESTED_EVENT, ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::app_info,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_catalog,
            commands::settings::refresh_catalog,
            commands::keys::key_status,
            commands::keys::set_api_key,
            commands::keys::clear_api_key,
            commands::keys::test_api_key,
            commands::offline::offline_status,
            commands::offline::download_model,
            commands::offline::cancel_model_download,
            commands::offline::remove_model,
            commands::presets::list_presets,
            commands::presets::save_preset,
            commands::presets::delete_preset,
            commands::presets::duplicate_preset,
            commands::presets::export_presets,
            commands::presets::import_presets,
            commands::jobs::expand_paths,
            commands::jobs::inspect_files,
            commands::jobs::estimate_cost,
            commands::jobs::create_job,
            commands::jobs::start_job,
            commands::jobs::pause_job,
            commands::jobs::cancel_job,
            commands::jobs::retry_failed,
            commands::jobs::delete_job,
            commands::jobs::list_jobs,
            commands::jobs::get_job,
            commands::jobs::job_pieces,
            commands::review::source_pieces,
            commands::review::piece_preview,
            commands::review::save_correction,
            commands::review::export_source,
            commands::review::compare_source,
            commands::review::diff_texts,
            commands::window::set_taskbar_progress,
            commands::window::set_tray_state,
            commands::window::hide_window,
            commands::window::quit_app,
        ])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        tracing::error!(error = %e, "the app could not start");
    }
}

#[cfg(test)]
mod tests {
    use super::command_names::COMMANDS;

    /// The handler, the command list and the capability must agree: a command missing from
    /// any of them is a rejected call at run time.
    #[test]
    fn every_command_is_registered_listed_and_granted() {
        let lib = include_str!("lib.rs");
        let capability = include_str!("../capabilities/main.json");
        for name in COMMANDS {
            assert!(
                lib.lines().any(|l| l.trim().starts_with("commands::")
                    && l.trim().ends_with(&format!("::{name},"))),
                "{name} is not in generate_handler!"
            );
            let permission = format!("\"allow-{}\"", name.replace('_', "-"));
            assert!(capability.contains(&permission), "{name} is not granted");
        }
        let registered = lib
            .lines()
            .filter(|l| {
                let l = l.trim();
                l.starts_with("commands::") && l.ends_with(',') && !l.contains('(')
            })
            .count();
        assert_eq!(
            registered,
            COMMANDS.len(),
            "generate_handler! has unlisted commands"
        );
    }
}
