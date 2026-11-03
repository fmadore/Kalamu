import { describe, expect, it } from 'vitest';
import type { Hit } from './bindings/Hit.js';
import type { ResultHits } from './bindings/ResultHits.js';
import { en } from './i18n/index.js';
import {
	blankRequest,
	hitOrigin,
	hitPlace,
	hitWords,
	keepSearch,
	lastSearch,
	reviewFind
} from './projectSearch.js';

function hit(over: Partial<Hit> = {}): Hit {
	return {
		index: 2,
		piece: { kind: 'page', number: 3 },
		term: 'mission catholique',
		nth: 1,
		field: null,
		entry: null,
		entry_number: null,
		reason: false,
		before: 'la ',
		words: 'Mission catholique',
		after: ' de Koupéla',
		marks: [{ start: 0, end: 18 }],
		at_ms: null,
		reviewed: false,
		corrected: false,
		...over
	};
}

const result: ResultHits = {
	job_id: 'j',
	job_name: 'Bulletins',
	setup_id: 's',
	setup_label: 'gemini',
	source_id: 'src',
	task: 'text_extraction',
	preferred: true,
	generated: false,
	stale: false,
	hits: []
};

describe('project search', () => {
	it('places a match on its page, or at the time of its line in a recording', () => {
		expect(hitPlace(hit(), en)).toBe('Page 3');
		const timed = hit({
			piece: { kind: 'segment', index: 1, start_ms: 1_200_000, end_ms: 2_400_000 },
			at_ms: 1_262_000
		});
		expect(hitPlace(timed, en)).toBe('00:21:02');
		// A line whose time cannot be told is placed at its piece.
		expect(hitPlace({ ...timed, at_ms: null }, en)).toBe('From 00:20:00');
	});

	it('says which answer a match was found in, and when it was the reason', () => {
		expect(hitOrigin(hit(), en)).toBe('');
		expect(hitOrigin(hit({ field: { key: 'people', label: 'People' } }), en)).toBe('People');
		expect(
			hitOrigin(
				hit({ field: { key: 'summary', label: 'Summary' }, entry: 'La zakat', reason: true }),
				en
			)
		).toBe('La zakat › Summary (reason)');
	});

	it('opens the review on the match chosen, with its words, field and entry', () => {
		expect(reviewFind(result, hit())).toEqual({
			query: 'mission catholique',
			setupId: 's',
			index: 2,
			nth: 1,
			field: null,
			entry: null
		});
		const entry = hit({
			term: 'zakat',
			nth: 0,
			field: { key: 'summary', label: 'Summary' },
			entry: 'La zakat',
			entry_number: 3
		});
		expect(reviewFind(result, entry)).toMatchObject({ query: 'zakat', field: 'summary', entry: 3 });
	});

	it('marks only the words looked for, not those between them', () => {
		expect(hitWords(hit())).toEqual([{ text: 'Mission catholique', found: true }]);
		const near = hit({
			words: 'imam est venu, et Ouédraogo',
			marks: [
				{ start: 0, end: 4 },
				{ start: 18, end: 27 }
			]
		});
		expect(hitWords(near)).toEqual([
			{ text: 'imam', found: true },
			{ text: ' est venu, et ', found: false },
			{ text: 'Ouédraogo', found: true }
		]);
		// Marks that overlap (a word inside a phrase also looked for) are marked once.
		const overlapping = hit({
			words: 'Union culturelle',
			marks: [
				{ start: 0, end: 16 },
				{ start: 0, end: 5 }
			]
		});
		expect(hitWords(overlapping)).toEqual([{ text: 'Union culturelle', found: true }]);
	});

	it('keeps the last search of each project for coming back to it', () => {
		expect(lastSearch('p1')).toBeUndefined();
		const request = { ...blankRequest(), query: 'zakat' };
		keepSearch('p1', request, null);
		expect(lastSearch('p1')?.request.query).toBe('zakat');
		expect(lastSearch('p2')).toBeUndefined();
		expect(blankRequest()).toMatchObject({
			mode: 'phrase',
			results: 'preferred',
			generated: false
		});
	});
});
