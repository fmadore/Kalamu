import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRow } from './bindings/EstimateRow.js';
import type { TextTooLong } from './bindings/TextTooLong.js';
import { en, fr } from './i18n/index.js';
import { notBatchedOf, tooLongOf } from './newJobEstimate.svelte.js';

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

// #242: a screening, a timed translation, and a model without a batch service, in economy mode.
describe('the models that get no discount in economy mode', () => {
	const priced = (
		provider: string,
		model: string,
		economy: number | null,
		more: Partial<EstimateRow> = {}
	): EstimateRow => ({
		...row(provider, model),
		estimate: { usd: 0.2, economy_usd: economy },
		...more
	});

	it('says nothing when every model is discounted', () => {
		expect(notBatchedOf([priced('gemini', 'gemini-4-hydrogen', 0.1)], catalog, en)).toEqual([]);
	});

	it('says an analysis that quotes the text is sent at once, though the model batches', () => {
		const rows = [
			priced('gemini', 'gemini-4-hydrogen', null, { at_once: 'quotes' }),
			priced('local-llm', 'gemma-4-e2b', null)
		];
		expect(notBatchedOf(rows, catalog, en)).toEqual([
			'Gemma 4 E2B has no batch service in Kalamu and still runs page by page, at its usual price.',
			'With Gemini 4 Hydrogen, an analysis that quotes the text is sent at once, at the usual price, so that Kalamu can check each quotation against the text.'
		]);
		expect(notBatchedOf(rows, catalog, fr)[1]).toBe(
			'Avec Gemini 4 Hydrogen, une analyse qui cite le texte est envoyée tout de suite, au prix habituel, pour que Kalamu puisse retrouver chaque citation dans le texte.'
		);
	});

	it('says a transcript translated line by line is sent at once', () => {
		const rows = [priced('gemini', 'gemini-4-hydrogen', null, { at_once: 'timed_lines' })];
		expect(notBatchedOf(rows, catalog, en)).toEqual([
			'With Gemini 4 Hydrogen, a transcript translated line by line is sent at once, at the usual price, so that Kalamu can check that no line is missing.'
		]);
		expect(notBatchedOf(rows, catalog, fr)[0]).toContain('qu’aucune ligne ne manque');
	});

	it('does not name a model for the step after the others, nor twice', () => {
		const rows = [
			priced('gemini', 'gemini-4-hydrogen', 0.1),
			priced('gemini', 'gemini-4-hydrogen', null, { merge: true }),
			priced('local-llm', 'gemma-4-e2b', null),
			priced('local-llm', 'gemma-4-e2b', null)
		];
		expect(notBatchedOf(rows, catalog, en)).toEqual([
			'Gemma 4 E2B has no batch service in Kalamu and still runs page by page, at its usual price.'
		]);
	});
});
