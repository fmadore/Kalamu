//! Adding files, estimating, and running jobs.

use std::path::{Path, PathBuf};

use scriptorium_catalog::CostEstimate;
use scriptorium_core::{InputKind, Job, JobId, JobSummary, PieceState, Task};
use scriptorium_engine::{Engine, NewJob};
use tauri::State;

use super::{AppState, EstimateRequest, EstimateRow, InspectedFile};
use crate::errors::{AppError, CmdResult, id};

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
