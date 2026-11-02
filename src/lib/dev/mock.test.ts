import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { DESKTOP_ONLY, NO_RESULT } from './noResult.js';

describe('the development mock', () => {
	it('answers every command the interface calls, or says why it does not', () => {
		const tauri = readFileSync('src/lib/tauri.ts', 'utf8');
		const called = [...tauri.matchAll(/invoke(?:<[^>]*>)?\(\s*'([a-z_]+)'/g)].map((m) => m[1]);
		const mocks = readdirSync('src/lib/dev')
			.filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
			.map((f) => readFileSync(`src/lib/dev/${f}`, 'utf8'))
			.join('\n');
		const answered = new Set([...mocks.matchAll(/case '([a-z_]+)'/g)].map((m) => m[1]));
		const unanswered = called.filter(
			(c) => !answered.has(c) && !NO_RESULT.has(c) && !DESKTOP_ONLY.has(c)
		);
		expect(unanswered).toEqual([]);
	});
});
