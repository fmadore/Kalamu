//! The review and comparison views.

use std::sync::Arc;

use base64::Engine as _;
use kalamu_core::{Correction, JobId, SetupId, SourceId, SpeakerNames};
use kalamu_engine::{ComparisonReport, DiffSpan, JobScores, PieceAudio, PieceView};
use kalamu_media::OpenSource;
use tauri::{AppHandle, Manager, State};

use super::{AppState, PagePreview};
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn source_pieces(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<Vec<PieceView>> {
    Ok(state.engine.source_pieces(job_id, source_id)?)
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
    let job = state.engine.job(job_id)?;
    let source = job
        .sources
        .iter()
        .find(|s| s.id == source_id)
        .cloned()
        .ok_or_else(|| AppError::new("job.not_found", source_id))?;
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
    let prepared = tokio::task::spawn_blocking(move || {
        opened.prepare(
            &piece,
            &["application/pdf", "image/png", "image/jpeg", "image/webp"],
            "preview",
        )
    })
    .await
    .map_err(|e| AppError::new(id::INTERNAL, e))?
    .map_err(|e| AppError::new("media.read", e))?;
    Ok(PagePreview {
        mime: prepared.mime,
        data: base64::engine::general_purpose::STANDARD.encode(prepared.bytes),
    })
}

#[tauri::command]
pub fn save_correction(
    state: State<'_, AppState>,
    correction: Correction,
) -> CmdResult<Vec<String>> {
    Ok(state.engine.save_correction(correction)?)
}

/// Names the speakers of one set-up's transcription; returns the files written again.
#[tauri::command]
pub fn save_speaker_names(
    state: State<'_, AppState>,
    job_id: JobId,
    names: SpeakerNames,
) -> CmdResult<Vec<String>> {
    Ok(state.engine.save_speaker_names(job_id, names)?)
}

#[tauri::command]
pub fn export_source(
    state: State<'_, AppState>,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
) -> CmdResult<Vec<String>> {
    Ok(state.engine.export(job_id, setup_id, source_id)?)
}

#[tauri::command]
pub fn compare_source(
    state: State<'_, AppState>,
    job_id: JobId,
    source_id: SourceId,
) -> CmdResult<ComparisonReport> {
    Ok(state.engine.compare(job_id, source_id)?)
}

#[tauri::command]
pub fn job_scores(state: State<'_, AppState>, job_id: JobId) -> CmdResult<JobScores> {
    Ok(state.engine.job_scores(job_id)?)
}

#[tauri::command]
pub fn diff_texts(left: String, right: String) -> Vec<DiffSpan> {
    kalamu_engine::diff(&left, &right)
}
