// The commands the front end may invoke.
//
// `build.rs` includes this file to generate an `allow-<command>` permission for each name; once
// the app has that manifest, Tauri rejects any command `capabilities/main.json` does not grant.
// Adding a command means three edits: the handler, this list and the capability. A test in
// `lib.rs` fails if one is missed.
//
// Plain `//` comments: `build.rs` includes this file inline, where an inner doc comment cannot
// appear.

/// Every command in `generate_handler!`, in the same order. Read by `build.rs` and the tests.
#[cfg_attr(not(test), allow(dead_code))]
pub const COMMANDS: &[&str] = &[
    "app_info",
    "get_settings",
    "save_settings",
    "get_catalog",
    "refresh_catalog",
    "key_status",
    "set_api_key",
    "clear_api_key",
    "test_api_key",
    "offline_status",
    "download_model",
    "cancel_model_download",
    "remove_model",
    "list_presets",
    "save_preset",
    "delete_preset",
    "duplicate_preset",
    "export_presets",
    "import_presets",
    "expand_paths",
    "inspect_files",
    "estimate_cost",
    "create_job",
    "start_job",
    "pause_job",
    "cancel_job",
    "retry_failed",
    "delete_job",
    "list_jobs",
    "get_job",
    "job_pieces",
    "job_text_outputs",
    "source_pieces",
    "piece_preview",
    "piece_audio",
    "save_correction",
    "save_speaker_names",
    "export_source",
    "compare_source",
    "diff_texts",
    "set_taskbar_progress",
    "set_tray_state",
    "hide_window",
    "quit_app",
];
