import { describe, expect, it, vi } from 'vitest';

import type { PassageEstimate } from './bindings/PassageEstimate.js';

// The app's answers: the bundled catalogue and rules, keys for every provider but OpenAI's, no
// consent left to ask, and differences worked out as "the whole text changed".
const calls = vi.hoisted(() => ({ diffs: [] as [string, string][] }));
vi.mock('./tauri.js', () => ({
	api: {
		providersToConsent: async () => [],
		listPresets: async () => [{ id: 'p1', name: 'One' }],
		diffTexts: async (a: string, b: string) => {
			calls.diffs.push([a, b]);
			return [
				{ kind: 'removed', text: a },
				{ kind: 'added', text: b }
			];
		}
	}
}));
vi.mock('./state.svelte.js', async () => {
	const { readFileSync } = await import('node:fs');
	const rules = (await import('./dev/taskRules.json')).default;
	return {
		app: {
			catalog: JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')),
			rules
		},
		isInstalled: () => true
	};
});

const { ModelChoice, ModelRequest, readSetup } = await import('./modelAsk.svelte.js');
const { IMAGES } = await import('./newJobModels.js');

const TEXT = ['text/plain'];
const keyed = (provider: string) => provider !== 'openai';

describe('the model a review asks', () => {
	it('reads a remembered set-up only when it names a provider and a model', () => {
		expect(readSetup({ provider: 'google', model: 'm' })).toEqual({
			provider: 'google',
			model: 'm',
			preset: null
		});
		expect(readSetup({ provider: 'google' })).toBeNull();
		expect(readSetup(null)).toBeNull();
	});

	it('starts from the first set-up still offered, its preset or the one given', () => {
		const gone = { provider: 'google', model: 'no-such-model', preset: 'x' };
		const kept = { provider: 'anthropic', model: 'claude-sonnet-5-5', preset: null };
		const c = new ModelChoice('text_correction', TEXT, keyed, [null, gone, kept], 'fallback');
		expect(c.provider).toBe('anthropic');
		expect(c.model).toBe('claude-sonnet-5-5');
		expect(c.preset).toBe('fallback');
		expect(c.keyed).toBe(true);
		expect(c.modelName).not.toBe('');
	});

	it('else the first provider that can be used, and its usual model', () => {
		const c = new ModelChoice('text_extraction', [IMAGES], keyed, [null], '');
		expect(c.providers.length).toBeGreaterThan(0);
		expect(c.provider).toBe(c.providers.find((p) => keyed(p.id))?.id);
		expect(c.models.some((m) => m.id === c.model)).toBe(true);
	});

	it('takes another provider’s usual model with it, and says when its key is missing', () => {
		const c = new ModelChoice('text_extraction', [IMAGES], keyed, [null], '');
		c.provider = 'anthropic';
		c.providerChanged();
		expect(c.model).toBe('claude-sonnet-5-5');
		c.provider = 'openai';
		expect(c.keyed).toBe(false);
	});

	it('reads the task’s prompt presets', async () => {
		const c = new ModelChoice('text_correction', TEXT, keyed, [null], '');
		c.loadPresets();
		await vi.waitFor(() => expect(c.presets).toHaveLength(1));
	});
});

type Answer = {
	ask: { id: string };
	status: 'answered' | 'cancelled' | 'failed';
	answer: string | null;
	error: string | null;
};

const texts = {
	waiting: (model: string) => `Waiting for ${model}`,
	ready: 'Ready',
	stopped: 'Stopped',
	failed: (reason: string) => `Failed: ${reason}`,
	changed: 'Changed'
};

function request() {
	const done = {
		cancelled: [] as string[],
		accepted: [] as [string, string][],
		discarded: [] as string[],
		refuse: null as unknown
	};
	const r = new ModelRequest<Answer, string>(texts, {
		cancel: async (id) => done.cancelled.push(id),
		accept: async (id, text) => {
			if (done.refuse) throw done.refuse;
			done.accepted.push([id, text]);
			return 'before';
		},
		discard: async (id) => done.discarded.push(id),
		changed: 'passage.changed'
	});
	return { r, done };
}

const answered = (id: string, answer: string): Answer => ({
	ask: { id },
	status: 'answered',
	answer,
	error: null
});
const estimate: PassageEstimate = { usd: 0.01, offline: false, cached: false };

describe('a request to a model from the review', () => {
	it('says what it would cost while it is asked, and why it would be refused', async () => {
		const { r } = request();
		r.estimateWith(async () => estimate);
		await vi.waitFor(() => expect(r.estimate).toEqual(estimate));
		r.estimateWith(() => Promise.reject({ id: 'passage.changed' }));
		await vi.waitFor(() => expect(r.estimateError).not.toBeNull());
		expect(r.estimate).toBeNull();
		// Nothing to ask about: no estimate, and no reason either.
		r.estimateWith(null);
		expect(r.estimate).toBeNull();
		expect(r.estimateError).toBeNull();
	});

	it('forgets an estimate that comes after the choice changed', async () => {
		const { r } = request();
		let answer: (e: PassageEstimate) => void = () => undefined;
		const forget = r.estimateWith(() => new Promise((resolve) => (answer = resolve)));
		forget?.();
		answer(estimate);
		await Promise.resolve();
		expect(r.estimate).toBeNull();
	});

	it('waits, then shows the answer marked against the words it replaces', async () => {
		const { r } = request();
		let sent = '';
		const remembered: string[] = [];
		calls.diffs = [];
		const asking = r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => remembered.push('kept'),
			send: async (id) => {
				sent = id;
				expect(r.stage).toBe('waiting');
				expect(r.said).toBe('Waiting for Gemini');
				return answered(id, 'new words');
			},
			base: 'old words'
		});
		await asking;
		expect(remembered).toEqual(['kept']);
		expect(sent).not.toBe('');
		expect(r.stage).toBe('answer');
		expect(r.draft).toBe('new words');
		expect(r.text).toBe('new words');
		expect(calls.diffs).toEqual([['old words', 'new words']]);
		expect(r.spans).toHaveLength(2);
		expect(r.said).toBe('Ready');
	});

	it('marks nothing when the answer replaces no words', async () => {
		const { r } = request();
		calls.diffs = [];
		await r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: async (id) => answered(id, 'a note'),
			base: null
		});
		expect(r.stage).toBe('answer');
		expect(r.spans).toEqual([]);
		expect(calls.diffs).toEqual([]);
	});

	it('goes back to asking when stopped, refused or failed', async () => {
		const { r, done } = request();
		const asking = r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: (id) =>
				new Promise((resolve) => {
					setTimeout(() => {
						r.stop();
						resolve({ ask: { id }, status: 'cancelled', answer: null, error: null });
					});
				}),
			base: null
		});
		await asking;
		expect(done.cancelled).toHaveLength(1);
		expect(r.stage).toBe('ask');
		expect(r.stopped).toBe(true);
		expect(r.said).toBe('Stopped');
		// Stopping when nothing is waited for asks nothing.
		r.stop();
		expect(done.cancelled).toHaveLength(1);

		await r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: async (id) => ({ ask: { id }, status: 'failed', answer: null, error: 'too long' }),
			base: null
		});
		expect(r.stage).toBe('ask');
		expect(r.stopped).toBe(false);
		expect(r.error).toBe('Failed: too long');

		await r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: () => Promise.reject(new Error('offline')),
			base: null
		});
		expect(r.stage).toBe('ask');
		expect(r.error).toBe('offline');
		expect(r.said).toBe('');
	});

	it('writes the answer or its edit, and says when the text changed meanwhile', async () => {
		const { r, done } = request();
		await r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: async (id) => answered(id, 'new words'),
			base: 'old words'
		});
		r.editing = true;
		r.draft = 'my words';
		const accepted: [string, string][] = [];
		await r.accept((id, before) => accepted.push([id, before]));
		expect(done.accepted[0]?.[1]).toBe('my words');
		expect(accepted[0]?.[1]).toBe('before');

		done.refuse = { id: 'passage.changed' };
		await r.accept(() => undefined);
		expect(r.stage).toBe('changed');
		expect(r.said).toBe('Changed');

		done.refuse = new Error('disk full');
		await r.accept(() => undefined);
		expect(r.error).toBe('disk full');
	});

	it('leaves an answer unused when discarded, and closes', async () => {
		const { r, done } = request();
		let closed = 0;
		r.discard(() => closed++);
		expect(done.discarded).toEqual([]);
		await r.ask({
			provider: 'google',
			modelName: 'Gemini',
			remember: () => undefined,
			send: async (id) => answered(id, 'new words'),
			base: null
		});
		r.discard(() => closed++);
		expect(done.discarded).toHaveLength(1);
		expect(closed).toBe(2);
	});
});
