//! The commands the interface calls, one module per area. Each is a thin wrapper: the work
//! happens in the engine and the library crates, so the interface holds no business logic.

pub mod dictionary;
pub mod jobs;
pub mod keys;
pub mod meaning;
pub mod offline;
pub mod passage;
pub mod pipelines;
pub mod presets;
pub mod profiles;
pub mod project_file;
pub mod projects;
pub mod region_reading;
pub mod review;
pub mod sample;
pub mod search;
pub mod settings;
pub mod units;
pub mod window;
pub mod workflows;

use std::path::{Path, PathBuf};
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
    /// A project saved as one file and opened again, by the same database.
    pub archive: Arc<dyn kalamu_core::ProjectArchive>,
    /// The same database, read for what clearing the unused results would remove; clearing
    /// them goes through the engine.
    pub result_cache: Arc<kalamu_store::SqliteStore>,
    /// Project search, over the same database and an index of its own.
    pub search: Arc<kalamu_search::Search>,
    /// The search by meaning and the map, where they call a model: preparation runs and
    /// questions.
    pub meaning: Arc<kalamu_engine::Meaning>,
    /// Raised by each search, so that a newer one stops an older one still indexing.
    pub search_generation: Arc<std::sync::atomic::AtomicU64>,
    /// Raised by each preparation of a project's text, so that a newer one stops an older one.
    pub search_preparation: Arc<std::sync::atomic::AtomicU64>,
    /// The stop of New job's photograph check under way: a newer check, or leaving New job,
    /// raises it.
    pub photo_check: Mutex<kalamu_core::StopSignal>,
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
    /// The offline embedding model of the search by meaning.
    pub embedder: Arc<kalamu_offline::LocalEmbedder>,
    pub keys: Arc<CredentialKeys>,
    /// Folders and files the researcher handed the app, kept where a sandbox requires it.
    pub grants: crate::grants::Grants,
    /// The files table exports wrote, so exporting again replaces them and nothing else.
    pub exports: Arc<crate::exports::ExportLedger>,
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
    /// Whether the interface holds changes not saved (a correction, a prompt, a workflow), as
    /// it last said: closing the window or quitting then asks it first.
    pub draft_open: std::sync::atomic::AtomicBool,
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

/// What New job's check of its photographs found: the same page photographed twice, a blurred
/// photograph, and each photograph warned about drawn small and close up.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PhotoChecks {
    pub warnings: Vec<InputWarning>,
    pub views: Vec<PhotoView>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PhotoView {
    pub source_id: SourceId,
    /// The whole photograph, as a base64 JPEG at most `kalamu_media::VIEW_WIDTH` wide.
    pub page: String,
    /// The part of it where the writing is densest, close up, as a base64 JPEG.
    pub detail: String,
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

/// Refuses a request from the review (a passage corrected, a region read again) that would send
/// to `provider` without the researcher's consent.
fn consented(state: &AppState, provider: &ProviderId) -> Result<(), AppError> {
    match unconsented(state, [provider.clone()]).first() {
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

/// The file of a table export the save dialog asked about, the one whose name the researcher
/// chose; the table goes beside it.
pub(crate) enum Asked {
    /// A JSON file, holding this text: the workbook and the CSV go beside it.
    Json(String),
    /// The workbook itself: the CSV goes beside it.
    Workbook,
}

/// Writes a table export where the researcher chose: the file the dialog asked about (`asked`,
/// at `path` with its extension), and beside it the table as `.xlsx`, which opens by
/// double-click in Excel set to any language, and as `.csv` for R, Python and other tools.
///
/// The dialog asked before replacing the chosen file, but not the files beside it: before
/// anything is written, those are named so that none replaces a file an export did not write
/// ([`kalamu_prompts::beside`], with the files an export wrote recognised by `ledger`). When one
/// is the researcher's, the files beside all go under the next free name, `name (2).csv`…, and
/// exporting again to the same name replaces Kalamu's own earlier files. Each file is written
/// whole through a temporary file beside it. Returns the files written under another name than
/// the chosen one's, for the interface to name. `failed` says a table that could not be made
/// as the export's own failure.
fn write_table(
    path: &Path,
    asked: Asked,
    table: &kalamu_core::Table,
    failed: impl Fn(String) -> AppError,
    ledger: &crate::exports::ExportLedger,
) -> Result<Vec<String>, AppError> {
    let xlsx = kalamu_export::table_xlsx(table).map_err(|e| failed(e.to_string()))?;
    let csv = kalamu_export::table_csv(table)
        .map_err(|e| failed(e.to_string()))?
        .into_bytes();
    let (chosen, chosen_bytes, beside) = match asked {
        Asked::Json(json) => (
            path.with_extension("json"),
            json.into_bytes(),
            vec![("xlsx", xlsx), ("csv", csv)],
        ),
        Asked::Workbook => (path.with_extension("xlsx"), xlsx, vec![("csv", csv)]),
    };
    let extensions: Vec<&str> = beside.iter().map(|(ext, _)| *ext).collect();
    let names = kalamu_prompts::beside(&chosen, &extensions, |p| ledger.wrote(p))
        .ok_or_else(|| AppError::new(crate::errors::id::FILE, chosen.display()))?;
    write_whole(&chosen, &chosen_bytes)?;
    ledger.record(&chosen, &chosen_bytes);
    let mut renamed = Vec::new();
    for ((ext, bytes), name) in beside.into_iter().zip(names) {
        write_whole(&name, &bytes)?;
        ledger.record(&name, &bytes);
        if name != chosen.with_extension(ext) {
            renamed.push(name.display().to_string());
        }
    }
    Ok(renamed)
}

/// Writes `bytes` at `path` through a temporary file in the same folder, renamed at the end, so
/// a failure leaves whatever stood there whole.
fn write_whole(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let file_error = |e: std::io::Error| AppError::new(crate::errors::id::FILE, e);
    let folder = match path.parent() {
        Some(folder) if !folder.as_os_str().is_empty() => folder,
        _ => Path::new("."),
    };
    let mut tmp = tempfile::Builder::new()
        .prefix(".kalamu-")
        .suffix(".pending")
        .tempfile_in(folder)
        .map_err(file_error)?;
    std::io::Write::write_all(&mut tmp, bytes).map_err(file_error)?;
    tmp.as_file().sync_all().map_err(file_error)?;
    tmp.persist(path).map_err(|e| file_error(e.error))?;
    Ok(())
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

/// What every table export is checked against, one test per export calling the function its
/// command calls.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod table_export_tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use crate::errors::AppError;
    use crate::exports::ExportLedger;

    pub(crate) type Write<'a> = &'a dyn Fn(&Path, &ExportLedger) -> Result<Vec<String>, AppError>;

    pub(crate) fn table() -> kalamu_core::Table {
        kalamu_core::Table {
            headings: vec!["Title".into(), "Answer".into()],
            rows: vec![vec!["Letter 1".into(), "Yes".into()]],
            cells: kalamu_core::CsvCells::default(),
        }
    }

    fn names(folder: &Path) -> BTreeSet<String> {
        std::fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    /// An export to `chosen` (the name the save dialog asked about, `inventory.json`) that
    /// writes the table beside it with each of `beside`: files of the researcher's at those
    /// names are kept byte for byte, the table going under `inventory (2)`, and exporting
    /// again replaces Kalamu's own files without piling up numbered copies.
    pub(crate) fn keeps_the_researchers_files(chosen: &str, beside: &[&str], write: Write<'_>) {
        let dir = tempfile::tempdir().unwrap();
        let ledger = ExportLedger::open(dir.path().join("exports.json"));
        let out = dir.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let path = out.join(chosen);
        let stem = Path::new(chosen).file_stem().unwrap().to_string_lossy();
        let mine = |ext: &str| format!("my own {ext}, with my columns");
        for ext in beside {
            std::fs::write(out.join(format!("{stem}.{ext}")), mine(ext)).unwrap();
        }
        let numbered: Vec<String> = beside
            .iter()
            .map(|ext| out.join(format!("{stem} (2).{ext}")).display().to_string())
            .collect();

        for _ in 0..2 {
            assert_eq!(write(&path, &ledger).unwrap(), numbered);
            for ext in beside {
                assert_eq!(
                    std::fs::read_to_string(out.join(format!("{stem}.{ext}"))).unwrap(),
                    mine(ext),
                    "the researcher's .{ext} is kept"
                );
            }
            let mut expected: BTreeSet<String> = beside
                .iter()
                .flat_map(|ext| [format!("{stem}.{ext}"), format!("{stem} (2).{ext}")])
                .collect();
            expected.insert(chosen.to_string());
            // Nothing else: no `(3)` copy, no temporary file left.
            assert_eq!(names(&out), expected);
        }
    }

    /// Exporting twice to the same name in a folder where nothing else is replaces the first
    /// export's files under their own names; once the researcher changed one, it is theirs.
    pub(crate) fn replaces_its_own_files(chosen: &str, beside: &[&str], write: Write<'_>) {
        let dir = tempfile::tempdir().unwrap();
        let ledger = ExportLedger::open(dir.path().join("exports.json"));
        let out = dir.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let path = out.join(chosen);
        let stem = Path::new(chosen).file_stem().unwrap().to_string_lossy();
        let mut expected: BTreeSet<String> =
            beside.iter().map(|ext| format!("{stem}.{ext}")).collect();
        expected.insert(chosen.to_string());
        for _ in 0..2 {
            assert!(write(&path, &ledger).unwrap().is_empty());
            assert_eq!(names(&out), expected);
        }
        let last = beside.last().unwrap();
        let edited = out.join(format!("{stem}.{last}"));
        std::fs::write(&edited, "edited by hand").unwrap();
        assert_eq!(write(&path, &ledger).unwrap().len(), beside.len());
        assert_eq!(std::fs::read_to_string(&edited).unwrap(), "edited by hand");
    }
}
