//! Project search: the passages of a project's documents that hold some words, found in the
//! local index of their corrected text. Nothing is sent; the index is brought up to date with
//! the store before each search, or ahead of it when the search opens, on a thread of its own so
//! the window stays free, telling the interface how far it has got.

use std::sync::atomic::Ordering;

use kalamu_core::{JobId, ProjectId, Task};
use kalamu_engine::{Engine, Freshness};
use kalamu_search::{SearchError, SearchProgress, SearchRequest, SearchView};
use tauri::{AppHandle, Emitter, State};

use super::AppState;
use crate::SEARCH_PROGRESS_EVENT;
use crate::errors::{AppError, CmdResult, id};

/// Tells the interface how far a project's text is prepared.
fn tell(app: &AppHandle, project_id: ProjectId) -> impl Fn(u32, u32) {
    let app = app.clone();
    move |done, total| {
        let _ = app.emit(
            SEARCH_PROGRESS_EVENT,
            SearchProgress {
                project_id,
                done,
                total,
            },
        );
    }
}

/// Marks the analyses that read text corrected since, as the job's page does.
fn mark_stale(engine: &Engine, view: &mut SearchView) {
    let mut checked: std::collections::HashMap<JobId, Vec<kalamu_engine::DerivedState>> =
        std::collections::HashMap::new();
    for result in view.documents.iter_mut().flat_map(|d| d.results.iter_mut()) {
        if result.task != Task::Analysis {
            continue;
        }
        let states = checked
            .entry(result.job_id)
            .or_insert_with(|| engine.derived_states(result.job_id).unwrap_or_default());
        result.stale = states
            .iter()
            .any(|s| s.source_id == result.source_id && s.freshness != Freshness::Current);
    }
}

/// Searches a project. A search begun while another is still indexing stops the older one,
/// which answers `search.stopped`.
#[tauri::command]
pub async fn project_search(
    app: AppHandle,
    state: State<'_, AppState>,
    project: ProjectId,
    request: SearchRequest,
) -> CmdResult<SearchView> {
    let generation = state.search_generation.clone();
    let mine = generation.fetch_add(1, Ordering::SeqCst) + 1;
    let search = state.search.clone();
    let engine = state.engine.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let stop = || generation.load(Ordering::SeqCst) != mine;
        let mut view = search.search(project, &request, &stop, &tell(&app, project))?;
        mark_stale(&engine, &mut view);
        Ok(view)
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
}

/// Empties the index; the next search makes again what it needs from the results.
#[tauri::command]
pub async fn rebuild_search_index(state: State<'_, AppState>) -> CmdResult<()> {
    state.search_generation.fetch_add(1, Ordering::SeqCst);
    let search = state.search.clone();
    tauri::async_runtime::spawn_blocking(move || search.rebuild())
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))??;
    Ok(())
}

/// Prepares the text of every result of a project ahead of its first search, telling the
/// interface how far it has got. A search begun meanwhile stops it, and finishes what it needs
/// itself; so does a newer preparation. Stopped, it answers without an error.
#[tauri::command]
pub async fn prepare_project_search(
    app: AppHandle,
    state: State<'_, AppState>,
    project: ProjectId,
) -> CmdResult<()> {
    let searches = state.search_generation.clone();
    let preparations = state.search_preparation.clone();
    let search_seen = searches.load(Ordering::SeqCst);
    let mine = preparations.fetch_add(1, Ordering::SeqCst) + 1;
    let search = state.search.clone();
    let prepared = tauri::async_runtime::spawn_blocking(move || {
        let stop = || {
            searches.load(Ordering::SeqCst) != search_seen
                || preparations.load(Ordering::SeqCst) != mine
        };
        search.prepare(project, &stop, &tell(&app, project))
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?;
    match prepared {
        Ok(_) | Err(SearchError::Stopped) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
