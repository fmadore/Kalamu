// What the whole interface shares: the current view, the catalogue, keys, settings, the job list,
// and live progress built from the engine's events. Components read it; only the functions
// here change it.

import type { AppInfo } from './bindings/AppInfo.js';
import type { Catalog } from './bindings/Catalog.js';
import type { JobSummary } from './bindings/JobSummary.js';
import type { KeyStatus } from './bindings/KeyStatus.js';
import type { OfflineStatus } from './bindings/OfflineStatus.js';
import type { PieceStatus } from './bindings/PieceStatus.js';
import type { ProgressEvent } from './bindings/ProgressEvent.js';
import type { Settings } from './bindings/Settings.js';
import type { Task } from './bindings/Task.js';
import { messages } from './i18n/index.js';
import { api, notify, openUrl } from './tauri.js';

export type View =
	| { name: 'new'; task?: Task; paths?: string[] }
	| { name: 'jobs'; jobId?: string }
	| { name: 'review'; jobId: string; sourceId: string }
	| { name: 'prompts' }
	| { name: 'settings' };

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
	keys: [] as KeyStatus[],
	offline: null as OfflineStatus | null,
	settings: null as Settings | null,
	jobs: [] as JobSummary[],
	live: {} as Record<string, LiveJob>,
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
	const live = liveJob(event.job_id);
	switch (event.type) {
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
			void refreshJobs();
			const m = messages();
			const job = app.jobs.find((j) => j.id === event.job_id);
			void notify(m.app.name, `${job?.name ?? ''}: ${m.status[event.status]}`);
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
