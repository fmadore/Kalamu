//! Adding files, estimating, and running jobs.

use std::path::{Path, PathBuf};

use kalamu_catalog::CostEstimate;
use kalamu_core::{InputKind, Job, JobId, JobSummary, PieceState, Task};
use kalamu_engine::NewJob;
use tauri::State;

use super::{AppState, EstimateRequest, EstimateRow, InspectedFile};
use crate::errors::{AppError, CmdResult, id};

/// Files and folders dropped or chosen, expanded to the supported files for a task, sorted so
/// numbered scans keep their order.
#[tauri::command]
pub fn expand_paths(paths: Vec<String>, task: Task) -> Vec<String> {
    fn wanted(task: Task, kind: InputKind) -> bool {
        match task {
            Task::TextExtraction => matches!(kind, InputKind::Pdf | InputKind::Image),
            Task::AudioTranscription => matches!(kind, InputKind::Audio | InputKind::Video),
            Task::Analysis => matches!(kind, InputKind::Text | InputKind::Pdf | InputKind::Image),
        }
    }
    fn walk(path: &Path, task: Task, out: &mut Vec<PathBuf>, depth: usize) {
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
        } else if InputKind::from_path(path).is_some_and(|k| wanted(task, k)) {
            out.push(path.to_path_buf());
        }
    }
    let mut out = Vec::new();
    for p in paths {
        walk(Path::new(&p), task, &mut out, 0);
    }
    out.dedup();
    out.into_iter().map(|p| p.display().to_string()).collect()
}

/// The text files a finished job wrote, to summarise them in a new job.
#[tauri::command]
pub fn job_text_outputs(state: State<'_, AppState>, id: JobId) -> CmdResult<Vec<String>> {
    Ok(state.engine.text_outputs(id)?)
}

#[tauri::command]
pub async fn inspect_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> CmdResult<Vec<InspectedFile>> {
    let engine = state.engine.clone();
    tokio::task::spawn_blocking(move || {
        engine
            .inspect(&paths)
            .into_iter()
            .zip(paths)
            .map(|(result, path)| match result {
                Ok((source, warnings)) => InspectedFile {
                    path,
                    source: Some(source),
                    warnings,
                    error: None,
                },
                Err(e) => InspectedFile {
                    path,
                    source: None,
                    warnings: Vec::new(),
                    error: Some(e.into()),
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
pub fn estimate_cost(state: State<'_, AppState>, request: EstimateRequest) -> Vec<EstimateRow> {
    let catalog = state.engine.catalog();
    let pages: u32 = request
        .sources
        .iter()
        .map(|s| {
            let n = s.page_count.unwrap_or(1) * if s.split_spreads { 2 } else { 1 };
            request.sample_pages.map_or(n, |cap| n.min(cap))
        })
        .sum();
    // A YouTube video's length is known only once the provider has read it.
    let unknown_length = request.sources.iter().any(|s| s.kind == InputKind::YouTube);
    let minutes: f64 = request
        .sources
        .iter()
        .map(|s| s.duration_ms.unwrap_or(0) as f64 / 60_000.0)
        .sum();
    request
        .setups
        .iter()
        .map(|setup| {
            let prompt_chars = setup
                .preset_id
                .as_ref()
                .and_then(|id| state.engine.with_prompts(|p| p.get(id)).ok())
                .map_or(2_000, |p| p.instructions.len());
            let estimate = match catalog.model(&setup.provider, &setup.model) {
                _ if unknown_length => CostEstimate {
                    usd: None,
                    economy_usd: None,
                    note: Some("A YouTube video's length is known once Gemini has read it.".into()),
                },
                Some(model) => {
                    let mut estimate =
                        catalog.estimate(model, request.task, pages, minutes, prompt_chars);
                    // Economy mode only where Kalamu sends this model's work as a batch.
                    let batches = state
                        .engine
                        .provider(&setup.provider)
                        .is_ok_and(|p| p.batches(request.task, &setup.model));
                    if !batches {
                        estimate.economy_usd = None;
                    }
                    estimate
                }
                None => CostEstimate {
                    usd: None,
                    economy_usd: None,
                    note: Some("This model is not in the catalogue.".into()),
                },
            };
            EstimateRow {
                provider: setup.provider.clone(),
                model: setup.model.clone(),
                pages,
                estimate,
            }
        })
        .collect()
}

#[tauri::command]
pub fn create_job(state: State<'_, AppState>, job: NewJob) -> CmdResult<Job> {
    Ok(state.engine.create_job(job)?)
}

// `async`, like every command that starts the engine: Tauri runs other commands on the main
// thread, outside the Tokio runtime the job is spawned on.
#[tauri::command]
pub async fn start_job(state: State<'_, AppState>, id: JobId) -> CmdResult<()> {
    Ok(state.engine.start(id)?)
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
    Ok(state.engine.delete_job(id)?)
}

#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> CmdResult<Vec<JobSummary>> {
    Ok(state.engine.jobs()?)
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
}
