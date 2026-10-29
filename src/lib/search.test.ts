import { describe, expect, it } from 'vitest';
import { context, find, search, searchable } from './search.js';

const words = (text: string, query: string) =>
	find(text, query).map((m) => text.slice(m.start, m.end));

describe('finding words in a text', () => {
	it('ignores capitals and accents', () => {
		expect(words('La Réfection du minaret. REFECTION faite.', 'refection')).toEqual([
			'Réfection',
			'REFECTION'
		]);
		expect(words('Le cercle de Dakar', 'CERCLE')).toEqual(['cercle']);
	});

	it('ignores Arabic vowel marks', () => {
		expect(words('قَالَ الإمامُ', 'قال')).toEqual(['قَالَ']);
	});

	it('matches any run of spaces or line breaks as one space', () => {
		const text = 'la Mission\ncatholique et la mission  catholique';
		expect(words(text, 'mission catholique')).toEqual([
			'Mission\ncatholique',
			'mission  catholique'
		]);
		expect(words(text, '  mission  ')).toEqual(['Mission', 'mission']);
	});

	it('gives offsets in the text itself, letters with marks of their own included', () => {
		// The e and its accent are two characters; the match still covers both.
		const text = 'une séance';
		expect(find(text, 'seance')).toEqual([{ start: 4, end: text.length }]);
	});

	it('does not overlap, and needs two letters', () => {
		expect(words('aaaa', 'aa')).toEqual(['aa', 'aa']);
		expect(find('a b a', 'a')).toEqual([]);
		expect(searchable(' a ')).toBe(false);
		expect(searchable('ab')).toBe(true);
	});

	it('shows a match among the words around it, on one line', () => {
		const text = 'Le Gouverneur général a reçu ce matin\nles délégués de la Mission catholique';
		const [m] = find(text, 'délégués');
		expect(context(text, m, 20)).toEqual({
			before: '…reçu ce matin les ',
			words: 'délégués',
			after: ' de la Mission…'
		});
		expect(context('les délégués', find('les délégués', 'les')[0])).toEqual({
			before: '',
			words: 'les',
			after: ' délégués'
		});
	});
});

describe('searching a source', () => {
	it('numbers the matches in each piece', () => {
		const hits = search(
			[
				{ index: 0, text: 'imam et imam', startMs: 0, segments: [] },
				{ index: 2, text: 'pas ici', startMs: 0, segments: [] },
				{ index: 3, text: "l'Imam", startMs: 0, segments: [] }
			],
			'imam'
		);
		expect(hits.map((h) => [h.index, h.nth])).toEqual([
			[0, 0],
			[0, 1],
			[3, 0]
		]);
		expect(hits.every((h) => h.lineMs === null)).toBe(true);
	});

	it('ties a recording’s matches to the lines they are in', () => {
		const segments = [
			{ start_ms: 0, text: 'Bonjour, merci.' },
			{ start_ms: 4_000, text: "Je suis l'imam." },
			{ start_ms: 9_000, text: 'Un imam venu de Bobo.' }
		];
		const text = segments.map((s) => s.text).join('\n');
		const hits = search([{ index: 1, text, startMs: 600_000, segments }], 'imam');
		expect(hits.map((h) => h.lineMs)).toEqual([4_000, 9_000]);
	});

	it('leaves the time out where a correction changed what the lines say', () => {
		const segments = [{ start_ms: 0, text: 'un imam' }];
		const hits = search(
			[{ index: 0, text: 'un imam, un autre imam', startMs: 0, segments }],
			'imam'
		);
		expect(hits.map((h) => h.lineMs)).toEqual([null, null]);
	});
});
