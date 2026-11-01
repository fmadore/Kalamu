// The one bridge to the Rust side, as in Live Translation: typed commands and events, and a
// graceful failure outside the desktop app (a plain `npm run dev` in a browser). Every type
// comes from `src/lib/bindings`, generated from the Rust definitions by `cargo test`.

import type { AppError } from './bindings/AppError.js';
import type { AppInfo } from './bindings/AppInfo.js';
import type { Catalog } from './bindings/Catalog.js';
import type { ComparisonReport } from './bindings/ComparisonReport.js';
import type { AttachChoice } from './bindings/AttachChoice.js';
import type { Collection } from './bindings/Collection.js';
import type { Correction } from './bindings/Correction.js';
import type { DocumentDetail } from './bindings/DocumentDetail.js';
import type { SpeakerNames } from './bindings/SpeakerNames.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { JobStep } from './bindings/JobStep.js';
import type { RunView } from './bindings/RunView.js';
import type { StartRun } from './bindings/StartRun.js';
import type { Workflow } from './bindings/Workflow.js';
import type { DerivedState } from './bindings/DerivedState.js';
import type { DiffSpan } from './bindings/DiffSpan.js';
import type { EstimateRequest } from './bindings/EstimateRequest.js';
import type { EstimateRow } from './bindings/EstimateRow.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { Job } from './bindings/Job.js';
import type { JobScores } from './bindings/JobScores.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { KeyStatus } from './bindings/KeyStatus.js';
import type { NewJob } from './bindings/NewJob.js';
import type { OfflineStatus } from './bindings/OfflineStatus.js';
import type { PagePreview } from './bindings/PagePreview.js';
import type { PieceAudio } from './bindings/PieceAudio.js';
import type { PieceState } from './bindings/PieceState.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Preset } from './bindings/Preset.js';
import type { Project } from './bindings/Project.js';
import type { ProjectSummary } from './bindings/ProjectSummary.js';
import type { ProjectView } from './bindings/ProjectView.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { SourceMatch } from './bindings/SourceMatch.js';
import type { ProgressEvent } from './bindings/ProgressEvent.js';
import type { Settings } from './bindings/Settings.js';
import type { Task } from './bindings/Task.js';
import type { TrayState } from './bindings/TrayState.js';

export const isTauri = (): boolean =>
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export type Unlisten = () => void;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) {
		throw { id: 'app.notDesktop', detail: cmd } satisfies AppError;
	}
	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(cmd, args);
}

async function listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
	if (!isTauri()) return () => {};
	const { listen } = await import('@tauri-apps/api/event');
	return listen<T>(event, (e) => handler(e.payload));
}

export const api = {
	appInfo: () => invoke<AppInfo>('app_info'),
	getSettings: () => invoke<Settings>('get_settings'),
	saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
	getCatalog: () => invoke<Catalog>('get_catalog'),
	refreshCatalog: () => invoke<Catalog>('refresh_catalog'),
	taskRules: () => invoke<TaskRules[]>('task_rules'),
	keyStatus: () => invoke<KeyStatus[]>('key_status'),
	setApiKey: (provider: string, key: string) => invoke<void>('set_api_key', { provider, key }),
	clearApiKey: (provider: string) => invoke<void>('clear_api_key', { provider }),
	testApiKey: (provider: string) => invoke<string[]>('test_api_key', { provider }),
	offlineStatus: () => invoke<OfflineStatus>('offline_status'),
	downloadModel: (id: string) => invoke<void>('download_model', { id }),
	cancelModelDownload: () => invoke<void>('cancel_model_download'),
	removeModel: (id: string) => invoke<void>('remove_model', { id }),
	listPresets: (task?: Task) => invoke<Preset[]>('list_presets', { task: task ?? null }),
	listPipelines: () => invoke<Pipeline[]>('list_pipelines'),
	savePipeline: (pipeline: Pipeline) => invoke<Pipeline>('save_pipeline', { pipeline }),
	deletePipeline: (id: string) => invoke<void>('delete_pipeline', { id }),
	exportPipeline: (id: string, path: string) => invoke<void>('export_pipeline', { id, path }),
	importPipeline: (path: string) => invoke<Pipeline>('import_pipeline', { path }),
	listWorkflows: () => invoke<Workflow[]>('list_workflows'),
	saveWorkflow: (workflow: Workflow) => invoke<Workflow>('save_workflow', { workflow }),
	duplicateWorkflow: (id: string, name: string) =>
		invoke<Workflow>('duplicate_workflow', { id, name }),
	deleteWorkflow: (id: string) => invoke<void>('delete_workflow', { id }),
	exportWorkflow: (id: string, path: string) => invoke<void>('export_workflow', { id, path }),
	importWorkflow: (path: string) => invoke<Workflow>('import_workflow', { path }),
	startRun: (start: StartRun) => invoke<RunView>('start_run', { start }),
	listRuns: () => invoke<RunView[]>('list_runs'),
	getRun: (id: string) => invoke<RunView>('get_run', { id }),
	jobStep: (id: string) => invoke<JobStep | null>('job_step', { id }),
	approveStep: (id: string, step: number) => invoke<void>('approve_step', { id, step }),
	cancelRun: (id: string) => invoke<void>('cancel_run', { id }),
	deleteRun: (id: string) => invoke<void>('delete_run', { id }),
	savePreset: (preset: Preset) => invoke<Preset>('save_preset', { preset }),
	deletePreset: (id: string) => invoke<void>('delete_preset', { id }),
	duplicatePreset: (id: string, name: string) => invoke<Preset>('duplicate_preset', { id, name }),
	exportPresets: (ids: string[], path: string) => invoke<void>('export_presets', { ids, path }),
	importPresets: (path: string) => invoke<Preset[]>('import_presets', { path }),
	expandPaths: (paths: string[], task: Task) => invoke<string[]>('expand_paths', { paths, task }),
	inspectFiles: (paths: string[]) => invoke<InspectedFile[]>('inspect_files', { paths }),
	inspectLinks: (links: string[]) => invoke<InspectedFile[]>('inspect_links', { links }),
	estimateCost: (request: EstimateRequest) => invoke<EstimateRow[]>('estimate_cost', { request }),
	createJob: (job: NewJob) => invoke<Job>('create_job', { job }),
	startJob: (id: string) => invoke<void>('start_job', { id }),
	pauseJob: (id: string) => invoke<void>('pause_job', { id }),
	cancelJob: (id: string) => invoke<void>('cancel_job', { id }),
	retryFailed: (id: string) => invoke<void>('retry_failed', { id }),
	deleteJob: (id: string) => invoke<void>('delete_job', { id }),
	renameJob: (id: string, name: string) => invoke<void>('rename_job', { id, name }),
	listJobs: () => invoke<JobSummary[]>('list_jobs'),
	getJob: (id: string) => invoke<Job>('get_job', { id }),
	jobPieces: (id: string) => invoke<PieceState[]>('job_pieces', { id }),
	jobTextOutputs: (id: string) => invoke<string[]>('job_text_outputs', { id }),
	snapshotJob: (id: string, setup: string) => invoke<SourceFile[]>('snapshot_job', { id, setup }),
	derivedState: (id: string) => invoke<DerivedState[]>('derived_state', { id }),
	jobOutputs: (id: string) => invoke<string[]>('job_outputs', { id }),
	openOutput: (id: string, path: string) => invoke<void>('open_output', { id, path }),
	sourcePieces: (jobId: string, sourceId: string) =>
		invoke<PieceView[]>('source_pieces', { jobId, sourceId }),
	piecePreview: (jobId: string, sourceId: string, index: number) =>
		invoke<PagePreview>('piece_preview', { jobId, sourceId, index }),
	pieceAudio: (jobId: string, sourceId: string, index: number, convert: boolean) =>
		invoke<PieceAudio>('piece_audio', { jobId, sourceId, index, convert }),
	/** A file `pieceAudio` named, as an address the player streams (Tauri's asset protocol). */
	fileUrl: async (path: string): Promise<string> => {
		const { convertFileSrc } = await import('@tauri-apps/api/core');
		return convertFileSrc(path);
	},
	sourceText: (jobId: string, sourceId: string) =>
		invoke<string>('source_text', { jobId, sourceId }),
	saveCorrection: (correction: Correction) => invoke<string[]>('save_correction', { correction }),
	saveSpeakerNames: (jobId: string, names: SpeakerNames) =>
		invoke<string[]>('save_speaker_names', { jobId, names }),
	exportSource: (jobId: string, setupId: string, sourceId: string) =>
		invoke<string[]>('export_source', { jobId, setupId, sourceId }),
	compareSource: (jobId: string, sourceId: string) =>
		invoke<ComparisonReport>('compare_source', { jobId, sourceId }),
	jobScores: (jobId: string) => invoke<JobScores>('job_scores', { jobId }),
	diffTexts: (left: string, right: string) => invoke<DiffSpan[]>('diff_texts', { left, right }),
	listProjects: () => invoke<ProjectSummary[]>('list_projects'),
	createProject: (name: string, description: string) =>
		invoke<Project>('create_project', { name, description }),
	updateProject: (id: string, name: string, description: string) =>
		invoke<Project>('update_project', { id, name, description }),
	archiveProject: (id: string, archived: boolean) =>
		invoke<Project>('archive_project', { id, archived }),
	deleteProject: (id: string) => invoke<void>('delete_project', { id }),
	projectView: (id: string) => invoke<ProjectView>('project_view', { id }),
	createCollection: (project: string, name: string) =>
		invoke<Collection>('create_collection', { project, name }),
	renameCollection: (project: string, id: string, name: string) =>
		invoke<void>('rename_collection', { project, id, name }),
	deleteCollection: (project: string, id: string) =>
		invoke<void>('delete_collection', { project, id }),
	addDocuments: (project: string, sources: SourceFile[], collection: string | null) =>
		invoke<string[]>('add_documents', { project, sources, collection }),
	renameDocument: (project: string, id: string, title: string) =>
		invoke<void>('rename_document', { project, id, title }),
	deleteDocuments: (project: string, ids: string[]) =>
		invoke<void>('delete_documents', { project, ids }),
	addToCollection: (project: string, collection: string, ids: string[]) =>
		invoke<void>('add_to_collection', { project, collection, ids }),
	removeFromCollection: (project: string, collection: string, ids: string[]) =>
		invoke<void>('remove_from_collection', { project, collection, ids }),
	mergeDocuments: (project: string, from: string, into: string) =>
		invoke<void>('merge_documents', { project, from, into }),
	documentDetail: (project: string, id: string) =>
		invoke<DocumentDetail>('document_detail', { project, id }),
	setPreferred: (
		project: string,
		document: string,
		task: Task,
		job: string | null,
		setup: string | null
	) => invoke<void>('set_preferred', { project, document, task, job, setup }),
	attachPlan: (project: string, job: string) =>
		invoke<SourceMatch[]>('attach_plan', { project, job }),
	attachJob: (project: string, job: string, choices: AttachChoice[], collection: string | null) =>
		invoke<string[]>('attach_job', { project, job, choices, collection }),
	linkJob: (project: string, job: string, collection: string | null) =>
		invoke<string[]>('link_job', { project, job, collection }),
	detachJob: (project: string, document: string, job: string, source: string) =>
		invoke<void>('detach_job', { project, document, job, source }),
	setTaskbarProgress: (percent: number | null) => invoke<void>('set_taskbar_progress', { percent }),
	setTrayState: (state: TrayState) => invoke<void>('set_tray_state', { state }),
	hideWindow: () => invoke<void>('hide_window'),
	quitApp: () => invoke<void>('quit_app')
};

export const on = {
	progress: (handler: (event: ProgressEvent) => void) => listen('progress', handler),
	closeRequested: (handler: () => void) => listen<null>('close-requested', handler)
};

const PAGES = ['pdf', 'png', 'jpg', 'jpeg', 'tif', 'tiff', 'webp', 'bmp', 'heic', 'heif'];
/** The files each task reads, by extension. */
const EXTENSIONS: Record<Task, string[]> = {
	text_extraction: PAGES,
	contents: PAGES,
	text_correction: ['txt', 'md', 'docx', 'pdf'],
	audio_transcription: [
		'mp3',
		'wav',
		'm4a',
		'aac',
		'flac',
		'ogg',
		'opus',
		'wma',
		'webm',
		'mp4',
		'mov',
		'mkv',
		'avi'
	],
	analysis: [
		'txt',
		'md',
		'docx',
		'csv',
		'pdf',
		'png',
		'jpg',
		'jpeg',
		'tif',
		'tiff',
		'webp',
		'bmp',
		'heic',
		'heif'
	]
};

/** File and folder pickers. */
export async function pickFiles(task: Task): Promise<string[]> {
	if (!isTauri()) return [];
	const { open } = await import('@tauri-apps/plugin-dialog');
	const extensions = EXTENSIONS[task];
	const picked = await open({ multiple: true, filters: [{ name: 'Sources', extensions }] });
	return picked === null ? [] : Array.isArray(picked) ? picked : [picked];
}

/** Files of any kind Kalamu reads, for a project's documents. */
export async function pickDocuments(): Promise<string[]> {
	if (!isTauri()) return [];
	const { open } = await import('@tauri-apps/plugin-dialog');
	const extensions = [...new Set(Object.values(EXTENSIONS).flat())];
	const picked = await open({ multiple: true, filters: [{ name: 'Sources', extensions }] });
	return picked === null ? [] : Array.isArray(picked) ? picked : [picked];
}

export async function pickFolder(): Promise<string | null> {
	if (!isTauri()) return null;
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({ directory: true, multiple: false });
	return typeof picked === 'string' ? picked : null;
}

export async function pickSaveFile(defaultPath: string): Promise<string | null> {
	if (!isTauri()) return null;
	const { save } = await import('@tauri-apps/plugin-dialog');
	return save({ defaultPath, filters: [{ name: 'JSON', extensions: ['json'] }] });
}

/** Where to write a pipeline's record: the JSON file, with its Markdown twin beside it. */
export async function pickPipelineExport(name: string): Promise<string | null> {
	if (!isTauri()) return null;
	const { save } = await import('@tauri-apps/plugin-dialog');
	return save({ defaultPath: `${name}.json`, filters: [{ name: 'JSON', extensions: ['json'] }] });
}

export async function pickPresetFile(): Promise<string | null> {
	if (!isTauri()) return null;
	const { open } = await import('@tauri-apps/plugin-dialog');
	const picked = await open({ multiple: false, filters: [{ name: 'JSON', extensions: ['json'] }] });
	return typeof picked === 'string' ? picked : null;
}

/** A yes/no question in a native dialog. */
export async function ask(message: string): Promise<boolean> {
	if (!isTauri()) return window.confirm(message);
	const { ask } = await import('@tauri-apps/plugin-dialog');
	return ask(message, { kind: 'warning' });
}

export async function revealInFolder(path: string): Promise<void> {
	if (!isTauri()) return;
	const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
	await revealItemInDir(path);
}

export async function openUrl(url: string): Promise<void> {
	if (!isTauri()) {
		window.open(url, '_blank', 'noopener');
		return;
	}
	const { openUrl } = await import('@tauri-apps/plugin-opener');
	await openUrl(url);
}

/** Paths dropped onto the window. */
export async function onDrop(handler: (paths: string[]) => void): Promise<Unlisten> {
	if (!isTauri()) return () => {};
	const { getCurrentWebview } = await import('@tauri-apps/api/webview');
	return getCurrentWebview().onDragDropEvent((event) => {
		if (event.payload.type === 'drop') handler(event.payload.paths);
	});
}

/** A system notification, when the window is not in front (or `always`). */
export async function notify(title: string, body: string, always = false): Promise<void> {
	if (!isTauri() || (!always && document.hasFocus())) return;
	const { isPermissionGranted, requestPermission, sendNotification } =
		await import('@tauri-apps/plugin-notification');
	let granted = await isPermissionGranted();
	if (!granted) granted = (await requestPermission()) === 'granted';
	if (granted) sendNotification({ title, body });
}
