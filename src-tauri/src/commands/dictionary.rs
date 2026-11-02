//! A project's dictionary: its names gathered from the analyses, and the researcher's entries.
//! Reading and deciding, and asking a model to propose merges, which runs as an ordinary job
//! through the engine; nothing here changes a result, and no proposal is applied unasked.

use kalamu_core::{
    CollectionId, Entry, EntryId, Job, JobSettings, OutputDestination, OutputSettings, Pace,
    ProjectId, SourceFile, Timestamp,
};
use kalamu_dictionary::{DictionaryView, Evidence, Item, ProposalsView};
use kalamu_engine::{NewJob, SetupRequest};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

/// What a request for proposals would send: the rows in one text, or in groups when they are
/// too many for one request, as the sources of a job, for its estimate.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ProposalPlan {
    pub sources: Vec<SourceFile>,
    pub rows: u32,
}

/// The groups of a field's rows written as texts the engine reads, the same texts each time
/// for the same rows.
fn prepared(
    state: &AppState,
    project: ProjectId,
    collection: Option<CollectionId>,
    namespace: &str,
) -> CmdResult<(kalamu_dictionary::Prepared, Vec<SourceFile>)> {
    let prepared = state
        .dictionary
        .prepare_proposals(project, collection, namespace)?;
    let texts: Vec<(String, String)> = prepared
        .groups
        .iter()
        .map(|g| (g.file_name.clone(), g.text.clone()))
        .collect();
    let sources = state.engine.text_sources(&texts)?;
    Ok((prepared, sources))
}

#[tauri::command]
pub fn dictionary_plan(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: Option<CollectionId>,
    namespace: String,
) -> CmdResult<ProposalPlan> {
    let (prepared, sources) = prepared(&state, project, collection, &namespace)?;
    Ok(ProposalPlan {
        sources,
        rows: prepared.rows,
    })
}

/// Asks a model to propose merges among a field's rows: a job of its own, named in the
/// interface's language, kept with the rows each of its sources sent, and started at once. It
/// writes only its provenance, into a folder of its own: the dictionary reads its answers from
/// the store, and the rows' texts lie in the snapshots folder, which never changes. Async, like
/// every command that starts the engine.
#[tauri::command]
pub async fn dictionary_propose(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: Option<CollectionId>,
    namespace: String,
    name: String,
    setup: SetupRequest,
    pace: Pace,
) -> CmdResult<Job> {
    let (prepared, sources) = prepared(&state, project, collection, &namespace)?;
    let sha256: Vec<String> = sources.iter().map(|s| s.sha256.clone()).collect();
    let folder = state
        .paths
        .data
        .join("dictionary")
        .join(project.to_string())
        .join(Timestamp::now().0.to_string());
    std::fs::create_dir_all(&folder).map_err(|e| AppError::new(id::INTERNAL, e))?;
    let job = state.engine.create_job(NewJob {
        name,
        task: kalamu_core::Task::Analysis,
        sources,
        setups: vec![SetupRequest {
            preset_id: Some(kalamu_prompts::DICTIONARY_MERGES.to_string()),
            ..setup
        }],
        settings: JobSettings {
            pace,
            ..JobSettings::default()
        },
        output: OutputSettings {
            formats: Vec::new(),
            destination: OutputDestination::Folder(folder.to_string_lossy().into_owned()),
            ..OutputSettings::default()
        },
    })?;
    // A job the dictionary does not know would be paid for and never read.
    if let Err(e) = state
        .dictionary
        .record_request(project, collection, &prepared, job.id, &sha256)
    {
        let _ = state.engine.delete_job(job.id);
        return Err(e.into());
    }
    state.engine.start(job.id)?;
    Ok(job)
}

#[tauri::command]
pub fn dictionary_proposals(
    state: State<'_, AppState>,
    project: ProjectId,
) -> CmdResult<ProposalsView> {
    Ok(state.dictionary.proposals(project)?)
}

#[tauri::command]
pub fn dictionary_dismiss(
    state: State<'_, AppState>,
    project: ProjectId,
    namespace: String,
    key: String,
    dismissed: bool,
) -> CmdResult<()> {
    Ok(state
        .dictionary
        .dismiss(project, &namespace, &key, dismissed)?)
}

/// Forgets a request and its proposals; the job stays among the jobs.
#[tauri::command]
pub fn dictionary_forget_request(
    state: State<'_, AppState>,
    project: ProjectId,
    request: String,
) -> CmdResult<()> {
    Ok(state.dictionary.forget_request(project, &request)?)
}

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
    let write = super::write_text;
    write(
        path.with_extension("json"),
        state.dictionary.export_json(project)?,
    )?;
    write(
        path.with_extension("csv"),
        state.dictionary.export_csv(project)?,
    )
}
