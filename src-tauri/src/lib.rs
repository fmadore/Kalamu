//! The Tauri shell: wires the engine to Windows (Credential Manager, the app data folder, the
//! window and taskbar) and exposes it to the interface through typed commands and events.

mod command_names;
mod commands;
mod errors;
mod secrets;
mod settings;
mod tray;

use std::sync::{Arc, Mutex};

use kalamu_catalog::Catalog;
use kalamu_engine::{Engine, EngineConfig};
use kalamu_prompts::PromptLibrary;
use kalamu_store::SqliteStore;
use tauri::{Emitter, Manager};
use tracing_subscriber::prelude::*;

use crate::commands::{AppState, Paths};
use crate::secrets::CredentialKeys;
use crate::settings::Settings;

/// Name of the event carrying [`kalamu_core::ProgressEvent`]s.
pub const PROGRESS_EVENT: &str = "progress";
/// Sent when the window is closed while jobs run, so the interface can ask first.
pub const CLOSE_REQUESTED_EVENT: &str = "close-requested";

/// The app's data folder in `%LOCALAPPDATA%`: Tauri names it after the bundle identifier in
/// `tauri.conf.json`. Not `Kalamu`, which the per-user installer takes for the program itself.
const DATA_FOLDER: &str = "io.github.fmadore.kalamu";
/// The data folder's name before the app was renamed from Scriptorium (0.1 and 0.2).
const LEGACY_IDENTIFIER: &str = "io.github.fmadore.scriptorium";

/// Moves the jobs, settings, presets and Whisper models of a Scriptorium install to the
/// renamed data folder, once, before anything is created there. Every stored path is relative
/// to the folder, so it moves whole. If the move fails the old folder is left untouched.
///
/// Runs before Tauri builds the window, since WebView2 creates its data in the new folder as
/// soon as the window exists.
fn move_legacy_data(data: &std::path::Path) -> Result<bool, std::io::Error> {
    let Some(legacy) = data.parent().map(|p| p.join(LEGACY_IDENTIFIER)) else {
        return Ok(false);
    };
    if data.exists() || !legacy.is_dir() {
        return Ok(false);
    }
    std::fs::rename(&legacy, data)?;
    Ok(true)
}

fn init_logging(dir: &std::path::Path) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "kalamu=info,kalamu_lib=info,kalamu_engine=info,warn".into());
    // Release builds have no console, and long batch jobs need a record: log to a daily file.
    let appender = tracing_appender::rolling::daily(dir, "kalamu.log");
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
    record_panics(dir);
    Some(guard)
}

/// Writes a panic to `panic.log` before the process ends. A panic in a command aborts the app
/// (it cannot unwind through WebView2), and the daily log is written from a background thread
/// that may not get to it.
fn record_panics(dir: &std::path::Path) {
    let path = dir.join("panic.log");
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;
        let thread = std::thread::current();
        let line = format!(
            "{} panic in thread '{}': {info}
{}
",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_default(),
            thread.name().unwrap_or("unnamed"),
            std::backtrace::Backtrace::force_capture(),
        );
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(line.as_bytes());
        }
        tracing::error!("{info}");
        previous(info);
    }));
}

pub fn run() {
    let moved = std::env::var_os("LOCALAPPDATA").map_or(Ok(false), |local| {
        move_legacy_data(&std::path::Path::new(&local).join(DATA_FOLDER))
    });
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Also when the window was hidden in the tray.
            commands::window::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let data = app.path().app_local_data_dir()?;
            let logs = app.path().app_log_dir()?;
            std::fs::create_dir_all(&data)?;
            std::fs::create_dir_all(&logs)?;
            if let Some(guard) = init_logging(&logs) {
                // Kept for the life of the process so buffered log lines are flushed.
                app.manage(Mutex::new(Some(guard)));
            }
            match moved {
                Ok(true) => {
                    tracing::info!("moved the Scriptorium data folder to {}", data.display())
                }
                Ok(false) => {}
                Err(e) => tracing::warn!(error = %e, "could not move the Scriptorium data folder"),
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
            let pipelines = kalamu_prompts::PipelineLibrary::open(&data.join("pipelines"))?;
            let cached = std::fs::read_to_string(&paths.catalog_cache).ok();
            let catalog = Catalog::newest(cached.as_deref());
            let http = kalamu_providers::client();
            let keys = Arc::new(CredentialKeys::default());
            let models = kalamu_offline::ModelManager::new(data.join("whisper-models"));
            // The bundled FFmpeg sits next to the executable (Tauri's sidecar layout).
            let app_dir: Vec<std::path::PathBuf> = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
                .into_iter()
                .collect();
            let ffmpeg = kalamu_media::Ffmpeg::locate(&app_dir);
            if ffmpeg.is_none() {
                tracing::warn!(
                    "FFmpeg not found: recordings are sent whole and video is unavailable"
                );
            }
            let mut providers = kalamu_providers::registry(http.clone());
            providers.push(Arc::new(kalamu_offline::Whisper::new(models.clone())));
            let engine = Engine::new(
                Arc::new(store),
                providers,
                catalog,
                prompts,
                keys.clone(),
                EngineConfig {
                    concurrency: settings.concurrency.clamp(1, 8) as usize,
                    app_version: env!("CARGO_PKG_VERSION").into(),
                    ffmpeg,
                    listening_cache: data.join("listening"),
                    ..EngineConfig::default()
                },
            );
            let mut interrupted = engine.recover().unwrap_or_default();
            // Jobs whose batches the provider still holds carry on by themselves, below; the
            // others wait for the researcher to resume them.
            let waiting = engine.jobs_waiting_for_batches().unwrap_or_default();
            interrupted.retain(|id| !waiting.contains(id));
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

            // Collect the answers of batches sent before the app was last closed.
            let resumed = engine.clone();
            tauri::async_runtime::spawn(async move {
                for id in waiting {
                    if let Err(e) = resumed.start(id) {
                        tracing::warn!(job = %id, error = %e, "batch job not resumed");
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
                pipelines: Mutex::new(pipelines),
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
            commands::pipelines::list_pipelines,
            commands::pipelines::save_pipeline,
            commands::pipelines::delete_pipeline,
            commands::pipelines::export_pipeline,
            commands::pipelines::import_pipeline,
            commands::jobs::expand_paths,
            commands::jobs::inspect_files,
            commands::jobs::inspect_links,
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
            commands::jobs::job_text_outputs,
            commands::jobs::job_outputs,
            commands::jobs::open_output,
            commands::jobs::rename_job,
            commands::review::source_pieces,
            commands::review::piece_preview,
            commands::review::piece_audio,
            commands::review::source_text,
            commands::review::save_correction,
            commands::review::save_speaker_names,
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::command_names::COMMANDS;
    use super::{Catalog, DATA_FOLDER, LEGACY_IDENTIFIER, move_legacy_data};

    /// The folder moved into before Tauri starts must be the one Tauri then opens.
    #[test]
    fn the_data_folder_is_the_bundle_identifier() {
        let config = include_str!("../tauri.conf.json");
        assert!(config.contains(&format!("\"identifier\": \"{DATA_FOLDER}\"")));
    }

    #[test]
    fn a_scriptorium_data_folder_moves_once() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(LEGACY_IDENTIFIER);
        let data = root.path().join(DATA_FOLDER);
        std::fs::create_dir_all(legacy.join("presets")).unwrap();
        std::fs::write(legacy.join("jobs.sqlite"), b"jobs").unwrap();

        assert!(move_legacy_data(&data).unwrap());
        assert_eq!(std::fs::read(data.join("jobs.sqlite")).unwrap(), b"jobs");
        assert!(data.join("presets").is_dir());
        assert!(!legacy.exists());

        // A later Scriptorium folder never replaces the Kalamu one.
        std::fs::create_dir_all(&legacy).unwrap();
        assert!(!move_legacy_data(&data).unwrap());
        assert!(legacy.exists());
    }

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

    /// The opener opens only the addresses its permission lists, and `opener:allow-open-url`
    /// alone lists none: every link in the built app did nothing. The app opens only https
    /// pages (videos, where keys are made, credits, support).
    #[test]
    fn web_pages_are_allowed_to_open() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let allowed: Vec<&str> = capability["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["identifier"] == "opener:allow-open-url")
            .flat_map(|p| p["allow"].as_array().into_iter().flatten())
            .filter_map(|entry| entry["url"].as_str())
            .collect();
        assert_eq!(allowed, ["https://*"]);
        // The addresses that come from data rather than code.
        for provider in Catalog::bundled().providers {
            let url = &provider.key_url;
            assert!(url.is_empty() || url.starts_with("https://"), "{url}");
        }
    }
}
