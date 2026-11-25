//! The Tauri shell: wires the engine to the operating system (its keychain, the app data
//! folder, the window, taskbar or Dock) and exposes it to the interface through typed commands
//! and events. What differs between Windows and macOS is in [`platform`].

mod activity;
mod command_names;
mod commands;
mod database;
mod errors;
mod exports;
mod grants;
mod platform;
mod secrets;
mod settings;
mod setup;
mod start_failure;
#[cfg(feature = "test-overrides")]
mod test_overrides;
mod tray;

// The test build leaves out the single-instance plugin and Tauri's dialogs (`run`), their
// dependencies still declared.
#[cfg(feature = "test-overrides")]
use {tauri_plugin_dialog as _, tauri_plugin_single_instance as _};

use tauri::{Emitter, Manager};

use crate::commands::AppState;

/// Name of the event carrying [`kalamu_core::ProgressEvent`]s.
pub const PROGRESS_EVENT: &str = "progress";
/// Sent when the window is closed or the app quit while jobs run or changes are not saved, so
/// the interface can ask first; it carries whether jobs run, to ask about them next.
pub const CLOSE_REQUESTED_EVENT: &str = "close-requested";
/// Sent by the application menu's Settings… on macOS, so the interface opens Settings.
pub const OPEN_SETTINGS_EVENT: &str = "open-settings";
/// Name of the event carrying [`kalamu_search::SearchProgress`]: how far a project's text is
/// prepared for searching.
pub const SEARCH_PROGRESS_EVENT: &str = "search-progress";
/// Name of the event carrying [`kalamu_engine::MeaningProgress`]: how far a project's text is
/// prepared for the search by meaning, after each batch.
pub const MEANING_PROGRESS_EVENT: &str = "meaning-progress";
/// Sent when what the offline status reports changed outside a command: the graphics processor
/// the OCR models use, found after start-up.
pub const OFFLINE_CHANGED_EVENT: &str = "offline-changed";
/// Sent when the catalogue was refreshed at start-up, after the interface first read it: the
/// models it lists and the release it names are read again.
pub const CATALOG_CHANGED_EVENT: &str = "catalog-changed";
/// Name of the event carrying [`kalamu_core::ProjectFileProgress`]: how far a project file is
/// written or opened, in bytes of the files it carries.
pub const PROJECT_FILE_PROGRESS_EVENT: &str = "project-file-progress";

/// The data folder each system keeps (`platform`); in the test build, the one its run names,
/// with nothing of a researcher's read or moved.
fn data_folder() -> (
    tauri::utils::config::AppDirectoriesOverride,
    platform::Moved,
) {
    #[cfg(feature = "test-overrides")]
    if let Some(folder) = test_overrides::data_folder() {
        return (
            tauri::utils::config::AppDirectoriesOverride::Root(folder),
            platform::Moved::Nothing,
        );
    }
    platform::data_folder()
}

pub fn run() {
    // Before any program is started: none may outlive the app.
    platform::keep_children();
    // The data folder is settled, and an earlier one moved into it, before Tauri builds the
    // window, whose webview data goes there too.
    let (folders, moved) = data_folder();
    // Known now, for a launch that fails before setup has started logging there.
    let logs = platform::log_folder(&folders);
    // WebView2 missing, said in Kalamu's message before Tauri says it in its own (Windows).
    if let Some(detail) = platform::webview_missing() {
        setup::fail_before_setup(
            &start_failure::StartError::new(start_failure::Cause::Window, detail),
            logs.as_deref(),
        );
    }
    let mut context = tauri::generate_context!();
    context.config_mut().app.app_directories_override = Some(folders);
    // The test build's window opens the DevTools protocol for its driver, when asked.
    #[cfg(feature = "test-overrides")]
    if let Some(arguments) = test_overrides::browser_arguments() {
        for window in &mut context.config_mut().app.windows {
            window.additional_browser_args = Some(arguments.clone());
        }
    }
    let builder = tauri::Builder::default();
    // The test build, its data apart, runs beside an installed Kalamu rather than focusing it.
    #[cfg(not(feature = "test-overrides"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        // Also when the window was hidden in the tray.
        commands::window::show_main_window(app);
    }));
    // In the test build, dialogs answered from a queue the driver writes (`test_overrides`).
    #[cfg(not(feature = "test-overrides"))]
    let builder = builder.plugin(tauri_plugin_dialog::init());
    #[cfg(feature = "test-overrides")]
    let builder = builder.plugin(test_overrides::dialogs());
    let result = builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| setup::setup(app, moved))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // While jobs run, or while changes not saved would be lost with the window (not
                // when it only hides, on macOS), the interface asks first: about the changes,
                // then about the jobs, which the event says run. Otherwise the window closes,
                // or on macOS hides with the app left in the Dock.
                let state = window.state::<AppState>();
                let jobs = commands::window::jobs_running(&state);
                let draft = !platform::CLOSE_HIDES_WINDOW && commands::window::draft_open(&state);
                if jobs || draft {
                    api.prevent_close();
                    let _ = window.emit(CLOSE_REQUESTED_EVENT, jobs);
                } else if platform::CLOSE_HIDES_WINDOW {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::app_info,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::providers_to_consent,
            commands::settings::consent_providers,
            commands::settings::get_catalog,
            commands::settings::refresh_catalog,
            commands::settings::catalog_refused,
            commands::settings::task_rules,
            commands::settings::newer_release,
            commands::settings::clear_unused_results,
            commands::settings::unused_results,
            commands::database::database_copies,
            commands::database::open_database_copy,
            commands::keys::key_status,
            commands::keys::set_api_key,
            commands::keys::clear_api_key,
            commands::keys::test_api_key,
            commands::offline::offline_status,
            commands::offline::download_model,
            commands::offline::cancel_model_download,
            commands::offline::remove_model,
            commands::presets::list_presets,
            commands::presets::save_preset,
            commands::presets::delete_preset,
            commands::presets::duplicate_preset,
            commands::presets::export_presets,
            commands::presets::import_presets,
            commands::pipelines::list_pipelines,
            commands::pipelines::save_pipeline,
            commands::pipelines::delete_pipeline,
            commands::pipelines::export_pipeline,
            commands::pipelines::import_pipeline,
            commands::workflows::list_workflows,
            commands::workflows::save_workflow,
            commands::workflows::duplicate_workflow,
            commands::workflows::delete_workflow,
            commands::workflows::export_workflow,
            commands::workflows::import_workflow,
            commands::workflows::start_run,
            commands::workflows::list_runs,
            commands::workflows::get_run,
            commands::workflows::job_step,
            commands::workflows::approve_step,
            commands::workflows::cancel_run,
            commands::workflows::delete_run,
            commands::jobs::expand_paths,
            commands::jobs::folders_to_ask,
            commands::jobs::keep_access,
            commands::jobs::inspect_files,
            commands::jobs::image_document,
            commands::jobs::check_photos,
            commands::jobs::stop_photo_checks,
            commands::jobs::inspect_links,
            commands::jobs::estimate_cost,
            commands::jobs::create_job,
            commands::jobs::prompt_fingerprints,
            commands::jobs::start_job,
            commands::sample::rest_estimate,
            commands::sample::rest_of_sample,
            commands::sample::compare_sample,
            commands::jobs::pause_job,
            commands::jobs::cancel_job,
            commands::jobs::retry_failed,
            commands::jobs::delete_job,
            commands::jobs::delete_jobs,
            commands::jobs::list_jobs,
            commands::jobs::job_sources,
            commands::jobs::get_job,
            commands::jobs::job_progress,
            commands::jobs::job_text_outputs,
            commands::jobs::snapshot_job,
            commands::jobs::derived_state,
            commands::units::units_view,
            commands::units::seed_units,
            commands::units::save_unit,
            commands::units::delete_unit,
            commands::units::join_units,
            commands::units::split_unit,
            commands::units::unit_sources,
            commands::units::export_units,
            commands::jobs::job_outputs,
            commands::jobs::open_output,
            commands::jobs::open_original,
            commands::jobs::rename_job,
            commands::review::source_pieces,
            commands::review::piece_preview,
            commands::review::page_regions,
            commands::review::drawn_regions,
            commands::review::save_region,
            commands::review::remove_region,
            commands::review::restore_region,
            commands::review::piece_thumbnail,
            commands::review::piece_audio,
            commands::review::source_text,
            commands::review::save_correction,
            commands::review::save_corrections,
            commands::review::edit_rows,
            commands::review::edit_cited,
            commands::review::cited_source,
            commands::review::rename_listed,
            commands::review::add_listed,
            commands::passage::passage_estimate,
            commands::passage::ask_passage,
            commands::passage::cancel_passage,
            commands::passage::accept_passage,
            commands::passage::discard_passage,
            commands::passage::undo_passage,
            commands::region_reading::region_reading_estimate,
            commands::region_reading::ask_region_reading,
            commands::region_reading::cancel_region_reading,
            commands::region_reading::accept_region_reading,
            commands::region_reading::discard_region_reading,
            commands::region_reading::undo_region_reading,
            commands::review::save_speaker_names,
            commands::review::export_source,
            commands::review::compare_source,
            commands::review::job_scores,
            commands::review::job_agreement,
            commands::review::diff_texts,
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::archive_project,
            commands::projects::delete_project,
            commands::projects::project_view,
            commands::projects::create_collection,
            commands::projects::rename_collection,
            commands::projects::delete_collection,
            commands::projects::add_documents,
            commands::projects::rename_document,
            commands::projects::delete_documents,
            commands::projects::add_to_collection,
            commands::projects::remove_from_collection,
            commands::projects::tag_documents,
            commands::projects::untag_documents,
            commands::projects::describe_document,
            commands::projects::set_source_defaults,
            commands::projects::rename_tag,
            commands::projects::export_descriptions,
            commands::projects::export_answers,
            commands::projects::document_sources,
            commands::profiles::list_export_profiles,
            commands::profiles::save_export_profile,
            commands::profiles::duplicate_export_profile,
            commands::profiles::delete_export_profile,
            commands::profiles::check_export_profile,
            commands::profiles::profile_starter,
            commands::profiles::project_answer_fields,
            commands::profiles::share_export_profile,
            commands::profiles::import_export_profile,
            commands::projects::merge_documents,
            commands::projects::document_detail,
            commands::projects::set_preferred,
            commands::projects::attach_plan,
            commands::projects::attach_job,
            commands::projects::link_job,
            commands::projects::link_jobs,
            commands::projects::detach_job,
            commands::projects::review_queue,
            commands::projects::set_review_sample,
            commands::projects::set_result_aside,
            commands::projects::keep_aside,
            commands::projects::export_review_manifest,
            commands::project_file::plan_project_file,
            commands::project_file::save_project_file,
            commands::project_file::open_project_file,
            commands::project_file::check_project_file,
            commands::dictionary::dictionary_view,
            commands::dictionary::dictionary_evidence,
            commands::dictionary::dictionary_merge,
            commands::dictionary::dictionary_split,
            commands::dictionary::dictionary_relabel,
            commands::dictionary::dictionary_set_reviewed,
            commands::dictionary::dictionary_keep_apart,
            commands::dictionary::dictionary_undo_apart,
            commands::dictionary::export_dictionary,
            commands::dictionary::dictionary_plan,
            commands::dictionary::dictionary_propose,
            commands::dictionary::dictionary_proposals,
            commands::dictionary::dictionary_dismiss,
            commands::dictionary::dictionary_forget_request,
            commands::search::project_search,
            commands::search::rebuild_search_index,
            commands::search::prepare_project_search,
            commands::meaning::meaning_status,
            commands::meaning::choose_meaning_model,
            commands::meaning::consent_meaning,
            commands::meaning::prepare_meaning,
            commands::meaning::pause_meaning,
            commands::meaning::cancel_meaning,
            commands::meaning::remove_meaning_space,
            commands::meaning::meaning_search,
            commands::meaning::meaning_map,
            commands::meaning::meaning_neighbours,
            commands::window::set_shell_labels,
            commands::window::hide_window,
            commands::window::set_draft_open,
            commands::window::quit_app,
            commands::interface_errors::log_interface_error,
        ])
        .build(context)
        .map(|app| {
            app.run(|app, event| {
                // Every way of quitting ends here: the files corrections left out of date are
                // written before the process ends, and Whisper's model is released before the
                // process's own clean-up would find it loaded.
                if matches!(event, tauri::RunEvent::Exit) {
                    commands::window::write_before_exit(app);
                    commands::window::release_before_exit(app);
                    tracing::info!("the app ends");
                }
                platform::on_run_event(app, event);
            });
        });
    // Tauri could not start at all (its event loop, a plugin): said as setup's failures are,
    // through the same message. A window that cannot be made is setup's own (`"create": false`).
    if let Err(e) = result {
        setup::fail_before_setup(
            &start_failure::StartError::new(start_failure::Cause::Window, e),
            logs.as_deref(),
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::command_names::COMMANDS;
    use kalamu_catalog::Catalog;

    /// The handler, the command list and the capability must agree: a command missing from
    /// any of them is a rejected call at run time, and a name listed twice hides a mistake.
    #[test]
    fn every_command_is_registered_listed_and_granted() {
        use std::collections::BTreeSet;

        fn duplicates<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
            let mut seen = BTreeSet::new();
            names.into_iter().filter(|n| !seen.insert(*n)).collect()
        }

        let lib = include_str!("lib.rs");
        let handler_start = lib.find("generate_handler![").unwrap();
        let handler_end = handler_start + lib[handler_start..].find("])").unwrap();
        let handler: Vec<&str> = lib[handler_start..handler_end]
            .lines()
            .filter_map(|l| l.trim().strip_prefix("commands::")?.strip_suffix(','))
            .map(|path| path.rsplit("::").next().unwrap())
            .collect();
        assert_eq!(
            duplicates(COMMANDS.iter().copied()),
            [] as [&str; 0],
            "listed twice"
        );
        assert_eq!(
            duplicates(handler.iter().copied()),
            [] as [&str; 0],
            "registered twice"
        );
        assert_eq!(handler, COMMANDS, "generate_handler! and COMMANDS differ");

        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let granted: Vec<String> = capability["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| p.as_str())
            .filter_map(|p| p.strip_prefix("allow-"))
            .map(|p| p.replace('-', "_"))
            .collect();
        assert_eq!(
            duplicates(granted.iter().map(String::as_str)),
            [] as [&str; 0],
            "granted twice"
        );
        let granted: BTreeSet<&str> = granted.iter().map(String::as_str).collect();
        let listed: BTreeSet<&str> = COMMANDS.iter().copied().collect();
        assert_eq!(
            listed.difference(&granted).collect::<Vec<_>>(),
            [] as [&&str; 0],
            "commands not granted"
        );
        assert_eq!(
            granted.difference(&listed).collect::<Vec<_>>(),
            [] as [&&str; 0],
            "permissions for commands that do not exist"
        );
    }

    /// Tauri makes no window before setup, which makes them itself, so a webview that cannot be
    /// made is told in the start-up message rather than ending in Tauri's own panic. A system's
    /// configuration that listed its windows again would replace the list, `create` with it.
    #[test]
    fn setup_makes_the_windows() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let windows = config["app"]["windows"].as_array().unwrap();
        assert!(!windows.is_empty());
        assert!(windows.iter().all(|w| w["create"] == false), "{windows:?}");
        for other in [
            include_str!("../tauri.macos.conf.json"),
            include_str!("../tauri.appstore.conf.json"),
        ] {
            let other: serde_json::Value = serde_json::from_str(other).unwrap();
            assert!(other["app"]["windows"].is_null(), "{other}");
        }
    }

    /// The opener opens only the addresses its permission lists, and `opener:allow-open-url`
    /// alone lists none: every link in the built app did nothing. The app opens only https
    /// pages (videos, where keys are made, credits, support).
    #[test]
    fn web_pages_are_allowed_to_open() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
        let allowed: Vec<&str> = capability["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["identifier"] == "opener:allow-open-url")
            .flat_map(|p| p["allow"].as_array().into_iter().flatten())
            .filter_map(|entry| entry["url"].as_str())
            .collect();
        assert_eq!(allowed, ["https://*"]);
        // The addresses that come from data rather than code.
        for provider in Catalog::bundled().providers {
            let url = &provider.key_url;
            assert!(url.is_empty() || url.starts_with("https://"), "{url}");
        }
    }
}
