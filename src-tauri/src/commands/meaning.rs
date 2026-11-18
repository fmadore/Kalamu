//! The search by meaning and the map of a project's documents. Choosing a model and reading
//! the estimate send nothing; preparing sends the project's text only once the researcher starts
//! it, and with a cloud model only after they agreed; each question is embedded through the
//! engine too. The work runs off the window's thread, the preparation's progress sent as
//! [`MEANING_PROGRESS_EVENT`](crate::MEANING_PROGRESS_EVENT).

use kalamu_core::{DocumentId, EmbeddingSpace, ProjectId, ProviderId, SpaceChoice};
use kalamu_engine::MeaningStatus;
use kalamu_search::{MapView, MeaningRequest, MeaningView, Neighbour};
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

/// Runs `work` on a thread of its own, where it may read the database at length.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, kalamu_engine::MeaningError> + Send + 'static,
) -> CmdResult<T> {
    Ok(tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))??)
}

/// The project's model, what preparing would embed and cost (nothing sent), the run in
/// progress, the runs recorded and what the index holds.
#[tauri::command]
pub async fn meaning_status(
    state: State<'_, AppState>,
    project: ProjectId,
) -> CmdResult<MeaningStatus> {
    let meaning = state.meaning.clone();
    blocking(move || meaning.status(project)).await
}

/// Makes a model the project's. Nothing is sent.
#[tauri::command]
pub fn choose_meaning_model(
    state: State<'_, AppState>,
    project: ProjectId,
    provider: ProviderId,
    model: String,
) -> CmdResult<SpaceChoice> {
    Ok(state.meaning.choose(project, &provider, &model)?)
}

/// Records the researcher's consent to send the project's text, and then its questions, to its
/// model's provider.
#[tauri::command]
pub fn consent_meaning(state: State<'_, AppState>, project: ProjectId) -> CmdResult<()> {
    Ok(state.meaning.consent(project)?)
}

/// Starts preparing the project's text: refused at once without a model, its consent, its key
/// or its download; otherwise it runs on, batch by batch, telling the interface.
#[tauri::command]
pub fn prepare_meaning(state: State<'_, AppState>, project: ProjectId) -> CmdResult<()> {
    let ready = state.meaning.check(project)?;
    let meaning = state.meaning.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = meaning.run(ready).await {
            tracing::warn!(project = %project, error = %e, "preparation for the search by meaning stopped");
        }
    });
    Ok(())
}

/// Pauses the preparation after the batch in flight; what was embedded is kept.
#[tauri::command]
pub fn pause_meaning(state: State<'_, AppState>, project: ProjectId) {
    state.meaning.pause(project);
}

/// Cancels the preparation after the batch in flight; what was embedded is kept.
#[tauri::command]
pub fn cancel_meaning(state: State<'_, AppState>, project: ProjectId) {
    state.meaning.cancel(project);
}

/// Removes the vectors one model made of the project's documents.
#[tauri::command]
pub async fn remove_meaning_space(
    state: State<'_, AppState>,
    project: ProjectId,
    space: EmbeddingSpace,
) -> CmdResult<()> {
    let meaning = state.meaning.clone();
    blocking(move || meaning.remove_space(project, &space)).await
}

/// Searches the project by meaning: the question embedded by its model, then compared with
/// every chunk prepared.
#[tauri::command]
pub async fn meaning_search(
    state: State<'_, AppState>,
    project: ProjectId,
    request: MeaningRequest,
) -> CmdResult<MeaningView> {
    let meaning = state.meaning.clone();
    Ok(meaning.search(project, request).await?)
}

/// The map of the project's documents.
#[tauri::command]
pub async fn meaning_map(state: State<'_, AppState>, project: ProjectId) -> CmdResult<MapView> {
    let meaning = state.meaning.clone();
    blocking(move || meaning.map(project)).await
}

/// The documents most like one of the project's.
#[tauri::command]
pub async fn meaning_neighbours(
    state: State<'_, AppState>,
    project: ProjectId,
    document: DocumentId,
) -> CmdResult<Vec<Neighbour>> {
    let meaning = state.meaning.clone();
    blocking(move || meaning.neighbours(project, document)).await
}
