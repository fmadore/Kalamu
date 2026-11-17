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

const { app, applyProgress } = await import('./state.svelte.js');

describe('live progress', () => {
	it('reads the offline status again when a job ends, so what it held can be removed', async () => {
		const released = { language_data: [{ id: 'tesseract-fra', in_use: false }] };
		app.offline = { language_data: [{ id: 'tesseract-fra', in_use: true }] } as OfflineStatus;
		offline.status = released;
		applyProgress({ type: 'job_finished', job_id: 'job', status: 'completed' });
		await vi.waitFor(() => expect(app.offline).toEqual(released));
	});
});
