//! A project saved as one file and opened on another computer (#213): what saving would write,
//! the save itself, the open and a check without opening (#246), each off the window's thread, telling the interface how far
//! it has got. The work is the job database's (`ProjectArchive`); this gives it the data
//! folder's places.

use kalamu_core::{
    ArchivePlaces, ProjectFileCheck, ProjectFileOpened, ProjectFilePlan, ProjectFileProgress,
    ProjectFileSaved, ProjectId,
};
use tauri::{AppHandle, Emitter, State};

use super::AppState;
use crate::PROJECT_FILE_PROGRESS_EVENT;
use crate::errors::CmdResult;

/// The frozen texts' folder (the engine's, set up beside the job database) and the folder the
/// originals a file carries are written to.
fn places(state: &AppState) -> ArchivePlaces {
    ArchivePlaces {
        snapshots: state.paths.data.join("snapshots"),
        originals: state.paths.data.join("originals"),
        app_version: env!("CARGO_PKG_VERSION").into(),
    }
}

/// Tells the interface how far the file is written or opened.
fn tell(app: &AppHandle) -> impl Fn(ProjectFileProgress) + use<> {
    let app = app.clone();
    move |progress| {
        let _ = app.emit(PROJECT_FILE_PROGRESS_EVENT, progress);
    }
}

/// What saving the project would write, with and without its originals.
#[tauri::command]
pub async fn plan_project_file(
    state: State<'_, AppState>,
    project: ProjectId,
) -> CmdResult<ProjectFilePlan> {
    let archive = state.archive.clone();
    let places = places(&state);
    super::off_thread(move || Ok(archive.plan_project_file(project, &places)?)).await
}

/// Saves the project to `path`, its originals in it when asked.
#[tauri::command]
pub async fn save_project_file(
    app: AppHandle,
    state: State<'_, AppState>,
    project: ProjectId,
    path: String,
    with_originals: bool,
) -> CmdResult<ProjectFileSaved> {
    let archive = state.archive.clone();
    let places = places(&state);
    let tell = tell(&app);
    super::off_thread(move || {
        Ok(archive.save_project_file(
            project,
            std::path::Path::new(&path),
            with_originals,
            &places,
            &tell,
        )?)
    })
    .await
}

/// Opens the file at `path` as a new project.
#[tauri::command]
pub async fn open_project_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<ProjectFileOpened> {
    let archive = state.archive.clone();
    let places = places(&state);
    let tell = tell(&app);
    super::off_thread(move || {
        Ok(archive.open_project_file(std::path::Path::new(&path), &places, &tell)?)
    })
    .await
}

/// Checks the file at `path` without opening it: nothing is written and no project created.
#[tauri::command]
pub async fn check_project_file(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> CmdResult<ProjectFileCheck> {
    let archive = state.archive.clone();
    let tell = tell(&app);
    super::off_thread(move || Ok(archive.check_project_file(std::path::Path::new(&path), &tell)?))
        .await
}
