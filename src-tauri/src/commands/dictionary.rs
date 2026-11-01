//! A project's dictionary: its names gathered from the analyses, and the researcher's entries.
//! Reading and deciding only: none of these calls a model or changes a result.

use kalamu_core::{CollectionId, Entry, EntryId, ProjectId};
use kalamu_dictionary::{DictionaryView, Evidence, Item};
use tauri::State;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn dictionary_view(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: Option<CollectionId>,
) -> CmdResult<DictionaryView> {
    Ok(state.dictionary.view(project, collection)?)
}

#[tauri::command]
pub fn dictionary_evidence(
    state: State<'_, AppState>,
    project: ProjectId,
    namespace: String,
    forms: Vec<String>,
) -> CmdResult<Vec<Evidence>> {
    Ok(state.dictionary.evidence(project, &namespace, &forms)?)
}

#[tauri::command]
pub fn dictionary_merge(
    state: State<'_, AppState>,
    project: ProjectId,
    namespace: String,
    items: Vec<Item>,
    label: String,
) -> CmdResult<Entry> {
    Ok(state
        .dictionary
        .merge(project, &namespace, &items, &label)?)
}

#[tauri::command]
pub fn dictionary_split(
    state: State<'_, AppState>,
    project: ProjectId,
    entry: EntryId,
    forms: Vec<String>,
) -> CmdResult<()> {
    Ok(state.dictionary.split(project, entry, &forms)?)
}

#[tauri::command]
pub fn dictionary_relabel(
    state: State<'_, AppState>,
    project: ProjectId,
    entry: EntryId,
    label: String,
) -> CmdResult<Entry> {
    Ok(state.dictionary.relabel(project, entry, &label)?)
}

#[tauri::command]
pub fn dictionary_set_reviewed(
    state: State<'_, AppState>,
    project: ProjectId,
    namespace: String,
    items: Vec<Item>,
    reviewed: bool,
) -> CmdResult<()> {
    Ok(state
        .dictionary
        .set_reviewed(project, &namespace, &items, reviewed)?)
}

#[tauri::command]
pub fn dictionary_keep_apart(
    state: State<'_, AppState>,
    project: ProjectId,
    namespace: String,
    a: Item,
    b: Item,
) -> CmdResult<()> {
    Ok(state.dictionary.keep_apart(project, &namespace, &a, &b)?)
}

#[tauri::command]
pub fn dictionary_undo_apart(
    state: State<'_, AppState>,
    project: ProjectId,
    a: EntryId,
    b: EntryId,
) -> CmdResult<()> {
    Ok(state.dictionary.undo_apart(project, a, b)?)
}

/// Writes the entries as `.json` and every listed form with its entry as `.csv`, beside each
/// other at the path chosen.
#[tauri::command]
pub fn export_dictionary(
    state: State<'_, AppState>,
    project: ProjectId,
    path: String,
) -> CmdResult<()> {
    let path = std::path::Path::new(&path);
    let write = |path: std::path::PathBuf, text: String| {
        std::fs::write(path, text).map_err(|e| AppError::new(id::FILE, e))
    };
    write(
        path.with_extension("json"),
        state.dictionary.export_json(project)?,
    )?;
    write(
        path.with_extension("csv"),
        state.dictionary.export_csv(project)?,
    )
}
