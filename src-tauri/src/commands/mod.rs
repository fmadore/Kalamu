//! The commands the interface calls, one module per area. Each is a thin wrapper: the work
//! happens in the engine and the library crates, so the interface holds no business logic.

pub mod jobs;
pub mod keys;
pub mod presets;
pub mod review;
pub mod settings;
pub mod window;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use scriptorium_catalog::CostEstimate;
use scriptorium_core::{InputWarning, JobId, ProviderId, SourceFile, SourceId, Task};
use scriptorium_engine::{Engine, SetupRequest};
use scriptorium_media::OpenSource;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::errors::AppError;
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

pub(crate) static INTERRUPTED: Mutex<Vec<JobId>> = Mutex::new(Vec::new());

pub fn remember_interrupted(ids: Vec<JobId>) {
    *INTERRUPTED.lock().unwrap_or_else(|p| p.into_inner()) = ids;
}
