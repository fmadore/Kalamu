import { describe, expect, it } from 'vitest';
import type { Hit } from './bindings/Hit.js';
import type { ResultHits } from './bindings/ResultHits.js';
import { en } from './i18n/index.js';
import {
	blankRequest,
	hitOrigin,
	hitPlace,
	keepSearch,
	lastSearch,
	reviewFind
} from './projectSearch.js';

function hit(over: Partial<Hit> = {}): Hit {
	return {
		index: 2,
		piece: { kind: 'page', number: 3 },
		nth: 1,
		field: null,
		entry: null,
		reason: false,
		before: 'la ',
		words: 'Mission catholique',
		after: ' de Koupéla',
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

	it('opens the review on the match chosen, with the words shown', () => {
		expect(reviewFind(result, hit(), 'mission catholique')).toEqual({
			query: 'mission catholique',
			setupId: 's',
			index: 2,
			nth: 1
		});
	});

	it('keeps the last search of each project for coming back to it', () => {
		expect(lastSearch('p1')).toBeUndefined();
		const request = { ...blankRequest(), query: 'zakat' };
		keepSearch('p1', request, null);
		expect(lastSearch('p1')?.request.query).toBe('zakat');
		expect(lastSearch('p2')).toBeUndefined();
		expect(blankRequest()).toMatchObject({ results: 'preferred', generated: false });
	});
});
