//! Offline transcription: the Whisper models, downloaded from the app as in Live Translation.

use kalamu_offline::ModelStatus;
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::AppState;
use crate::errors::CmdResult;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct OfflineStatus {
    pub models: Vec<ModelStatus>,
    /// Instruction-set extensions this processor lacks; empty when Whisper can run here.
    pub missing_cpu: Vec<String>,
}

#[tauri::command]
pub fn offline_status(state: State<'_, AppState>) -> OfflineStatus {
    OfflineStatus {
        models: state.models.list(),
        missing_cpu: kalamu_offline::missing_features().to_vec(),
    }
}

/// Returns when the download has finished, failed or been cancelled; the interface polls
/// [`offline_status`] meanwhile for the bytes received.
#[tauri::command]
pub async fn download_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let models = state.models.clone();
    Ok(models.download(&id).await?)
}

#[tauri::command]
pub fn cancel_model_download(state: State<'_, AppState>) {
    state.models.cancel_download();
}

#[tauri::command]
pub fn remove_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    Ok(state.models.remove(&id)?)
}
