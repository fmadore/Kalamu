//! Adding files, estimating, and running jobs.

use std::path::{Path, PathBuf};

use kalamu_core::{InputKind, Job, JobId, JobSources, JobSummary, SourceFile, SourceId, Task};
use kalamu_engine::{Estimate, EstimateRequest, NewJob};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::{AppState, InspectedFile, off_thread};
use crate::errors::{AppError, CmdResult, id};

/// Files and folders dropped or chosen, expanded to the supported files for a task, sorted so
/// numbered scans keep their order, with the files and folders that gave nothing the task can
/// read, so New job can say so rather than drop them unseen. Each is kept for later launches
/// where a sandbox requires it, while the app may still reach it.
///
/// Walking folders can take a while on a network share, so it runs off the window's thread.
#[tauri::command]
pub async fn expand_paths(
    state: State<'_, AppState>,
    paths: Vec<String>,
    task: Task,
) -> CmdResult<Expanded> {
    state.grants.keep(&paths);
    off_thread(move || Ok(expand(paths, task))).await
}

/// The folders results would be written to beside these files that the app may not write to
/// yet: in the Mac App Store's build, a file picked or dropped is granted, not its folder.
/// Always empty elsewhere.
#[tauri::command]
pub fn folders_to_ask(state: State<'_, AppState>, paths: Vec<String>) -> Vec<String> {
    state
        .grants
        .folders_to_ask(&paths)
        .into_iter()
        .map(|p| p.display().to_string())
        .collect()
}

/// Folders the researcher has just granted, kept for later launches.
#[tauri::command]
pub fn keep_access(state: State<'_, AppState>, paths: Vec<String>) {
    state.grants.keep(&paths);
}

/// What dropping or choosing files gave a task.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Expanded {
    /// The files the task reads, in order.
    pub paths: Vec<String>,
    /// The files and folders dropped or chosen that gave none: a file of another kind, or a
    /// folder with nothing the task reads inside. Files of another kind inside a folder are not
    /// listed: a folder of scans often holds a stray file or two.
    pub refused: Vec<Refused>,
}

/// A file or folder that gave the task nothing to read.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Refused {
    pub path: String,
    pub file_name: String,
    /// The kind of a file Kalamu reads in another task; `None` for a folder or a file of a
    /// kind Kalamu does not read at all.
    pub kind: Option<InputKind>,
    pub folder: bool,
}

fn expand(paths: Vec<String>, task: Task) -> Expanded {
    fn walk(path: &Path, task: Task, out: &mut Vec<PathBuf>, depth: usize) {
        if is_apple_double(path) {
            return;
        }
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
        } else if InputKind::from_path(path).is_some_and(|k| task.reads_source(k)) {
            out.push(path.to_path_buf());
        }
    }
    let mut out = Vec::new();
    let mut refused = Vec::new();
    for p in paths {
        let path = Path::new(&p);
        let before = out.len();
        walk(path, task, &mut out, 0);
        if out.len() == before && !is_apple_double(path) {
            let folder = path.is_dir();
            refused.push(Refused {
                file_name: path
                    .file_name()
                    .map_or_else(|| p.clone(), |n| n.to_string_lossy().into_owned()),
                kind: if folder {
                    None
                } else {
                    InputKind::from_path(path)
                },
                folder,
                path: p,
            });
        }
    }
    // A folder dropped with a file inside it names that file twice: once each, in order.
    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.clone()));
    Expanded {
        paths: out.into_iter().map(|p| p.display().to_string()).collect(),
        refused,
    }
}

/// The `._name.pdf` beside `name.pdf` that macOS writes on USB drives and network shares to
/// hold the file's metadata: it has the extension of the file but none of its content, and
/// the folder reaches Windows PCs too.
fn is_apple_double(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("._"))
}

/// Freezes the corrected text of a job's sources under one set-up, as sources a new analysis
/// reads without reading the originals again.
#[tauri::command]
pub async fn snapshot_job(
    state: State<'_, AppState>,
    id: JobId,
    setup: kalamu_core::SetupId,
    source: Option<kalamu_core::SourceId>,
) -> CmdResult<Vec<SourceFile>> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.snapshot_sources(id, setup, source)?)).await
}

/// For a job that read frozen text, whether each text has been corrected since.
#[tauri::command]
pub fn derived_state(
    state: State<'_, AppState>,
    id: JobId,
) -> CmdResult<Vec<kalamu_engine::DerivedState>> {
    Ok(state.engine.derived_states(id)?)
}

/// The text files a finished job wrote, to summarise them in a new job. Files left out of date
/// by corrections are written first, off the window's thread.
#[tauri::command]
pub async fn job_text_outputs(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<String>> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.text_outputs(id)?)).await
}

/// Every file a job wrote that is still on disk, for its page's "Written" list. Files left out
/// of date by corrections are written first, off the window's thread.
#[tauri::command]
pub async fn job_outputs(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<String>> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.outputs(id)?)).await
}

/// Opens a file the job wrote in the app the system has for it; JSON opens in the plain text
/// editor every computer has (`platform`), as there is often no app for it. Only the job's own
/// files, so the interface cannot open anything else on the computer.
#[tauri::command]
pub async fn open_output(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: JobId,
    path: String,
) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let engine = state.engine.clone();
    if !off_thread(move || Ok(engine.outputs(id)?))
        .await?
        .contains(&path)
    {
        return Err(AppError::new(id::OPEN, &path));
    }
    let with = path
        .to_ascii_lowercase()
        .ends_with(".json")
        .then_some(crate::platform::JSON_APP)
        .flatten();
    app.opener()
        .open_path(&path, with)
        .map_err(|e| AppError::new(id::OPEN, e))
}

/// Opens the original file of a page of one of the job's sources, in the app the system has
/// for it, from the review. Only files the job read, so the interface cannot open anything else
/// on the computer.
#[tauri::command]
pub async fn open_original(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: JobId,
    path: String,
) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let job = state.engine.job(id)?;
    if !job
        .sources
        .iter()
        .any(|s| s.files().contains(&path.as_str()))
    {
        return Err(AppError::new(id::OPEN, &path));
    }
    if !std::path::Path::new(&path).is_file() {
        return Err(AppError::new(id::FILE_MISSING, &path));
    }
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| AppError::new(id::OPEN, e))
}

/// Image files already inspected, made one document whose pages they are: in the natural order
/// of their names, with what their numbers say and every page's checks. Nothing is read again
/// but the images' sizes, for the checks.
#[tauri::command]
pub async fn image_document(
    state: State<'_, AppState>,
    name: String,
    sources: Vec<SourceFile>,
) -> CmdResult<kalamu_engine::ImageDocument> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.image_document(&name, &sources)?)).await
}

#[tauri::command]
pub async fn inspect_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> CmdResult<Vec<InspectedFile>> {
    state.grants.keep(&paths);
    let engine = state.engine.clone();
    tokio::task::spawn_blocking(move || {
        engine
            .inspect(&paths)
            .into_iter()
            .zip(paths)
            .map(|(result, path)| match result {
                Ok((source, warnings)) => InspectedFile {
                    path,
                    pieces: kalamu_engine::Engine::recording_pieces(&source),
                    source: Some(source),
                    warnings,
                    error: None,
                },
                Err(e) => InspectedFile {
                    path,
                    source: None,
                    warnings: Vec::new(),
                    error: Some(e.into()),
                    pieces: None,
                },
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))
}

/// YouTube links pasted for a new job, each checked; `path` is the link as pasted. Each video's
/// title is asked of YouTube's public oEmbed endpoint, which needs no key and also says when a
/// video is private or gone. Without a connection the links are kept, untitled.
#[tauri::command]
pub async fn inspect_links(
    state: State<'_, AppState>,
    links: Vec<String>,
) -> CmdResult<Vec<InspectedFile>> {
    let checked = state.engine.inspect_links(&links);
    let mut lookups = tokio::task::JoinSet::new();
    for (index, result) in checked.iter().enumerate() {
        if let Ok(source) = result {
            let (http, url) = (state.http.clone(), source.path.clone());
            lookups.spawn(async move { (index, youtube_title(&http, &url).await) });
        }
    }
    let mut found = vec![Ok(None); checked.len()];
    while let Some(done) = lookups.join_next().await {
        let (index, title) = done.map_err(|e| AppError::new(id::INTERNAL, e))?;
        found[index] = title;
    }
    Ok(checked
        .into_iter()
        .zip(found)
        .zip(links)
        .map(|((result, title), link)| {
            let (source, error) = match (result, title) {
                (Ok(source), Ok(Some(title))) => (Some(source.with_title(&title)), None),
                (Ok(source), Ok(None)) => (Some(source), None),
                (Ok(_), Err(refused)) => (
                    None,
                    Some(AppError {
                        id: refused.into(),
                        detail: None,
                    }),
                ),
                (Err(e), _) => (None, Some(e.into())),
            };
            InspectedFile {
                path: link,
                source,
                warnings: Vec::new(),
                error,
                // Gemini reads a video whole.
                pieces: None,
            }
        })
        .collect())
}

/// A video's title from YouTube's oEmbed endpoint; `Ok(None)` when YouTube could not be asked.
async fn youtube_title(http: &reqwest::Client, url: &str) -> Result<Option<String>, &'static str> {
    let response = http
        .get(format!(
            "https://www.youtube.com/oembed?format=json&url={}",
            encode(url)
        ))
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await;
    let Ok(response) = response else {
        return Ok(None);
    };
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    oembed_title(status, &body)
}

/// Percent-encodes a query value, keeping only URL-safe characters as they are.
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// What oEmbed's answer says about a video: its title, or why it cannot be read. Unlisted videos
/// answer like public ones, so Gemini may still refuse them.
fn oembed_title(status: u16, body: &str) -> Result<Option<String>, &'static str> {
    match status {
        200 => Ok(serde_json::from_str::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("title")?.as_str().map(str::to_string))
            .filter(|t| !t.trim().is_empty())),
        401 | 403 => Err(id::SOURCE_PRIVATE),
        400 | 404 => Err(id::SOURCE_MISSING),
        _ => Ok(None),
    }
}

#[tauri::command]
pub fn estimate_cost(state: State<'_, AppState>, request: EstimateRequest) -> Estimate {
    state.engine.estimate(&request)
}

#[tauri::command]
pub fn create_job(state: State<'_, AppState>, job: NewJob) -> CmdResult<Job> {
    super::require_consent(&state, &job.setups)?;
    state
        .grants
        .keep(super::settings::destination_folder(&job.output.destination));
    Ok(state.engine.create_job(job)?)
}

// `async`, like every command that starts the engine: Tauri runs other commands on the main
// thread, outside the Tokio runtime the job is spawned on.
#[tauri::command]
pub async fn start_job(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.start(id)?)
}

#[tauri::command]
pub fn rename_job(state: State<'_, AppState>, id: JobId, name: String) -> CmdResult<()> {
    Ok(state.engine.rename_job(id, &name)?)
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
pub async fn retry_failed(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.retry_failed(id)?)
}

// `async`: a job waiting for its batches asks the provider to stop them first, so a deleted
// job is not billed for answers nobody will read.
#[tauri::command]
pub async fn delete_job(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    state.engine.cancel(id);
    if let Err(e) = state.engine.cancel_batches(id).await {
        tracing::warn!(job = %id, error = %e, "batches not cancelled");
    }
    state.engine.delete_job(id)?;
    // Its passages leave the search index with it; one missed is pruned at the next start.
    if let Err(e) = state.search.forget_job(id) {
        tracing::warn!(job = %id, error = %e, "passages not removed from the search index");
    }
    Ok(())
}

/// Deletes the jobs chosen on the Jobs page, all or none; their batches are cancelled and their
/// passages leave the search index as [`delete_job`] does for one.
#[tauri::command]
pub async fn delete_jobs(state: State<'_, AppState>, ids: Vec<JobId>) -> CmdResult<()> {
    for &id in &ids {
        state.engine.cancel(id);
        if let Err(e) = state.engine.cancel_batches(id).await {
            tracing::warn!(job = %id, error = %e, "batches not cancelled");
        }
    }
    state.engine.delete_jobs(&ids)?;
    for id in ids {
        if let Err(e) = state.search.forget_job(id) {
            tracing::warn!(job = %id, error = %e, "passages not removed from the search index");
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> CmdResult<Vec<JobSummary>> {
    Ok(state.engine.jobs()?)
}

/// The names of every job's sources, for the Jobs page's search.
#[tauri::command]
pub fn job_sources(state: State<'_, AppState>) -> CmdResult<Vec<JobSources>> {
    Ok(state.engine.job_sources()?)
}

#[tauri::command]
pub fn get_job(state: State<'_, AppState>, id: JobId) -> CmdResult<Job> {
    Ok(state.engine.job(id)?)
}

/// A job's progress for its page: the pieces held in batches, and each of `sources` counted
/// under every set-up. Read again as progress arrives, so it sends counts, not pieces.
#[tauri::command]
pub async fn job_progress(
    state: State<'_, AppState>,
    id: JobId,
    sources: Vec<SourceId>,
) -> CmdResult<kalamu_engine::JobProgress> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.job_progress(id, &sources)?)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oembed_gives_the_title_or_says_why_a_video_cannot_be_read() {
        assert_eq!(
            oembed_title(
                200,
                r#"{"title":"Prêche du vendredi","author_name":"Mosquée"}"#
            ),
            Ok(Some("Prêche du vendredi".to_string()))
        );
        assert_eq!(oembed_title(401, "Unauthorized"), Err(id::SOURCE_PRIVATE));
        assert_eq!(oembed_title(404, "Not Found"), Err(id::SOURCE_MISSING));
        // YouTube busy or changed: the link is kept, untitled.
        assert_eq!(oembed_title(503, ""), Ok(None));
        assert_eq!(oembed_title(200, "not json"), Ok(None));
        assert_eq!(
            encode("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            "https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ"
        );
    }
    #[test]
    #[allow(clippy::unwrap_used)]
    fn a_folder_expands_without_the_metadata_files_macos_leaves() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("page-01.jpg"), b"jpeg").unwrap();
        std::fs::write(dir.path().join("._page-01.jpg"), b"metadata").unwrap();
        let found = expand(vec![dir.path().display().to_string()], Task::TextExtraction).paths;
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("page-01.jpg") && !found[0].contains("._"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn files_the_task_cannot_read_are_named_with_their_kind() {
        let dir = tempfile::tempdir().unwrap();
        let scan = dir.path().join("page-01.jpg");
        let interview = dir.path().join("interview.mp3");
        let archive = dir.path().join("letters.zip");
        let empty = dir.path().join("empty");
        for f in [&scan, &interview, &archive] {
            std::fs::write(f, b"bytes").unwrap();
        }
        std::fs::create_dir(&empty).unwrap();
        let all = [&scan, &interview, &archive, &empty].map(|p| p.display().to_string());
        let found = expand(all.to_vec(), Task::TextExtraction);
        assert_eq!(found.paths, vec![all[0].clone()]);
        let refused: Vec<_> = found
            .refused
            .iter()
            .map(|r| (r.file_name.as_str(), r.kind, r.folder))
            .collect();
        assert_eq!(
            refused,
            vec![
                ("interview.mp3", Some(InputKind::Audio), false),
                ("letters.zip", None, false),
                ("empty", None, true),
            ]
        );
    }
}
