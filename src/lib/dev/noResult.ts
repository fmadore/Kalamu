/**
 * Commands that give nothing back, answered with nothing here. Any other command the mock does
 * not answer fails as it would outside the app, rather than returning nothing to a caller that
 * expects a value.
 */
export const NO_RESULT = new Set([
	'cancel_job',
	'cancel_model_download',
	'clear_api_key',
	'delete_job',
	'delete_preset',
	'download_model',
	'export_presets',
	'hide_window',
	'pause_job',
	'quit_app',
	'remove_model',
	'retry_failed',
	'save_settings',
	'set_api_key',
	'set_shell_labels',
	'start_job'
]);

/**
 * Commands that need the desktop app (a job run, a file read, a provider called): the mock
 * fails them as the interface does outside the app.
 */
export const DESKTOP_ONLY = new Set([
	'create_job',
	'duplicate_preset',
	'import_pipeline',
	'import_presets',
	'import_workflow',
	'refresh_catalog',
	'save_preset',
	'test_api_key'
]);
