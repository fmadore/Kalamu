//! The Tauri shell: wires the engine to the operating system (its keychain, the app data
//! folder, the window, taskbar or Dock) and exposes it to the interface through typed commands
//! and events. What differs between Windows and macOS is in [`platform`].

mod activity;
mod command_names;
mod commands;
mod errors;
mod grants;
mod platform;
mod secrets;
mod settings;
mod setup;
mod start_failure;
mod tray;

use tauri::{Emitter, Manager};

use crate::commands::AppState;

/// Name of the event carrying [`kalamu_core::ProgressEvent`]s.
pub const PROGRESS_EVENT: &str = "progress";
/// Sent when the window is closed while jobs run, so the interface can ask first.
pub const CLOSE_REQUESTED_EVENT: &str = "close-requested";
/// Name of the event carrying [`kalamu_search::SearchProgress`]: how far a project's text is
/// prepared for searching.
pub const SEARCH_PROGRESS_EVENT: &str = "search-progress";
/// Sent when what the offline status reports changed outside a command: the graphics processor
/// the OCR models use, found after start-up.
pub const OFFLINE_CHANGED_EVENT: &str = "offline-changed";
/// Sent when the catalogue was refreshed at start-up, after the interface first read it: the
/// models it lists and the release it names are read again.
pub const CATALOG_CHANGED_EVENT: &str = "catalog-changed";

pub fn run() {
    // Before any program is started: none may outlive the app.
    platform::keep_children();
    // The data folder is settled, and an earlier one moved into it, before Tauri builds the
    // window, whose webview data goes there too.
    let (folders, moved) = platform::data_folder();
    let mut context = tauri::generate_context!();
    context.config_mut().app.app_directories_override = Some(folders);
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Also when the window was hidden in the tray.
            commands::window::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| setup::setup(app, moved))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // While jobs run the interface asks first; otherwise the window closes, or on
                // macOS hides with the app left in the Dock.
                if commands::window::jobs_running(&window.state::<AppState>()) {
                    api.prevent_close();
                    let _ = window.emit(CLOSE_REQUESTED_EVENT, ());
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
            commands::settings::task_rules,
            commands::settings::newer_release,
            commands::settings::clear_unused_results,
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
            commands::jobs::inspect_links,
            commands::jobs::estimate_cost,
            commands::jobs::create_job,
            commands::jobs::start_job,
            commands::jobs::pause_job,
            commands::jobs::cancel_job,
            commands::jobs::retry_failed,
            commands::jobs::delete_job,
            commands::jobs::delete_jobs,
            commands::jobs::list_jobs,
            commands::jobs::job_sources,
            commands::jobs::get_job,
            commands::jobs::job_pieces,
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
            commands::jobs::rename_job,
            commands::review::source_pieces,
            commands::review::piece_preview,
            commands::review::piece_thumbnail,
            commands::review::piece_audio,
            commands::review::source_text,
            commands::review::save_correction,
            commands::review::save_corrections,
            commands::review::rename_listed,
            commands::review::save_speaker_names,
            commands::review::export_source,
            commands::review::compare_source,
            commands::review::job_scores,
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
            commands::window::set_shell_labels,
            commands::window::hide_window,
            commands::window::quit_app,
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
                }
                platform::on_run_event(app, event);
            });
        });
    if let Err(e) = result {
        tracing::error!(error = %e, "the app could not start");
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
