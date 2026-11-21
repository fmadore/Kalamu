import { describe, expect, it, vi } from 'vitest';
import type { OfflineStatus } from './bindings/OfflineStatus.js';

// The engine's answers, as the app would give them: the offline status read again shows the
// language data a job held released.
const offline = vi.hoisted(() => ({ status: null as unknown }));
vi.mock('./tauri.js', () => ({
	api: {
		offlineStatus: async () => offline.status,
		listJobs: async () => [],
		listRuns: async () => []
	},
	openUrl: async () => undefined
}));

const { app, applyProgress, holdToast, toast, undoFromToast, TOAST_ACTION_MS, TOAST_MS } =
	await import('./state.svelte.js');

describe('toasts', () => {
	it('keeps a toast that offers to undo for twelve seconds, and while it is pointed at', () => {
		vi.useFakeTimers();
		try {
			const run = vi.fn();
			toast('IMG_0415.jpg left out of this job', { action: { label: 'Undo', run } });
			vi.advanceTimersByTime(TOAST_MS + 1000);
			expect(app.toast).toBe('IMG_0415.jpg left out of this job');
			// Pointed at, it stays however long; once left, its time starts again.
			holdToast('pointer', true);
			vi.advanceTimersByTime(TOAST_ACTION_MS * 2);
			expect(app.toast).not.toBeNull();
			holdToast('pointer', false);
			vi.advanceTimersByTime(TOAST_ACTION_MS - 1);
			expect(app.toast).not.toBeNull();
			vi.advanceTimersByTime(1);
			expect(app.toast).toBeNull();
			expect(run).not.toHaveBeenCalled();
			// A plain confirmation goes after four seconds.
			toast('Saved');
			vi.advanceTimersByTime(TOAST_MS);
			expect(app.toast).toBeNull();
		} finally {
			vi.useRealTimers();
		}
	});

	it('undoes from the keyboard what the toast shown offers to undo, and nothing else', () => {
		const run = vi.fn();
		toast('Saved');
		expect(undoFromToast()).toBe(false);
		toast('IMG_0415.jpg left out of this job', { action: { label: 'Undo', run } });
		expect(undoFromToast()).toBe(true);
		expect(run).toHaveBeenCalledOnce();
		expect(app.toast).toBeNull();
		expect(undoFromToast()).toBe(false);
	});
});

describe('live progress', () => {
	it('reads the offline status again when a job ends, so what it held can be removed', async () => {
		const released = { language_data: [{ id: 'tesseract-fra', in_use: false }] };
		app.offline = { language_data: [{ id: 'tesseract-fra', in_use: true }] } as OfflineStatus;
		offline.status = released;
		applyProgress({ type: 'job_finished', job_id: 'job', status: 'completed' });
		await vi.waitFor(() => expect(app.offline).toEqual(released));
	});
});
