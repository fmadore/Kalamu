import { describe, expect, it } from 'vitest';
import { apparatus } from './apparatus.js';
import { withReading } from './withReading.js';

/** The entry for the words of `base` from `start`, `lemma` long, as the apparatus keys it. */
const at = (base: string, lemma: string, from = 0) => {
	const start = base.indexOf(lemma, from);
	return { start, end: start + lemma.length, lemma };
};

describe('a reading taken into the text', () => {
	it('replaces words at the start, in the middle and at the end', () => {
		const base = 'Le minaret est haut';
		expect(withReading(base, at(base, 'Le'), 'La')).toBe('La minaret est haut');
		expect(withReading(base, at(base, 'minaret'), 'marché')).toBe('Le marché est haut');
		expect(withReading(base, at(base, 'haut'), 'bas')).toBe('Le minaret est bas');
	});

	it('counts places as the interface does, in UTF-16 units, through accents, Arabic and emoji', () => {
		const base = 'Écoute 🎧 الإمام à Bobo';
		expect(withReading(base, at(base, 'الإمام'), 'الشيخ')).toBe('Écoute 🎧 الشيخ à Bobo');
		expect(withReading(base, at(base, 'à'), 'de')).toBe('Écoute 🎧 الإمام de Bobo');
		expect(withReading(base, at(base, '🎧'), '📻')).toBe('Écoute 📻 الإمام à Bobo');
		expect(withReading(base, at(base, 'Écoute'), 'Écouté')).toBe('Écouté 🎧 الإمام à Bobo');
	});

	it('takes a reading with accents and emoji in the place of plain words', () => {
		const base = 'a cafe here';
		expect(withReading(base, at(base, 'cafe'), 'café ☕')).toBe('a café ☕ here');
	});

	it('takes an empty reading by leaving the words out, the spaces around them kept', () => {
		const base = 'one two three';
		// Both spaces stay: the text reads "one  three".
		expect(withReading(base, at(base, 'two'), '')).toBe('one  three');
		expect(withReading(base, at(base, 'one'), '')).toBe(' two three');
	});

	it('puts words another witness adds where the empty lemma stands', () => {
		const base = 'one three';
		expect(withReading(base, { start: 3, end: 3, lemma: '' }, ' two')).toBe('one two three');
		expect(withReading(base, { start: 0, end: 0, lemma: '' }, 'zero ')).toBe('zero one three');
		expect(withReading(base, { start: 9, end: 9, lemma: '' }, ' four')).toBe('one three four');
	});

	it('keeps punctuation shared by every reading after the reading taken', () => {
		const base = 'Il vit le minaret. Puis';
		const [entry] = apparatus(base, [
			{
				siglum: 'b',
				spans: [
					{ kind: 'same', text: 'Il vit le ' },
					{ kind: 'removed', text: 'minaret.' },
					{ kind: 'added', text: 'marché.' },
					{ kind: 'same', text: ' Puis' }
				]
			}
		]);
		expect(entry.lemma).toBe('minaret');
		expect(base.slice(entry.start, entry.end)).toBe('minaret.');
		expect(entry.readings[0].text).toBe('marché');
		expect(withReading(base, entry, entry.readings[0].text)).toBe('Il vit le marché. Puis');
	});

	it('keeps a whole lemma of shared punctuation when the reading is empty', () => {
		const base = 'Fin. Puis';
		expect(withReading(base, { start: 3, end: 4, lemma: '' }, '!')).toBe('Fin!. Puis');
	});

	it('keeps no tail when the lemma is not what the text holds at that place', () => {
		// A stale entry: its words are replaced whole, whatever they were.
		const base = 'Le minaret est haut';
		expect(withReading(base, { start: 3, end: 10, lemma: 'marché' }, 'toit')).toBe(
			'Le toit est haut'
		);
	});

	it('appends past the end of the text, as slicing does', () => {
		expect(withReading('abc', { start: 5, end: 7, lemma: '' }, 'z')).toBe('abcz');
	});
});
