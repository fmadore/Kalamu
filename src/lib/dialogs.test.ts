import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { Catalog } from './bindings/Catalog.js';

// The app's answers: the providers a job or a run would send to, and those not yet agreed to;
// what was agreed to is recorded. Outside the desktop app the question is `window.confirm`.
const app = vi.hoisted(() => ({
	ahead: [] as string[],
	runAhead: [] as string[],
	unagreed: [] as string[],
	agreed: [] as string[][],
	asked: [] as string[][]
}));
vi.mock('./tauri.js', () => ({
	isTauri: () => false,
	PRIVACY_URL: 'https://example.org/privacy',
	api: {
		providersAhead: async () => app.ahead,
		runProvidersAhead: async () => app.runAhead,
		providersToConsent: async (providers: string[]) => {
			app.asked.push(providers);
			return providers.filter((p) => app.unagreed.includes(p));
		},
		consentProviders: async (providers: string[]) => {
			app.agreed.push(providers);
		}
	}
}));

const { mayGoOn, mayResume } = await import('./dialogs.js');

const catalog = {
	providers: [
		{
			id: 'own-0000abcd',
			name: 'Lab Ollama',
			key_url: '',
			key_prefix: null,
			local: false,
			own: {
				address: 'http://10.0.0.5:11434',
				host: '10.0.0.5',
				this_computer: false,
				key: false,
				dialect: 'ollama'
			}
		}
	],
	models: []
} as unknown as Catalog;

describe('consent before work starts again', () => {
	const confirm = vi.fn<(message?: string) => boolean>();
	beforeEach(() => {
		Object.assign(app, { ahead: [], runAhead: [], unagreed: [], agreed: [], asked: [] });
		confirm.mockReset();
		vi.stubGlobal('window', { confirm });
	});

	it('asks for the server a resumed job sends to, by its name and its host', async () => {
		app.ahead = ['mistral', 'own-0000abcd'];
		app.unagreed = ['own-0000abcd'];
		confirm.mockReturnValue(true);
		expect(await mayResume('job-1', catalog)).toBe(true);
		expect(app.asked).toEqual([['mistral', 'own-0000abcd']]);
		expect(confirm).toHaveBeenCalledOnce();
		expect(confirm.mock.calls[0][0]).toContain('Lab Ollama');
		expect(confirm.mock.calls[0][0]).toContain('10.0.0.5');
		expect(app.agreed).toEqual([['own-0000abcd']]);
	});

	it('records nothing when the researcher declines, so the job is not started', async () => {
		app.ahead = ['own-0000abcd'];
		app.unagreed = ['own-0000abcd'];
		confirm.mockReturnValue(false);
		expect(await mayResume('job-1', catalog)).toBe(false);
		expect(app.agreed).toEqual([]);
	});

	it('asks nothing when every provider was agreed to', async () => {
		app.ahead = ['own-0000abcd'];
		expect(await mayResume('job-1', catalog)).toBe(true);
		expect(confirm).not.toHaveBeenCalled();
	});

	it('asks, before a step is approved, for the steps it may start', async () => {
		app.runAhead = ['own-0000abcd'];
		app.unagreed = ['own-0000abcd'];
		confirm.mockReturnValue(false);
		expect(await mayGoOn('run-1', catalog)).toBe(false);
		expect(app.asked).toEqual([['own-0000abcd']]);
		app.runAhead = [];
		expect(await mayGoOn('run-1', catalog)).toBe(true);
	});
});
