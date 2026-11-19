//! Reading a region of a page again with a model, from the review: what it would cost, the
//! request, and the researcher's decision on the reading. Thin wrappers over the engine, as the
//! passages' are.

use kalamu_core::{ReadingEffect, RegionAsk, RegionReading};
use kalamu_engine::PassageEstimate;
use tauri::State;

use super::{AppState, consented, off_thread};
use crate::errors::CmdResult;

/// What asking a model to read a region again would cost, and whether the answer is stored
/// already; refused as asking would be (a box off the page, a model that reads no pictures, the
/// words where the reading would go changed). Cuts the region from the original to know.
#[tauri::command]
pub async fn region_reading_estimate(
    state: State<'_, AppState>,
    ask: RegionAsk,
) -> CmdResult<PassageEstimate> {
    let engine = state.engine.clone();
    Ok(engine.region_reading_estimate(&ask).await?)
}

/// Asks a model to read a region of a page again, through the engine, and gives back its
/// reading; nothing is written until it is accepted. Refused without the researcher's consent
/// to send to the model's provider.
#[tauri::command]
pub async fn ask_region_reading(
    state: State<'_, AppState>,
    ask: RegionAsk,
) -> CmdResult<RegionReading> {
    consented(&state, &ask.provider)?;
    let engine = state.engine.clone();
    Ok(engine.ask_region_reading(ask).await?)
}

/// Stops waiting for a region's reading; an answer that still comes is kept for next time.
#[tauri::command]
pub fn cancel_region_reading(state: State<'_, AppState>, id: String) {
    state.engine.cancel_region_reading(&id);
}

/// Writes a reading (or the researcher's edit of it) where it goes: the text of the region
/// they drew, or the page's text in place of the region's words; gives back what it replaced,
/// for *Undo*.
#[tauri::command]
pub async fn accept_region_reading(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> CmdResult<ReadingEffect> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.accept_region_reading(&id, text)?)).await
}

/// Leaves a reading unused.
#[tauri::command]
pub async fn discard_region_reading(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.discard_region_reading(&id)?)).await
}

/// Puts back what an accepted reading replaced.
#[tauri::command]
pub async fn undo_region_reading(
    state: State<'_, AppState>,
    id: String,
    before: ReadingEffect,
) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.undo_region_reading(&id, before)?)).await
}
