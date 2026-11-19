//! Correcting a passage selected in the review with a model: what it would cost, the request,
//! and the researcher's decision on the proposal. Thin wrappers over the engine, as the regions
//! read again are.

use kalamu_core::{Correction, PassageAsk, PassageProposal};
use kalamu_engine::PassageEstimate;
use tauri::State;

use super::{AppState, consented, off_thread};
use crate::errors::CmdResult;

/// What asking a model to correct a passage selected in the review would cost, and whether the
/// answer is stored already; refused as asking would be (a passage too short or too long, a text
/// changed, a model that cannot correct text).
#[tauri::command]
pub async fn passage_estimate(
    state: State<'_, AppState>,
    ask: PassageAsk,
) -> CmdResult<PassageEstimate> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.passage_estimate(&ask)?)).await
}

/// Asks a model to correct a passage selected in the review, through the engine, and gives back
/// its proposal; nothing is written into the text until it is accepted. Refused without the
/// researcher's consent to send to the model's provider.
#[tauri::command]
pub async fn ask_passage(
    state: State<'_, AppState>,
    ask: PassageAsk,
) -> CmdResult<PassageProposal> {
    consented(&state, &ask.provider)?;
    let engine = state.engine.clone();
    Ok(engine.ask_passage(ask).await?)
}

/// Stops waiting for a passage's proposal; an answer that still comes is kept for next time.
#[tauri::command]
pub fn cancel_passage(state: State<'_, AppState>, id: String) {
    state.engine.cancel_passage(&id);
}

/// Writes a proposal (or the researcher's edit of it) into the corrections of its piece; gives
/// back the piece's correction as it stood, for *Undo*.
#[tauri::command]
pub async fn accept_passage(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> CmdResult<Correction> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.accept_passage(&id, text)?)).await
}

/// Leaves a proposal unused.
#[tauri::command]
pub async fn discard_passage(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.discard_passage(&id)?)).await
}

/// Puts back the correction an accepted passage replaced.
#[tauri::command]
pub async fn undo_passage(
    state: State<'_, AppState>,
    id: String,
    before: Correction,
) -> CmdResult<()> {
    let engine = state.engine.clone();
    off_thread(move || Ok(engine.undo_passage(&id, before)?)).await
}
