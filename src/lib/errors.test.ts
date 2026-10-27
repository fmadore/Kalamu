import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { describeError } from './errors.js';
import { en } from './i18n/index.js';

/** Every id the Rust side can send, read from its source so a new id needs a sentence. */
function rustIds(): string[] {
	const sources = [
		'crates/engine/src/lib.rs',
		'crates/core/src/provider.rs',
		'crates/offline/src/lib.rs',
		'src-tauri/src/errors.rs',
		...readdirSync('src-tauri/src/commands').map((f) => `src-tauri/src/commands/${f}`)
	].map((p) => readFileSync(p, 'utf8'));
	const ids = new Set<string>();
	for (const source of sources) {
		for (const match of source.matchAll(/"([a-z]+\.[a-z_]+)"/g)) {
			const id = match[1];
			if (
				/^(job|media|prompt|provider|output|keys|settings|catalog|file|app|offline|source)\./.test(
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

	it('add the detail after the sentence and fall back to it for unknown ids', () => {
		expect(describeError({ id: 'provider.auth', detail: '401' }, en)).toBe(
			'The provider rejected the API key. Check it in Settings. (401)'
		);
		expect(describeError({ id: 'future.thing', detail: 'disk full' }, en)).toBe('disk full');
		expect(describeError(new Error('boom'), en)).toBe('boom');
	});
});
