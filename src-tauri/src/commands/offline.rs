//! Offline engines' downloads: Whisper's models, as in Live Translation, and Tesseract's
//! language data, one file per language.

use kalamu_offline::{ModelManager, ModelStatus};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::AppState;
use crate::errors::CmdResult;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct OfflineStatus {
    /// Whisper's speech models.
    pub models: Vec<ModelStatus>,
    /// Tesseract's language data, each file with the languages it reads.
    pub language_data: Vec<ModelStatus>,
    /// The version of the Tesseract built into the app.
    pub tesseract_version: String,
    /// Instruction-set extensions this processor lacks; empty when Whisper can run here.
    pub missing_cpu: Vec<String>,
}

#[tauri::command]
pub fn offline_status(state: State<'_, AppState>) -> OfflineStatus {
    OfflineStatus {
        models: state.models.list(),
        language_data: state.tessdata.list(),
        tesseract_version: kalamu_offline::Tesseract::version(),
        missing_cpu: kalamu_offline::missing_features().to_vec(),
    }
}

/// The engine whose file `id` is: Tesseract's language data or, failing that, Whisper's
/// models, which report an unknown id.
fn manager<'a>(state: &'a AppState, id: &str) -> &'a ModelManager {
    if state.tessdata.find(id).is_some() {
        &state.tessdata
    } else {
        &state.models
    }
}

/// Returns when the download has finished, failed or been cancelled; the interface polls
/// [`offline_status`] meanwhile for the bytes received.
#[tauri::command]
pub async fn download_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let models = manager(&state, &id).clone();
    Ok(models.download(&id).await?)
}

#[tauri::command]
pub fn cancel_model_download(state: State<'_, AppState>) {
    state.models.cancel_download();
    state.tessdata.cancel_download();
}

#[tauri::command]
pub fn remove_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(manager(&state, &id).remove(&id)?)
}
