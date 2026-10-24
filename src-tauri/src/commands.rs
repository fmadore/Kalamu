//! The commands the interface calls. Each is a thin wrapper: the work happens in the engine and
//! the library crates, so the interface holds no business logic.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use scriptorium_catalog::{Catalog, CostEstimate};
use scriptorium_core::{
    Correction, InputKind, InputWarning, Job, JobId, JobSummary, PieceState, ProviderId, SetupId,
    SourceFile, SourceId, Task,
};
use scriptorium_engine::{
    ComparisonReport, DiffSpan, Engine, KeySource, NewJob, PieceView, SetupRequest,
};
use scriptorium_media::OpenSource;
use scriptorium_prompts::Preset;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

use crate::errors::{AppError, CmdResult, id};
use crate::secrets::CredentialKeys;
use crate::settings::Settings;

pub struct Paths {
    pub data: PathBuf,
    pub settings: PathBuf,
    pub catalog_cache: PathBuf,
    pub logs: PathBuf,
}

pub struct AppState {
    pub engine: Arc<Engine>,
    pub keys: Arc<CredentialKeys>,
    pub settings: Mutex<Settings>,
    pub paths: Paths,
    pub http: reqwest::Client,
    /// The last source opened for page previews, so flipping pages does not parse a large PDF
    /// again each time.
    pub preview: Mutex<Option<(SourceId, Arc<OpenSource>)>>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub logs_dir: String,
    /// Jobs that were running when the app last closed.
    pub interrupted: Vec<JobId>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct KeyStatus {
    pub provider: ProviderId,
    pub saved: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct InspectedFile {
    pub path: String,
    pub source: Option<SourceFile>,
    pub warnings: Vec<InputWarning>,
    pub error: Option<AppError>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct EstimateRow {
    pub provider: ProviderId,
    pub model: String,
    pub pages: u32,
    pub estimate: CostEstimate,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PagePreview {
    pub mime: String,
    /// Base64 of a single-page PDF or an image, rendered by the review view.
    pub data: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct EstimateRequest {
    pub task: Task,
    pub sources: Vec<SourceFile>,
    pub setups: Vec<SetupRequest>,
    pub sample_pages: Option<u32>,
}

static INTERRUPTED: Mutex<Vec<JobId>> = Mutex::new(Vec::new());

pub fn remember_interrupted(ids: Vec<JobId>) {
    *INTERRUPTED.lock().unwrap_or_else(|p| p.into_inner()) = ids;
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        data_dir: state.paths.data.display().to_string(),
        logs_dir: state.paths.logs.display().to_string(),
        interrupted: INTERRUPTED.lock().map(|v| v.clone()).unwrap_or_default(),
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.lock().map(|s| s.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> CmdResult<()> {
    settings
        .save(&state.paths.settings)
        .map_err(|e| AppError::new(id::SETTINGS, e))?;
    *state.settings.lock().unwrap_or_else(|p| p.into_inner()) = settings;
    Ok(())
}

#[tauri::command]
pub fn get_catalog(state: State<'_, AppState>) -> Catalog {
    state.engine.catalog()
}

/// Downloads the catalogue from the repository and keeps it when it is newer than the one in use.
#[tauri::command]
pub async fn refresh_catalog(state: State<'_, AppState>) -> CmdResult<Catalog> {
    let response = state
        .http
        .get(scriptorium_catalog::UPDATE_URL)
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

#[tauri::command]
pub fn key_status(state: State<'_, AppState>) -> Vec<KeyStatus> {
    state
        .engine
        .catalog()
        .providers
        .iter()
        .map(|p| KeyStatus {
            provider: p.id.clone(),
            saved: state.keys.has(&p.id),
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
    let p = scriptorium_providers::find(&state.engine_providers(), &provider)
        .ok_or_else(|| AppError::new("provider.unknown", &provider))?;
    Ok(p.list_models(&key).await?)
}

impl AppState {
    fn engine_providers(&self) -> Vec<Arc<dyn scriptorium_core::Provider>> {
        scriptorium_providers::registry(self.http.clone())
    }
}

#[tauri::command]
pub fn list_presets(state: State<'_, AppState>, task: Option<Task>) -> Vec<Preset> {
    state.engine.with_prompts(|p| p.list(task))
}

#[tauri::command]
pub fn save_preset(state: State<'_, AppState>, preset: Preset) -> CmdResult<Preset> {
    Ok(state.engine.with_prompts(|p| p.save(preset))?)
}

/// Deletes a user preset, or resets an edited default.
#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state.engine.with_prompts(|p| p.delete(&id))?)
}

#[tauri::command]
pub fn duplicate_preset(state: State<'_, AppState>, id: String, name: String) -> CmdResult<Preset> {
    Ok(state.engine.with_prompts(|p| p.duplicate(&id, &name))?)
}

#[tauri::command]
pub fn export_presets(state: State<'_, AppState>, ids: Vec<String>, path: String) -> CmdResult<()> {
    let json = state.engine.with_prompts(|p| p.export(&ids))?;
    std::fs::write(&path, json).map_err(|e| AppError::new(id::FILE, e))
}

#[tauri::command]
pub fn import_presets(state: State<'_, AppState>, path: String) -> CmdResult<Vec<Preset>> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::new(id::FILE, e))?;
    Ok(state.engine.with_prompts(|p| p.import(&json))?)
}

/// Files and folders dropped or chosen, expanded to the supported files for a task, sorted so
/// numbered scans keep their order.
#[tauri::command]
pub fn expand_paths(paths: Vec<String>, task: Task) -> Vec<String> {
    fn wanted(task: Task, kind: InputKind) -> bool {
        match task {
            Task::TextExtraction => matches!(kind, InputKind::Pdf | InputKind::Image),
            Task::AudioTranscription => matches!(kind, InputKind::Audio | InputKind::Video),
        }
    }
    fn walk(path: &Path, task: Task, out: &mut Vec<PathBuf>, depth: usize) {
        if path.is_dir() {
            if depth > 8 {
                return;
            }
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            let mut children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            children.sort();
            for child in children {
                walk(&child, task, out, depth + 1);
            }
        } else if InputKind::from_path(path).is_some_and(|k| wanted(task, k)) {
            out.push(path.to_path_buf());
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(Path::new(&p), task, &mut out, 0);
    }
    out.dedup();
    out.into_iter().map(|p| p.display().to_string()).collect()
}

#[tauri::command]
pub async fn inspect_files(paths: Vec<String>) -> CmdResult<Vec<InspectedFile>> {
    tokio::task::spawn_blocking(move || {
        Engine::inspect(&paths)
            .into_iter()
            .zip(paths)
            .map(|(result, path)| match result {
                Ok((source, warnings)) => InspectedFile {
                    path,
                    source: Some(source),
                    warnings,
                    error: None,
                },
                Err(e) => InspectedFile {
                    path,
                    source: None,
                    warnings: Vec::new(),
                    error: Some(e.into()),
                },
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))
}

#[tauri::command]
pub fn estimate_cost(state: State<'_, AppState>, request: EstimateRequest) -> Vec<EstimateRow> {
    let catalog = state.engine.catalog();
    let pages: u32 = request
        .sources
        .iter()
        .map(|s| {
            let n = s.page_count.unwrap_or(1) * if s.split_spreads { 2 } else { 1 };
            request.sample_pages.map_or(n, |cap| n.min(cap))
        })
        .sum();
    let minutes: f64 = request
        .sources
        .iter()
        .map(|s| s.duration_ms.unwrap_or(0) as f64 / 60_000.0)
        .sum();
    request
        .setups
        .iter()
        .map(|setup| {
            let prompt_chars = setup
                .preset_id
                .as_ref()
                .and_then(|id| state.engine.with_prompts(|p| p.get(id)).ok())
                .map_or(2_000, |p| p.instructions.len());
            let estimate = match catalog.model(&setup.provider, &setup.model) {
                Some(model) => catalog.estimate(model, request.task, pages, minutes, prompt_chars),
                None => CostEstimate {
                    usd: None,
                    note: Some("This model is not in the catalogue.".into()),
                },
            };
            EstimateRow {
                provider: setup.provider.clone(),
                model: setup.model.clone(),
                pages,
                estimate,
            }
        })
        .collect()
}

#[tauri::command]
pub fn create_job(state: State<'_, AppState>, job: NewJob) -> CmdResult<Job> {
    Ok(state.engine.create_job(job)?)
}

#[tauri::command]
pub fn start_job(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.start(id)?)
}

#[tauri::command]
pub fn pause_job(state: State<'_, AppState>, id: JobId) {
    state.engine.pause(id);
}

#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, id: JobId) {
    state.engine.cancel(id);
}

#[tauri::command]
pub fn retry_failed(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.retry_failed(id)?)
}

#[tauri::command]
pub fn delete_job(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.delete_job(id)?)
}

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> CmdResult<Vec<JobSummary>> {
    Ok(state.engine.jobs()?)
}

#[tauri::command]
pub fn get_job(state: State<'_, AppState>, id: JobId) -> CmdResult<Job> {
    Ok(state.engine.job(id)?)
}

#[tauri::command]
pub fn job_pieces(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<PieceState>> {
    Ok(state.engine.pieces(id)?)
}

#[tauri::command]
pub fn source_pieces(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<Vec<PieceView>> {
    Ok(state.engine.source_pieces(job_id, source_id)?)
}

/// One page of a source as a single-page PDF or an image, for the review view.
#[tauri::command]
pub async fn piece_preview(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
) -> CmdResult<PagePreview> {
    let job = state.engine.job(job_id)?;
    let source = job
        .sources
        .iter()
        .find(|s| s.id == source_id)
        .cloned()
        .ok_or_else(|| AppError::new("job.not_found", source_id))?;
    let cached = state.preview.lock().ok().and_then(|c| {
        c.as_ref()
            .filter(|(id, _)| *id == source_id)
            .map(|(_, o)| o.clone())
    });
    let opened = match cached {
        Some(o) => o,
        None => {
            let s = source.clone();
            let o = tokio::task::spawn_blocking(move || OpenSource::open(&s))
                .await
                .map_err(|e| AppError::new(id::INTERNAL, e))?
                .map_err(|e| AppError::new("media.read", e))?;
            let o = Arc::new(o);
            *state.preview.lock().unwrap_or_else(|p| p.into_inner()) = Some((source_id, o.clone()));
            o
        }
    };
    let pieces = opened.pieces(source.split_spreads, job.settings.sample_pages);
    let piece = pieces
        .get(index as usize)
        .cloned()
        .ok_or_else(|| AppError::new("media.read", format!("no piece {index}")))?;
    let prepared = tokio::task::spawn_blocking(move || {
        opened.prepare(
            &piece,
            &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            "preview",
        )
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
    .map_err(|e| AppError::new("media.read", e))?;
    Ok(PagePreview {
        mime: prepared.mime,
        data: base64::engine::general_purpose::STANDARD.encode(prepared.bytes),
    })
}

#[tauri::command]
pub fn save_correction(
    state: State<'_, AppState>,
    correction: Correction,
) -> CmdResult<Vec<String>> {
    Ok(state.engine.save_correction(correction)?)
}

#[tauri::command]
pub fn export_source(
    state: State<'_, AppState>,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
) -> CmdResult<Vec<String>> {
    Ok(state.engine.export(job_id, setup_id, source_id)?)
}

#[tauri::command]
pub fn compare_source(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<ComparisonReport> {
    Ok(state.engine.compare(job_id, source_id)?)
}

#[tauri::command]
pub fn diff_texts(left: String, right: String) -> Vec<DiffSpan> {
    scriptorium_engine::diff(&left, &right)
}

/// Progress on the taskbar button: 0–100, or `None` to clear it.
#[tauri::command]
pub fn set_taskbar_progress(app: AppHandle, percent: Option<u32>) {
    use tauri::window::{ProgressBarState, ProgressBarStatus};
    if let Some(window) = app.get_webview_window("main") {
        let state = match percent {
            Some(p) => ProgressBarState {
                status: Some(ProgressBarStatus::Normal),
                progress: Some(u64::from(p.min(100))),
            },
            None => ProgressBarState {
                status: Some(ProgressBarStatus::None),
                progress: None,
            },
        };
        let _ = window.set_progress_bar(state);
    }
}

/// Closes the app after the user confirmed. Running jobs are paused first so they resume
/// cleanly; even without that, the next launch marks them as interrupted.
#[tauri::command]
pub async fn quit_app(app: AppHandle, state: State<'_, AppState>) -> CmdResult<()> {
    let running: Vec<JobId> = state
        .engine
        .jobs()?
        .into_iter()
        .filter(|j| state.engine.is_running(j.id))
        .map(|j| j.id)
        .collect();
    for id in &running {
        state.engine.pause(*id);
    }
    // Give pages in flight a moment to land in the cache.
    for _ in 0..50 {
        if running.iter().all(|id| !state.engine.is_running(*id)) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    app.exit(0);
    Ok(())
}
