//! What the app does once at launch, before the window shows: the data folder and the logs, the
//! state every command shares, progress forwarded to the interface, and the work left from the
//! last close picked up again.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kalamu_catalog::Catalog;
use kalamu_core::JobId;
use kalamu_engine::{Engine, EngineConfig};
use kalamu_prompts::PromptLibrary;
use kalamu_store::SqliteStore;
use tauri::{Emitter, Manager};
use tracing_subscriber::prelude::*;

use crate::commands::{self, AppState, Paths};
use crate::secrets::CredentialKeys;
use crate::settings::Settings;
use crate::{PROGRESS_EVENT, grants, platform, tray};

type SetupResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Tauri's setup: everything the app needs before its first command.
pub(crate) fn setup(app: &mut tauri::App, moved: platform::Moved) -> SetupResult<()> {
    let (data, logs) = folders(app, moved)?;
    let (state, store, waiting) = assemble(data, logs)?;
    forward_progress(app, &state.engine, &store);
    resume(&state.engine, waiting);
    let refresh = state
        .settings
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .refresh_catalog;
    app.manage(state);
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
}

/// The data and log folders, made if needed, with logging started and the move of an earlier
/// data folder recorded.
fn folders(app: &mut tauri::App, moved: platform::Moved) -> SetupResult<(PathBuf, PathBuf)> {
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
    Ok((data, logs))
}

/// The state the commands share, the job database it opened, and the jobs whose batches the
/// providers still hold, to collect.
fn assemble(data: PathBuf, logs: PathBuf) -> SetupResult<(AppState, Arc<SqliteStore>, Vec<JobId>)> {
    let paths = Paths {
        settings: data.join("settings.json"),
        catalog_cache: data.join("catalog.json"),
        logs,
        data: data.clone(),
    };
    let settings = Settings::load(&paths.settings);
    // Folders and files granted at earlier launches, opened again before the engine
    // resumes any job or workflow run that reads or writes there.
    let grants = grants::Grants::open(data.join("grants.json"), platform::access());
    let open = grants.restore();
    if open > 0 {
        tracing::info!(count = open, "granted folders and files opened again");
    }
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
    let profiles = kalamu_prompts::ProfileLibrary::open(&data.join("export-profiles"))?;
    let cached = std::fs::read_to_string(&paths.catalog_cache).ok();
    let catalog = Catalog::newest(cached.as_deref());
    let http = kalamu_providers::client();
    let keys = Arc::new(CredentialKeys::default());
    let models =
        kalamu_offline::ModelManager::new(data.join("whisper-models"), &kalamu_offline::WHISPER);
    let tessdata =
        kalamu_offline::ModelManager::new(data.join("tesseract-data"), &kalamu_offline::TESSDATA);
    // The bundled FFmpeg and llama.cpp sit next to the executable (Tauri's sidecar layout).
    let app_dir: Vec<std::path::PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
        .into_iter()
        .collect();
    let ffmpeg = kalamu_media::Ffmpeg::locate(&app_dir);
    if ffmpeg.is_none() {
        tracing::warn!("FFmpeg not found: recordings are sent whole and video is unavailable");
    }
    let llama = kalamu_offline::Llama::locate(&app_dir);
    if llama.is_none() {
        tracing::warn!("llama.cpp not found: offline OCR models are unavailable");
    }
    // On Windows, its builds for the graphics processor (Vulkan; OpenCL for Snapdragon's
    // Adreno), asked at once which one they find, so Settings can name it and the first page
    // need not wait.
    let graphics = [
        kalamu_offline::Llama::locate_vulkan(&app_dir)
            .map(|p| (kalamu_offline::Backend::Vulkan, p)),
        kalamu_offline::Llama::locate_opencl(&app_dir)
            .map(|p| (kalamu_offline::Backend::OpenCl, p)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let vision = Arc::new(kalamu_offline::Vision::new(
        kalamu_offline::ModelManager::new(data.join("ocr-models"), &kalamu_offline::VISION),
        llama,
        graphics,
        settings.ocr_graphics_processor,
    ));
    let finding = vision.clone();
    std::thread::spawn(move || finding.find_graphics_processor());
    let mut providers = kalamu_providers::registry(http.clone());
    providers.push(Arc::new(kalamu_offline::Whisper::new(models.clone())));
    providers.push(Arc::new(kalamu_offline::Tesseract::new(tessdata.clone())));
    providers.push(vision.clone());
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
    // Each result's provenance names the project documents its source is.
    engine.set_source_documents(Arc::new(kalamu_projects::Projects::new(
        store.clone(),
        store.clone(),
    )));
    let interrupted = engine.recover().unwrap_or_default();
    // Batches the provider still holds are collected below; nothing else is sent until
    // the researcher resumes the job.
    let waiting = engine.jobs_waiting_for_batches().unwrap_or_default();
    if !interrupted.is_empty() {
        tracing::info!(count = interrupted.len(), "jobs interrupted at last close");
    }
    commands::remember_interrupted(interrupted);
    let state = AppState {
        engine,
        projects,
        dictionary,
        search,
        search_generation: Arc::default(),
        search_preparation: Arc::default(),
        models,
        tessdata,
        vision,
        keys,
        grants,
        settings: Mutex::new(settings),
        paths,
        http,
        preview: Mutex::new(None),
        pipelines: Mutex::new(pipelines),
        workflows: Mutex::new(workflows),
        profiles: Mutex::new(profiles),
        quitting: std::sync::atomic::AtomicBool::new(false),
    };
    Ok((state, store, waiting))
}

/// Progress to the interface. A workflow step started from a project is tied to its documents as
/// a job started from the project is.
fn forward_progress(app: &tauri::App, engine: &Arc<Engine>, store: &Arc<SqliteStore>) {
    let mut events = engine.subscribe();
    let handle = app.handle().clone();
    let linker = kalamu_projects::Projects::new(store.clone(), store.clone());
    let runs = engine.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let kalamu_core::ProgressEvent::StepStarted { run_id, job_id, .. } = &event {
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
}

/// Work left from the last close: workflow steps whose input became ready, and batches the
/// providers still hold.
fn resume(engine: &Arc<Engine>, waiting: Vec<JobId>) {
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
