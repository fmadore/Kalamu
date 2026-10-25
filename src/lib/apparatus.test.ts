import { describe, expect, it } from 'vitest';
import type { DiffSpan } from './bindings/DiffSpan.js';
import { apparatus, lines, numbered, siglum, variants } from './apparatus.js';

const same = (text: string): DiffSpan => ({ kind: 'same', text });
const removed = (text: string): DiffSpan => ({ kind: 'removed', text });
const added = (text: string): DiffSpan => ({ kind: 'added', text });

const base = 'Dakar, le 3 mai 1957\nplus de deux cents fidèles';

describe('apparatus', () => {
	it('turns a word diff into places of variation', () => {
		const spans = [same('Dakar, le '), removed('3'), added('5'), same(' mai 1957')];
		expect(variants(spans)).toEqual([{ start: 10, end: 11, text: '5' }]);
		// Two changed words separated by a space are one variant.
		const two = [
			same('plus de '),
			removed('deux'),
			added('trois'),
			same(' '),
			removed('cents'),
			added('cent')
		];
		expect(variants(two)).toEqual([{ start: 8, end: 18, text: 'trois cent' }]);
	});

	it('numbers entries by line and merges witnesses that vary at the same place', () => {
		const b = [
			same('Dakar, le '),
			removed('3'),
			added('5'),
			same(' mai 1957\nplus de '),
			removed('deux'),
			added('trois'),
			same(' cents fidèles')
		];
		const corr = [
			same('Dakar, le '),
			removed('3'),
			added('8'),
			same(' mai 1957\nplus de deux cents fidèles')
		];
		const entries = apparatus(base, [
			{ siglum: 'b', spans: b },
			{ siglum: 'a', spans: corr }
		]);
		expect(entries.map((e) => [e.id, e.line, e.lemma])).toEqual([
			[1, 1, '3'],
			[2, 2, 'deux']
		]);
		expect(entries[0].readings).toEqual([
			{ siglum: 'b', text: '5' },
			{ siglum: 'a', text: '8' }
		]);
	});

	it('cuts lines at lemmata and numbers every fifth line and every line with a variant', () => {
		const entries = apparatus(base, [
			{
				siglum: 'b',
				spans: [
					same('Dakar, le '),
					removed('3'),
					added('5'),
					same(' mai 1957\nplus de deux cents fidèles')
				]
			}
		]);
		const [first, second] = lines(base, entries);
		expect(first.parts).toEqual([
			{ text: 'Dakar, le ', entry: null },
			{ text: '3', entry: 1 },
			{ text: ' mai 1957', entry: null }
		]);
		expect(second.parts).toEqual([{ text: 'plus de deux cents fidèles', entry: null }]);
		expect(numbered(1, entries)).toBe(true);
		expect(numbered(2, entries)).toBe(false);
		expect(numbered(5, entries)).toBe(true);
	});

	it('does not count blank lines between paragraphs', () => {
		const text = 'Titre\n\nPremier paragraphe\nsuite 3 mai';
		const entries = apparatus(text, [
			{
				siglum: 'b',
				spans: [same('Titre\n\nPremier paragraphe\nsuite '), removed('3'), added('5'), same(' mai')]
			}
		]);
		expect(entries[0].line).toBe(3);
		expect(lines(text, entries).map((l) => l.number)).toEqual([1, 0, 2, 3]);
		expect(numbered(0, entries)).toBe(false);
	});

	it('leaves out punctuation that the lemma and every reading share', () => {
		const [entry] = apparatus('du minaret.', [
			{ siglum: 'b', spans: [same('du '), removed('minaret.'), added('marché.')] }
		]);
		expect(entry).toMatchObject({ lemma: 'minaret', readings: [{ siglum: 'b', text: 'marché' }] });
		const [kept] = apparatus('fin.', [{ siglum: 'b', spans: [removed('fin.'), added('fin!')] }]);
		expect(kept.lemma).toBe('fin.');
	});

	it('names set-ups a, b, c', () => {
		expect([0, 1, 2].map(siglum)).toEqual(['a', 'b', 'c']);
	});

	it('records additions and omissions', () => {
		const omitted = apparatus('a b c', [
			{ siglum: 'b', spans: [same('a '), removed('b'), same(' c')] }
		]);
		expect(omitted[0]).toMatchObject({ lemma: 'b', readings: [{ siglum: 'b', text: '' }] });
		const addition = apparatus('a c', [
			{ siglum: 'b', spans: [same('a '), added('b '), same('c')] }
		]);
		expect(addition[0]).toMatchObject({ lemma: '', readings: [{ siglum: 'b', text: 'b' }] });
	});
});
