//! Project workspaces: projects, collections, documents and their ties to jobs. Organising only:
//! none of these calls a model, moves a file or changes a result.

use kalamu_core::{
    Collection, CollectionId, DocumentId, JobId, Project, ProjectId, ReviewQueue, SetupId,
    SourceDescription, SourceFile, SourceId, Task,
};
use kalamu_projects::{
    AttachChoice, DocumentDetail, ProjectSummary, ProjectView, QueueSelection, ResultChoice,
    ResultKey, SourceMatch,
};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::AppState;
use crate::errors::{AppError, CmdResult, id};

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> CmdResult<Vec<ProjectSummary>> {
    Ok(state.projects.list()?)
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    name: String,
    description: String,
) -> CmdResult<Project> {
    Ok(state.projects.create(&name, &description)?)
}

#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    id: ProjectId,
    name: String,
    description: String,
) -> CmdResult<Project> {
    Ok(state.projects.update(id, &name, &description)?)
}

#[tauri::command]
pub fn archive_project(
    state: State<'_, AppState>,
    id: ProjectId,
    archived: bool,
) -> CmdResult<Project> {
    Ok(state.projects.set_archived(id, archived)?)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: ProjectId) -> CmdResult<()> {
    Ok(state.projects.delete(id)?)
}

#[tauri::command]
pub fn project_view(state: State<'_, AppState>, id: ProjectId) -> CmdResult<ProjectView> {
    Ok(state.projects.view(id)?)
}

#[tauri::command]
pub fn create_collection(
    state: State<'_, AppState>,
    project: ProjectId,
    name: String,
) -> CmdResult<Collection> {
    Ok(state.projects.create_collection(project, &name)?)
}

#[tauri::command]
pub fn rename_collection(
    state: State<'_, AppState>,
    project: ProjectId,
    id: CollectionId,
    name: String,
) -> CmdResult<()> {
    Ok(state.projects.rename_collection(project, id, &name)?)
}

#[tauri::command]
pub fn delete_collection(
    state: State<'_, AppState>,
    project: ProjectId,
    id: CollectionId,
) -> CmdResult<()> {
    Ok(state.projects.delete_collection(project, id)?)
}

/// Files inspected by `inspect_files`, added as documents.
#[tauri::command]
pub fn add_documents(
    state: State<'_, AppState>,
    project: ProjectId,
    sources: Vec<SourceFile>,
    collection: Option<CollectionId>,
) -> CmdResult<Vec<DocumentId>> {
    Ok(state.projects.add_documents(project, sources, collection)?)
}

#[tauri::command]
pub fn rename_document(
    state: State<'_, AppState>,
    project: ProjectId,
    id: DocumentId,
    title: String,
) -> CmdResult<()> {
    Ok(state.projects.rename_document(project, id, &title)?)
}

#[tauri::command]
pub fn delete_documents(
    state: State<'_, AppState>,
    project: ProjectId,
    ids: Vec<DocumentId>,
) -> CmdResult<()> {
    Ok(state.projects.delete_documents(project, &ids)?)
}

#[tauri::command]
pub fn add_to_collection(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: CollectionId,
    ids: Vec<DocumentId>,
) -> CmdResult<()> {
    Ok(state
        .projects
        .add_to_collection(project, collection, &ids)?)
}

#[tauri::command]
pub fn remove_from_collection(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: CollectionId,
    ids: Vec<DocumentId>,
) -> CmdResult<()> {
    Ok(state
        .projects
        .remove_from_collection(project, collection, &ids)?)
}

#[tauri::command]
pub fn tag_documents(
    state: State<'_, AppState>,
    project: ProjectId,
    ids: Vec<DocumentId>,
    tag: String,
) -> CmdResult<()> {
    Ok(state.projects.tag_documents(project, &ids, &tag)?)
}

#[tauri::command]
pub fn untag_documents(
    state: State<'_, AppState>,
    project: ProjectId,
    ids: Vec<DocumentId>,
    tag: String,
) -> CmdResult<()> {
    Ok(state.projects.untag_documents(project, &ids, &tag)?)
}

/// Sets a document's own description of its source.
#[tauri::command]
pub fn describe_document(
    state: State<'_, AppState>,
    project: ProjectId,
    id: DocumentId,
    description: SourceDescription,
) -> CmdResult<()> {
    Ok(state.projects.describe_document(project, id, description)?)
}

/// Sets the parts of the description a project's documents share.
#[tauri::command]
pub fn set_source_defaults(
    state: State<'_, AppState>,
    id: ProjectId,
    defaults: SourceDescription,
) -> CmdResult<Project> {
    Ok(state.projects.set_source_defaults(id, defaults)?)
}

/// Renames a tag on every document of the project; returns how many changed.
#[tauri::command]
pub fn rename_tag(
    state: State<'_, AppState>,
    project: ProjectId,
    from: String,
    to: String,
) -> CmdResult<u32> {
    Ok(state.projects.rename_tag(project, &from, &to)?)
}

/// Writes the documents' descriptions where the researcher chose, as `.json` and `.csv`:
/// every part with where it came from, or the columns of a saved export profile.
#[tauri::command]
pub fn export_descriptions(
    state: State<'_, AppState>,
    project: ProjectId,
    collection: Option<CollectionId>,
    profile: Option<String>,
    path: String,
) -> CmdResult<()> {
    let path = std::path::Path::new(&path);
    let write = super::write_text;
    if let Some(id) = profile {
        let profile = state
            .profiles
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&id)?;
        let projects = &state.projects;
        write(
            path.with_extension("json"),
            projects.export_profiled_json(project, collection, &profile)?,
        )?;
        return write(
            path.with_extension("csv"),
            projects.export_profiled_csv(project, collection, &profile)?,
        );
    }
    write(
        path.with_extension("json"),
        state
            .projects
            .export_descriptions_json(project, collection)?,
    )?;
    write(
        path.with_extension("csv"),
        state
            .projects
            .export_descriptions_csv(project, collection)?,
    )
}

#[tauri::command]
pub fn merge_documents(
    state: State<'_, AppState>,
    project: ProjectId,
    from: DocumentId,
    into: DocumentId,
) -> CmdResult<()> {
    Ok(state.projects.merge_documents(project, from, into)?)
}

#[tauri::command]
pub fn document_detail(
    state: State<'_, AppState>,
    project: ProjectId,
    id: DocumentId,
) -> CmdResult<DocumentDetail> {
    Ok(state.projects.document(project, id)?)
}

#[tauri::command]
pub fn set_preferred(
    state: State<'_, AppState>,
    project: ProjectId,
    document: DocumentId,
    task: Task,
    job: Option<JobId>,
    setup: Option<SetupId>,
) -> CmdResult<()> {
    let choice = job.zip(setup);
    Ok(state
        .projects
        .set_preferred(project, document, task, choice)?)
}

#[tauri::command]
pub fn attach_plan(
    state: State<'_, AppState>,
    project: ProjectId,
    job: JobId,
) -> CmdResult<Vec<SourceMatch>> {
    Ok(state.projects.attach_plan(project, job)?)
}

#[tauri::command]
pub fn attach_job(
    state: State<'_, AppState>,
    project: ProjectId,
    job: JobId,
    choices: Vec<AttachChoice>,
    collection: Option<CollectionId>,
) -> CmdResult<Vec<DocumentId>> {
    Ok(state.projects.attach(project, job, &choices, collection)?)
}

/// Ties a job just created from the project to its documents.
#[tauri::command]
pub fn link_job(
    state: State<'_, AppState>,
    project: ProjectId,
    job: JobId,
    collection: Option<CollectionId>,
) -> CmdResult<Vec<DocumentId>> {
    Ok(state.projects.link_job(project, job, collection)?)
}

#[tauri::command]
pub fn detach_job(
    state: State<'_, AppState>,
    project: ProjectId,
    document: DocumentId,
    job: JobId,
    source: SourceId,
) -> CmdResult<()> {
    Ok(state.projects.detach(project, document, job, source)?)
}

/// The review queue of a project, a collection or a selection, with the results it reads.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct QueueView {
    pub queue: ReviewQueue,
    /// Results that stand for documents without a preferred one, which the researcher can set
    /// aside.
    pub choices: Vec<ResultChoice>,
    /// Documents and tasks read through a preferred result.
    pub preferred: u32,
}

/// The pages and answers that need the researcher, read from what is stored: no model is
/// called. The project chooses the results, the engine says what each needs.
#[tauri::command]
pub async fn review_queue(
    state: State<'_, AppState>,
    project: ProjectId,
    selection: QueueSelection,
    excluded: Vec<ResultKey>,
) -> CmdResult<QueueView> {
    let plan = state.projects.queue_plan(project, &selection, &excluded)?;
    let engine = state.engine.clone();
    let scope = plan.scope;
    let queue = tokio::task::spawn_blocking(move || engine.review_queue(&scope))
        .await
        .map_err(|e| AppError::new(id::INTERNAL, e))??;
    Ok(QueueView {
        queue,
        choices: plan.choices,
        preferred: plan.preferred,
    })
}
