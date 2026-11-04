//! Adding files, estimating, and running jobs.

use std::path::{Path, PathBuf};

use kalamu_core::{InputKind, Job, JobId, JobSources, JobSummary, PieceState, SourceFile, Task};
use kalamu_engine::{Estimate, EstimateRequest, NewJob};
use tauri::State;

use super::{AppState, InspectedFile, off_thread};
use crate::errors::{AppError, CmdResult, id};

/// Files and folders dropped or chosen, expanded to the supported files for a task, sorted so
/// numbered scans keep their order. Each is kept for later launches where a sandbox requires
/// it, while the app may still reach it.
///
/// Walking folders can take a while on a network share, so it runs off the window's thread.
#[tauri::command]
pub async fn expand_paths(
    state: State<'_, AppState>,
    paths: Vec<String>,
    task: Task,
) -> CmdResult<Vec<String>> {
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

fn expand(paths: Vec<String>, task: Task) -> Vec<String> {
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
    for p in paths {
        walk(Path::new(&p), task, &mut out, 0);
    }
    // A folder dropped with a file inside it names that file twice: once each, in order.
    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.clone()));
    out.into_iter().map(|p| p.display().to_string()).collect()
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

/// The text files a finished job wrote, to summarise them in a new job.
#[tauri::command]
pub fn job_text_outputs(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<String>> {
    Ok(state.engine.text_outputs(id)?)
}

/// Every file a job wrote that is still on disk, for its page's "Written" list.
#[tauri::command]
pub fn job_outputs(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<String>> {
    Ok(state.engine.outputs(id)?)
}

/// Opens a file the job wrote in the app the system has for it; JSON opens in the plain text
/// editor every computer has (`platform`), as there is often no app for it. Only the job's own
/// files, so the interface cannot open anything else on the computer.
#[tauri::command]
pub fn open_output(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: JobId,
    path: String,
) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    if !state.engine.outputs(id)?.contains(&path) {
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

#[tauri::command]
pub fn job_pieces(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<PieceState>> {
    Ok(state.engine.pieces(id)?)
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
        let found = expand(vec![dir.path().display().to_string()], Task::TextExtraction);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("page-01.jpg") && !found[0].contains("._"));
    }
}
