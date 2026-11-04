import { describe, expect, it } from 'vitest';
import type { PieceView } from './bindings/PieceView.js';
import { pieces } from './dev/mockJobs.js';
import { en } from './i18n/en.js';
import { searchedPieces, searchSummary } from './reviewSearch.svelte.js';

const view = (over: Partial<PieceView>) =>
	({ ...pieces[0], corrected_text: null, source_text: null, ...over }) as PieceView;

describe('the review’s search', () => {
	it('searches the first set-up’s text of each piece, corrections included', () => {
		const views = [
			view({ index: 0, setup_id: 'a', ai_text: 'model', corrected_text: 'mine' }),
			view({ index: 0, setup_id: 'b', ai_text: 'other' }),
			view({
				index: 1,
				setup_id: 'a',
				ai_text: 'later',
				piece: { kind: 'segment', index: 1, start_ms: 60_000, end_ms: 120_000 }
			})
		];
		expect(searchedPieces([0, 1, 2], views, 'a')).toEqual([
			{ index: 0, text: 'mine', startMs: 0, segments: views[0].segments },
			{ index: 1, text: 'later', startMs: 60_000, segments: views[2].segments },
			{ index: 2, text: '', startMs: 0, segments: [] }
		]);
	});

	it('says how many matches there are and which is shown, once the words can be searched', () => {
		expect(searchSummary('a', 0, -1, en)).toBe('');
		expect(searchSummary('imam', 0, -1, en)).toBe('Not found');
		expect(searchSummary('imam', 3, -1, en)).toBe('3 found');
		expect(searchSummary('imam', 3, 1, en)).toBe('2 of 3');
	});
});
