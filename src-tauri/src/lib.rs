//! The Tauri shell: wires the engine to Windows (Credential Manager, the app data folder, the
//! window and taskbar) and exposes it to the interface through typed commands and events.

mod command_names;
mod commands;
mod errors;
mod secrets;
mod settings;

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
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
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
            let engine = Engine::new(
                Arc::new(store),
                scriptorium_providers::registry(http.clone()),
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
                keys,
                settings: Mutex::new(settings),
                paths,
                http,
                preview: Mutex::new(None),
            });
            if refresh {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = handle.state::<AppState>();
                    if let Err(e) = commands::refresh_catalog(state).await {
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
            commands::app_info,
            commands::get_settings,
            commands::save_settings,
            commands::get_catalog,
            commands::refresh_catalog,
            commands::key_status,
            commands::set_api_key,
            commands::clear_api_key,
            commands::test_api_key,
            commands::list_presets,
            commands::save_preset,
            commands::delete_preset,
            commands::duplicate_preset,
            commands::export_presets,
            commands::import_presets,
            commands::expand_paths,
            commands::inspect_files,
            commands::estimate_cost,
            commands::create_job,
            commands::start_job,
            commands::pause_job,
            commands::cancel_job,
            commands::retry_failed,
            commands::delete_job,
            commands::list_jobs,
            commands::get_job,
            commands::job_pieces,
            commands::source_pieces,
            commands::piece_preview,
            commands::save_correction,
            commands::export_source,
            commands::compare_source,
            commands::diff_texts,
            commands::set_taskbar_progress,
            commands::quit_app,
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
                lib.contains(&format!("commands::{name},")),
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
