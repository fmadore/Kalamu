import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { describeError, errorStep } from './errors.js';
import { en } from './i18n/index.js';
import { fr } from './i18n/fr.js';

/** Every id the Rust side can send, read from its source so a new id needs a sentence. */
function rustIds(): string[] {
	const sources = [
		'crates/engine/src/lib.rs',
		'crates/engine/src/error.rs',
		'crates/prompts/src/lib.rs',
		'crates/core/src/analysis/cited/edit.rs',
		'crates/core/src/workflow.rs',
		'crates/core/src/provider.rs',
		'crates/core/src/units.rs',
		'crates/core/src/profile.rs',
		'crates/core/src/dictionary.rs',
		'crates/core/src/project.rs',
		'crates/core/src/project_file.rs',
		'crates/offline/src/lib.rs',
		'crates/search/src/lib.rs',
		'crates/engine/src/meaning/error.rs',
		'crates/core/src/address.rs',
		'crates/providers/src/own/discover.rs',
		'crates/catalog/src/own/problems.rs',
		'crates/catalog/src/own/draft.rs',
		'src-tauri/src/errors.rs',
		...readdirSync('src-tauri/src/commands').map((f) => `src-tauri/src/commands/${f}`)
	].map((p) => readFileSync(p, 'utf8'));
	const ids = new Set<string>();
	for (const source of sources) {
		for (const match of source.matchAll(/"([a-z]+\.[a-z_]+)"/g)) {
			const id = match[1];
			if (
				/^(job|media|prompt|provider|output|keys|settings|catalog|file|app|offline|source|project|workflow|search|meaning|unit|profile|dictionary|cited|server)\./.test(
					id
				)
			) {
				ids.add(id);
			}
		}
	}
	return [...ids];
}

describe('error sentences', () => {
	it('exist for every id the core sends', () => {
		const missing = rustIds().filter((id) => !(id in en.error));
		expect(missing).toEqual([]);
	});

	it('exist in French for every id the core sends', () => {
		const missing = rustIds().filter((id) => !(id in fr.error));
		expect(missing).toEqual([]);
	});

	it('name the server, the host or the model a server’s failure is about', () => {
		const sentence = (id: string, params?: Record<string, string>) =>
			describeError(params ? { id, params } : { id }, en);
		const nothing = sentence('server.nothing_answers', { host: '192.168.1.20:11434' });
		expect(nothing).toContain('Nothing answers at 192.168.1.20:11434');
		// Never blamed on the internet connection: what to check is the server's program.
		expect(nothing).not.toMatch(/internet/i);
		expect(nothing).toContain('running');
		expect(sentence('server.key_refused', { host: 'locked.lan' })).toContain('locked.lan');
		expect(sentence('server.did_nothing', { model: 'llama3.1:8b' })).toContain('llama3.1:8b');
		expect(sentence('server.at_once', { server: 'Lab', most: '8' })).toContain('from 1 to 8');
		// An address typed is said plainly; one from the saved list names its server.
		expect(sentence('server.address_scheme')).toBe(
			'The address must start with http:// or https://.'
		);
		expect(sentence('server.address_scheme', { server: 'Lab' })).toContain('“Lab”');
		const french = describeError({ id: 'server.key_needed', params: { host: 'uni.example' } }, fr);
		expect(french).toContain('uni.example');
		expect(french).toContain('Ce serveur demande une clé');
	});

	it('add the detail after the sentence and fall back to it for unknown ids', () => {
		expect(describeError({ id: 'provider.auth', detail: '401' }, en)).toBe(
			'The provider rejected the API key. Check it in Settings. (401)'
		);
		expect(describeError({ id: 'future.thing', detail: 'disk full' }, en)).toBe('disk full');
		expect(describeError(new Error('boom'), en)).toBe('boom');
	});

	it('put the values an error names into its sentence', () => {
		const m = {
			...en,
			error: {
				...en.error,
				'server.nothing_answers': (p: Record<string, string>) => `No ${p.host}`
			}
		} as unknown as typeof en;
		expect(
			describeError({ id: 'server.nothing_answers', params: { host: '10.0.0.2:11434' } }, m)
		).toBe('No 10.0.0.2:11434');
	});

	it('name the step of a workflow once, before the sentence', () => {
		const refused = { id: 'job.no_language', step: 2 };
		expect(describeError(refused, en)).toBe('Step 2: Choose the language to translate into.');
		expect(errorStep(refused)).toBe(2);
		expect(errorStep({ id: 'job.no_language' })).toBeNull();
	});

	it('says a piece whose answer stayed unusable failed for good, with the engine’s reason', () => {
		const failed = {
			id: 'provider.invalid_answer',
			detail: 'the model left every field empty, though the text runs to about 4,800 words'
		};
		const said = describeError(failed, en);
		expect(said).toContain('even when asked again');
		expect(said).not.toContain('asks it again');
		expect(said).toContain('(the model left every field empty');
	});
});
