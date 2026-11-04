//! The commands the interface calls, one module per area. Each is a thin wrapper: the work
//! happens in the engine and the library crates, so the interface holds no business logic.

pub mod dictionary;
pub mod jobs;
pub mod keys;
pub mod offline;
pub mod pipelines;
pub mod presets;
pub mod profiles;
pub mod projects;
pub mod review;
pub mod search;
pub mod settings;
pub mod units;
pub mod window;
pub mod workflows;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use kalamu_core::{InputWarning, JobId, ProviderId, SourceFile, SourceId};
use kalamu_engine::Engine;
use kalamu_media::OpenSource;
use serde::Serialize;
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
    /// Project workspaces, over the same database as the engine's jobs.
    pub projects: kalamu_projects::Projects,
    /// Each project's dictionary, over the same database.
    pub dictionary: kalamu_dictionary::Dictionary,
    /// Project search, over the same database and an index of its own.
    pub search: Arc<kalamu_search::Search>,
    /// Raised by each search, so that a newer one stops an older one still indexing.
    pub search_generation: Arc<std::sync::atomic::AtomicU64>,
    /// Raised by each preparation of a project's text, so that a newer one stops an older one.
    pub search_preparation: Arc<std::sync::atomic::AtomicU64>,
    /// Whisper models, shared with the Whisper provider inside the engine.
    pub models: kalamu_offline::ModelManager,
    /// Tesseract's language data, shared with the Tesseract provider inside the engine.
    pub tessdata: kalamu_offline::ModelManager,
    pub keys: Arc<CredentialKeys>,
    pub settings: Mutex<Settings>,
    pub paths: Paths,
    pub http: reqwest::Client,
    /// The last source opened for page previews, so flipping pages does not parse a large PDF
    /// again each time.
    pub preview: Mutex<Option<(SourceId, Arc<OpenSource>)>>,
    /// Saved job settings; the engine does not need them.
    pub pipelines: Mutex<kalamu_prompts::PipelineLibrary>,
    /// Saved workflows; their runs are the engine's.
    pub workflows: Mutex<kalamu_prompts::WorkflowLibrary>,
    /// Saved export profiles for project descriptions.
    pub profiles: Mutex<kalamu_prompts::ProfileLibrary>,
    /// Set once quitting has begun: asked to quit again, the app stops waiting.
    pub quitting: std::sync::atomic::AtomicBool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub logs_dir: String,
    /// Jobs that were running when the app last closed.
    pub interrupted: Vec<JobId>,
    /// The system's languages in order of preference (`fr-FR`, `en-US`…), from which the
    /// interface takes its own when the setting is `system`.
    pub system_languages: Vec<String>,
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
    /// How many pieces a recording is sent in at the usual length, for New job's diagram.
    pub pieces: Option<u32>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PagePreview {
    pub mime: String,
    /// Base64 of a single-page PDF or an image, rendered by the review view.
    pub data: String,
}

pub(crate) static INTERRUPTED: Mutex<Vec<JobId>> = Mutex::new(Vec::new());

pub fn remember_interrupted(ids: Vec<JobId>) {
    *INTERRUPTED.lock().unwrap_or_else(|p| p.into_inner()) = ids;
}

/// Writes an exported file (a record, a dictionary) where the researcher chose.
fn write_text(path: PathBuf, text: String) -> Result<(), AppError> {
    std::fs::write(path, text).map_err(|e| AppError::new(crate::errors::id::FILE, e))
}
