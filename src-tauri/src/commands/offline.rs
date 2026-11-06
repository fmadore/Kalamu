//! Offline engines' downloads: Whisper's models, as in Live Translation, Tesseract's language
//! data, one file per language, and the OCR vision models, each with its image encoder.

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
    /// The OCR vision models, each counted with its image encoder.
    pub ocr_models: Vec<ModelStatus>,
    /// Whether this copy of the app carries the engine that runs them.
    pub ocr_engine: bool,
    /// Seconds a page has taken on this computer, by model, from the pages it read last.
    pub ocr_seconds_per_page: std::collections::BTreeMap<String, f32>,
    /// Instruction-set extensions this processor lacks; empty when Whisper can run here.
    pub missing_cpu: Vec<String>,
}

#[tauri::command]
pub fn offline_status(state: State<'_, AppState>) -> OfflineStatus {
    OfflineStatus {
        models: state.models.list(),
        language_data: state.tessdata.list(),
        tesseract_version: kalamu_offline::Tesseract::version(),
        ocr_models: state.vision.models().list(),
        ocr_engine: state.vision.available(),
        ocr_seconds_per_page: state.vision.speeds(),
        missing_cpu: kalamu_offline::missing_features().to_vec(),
    }
}

/// The engine whose file `id` is: Tesseract's language data, an OCR vision model or, failing
/// those, Whisper's models, which report an unknown id.
fn manager<'a>(state: &'a AppState, id: &str) -> &'a ModelManager {
    if state.tessdata.find(id).is_some() {
        &state.tessdata
    } else if state.vision.models().find(id).is_some() {
        state.vision.models()
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
    state.vision.models().cancel_download();
}

#[tauri::command]
pub fn remove_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(manager(&state, &id).remove(&id)?)
}
