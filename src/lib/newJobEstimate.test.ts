import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRow } from './bindings/EstimateRow.js';
import type { TextTooLong } from './bindings/TextTooLong.js';
import { en, fr } from './i18n/index.js';
import { tooLongOf } from './newJobEstimate.svelte.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;

const row = (provider: string, model: string, tooLong: TextTooLong[] = []): EstimateRow => ({
	provider,
	model,
	pages: 19,
	estimate: { usd: 0, economy_usd: null },
	merge: false,
	...(tooLong.length ? { too_long: tooLong } : {})
});

const estimate = (...rows: EstimateRow[]): Estimate => ({
	rows,
	total_usd: 0,
	total_economy_usd: 0
});

const long = (file: string): TextTooLong => ({
	source: file,
	file,
	words: 48_000,
	most_words: 11_000
});

// B15: Gemma 4 E2B and a 47,621-word text, analysed with Summary and keywords.
describe('a text too long for a model chosen in New job', () => {
	it('says nothing when the estimate names no text, or there is none yet', () => {
		expect(tooLongOf(null, catalog, en)).toBeNull();
		expect(tooLongOf(estimate(row('local-llm', 'gemma-4-e2b')), catalog, en)).toBeNull();
	});

	it('names the file, the model and both counts, in the interface language', () => {
		const one = estimate(row('local-llm', 'gemma-4-e2b', [long('long-text.txt')]));
		expect(tooLongOf(one, catalog, en)).toBe(
			'long-text.txt is too long for Gemma 4 E2B: about 48,000 words, where it reads about 11,000 at once. Choose another model, or split the text into shorter documents.'
		);
		expect(tooLongOf(one, catalog, fr)).toMatch(
			/^long-text\.txt est trop long pour Gemma 4 E2B\s: environ 48\s000 mots, alors qu’il en lit environ 11\s000 à la fois\. Choisissez un autre modèle/
		);
	});

	it('counts the other texts too long after the first', () => {
		const two = estimate(row('local-llm', 'gemma-4-e2b', [long('a.txt'), long('b.txt')]));
		expect(tooLongOf(two, catalog, en)).toContain('at once, and so is 1 other text.');
		const three = estimate(
			row('local-llm', 'gemma-4-e2b', [long('a.txt'), long('b.txt'), long('c.txt')])
		);
		expect(tooLongOf(three, catalog, en)).toContain('at once, and so are 2 other texts.');
		expect(tooLongOf(three, catalog, fr)).toContain('de même que 2 autres textes.');
	});

	it('takes the first set-up whose model cannot read a text', () => {
		const second = estimate(
			row('gemini', 'gemini-4-hydrogen'),
			row('local-llm', 'gemma-4-e2b', [long('a.txt')])
		);
		expect(tooLongOf(second, catalog, en)).toContain('too long for Gemma 4 E2B:');
	});
});
