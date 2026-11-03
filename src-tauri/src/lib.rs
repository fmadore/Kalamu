//! The Tauri shell: wires the engine to the operating system (its keychain, the app data
//! folder, the window, taskbar or Dock) and exposes it to the interface through typed commands
//! and events. What differs between Windows and macOS is in [`platform`].

mod command_names;
mod commands;
mod errors;
mod platform;
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
/// Name of the event carrying [`kalamu_search::SearchProgress`]: how far a project's text is
/// prepared for searching.
pub const SEARCH_PROGRESS_EVENT: &str = "search-progress";

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
/// (it cannot unwind through the webview), and the daily log is written from a background thread
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

/// Ties a workflow step's job to the documents of the project its run was started from.
fn link_step(
    engine: &Engine,
    projects: &kalamu_projects::Projects,
    run: kalamu_core::RunId,
    job: kalamu_core::JobId,
) {
    let Ok(view) = engine.run_view(run) else {
        return;
    };
    if let Some(project) = view.run.project
        && let Err(e) = projects.link_job(project, job, view.run.collection)
    {
        tracing::warn!(run = %run, job = %job, error = %e, "workflow step not tied to its project");
    }
}

pub fn run() {
    // The data folder is settled, and an earlier one moved into it, before Tauri builds the
    // window, whose webview data goes there too.
    let (folders, moved) = platform::data_folder();
    let mut context = tauri::generate_context!();
    context.config_mut().app.app_directories_override = Some(folders);
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Also when the window was hidden in the tray.
            commands::window::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            platform::init(app)?;
            let data = app.path().app_local_data_dir()?;
            let logs = app.path().app_log_dir()?;
            std::fs::create_dir_all(&data)?;
            std::fs::create_dir_all(&logs)?;
            if let Some(guard) = init_logging(&logs) {
                // Kept for the life of the process so buffered log lines are flushed.
                app.manage(Mutex::new(Some(guard)));
            }
            match moved {
                platform::Moved::Nothing => {}
                platform::Moved::From(old) => {
                    tracing::info!(
                        "moved the data folder {} to {}",
                        old.display(),
                        data.display()
                    )
                }
                platform::Moved::Failed(old, e) => tracing::warn!(
                    error = %e,
                    "could not move the data folder {}, which stays in use",
                    old.display()
                ),
            }

            let paths = Paths {
                settings: data.join("settings.json"),
                catalog_cache: data.join("catalog.json"),
                logs,
                data: data.clone(),
            };
            let settings = Settings::load(&paths.settings);
            let store = Arc::new(SqliteStore::open(&data.join("jobs.sqlite"))?);
            // Projects share the job database, so deleting a job unlinks it from its documents.
            let projects = kalamu_projects::Projects::new(store.clone(), store.clone());
            let dictionary =
                kalamu_dictionary::Dictionary::new(store.clone(), store.clone(), store.clone());
            // The search index is a file of its own, made again from the job database when it is
            // lost or cannot be read; it holds nothing the database does not.
            let index: Arc<dyn kalamu_core::SearchIndex> =
                match kalamu_store::SqliteIndex::open(&data.join("search.sqlite")) {
                    Ok(index) => Arc::new(index),
                    Err(e) => {
                        tracing::warn!(error = %e, "search index kept in memory only");
                        Arc::new(kalamu_store::SqliteIndex::in_memory()?)
                    }
                };
            let search = Arc::new(kalamu_search::Search::new(
                store.clone(),
                store.clone(),
                index,
            ));
            // Passages of jobs deleted while the index could not be told.
            let pruning = search.clone();
            std::thread::spawn(move || match pruning.prune() {
                Ok(0) => {}
                Ok(n) => tracing::info!(jobs = n, "deleted jobs removed from the search index"),
                Err(e) => tracing::warn!(error = %e, "search index not pruned"),
            });
            let prompts = PromptLibrary::open(&data.join("presets"))?;
            let pipelines = kalamu_prompts::PipelineLibrary::open(&data.join("pipelines"))?;
            let workflows = kalamu_prompts::WorkflowLibrary::open(&data.join("workflows"))?;
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
                store.clone(),
                providers,
                catalog,
                prompts,
                keys.clone(),
                EngineConfig {
                    concurrency: settings.concurrency.clamp(1, 8) as usize,
                    app_version: env!("CARGO_PKG_VERSION").into(),
                    ffmpeg,
                    listening_cache: data.join("listening"),
                    playback: platform::PLAYBACK,
                    snapshots: data.join("snapshots"),
                    ..EngineConfig::default()
                },
            );
            let interrupted = engine.recover().unwrap_or_default();
            // Batches the provider still holds are collected below; nothing else is sent until
            // the researcher resumes the job.
            let waiting = engine.jobs_waiting_for_batches().unwrap_or_default();
            if !interrupted.is_empty() {
                tracing::info!(count = interrupted.len(), "jobs interrupted at last close");
            }
            commands::remember_interrupted(interrupted);
            // Progress to the interface. A workflow step started from a project is tied to its
            // documents as a job started from the project is.
            let mut events = engine.subscribe();
            let handle = app.handle().clone();
            let linker = kalamu_projects::Projects::new(store.clone(), store.clone());
            let runs = engine.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    match events.recv().await {
                        Ok(event) => {
                            if let kalamu_core::ProgressEvent::StepStarted {
                                run_id, job_id, ..
                            } = &event
                            {
                                link_step(&runs, &linker, *run_id, *job_id);
                            }
                            let _ = handle.emit(PROGRESS_EVENT, &event);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!(n, "progress events dropped");
                        }
                        Err(_) => break,
                    }
                }
            });

            // Workflow steps whose input became ready as the app closed: their jobs are made,
            // and wait for the researcher like any interrupted job. After subscribing, so a
            // step of a project's run is tied to its documents as any other.
            if let Err(e) = engine.resume_runs() {
                tracing::warn!(error = %e, "workflow runs not resumed");
            }

            // Collect the answers of batches sent before the app was last closed.
            let resumed = engine.clone();
            tauri::async_runtime::spawn(async move {
                for id in waiting {
                    if let Err(e) = resumed.collect_waiting(id) {
                        tracing::warn!(job = %id, error = %e, "batches not collected");
                    }
                }
            });

            let refresh = settings.refresh_catalog;
            app.manage(AppState {
                engine,
                projects,
                dictionary,
                search,
                search_generation: Arc::default(),
                search_preparation: Arc::default(),
                models,
                keys,
                settings: Mutex::new(settings),
                paths,
                http,
                preview: Mutex::new(None),
                pipelines: Mutex::new(pipelines),
                workflows: Mutex::new(workflows),
                quitting: std::sync::atomic::AtomicBool::new(false),
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
                // While jobs run the interface asks first; otherwise the window closes, or on
                // macOS hides with the app left in the Dock.
                if commands::window::jobs_running(&window.state::<AppState>()) {
                    api.prevent_close();
                    let _ = window.emit(CLOSE_REQUESTED_EVENT, ());
                } else if platform::CLOSE_HIDES_WINDOW {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::app_info,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_catalog,
            commands::settings::refresh_catalog,
            commands::settings::task_rules,
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
            commands::workflows::list_workflows,
            commands::workflows::save_workflow,
            commands::workflows::duplicate_workflow,
            commands::workflows::delete_workflow,
            commands::workflows::export_workflow,
            commands::workflows::import_workflow,
            commands::workflows::start_run,
            commands::workflows::list_runs,
            commands::workflows::get_run,
            commands::workflows::job_step,
            commands::workflows::job_step,
            commands::workflows::approve_step,
            commands::workflows::cancel_run,
            commands::workflows::delete_run,
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
            commands::jobs::snapshot_job,
            commands::jobs::derived_state,
            commands::units::units_view,
            commands::units::seed_units,
            commands::units::save_unit,
            commands::units::delete_unit,
            commands::units::join_units,
            commands::units::split_unit,
            commands::units::unit_sources,
            commands::units::export_units,
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
            commands::review::job_scores,
            commands::review::diff_texts,
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::archive_project,
            commands::projects::delete_project,
            commands::projects::project_view,
            commands::projects::create_collection,
            commands::projects::rename_collection,
            commands::projects::delete_collection,
            commands::projects::add_documents,
            commands::projects::rename_document,
            commands::projects::delete_documents,
            commands::projects::add_to_collection,
            commands::projects::remove_from_collection,
            commands::projects::tag_documents,
            commands::projects::untag_documents,
            commands::projects::describe_document,
            commands::projects::set_source_defaults,
            commands::projects::rename_tag,
            commands::projects::export_descriptions,
            commands::projects::merge_documents,
            commands::projects::document_detail,
            commands::projects::set_preferred,
            commands::projects::attach_plan,
            commands::projects::attach_job,
            commands::projects::link_job,
            commands::projects::detach_job,
            commands::projects::review_queue,
            commands::dictionary::dictionary_view,
            commands::dictionary::dictionary_evidence,
            commands::dictionary::dictionary_merge,
            commands::dictionary::dictionary_split,
            commands::dictionary::dictionary_relabel,
            commands::dictionary::dictionary_set_reviewed,
            commands::dictionary::dictionary_keep_apart,
            commands::dictionary::dictionary_undo_apart,
            commands::dictionary::export_dictionary,
            commands::dictionary::dictionary_plan,
            commands::dictionary::dictionary_propose,
            commands::dictionary::dictionary_proposals,
            commands::dictionary::dictionary_dismiss,
            commands::dictionary::dictionary_forget_request,
            commands::search::project_search,
            commands::search::rebuild_search_index,
            commands::search::prepare_project_search,
            commands::window::set_taskbar_progress,
            commands::window::set_tray_state,
            commands::window::hide_window,
            commands::window::quit_app,
        ])
        .build(context)
        .map(|app| app.run(platform::on_run_event));
    if let Err(e) = result {
        tracing::error!(error = %e, "the app could not start");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Catalog;
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
