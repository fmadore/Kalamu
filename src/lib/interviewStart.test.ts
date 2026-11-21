import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { EstimateRow } from './bindings/EstimateRow.js';
import type { TextTooLong } from './bindings/TextTooLong.js';
import { describeError } from './errors.js';
import { en, fr } from './i18n/index.js';
import { tooLongOf } from './interviewStart.svelte.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;

const row = (provider: string, model: string, tooLong?: TextTooLong): EstimateRow => ({
	provider,
	model,
	pages: 40,
	estimate: { usd: 0.4, economy_usd: null },
	merge: false,
	...(tooLong ? { too_long: [tooLong] } : {})
});

const estimate = (...rows: EstimateRow[]): Estimate => ({
	rows,
	total_usd: 0.4,
	total_economy_usd: 0.4
});

describe('a transcript too long for the model chosen', () => {
	it('says nothing when the estimate says nothing, or there is none yet', () => {
		expect(tooLongOf(null, catalog, en)).toBeNull();
		expect(tooLongOf(estimate(row('gemini', 'gemini-4-hydrogen')), catalog, en)).toBeNull();
	});

	it('names the model and both counts, written as the interface language writes numbers', () => {
		const long = estimate(
			row('local-llm', 'gemma-4-e4b', {
				source: 's1',
				file: 'mariam.txt',
				words: 100_000,
				most_words: 11_000
			})
		);
		expect(tooLongOf(long, catalog, en)).toBe(
			'This transcript is too long for Gemma 4 E4B: about 100,000 words, where it reads about 11,000 at once. Choose another model under More choices.'
		);
		expect(tooLongOf(long, catalog, fr)).toMatch(
			/^Cette transcription est trop longue pour Gemma 4 E4B : environ 100\s000 mots, alors qu’il en lit environ 11\s000 à la fois\./
		);
	});

	it('takes the model a row names, and its id when the catalogue does not list it', () => {
		const long = estimate(
			row('gemini', 'gemini-4-hydrogen'),
			row('elsewhere', 'unknown-model', { source: 's1', file: 'a.txt', words: 20, most_words: 10 })
		);
		expect(tooLongOf(long, catalog, en)).toContain('too long for unknown-model:');
	});

	it('is said by New job as its other refusals, the engine’s numbers after the sentence', () => {
		const refused = {
			id: 'job.text_too_long',
			detail:
				'mariam.txt is too long for Gemma 4 E4B: about 12,000 words, where it reads about 11,000 at once'
		};
		expect(describeError(refused, en)).toBe(
			'This text is too long for the model chosen to read at once. Choose a model that reads more. (mariam.txt is too long for Gemma 4 E4B: about 12,000 words, where it reads about 11,000 at once)'
		);
		expect(describeError(refused, fr)).toMatch(/^Ce texte est trop long .* Choisissez un modèle/);
	});
});
