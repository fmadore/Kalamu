//! The review and comparison views.

use std::sync::Arc;

use base64::Engine as _;
use kalamu_core::{Correction, JobId, SetupId, SourceId, SpeakerNames};
use kalamu_engine::{ComparisonReport, DiffSpan, JobScores, PieceAudio, PieceView};
use kalamu_media::OpenSource;
use tauri::{AppHandle, Manager, State};

use super::{AppState, PagePreview, off_thread};
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub async fn source_pieces(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<Vec<PieceView>> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.source_pieces(job_id, source_id)?)).await
}

/// Where to hear one piece of a recording in the review view. The file is served to the player
/// through the asset protocol, which streams it with range requests; only the files this
/// command names are opened to it, for the life of the app. `convert` asks for a copy FFmpeg
/// converted, when the player could not play the recording itself.
#[tauri::command]
pub async fn piece_audio(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
    convert: bool,
) -> CmdResult<PieceAudio> {
    let engine = state.engine.clone();
    let audio =
        tokio::task::spawn_blocking(move || engine.piece_audio(job_id, source_id, index, convert))
            .await
            .map_err(|e| AppError::new(id::INTERNAL, e))??;
    app.asset_protocol_scope()
        .allow_file(&audio.path)
        .map_err(|e| AppError::new(id::INTERNAL, e))?;
    Ok(audio)
}

/// The text of a text source (plain text, Markdown, CSV, Word), for reviewing its analysis.
#[tauri::command]
pub async fn source_text(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<String> {
    let job = state.engine.job(job_id)?;
    let source = job
        .sources
        .into_iter()
        .find(|s| s.id == source_id)
        .ok_or_else(|| AppError::new("job.not_found", source_id))?;
    tokio::task::spawn_blocking(move || match OpenSource::open(&source, None)? {
        OpenSource::Text(text) => text.text(),
        _ => Err(kalamu_media::MediaError::UnsupportedKind),
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
    .map_err(|e| AppError::new("media.read", e))
}

/// One page of a source as a single-page PDF or an image, for the review view.
#[tauri::command]
pub async fn piece_preview(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
) -> CmdResult<PagePreview> {
    let prepared = page_of(&state, job_id, source_id, index).await?;
    Ok(PagePreview {
        mime: prepared.mime,
        data: base64::engine::general_purpose::STANDARD.encode(prepared.bytes),
    })
}

/// One page of a source drawn `width` pixels wide, for the review's strip of thumbnails: a
/// small PNG or JPEG instead of the whole page that `piece_preview` sends.
#[tauri::command]
pub async fn piece_thumbnail(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
    width: u32,
) -> CmdResult<PagePreview> {
    let prepared = page_of(&state, job_id, source_id, index).await?;
    let (mime, bytes) = tokio::task::spawn_blocking(move || {
        kalamu_media::thumbnail(&prepared.bytes, &prepared.mime, width)
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
    .map_err(|e| AppError::new("media.read", e))?;
    Ok(PagePreview {
        mime,
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
    })
}

/// One page of a source as a single-page PDF or an image, its source opened once for the
/// pages after it.
async fn page_of(
    state: &State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
) -> CmdResult<kalamu_core::PreparedInput> {
    let job = state.engine.job(job_id)?;
    let source = job
        .sources
        .iter()
        .find(|s| s.id == source_id)
        .cloned()
        .ok_or_else(|| AppError::new("job.not_found", source_id))?;
    // Only scans and images are previewed, never a link: a path that is not a file was moved
    // or deleted since the job read it.
    if !std::path::Path::new(&source.path).is_file() {
        return Err(AppError::new(id::FILE_MISSING, &source.path));
    }
    let cached = state.preview.lock().ok().and_then(|c| {
        c.as_ref()
            .filter(|(id, _)| *id == source_id)
            .map(|(_, o)| o.clone())
    });
    let opened = match cached {
        Some(o) => o,
        None => {
            let s = source.clone();
            // Previews are of pages only; recordings need no FFmpeg here.
            let o = tokio::task::spawn_blocking(move || OpenSource::open(&s, None))
                .await
                .map_err(|e| AppError::new(id::INTERNAL, e))?
                .map_err(|e| AppError::new("media.read", e))?;
            let o = Arc::new(o);
            *state.preview.lock().unwrap_or_else(|p| p.into_inner()) = Some((source_id, o.clone()));
            o
        }
    };
    let pieces = opened.pieces(
        source.split_spreads,
        job.settings.sample_pages,
        kalamu_media::DEFAULT_PIECE_MS,
    );
    let piece = pieces
        .get(index as usize)
        .cloned()
        .ok_or_else(|| AppError::new("media.read", format!("no piece {index}")))?;
    tokio::task::spawn_blocking(move || {
        opened.prepare(
            &piece,
            &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            "preview",
        )
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
    .map_err(|e| AppError::new("media.read", e))
}

/// Stores a correction; the source's files are written again a few seconds after the last
/// one, or sooner when they are listed or exported.
#[tauri::command]
pub async fn save_correction(state: State<'_, AppState>, correction: Correction) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.save_correction(correction)?)).await
}

/// Stores several corrections at once, all or none: a change across set-ups put back.
#[tauri::command]
pub async fn save_corrections(
    state: State<'_, AppState>,
    corrections: Vec<Correction>,
) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.save_corrections(corrections)?)).await
}

/// Renames a name an analysis listed, or takes it out when `to` is none, in every set-up that
/// found it, all or none; gives back those set-ups' corrections as they stood, to put back.
#[tauri::command]
pub async fn rename_listed(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
    index: u32,
    key: String,
    from: String,
    to: Option<String>,
) -> CmdResult<Vec<Correction>> {
    let engine = state.engine.clone();
    off_thread(move || {
        Ok(engine.rename_listed(job_id, source_id, index, &key, &from, to.as_deref())?)
    })
    .await
}

/// Names the speakers of one set-up's transcription; the files are written again as after a
/// correction.
#[tauri::command]
pub async fn save_speaker_names(
    state: State<'_, AppState>,
    job_id: JobId,
    names: SpeakerNames,
) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.save_speaker_names(job_id, names)?)).await
}

#[tauri::command]
pub async fn export_source(
    state: State<'_, AppState>,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
) -> CmdResult<Vec<String>> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.export(job_id, setup_id, source_id)?)).await
}

#[tauri::command]
pub async fn compare_source(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<ComparisonReport> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.compare(job_id, source_id)?)).await
}

#[tauri::command]
pub async fn job_scores(state: State<'_, AppState>, job_id: JobId) -> CmdResult<JobScores> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.job_scores(job_id)?)).await
}

#[tauri::command]
pub fn diff_texts(left: String, right: String) -> Vec<DiffSpan> {
    kalamu_engine::diff(&left, &right)
}
