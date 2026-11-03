// What the whole interface shares: the current view, the catalogue, keys, settings, the job list,
// and live progress built from the engine's events. Components read it; only the functions
// here change it.

import type { AppInfo } from './bindings/AppInfo.js';
import type { Catalog } from './bindings/Catalog.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { KeyStatus } from './bindings/KeyStatus.js';
import type { OfflineStatus } from './bindings/OfflineStatus.js';
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
import { messages } from './i18n/index.js';
import { api, notify, openUrl } from './tauri.js';

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
			/** Opens the project on its list of what to check, over these documents if given. */
			pane?: 'queue';
			selection?: string[];
	  }
	| { name: 'jobs'; jobId?: string }
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
	| { name: 'settings' }
	| { name: 'glossary'; term?: Term | 'formats' };

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
	live: {} as Record<string, LiveJob>,
	/** Raised whenever a workflow run changes, so its views read it again. */
	runs: 0,
	/** Jobs of workflow steps that wait for the researcher: checked, or pages not read. */
	waiting: {} as Record<string, { runId: string; step: number; state: StepState }>,
	/** The list of what to check the review was opened from, if it was. */
	queue: null as QueueTrail | null,
	closeRequested: false,
	toast: null as string | null
});

export function go(view: View): void {
	app.view = view;
}

export function hasKey(provider: string): boolean {
	return app.keys.some((k) => k.provider === provider && k.saved);
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

/** Whether a model can run now: always for cloud models, once downloaded for offline ones. */
export function isInstalled(provider: string, model: string): boolean {
	if (!app.catalog?.providers.find((p) => p.id === provider)?.local) return true;
	return app.offline?.models.some((m) => m.id === model && m.installed) ?? false;
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
				void notify(m.app.name, `${job?.name ?? ''}: ${said}`);
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
				tooltip: `${m.app.name}: ${status}`,
				open: m.tray.open,
				pause: m.tray.pause,
				quit: running.length > 0 ? m.tray.quitPausing : m.tray.quit
			}
		})
		.catch(() => undefined);
}

export function toast(message: string): void {
	app.toast = message;
	setTimeout(() => {
		if (app.toast === message) app.toast = null;
	}, 4000);
}

/** Open a web page in the default browser, and say so when it cannot be opened. */
export async function openLink(url: string): Promise<void> {
	try {
		await openUrl(url);
	} catch {
		toast(messages().common.linkFailed(url));
	}
}
