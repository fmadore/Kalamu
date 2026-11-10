// What the whole interface shares: the current view, the catalogue, keys, settings, the job list,
// and live progress built from the engine's events. Components read it; only the functions
// here change it.

import type { AppInfo } from './bindings/AppInfo.js';
import type { Catalog } from './bindings/Catalog.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { KeyStatus } from './bindings/KeyStatus.js';
import type { OfflineStatus } from './bindings/OfflineStatus.js';
import type { ProjectSummary } from './bindings/ProjectSummary.js';
import type { PieceStatus } from './bindings/PieceStatus.js';
import type { QueueItem } from './bindings/QueueItem.js';
import type { ResultKey } from './bindings/ResultKey.js';
import type { ProgressEvent } from './bindings/ProgressEvent.js';
import type { Settings } from './bindings/Settings.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { StepState } from './bindings/StepState.js';
import type { Task } from './bindings/Task.js';
import type { Term } from './glossary.js';
import type { Example } from './workflows.js';
import type { QueueFilter } from './queue.js';
import { leaving, samePlace } from './history.js';
import { messages, resolveLocale, setLocale, type Locale } from './i18n/index.js';
import { api, ask, notify, openUrl } from './tauri.js';

/** A project New job was opened from: the job it starts is tied to the project's documents. */
export interface FromProject {
	id: string;
	name: string;
	/** The collection shown, which new documents join. */
	collectionId?: string;
}

/** A match chosen in a project's search: the review opens on its piece with the words
 * searched, and goes to it. */
export interface ReviewFind {
	query: string;
	setupId: string;
	/** The piece it is in, as the review numbers them. */
	index: number;
	/** Which match it is in that piece's text, from 0. */
	nth: number;
	/** For an answer, its field's key. */
	field?: string | null;
	/** For a table of contents' entry, its place in the table, from 1. */
	entry?: number | null;
}

export type View =
	| {
			name: 'new';
			task?: Task;
			paths?: string[];
			/** Sources already read: text frozen from an earlier job's corrected results. */
			sources?: SourceFile[];
			project?: FromProject;
	  }
	| {
			name: 'projects';
			projectId?: string;
			collectionId?: string;
			documentId?: string;
			/** The pane the project shows; when not given, the one it was left on. */
			pane?: ProjectPane;
			/** Documents chosen in the table, for the list of what to check. */
			selection?: string[];
	  }
	| { name: 'jobs'; jobId?: string }
	/** Every job in a table, found, filtered and sorted. */
	| { name: 'allJobs' }
	| {
			name: 'review';
			jobId: string;
			sourceId: string;
			/** The piece to open on, from 0. */
			index?: number;
			find?: ReviewFind;
	  }
	| {
			name: 'workflows';
			/** A saved workflow to edit, or `new` for a blank one. */
			workflowId?: string;
			/** A ready-made example to start a new one from. */
			example?: Example;
			runId?: string;
			/** Files chosen elsewhere (a project's selection) to run a workflow on. */
			paths?: string[];
			/** Text frozen from earlier jobs' results, to run a workflow on. */
			sources?: SourceFile[];
			project?: FromProject;
	  }
	| { name: 'prompts'; presetId?: string }
	| { name: 'settings'; tab?: SettingsTab }
	| { name: 'glossary'; term?: Term | 'formats' };

/** The panes of a project, down its side. */
export type ProjectPane = 'documents' | 'search' | 'queue' | 'dictionary' | 'activity';

/** The sections of Settings, each a tab. */
export type SettingsTab = 'keys' | 'offline' | 'general' | 'about';

/** The list of what to check a review was opened from: the items shown, the one open, and how
 * the list was set, so stepping on and going back keep it as it was. */
export interface QueueTrail {
	projectId: string;
	/** A collection's id, `selection`, or empty for the whole project. */
	scope: string;
	selection: string[];
	excluded: ResultKey[];
	filter: QueueFilter;
	items: QueueItem[];
	at: number;
}

export interface LivePiece {
	status: PieceStatus;
	error: string | null;
}

/** Progress of one job as events arrive, keyed `setup/source/index`. */
export interface LiveJob {
	pieces: Record<string, LivePiece>;
	spent: number;
	preview: string | null;
	written: string[];
	problems: string[];
	/** Batches with the provider in economy mode, by set-up: size, answers so far, due time. */
	batches: Record<string, { pieces: number; finished: number; dueBy: number }>;
}

export const app = $state({
	view: { name: 'new' } as View,
	info: null as AppInfo | null,
	catalog: null as Catalog | null,
	/** Which sources, models and formats suit each task, from the engine. */
	rules: [] as TaskRules[],
	keys: [] as KeyStatus[],
	offline: null as OfflineStatus | null,
	settings: null as Settings | null,
	jobs: [] as JobSummary[],
	/** The projects, for the sidebar's list of them. */
	projects: [] as ProjectSummary[],
	live: {} as Record<string, LiveJob>,
	/** Raised whenever a workflow run changes, so its views read it again. */
	runs: 0,
	/** Jobs of workflow steps that wait for the researcher: checked, or pages not read. */
	waiting: {} as Record<string, { runId: string; step: number; state: StepState }>,
	/** The list of what to check the review was opened from, if it was. */
	queue: null as QueueTrail | null,
	closeRequested: false,
	toast: null as string | null,
	/** A failure shown until the researcher dismisses it, numbered so the same one shown again
	 * is read out again. */
	toastError: null as { text: string; n: number } | null,
	/** What the toast offers to do about what it says, such as undoing it. */
	toastAction: null as ToastAction | null,
	/** Whether files are being dragged over the window, for the screen that takes them to show
	 * it will. */
	filesOver: false,
	/** The screen on view that takes files dropped on the window, and what it does with them. */
	dropTarget: null as { id: number; label: string } | null,
	/** The interface language in use; the screens are drawn again when it changes. */
	locale: 'en' as Locale
});

let dialogs = 0;

/** A dialog opened over the window: until it closes, the window's own shortcuts wait. */
export function holdShortcuts(): () => void {
	dialogs++;
	return () => {
		dialogs--;
	};
}

/** Whether a dialog is open, so a shortcut of the screen behind it does nothing. */
export function shortcutsHeld(): boolean {
	return dialogs > 0;
}

/** Work on screen that is not saved yet: a correction, a prompt, a workflow. */
export interface Draft {
	/** Whether it differs from what was opened. */
	dirty: () => boolean;
	/** Drops it, once the researcher has agreed to. */
	discard: () => void;
}

const drafts = new Set<Draft>();
let asking: Promise<boolean> | null = null;

/** Keeps a draft from being dropped unasked while it is on screen; the function returned lets
 * it go, for a component to call as it unmounts. */
export function guardDraft(draft: Draft): () => void {
	drafts.add(draft);
	return () => {
		drafts.delete(draft);
	};
}

/** Does `then` at once when no draft holds changes; otherwise asks first, and drops the drafts
 * before doing it only if the researcher agrees. Leaving the screen and turning to another
 * piece in it both go through here, so no way out loses a draft silently. */
export function leaveThen(then: () => void): void {
	const dirty = [...drafts].filter((d) => d.dirty());
	if (dirty.length === 0) {
		then();
		return;
	}
	const m = messages();
	// A second way out tried while the question is up waits for the same answer.
	asking ??= ask(m.common.unsaved, { ok: m.common.discardChanges, cancel: m.common.keepEditing })
		.catch(() => false)
		.finally(() => (asking = null));
	void asking.then((discard) => {
		if (!discard) return;
		dirty.forEach((d) => d.discard());
		then();
	});
}

/** The screens left, to go back through; see `history.ts`. */
let past = $state<View[]>([]);

let dropTargets = 0;

/** Says that the screen on view takes files dropped on the window, and what for (`label`), so
 * the window shows it while files are dragged over it; the function returned stops, for a
 * component to call as it unmounts. */
export function takeFiles(label: string): () => void {
	const id = ++dropTargets;
	app.dropTarget = { id, label };
	return () => {
		if (app.dropTarget?.id === id) app.dropTarget = null;
	};
}

/** Opens a screen, the one left kept to come back to; with `replace`, in its place, as stepping
 * through a list of what to check does, so Back returns to the list. */
export function go(view: View, { replace = false } = {}): void {
	leaveThen(() => {
		if (!replace) past = leaving(past, $state.snapshot(app.view) as View, view);
		app.view = view;
	});
}

/** Sets what the screen on view shows, as it changes inside it (a project's pane), so that
 * coming back to it shows the same; nothing is added to go back through. */
export function settle(view: View): void {
	app.view = view;
}

/** Whether there is a screen to go back to. */
export function canGoBack(): boolean {
	return past.length > 0;
}

/** Goes back to the screen left last, as it was left. */
export function back(): void {
	if (past.length === 0) return;
	leaveThen(() => {
		const previous = past[past.length - 1];
		past = past.slice(0, -1);
		app.view = previous;
	});
}

/** Goes to `view` by going back, when that is where the screen was opened from, so it comes
 * back as it was left; otherwise opens it. For a link that names where it goes. */
export function returnTo(view: View): void {
	const previous = past[past.length - 1];
	if (previous && samePlace(previous, view)) back();
	else go(view);
}

/** Puts the interface in the language the settings ask for, the tray menu included. */
export function applyLanguage(): void {
	const locale = resolveLocale(
		app.settings?.language ?? 'system',
		app.info?.system_languages ?? []
	);
	setLocale(locale);
	app.locale = locale;
	updateProgressDisplays();
}

export function hasKey(provider: string): boolean {
	return app.keys.some((k) => k.provider === provider && k.saved);
}

/** The projects as the sidebar lists them; left as they were when they cannot be read. */
export async function refreshProjects(): Promise<void> {
	app.projects = await api.listProjects().catch(() => app.projects);
}

export async function refreshJobs(): Promise<void> {
	app.jobs = await api.listJobs();
	updateProgressDisplays();
}

/** Which workflow steps wait for the researcher, for the job list, the job's page and review. */
export async function refreshWaiting(): Promise<void> {
	const runs = await api.listRuns().catch(() => []);
	const waiting: typeof app.waiting = {};
	for (const { run, steps } of runs) {
		steps.forEach((s, step) => {
			if (s.job && s.needs_you) {
				waiting[s.job.id] = { runId: run.id, step, state: s.state };
			}
		});
	}
	app.waiting = waiting;
}

/** A run changed: the views that show it read it again. */
function runsChanged(): void {
	app.runs++;
	void refreshWaiting();
}

export async function refreshKeys(): Promise<void> {
	app.keys = await api.keyStatus();
}

export async function refreshOffline(): Promise<void> {
	app.offline = await api.offlineStatus();
}

/**
 * Whether a model can run now: always for cloud models, once downloaded for offline ones. An
 * offline engine built into the app (Tesseract) has nothing of its own to download; its
 * language data is checked by `missingLanguageData`.
 */
export function isInstalled(provider: string, model: string): boolean {
	if (!app.catalog?.providers.find((p) => p.id === provider)?.local) return true;
	const status = [
		...(app.offline?.models ?? []),
		...(app.offline?.ocr_models ?? []),
		...(app.offline?.llm_models ?? [])
	].find((m) => m.id === model);
	return status ? status.installed : app.offline !== null;
}

/**
 * For an offline OCR model, the seconds a page has taken on this computer, rounded, or `null`
 * before it has read any; `undefined` for any other model.
 */
export function secondsPerPage(model: string): number | null | undefined {
	if (!app.offline?.ocr_models.some((s) => s.id === model)) return undefined;
	const seconds = app.offline.ocr_seconds_per_page[model];
	return seconds === undefined ? null : Math.max(1, Math.round(seconds));
}

/** Whether an offline OCR model's next page uses the graphics processor, wholly or in part. */
export function onGraphics(model: string): boolean {
	return (app.offline?.ocr_devices[model] ?? 'cpu') !== 'cpu';
}

/** The chosen languages whose language data, for an engine that needs it, is not downloaded. */
export function missingLanguageData(codes: string[]): string[] {
	const data = app.offline?.language_data ?? [];
	return codes.filter((code) => {
		const file = data.find((d) => d.languages.includes(code));
		return file !== undefined && !file.installed;
	});
}

export function liveJob(id: string): LiveJob {
	if (!app.live[id]) {
		app.live[id] = {
			pieces: {},
			spent: 0,
			preview: null,
			written: [],
			problems: [],
			batches: {}
		};
	}
	return app.live[id];
}

let refreshTimer: ReturnType<typeof setTimeout> | null = null;

/** Refresh the job list at most every half second while events stream in. */
function scheduleRefresh(): void {
	if (refreshTimer) return;
	refreshTimer = setTimeout(() => {
		refreshTimer = null;
		void refreshJobs();
	}, 500);
}

export function applyProgress(event: ProgressEvent): void {
	if (event.type === 'run_changed') {
		runsChanged();
		return;
	}
	const live = liveJob(event.job_id);
	switch (event.type) {
		case 'step_started':
			runsChanged();
			scheduleRefresh();
			break;
		case 'job_started':
		case 'source_prepared':
			scheduleRefresh();
			break;
		case 'piece_updated':
			live.pieces[`${event.setup_id}/${event.source_id}/${event.index}`] = {
				status: event.status,
				error: event.error
			};
			if (event.preview) live.preview = event.preview;
			if (event.status === 'failed' && event.error) live.problems.push(event.error);
			scheduleRefresh();
			break;
		case 'source_exported':
			live.written.push(...event.files);
			break;
		case 'source_failed':
			live.problems.push(event.error);
			break;
		case 'cost_updated':
			live.spent = event.spent_usd;
			break;
		case 'batch_waiting':
			live.batches[event.setup_id] = {
				pieces: event.pieces,
				finished: event.finished,
				dueBy: event.due_by
			};
			break;
		case 'job_finished': {
			app.runs++;
			void refreshJobs();
			const m = messages();
			const job = app.jobs.find((j) => j.id === event.job_id);
			// A workflow step that now waits says so: the next step needs the researcher.
			void refreshWaiting().then(() => {
				const waits = app.waiting[event.job_id];
				const said = waits
					? m.workflows.notify[waits.state === 'review' ? 'review' : 'attention']
					: m.status[event.status];
				void notify(m.app.name, `${job?.name ?? ''}${m.common.colon} ${said}`);
			});
			break;
		}
	}
}

/** Overall progress of running jobs on the taskbar button and in the tray menu. */
function updateProgressDisplays(): void {
	const running = app.jobs.filter((j) => j.status === 'running');
	const total = running.reduce((n, j) => n + j.pieces_total, 0);
	const done = running.reduce((n, j) => n + j.pieces_finished, 0);
	const percent = total > 0 ? Math.round((100 * done) / total) : null;
	void api.setTaskbarProgress(percent).catch(() => undefined);
	const m = messages();
	const status = running.length === 0 ? m.tray.idle : m.tray.running(running.length, percent ?? 0);
	void api
		.setTrayState({
			running: running.length > 0,
			labels: {
				status,
				tooltip: `${m.app.name}${m.common.colon} ${status}`,
				open: m.tray.open,
				pause: m.tray.pause,
				quit: running.length > 0 ? m.tray.quitPausing : m.tray.quit
			}
		})
		.catch(() => undefined);
}

/** A button in the toast: what it says, and what it does. */
export interface ToastAction {
	label: string;
	run: () => void;
}

let toastTimer: ReturnType<typeof setTimeout> | undefined;

/** Shows a message for four seconds from the last time it was shown, even when it is the same
 * message again; with an action (`action`, such as undoing it), eight, to leave time to take it.
 * A failure (`kind: 'error'`) is read out at once and stays until dismissed, so it can be read
 * again. */
export function toast(
	message: string,
	options: { kind?: 'error'; action?: ToastAction } = {}
): void {
	if (options.kind === 'error') {
		app.toastError = { text: message, n: (app.toastError?.n ?? 0) + 1 };
		return;
	}
	const action = options.action ?? null;
	app.toast = message;
	app.toastAction = action;
	clearTimeout(toastTimer);
	toastTimer = setTimeout(
		() => {
			app.toast = null;
			app.toastAction = null;
		},
		action ? 8000 : 4000
	);
}

/** Takes the toast's action, and puts the toast away. */
export function takeToastAction(): void {
	const action = app.toastAction;
	clearTimeout(toastTimer);
	app.toast = null;
	app.toastAction = null;
	action?.run();
}

/** Puts away the failure shown. */
export function dismissError(): void {
	app.toastError = null;
}

/** Open a web page in the default browser, and say so when it cannot be opened. */
export async function openLink(url: string): Promise<void> {
	try {
		await openUrl(url);
	} catch {
		toast(messages().common.linkFailed(url), { kind: 'error' });
	}
}
