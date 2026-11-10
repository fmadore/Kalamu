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
    /// The Whisper provider, for its graphics processor (Settings, the offline status).
    pub whisper: Arc<kalamu_offline::Whisper>,
    /// Tesseract's language data, shared with the Tesseract provider inside the engine.
    pub tessdata: kalamu_offline::ModelManager,
    /// llama.cpp's programs, shared by the offline OCR and language models.
    pub llama: Arc<kalamu_offline::LlamaHost>,
    /// The offline OCR models, each with its image encoder.
    pub vision: Arc<kalamu_offline::Vision>,
    /// The offline language models, each with its encoder.
    pub llm: Arc<kalamu_offline::LocalLlm>,
    pub keys: Arc<CredentialKeys>,
    /// Folders and files the researcher handed the app, kept where a sandbox requires it.
    pub grants: crate::grants::Grants,
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
    /// The Mac App Store's build, which runs in the App Sandbox: it asks for a folder before
    /// writing results beside sources, and offers no support link (App Review Guideline 3.1.1).
    pub app_store: bool,
    /// Where the settings file that could not be read at this launch was moved, as File
    /// Explorer or Finder shows it; the defaults are in use. The interface says so once.
    pub settings_set_aside: Option<String>,
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

/// Where an unreadable settings file was moved at this launch, if one was.
pub(crate) static SETTINGS_SET_ASIDE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn remember_settings_reset(set_aside: Option<PathBuf>) {
    *SETTINGS_SET_ASIDE.lock().unwrap_or_else(|p| p.into_inner()) = set_aside;
}

/// The cloud providers among `providers` the researcher has not agreed to send their files to,
/// each once, in order. A provider that needs no key is an offline engine: nothing is sent.
fn unconsented(
    state: &AppState,
    providers: impl IntoIterator<Item = ProviderId>,
) -> Vec<ProviderId> {
    let agreed = state
        .settings
        .lock()
        .map(|s| s.ai_consent.clone())
        .unwrap_or_default();
    let mut out: Vec<ProviderId> = Vec::new();
    for provider in providers {
        let cloud = state
            .engine
            .provider(&provider)
            .map_or(true, |p| p.needs_key());
        if cloud && !agreed.contains(&provider) && !out.contains(&provider) {
            out.push(provider);
        }
    }
    out
}

/// Refuses work that would send files to a provider the researcher has not agreed to: the
/// interface asks first (`askConsent`), and this keeps any other path from skipping it.
fn require_consent<'a>(
    state: &AppState,
    setups: impl IntoIterator<Item = &'a kalamu_core::SetupRequest>,
) -> Result<(), AppError> {
    let providers = setups.into_iter().flat_map(|s| {
        std::iter::once(s.provider.clone()).chain(s.merge.as_ref().map(|m| m.provider.clone()))
    });
    match unconsented(state, providers).first() {
        Some(provider) => Err(AppError::new(crate::errors::id::CONSENT, provider)),
        None => Ok(()),
    }
}

/// Runs work that reads or writes files off the window's thread, where a synchronous command
/// would run it and freeze the window: writing a result waits for a running job to finish
/// writing its own.
async fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| AppError::new(crate::errors::id::INTERNAL, e))?
}

/// Writes an exported file (a record, a dictionary) where the researcher chose.
fn write_text(path: PathBuf, text: String) -> Result<(), AppError> {
    std::fs::write(path, text).map_err(|e| AppError::new(crate::errors::id::FILE, e))
}

/// Writes a pipeline's or a workflow's record: the JSON at `path`, whose name the researcher
/// chose (the save dialog asked before replacing it), and the Markdown beside it, which never
/// replaces a file an export did not write ([`kalamu_prompts::markdown_beside`]). Returns the
/// Markdown file's path, for the interface to name when it is not `name.md`.
fn write_record(
    path: &std::path::Path,
    json: String,
    markdown: String,
) -> Result<String, AppError> {
    let json_path = path.with_extension("json");
    let markdown_path = kalamu_prompts::markdown_beside(&json_path)
        .ok_or_else(|| AppError::new(crate::errors::id::FILE, json_path.display()))?;
    write_text(json_path, json)?;
    write_text(markdown_path.clone(), markdown)?;
    Ok(markdown_path.display().to_string())
}
