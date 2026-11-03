//! A table of contents' document units (#50): made from its entries, corrected, and read,
//! analysed or exported one by one.

use std::path::Path;

use kalamu_core::{DocumentUnit, JobId, SetupId, SourceFile, SourceId, UnitId};
use kalamu_engine::{SeedReport, UnitEdit, UnitsView};
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn units_view(
    state: State<'_, AppState>,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
) -> CmdResult<UnitsView> {
    Ok(state.engine.units_view(job_id, setup_id, source_id)?)
}

/// One unit per corrected entry, once.
#[tauri::command]
pub fn seed_units(
    state: State<'_, AppState>,
    job_id: JobId,
    setup_id: SetupId,
    source_id: SourceId,
) -> CmdResult<SeedReport> {
    Ok(state.engine.seed_units(job_id, setup_id, source_id)?)
}

#[tauri::command]
pub fn save_unit(state: State<'_, AppState>, edit: UnitEdit) -> CmdResult<DocumentUnit> {
    Ok(state.engine.save_unit(edit)?)
}

#[tauri::command]
pub fn delete_unit(state: State<'_, AppState>, id: UnitId) -> CmdResult<()> {
    Ok(state.engine.delete_unit(id)?)
}

#[tauri::command]
pub fn join_units(
    state: State<'_, AppState>,
    first: UnitId,
    second: UnitId,
) -> CmdResult<DocumentUnit> {
    Ok(state.engine.join_units(first, second)?)
}

#[tauri::command]
pub fn split_unit(state: State<'_, AppState>, id: UnitId, at: u32) -> CmdResult<[DocumentUnit; 2]> {
    Ok(state.engine.split_unit(id, at)?)
}

/// The units' pages frozen as PDFs, as sources for a new job to read. Cutting a large volume's
/// pages takes a while, so it runs off the window's thread.
#[tauri::command]
pub async fn unit_sources(
    state: State<'_, AppState>,
    ids: Vec<UnitId>,
) -> CmdResult<Vec<SourceFile>> {
    let engine = state.engine.clone();
    tokio::task::spawn_blocking(move || Ok(engine.unit_sources(&ids)?))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}

/// Each unit's pages as a PDF in the chosen folder, with the list of them, off the window's
/// thread.
#[tauri::command]
pub async fn export_units(
    state: State<'_, AppState>,
    ids: Vec<UnitId>,
    folder: String,
) -> CmdResult<Vec<String>> {
    let engine = state.engine.clone();
    tokio::task::spawn_blocking(move || Ok(engine.export_units(&ids, Path::new(&folder))?))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))?
}
