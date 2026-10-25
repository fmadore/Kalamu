import { describe, expect, it } from 'vitest';
import { collate, findTerm } from './analysis.js';

const a = {
	siglum: 'a',
	answers: [
		{ key: 'places', value: ['Bamako', 'Ségou'] },
		{ key: 'tone', value: 'Neutral', reason: 'Factual.' }
	]
};
const b = {
	siglum: 'b',
	answers: [
		{ key: 'places', value: ['bamako', 'Dakar'] },
		{ key: 'tone', value: 'Negative' }
	]
};

describe('analysis collation', () => {
	it('gives each list item the sigla of the set-ups that found it', () => {
		expect(collate('places', [a, b])).toEqual([
			{ value: 'Bamako', sigla: ['a', 'b'] },
			{ value: 'Ségou', sigla: ['a'] },
			{ value: 'Dakar', sigla: ['b'] }
		]);
	});

	it('groups whole answers to a choice and skips empty ones', () => {
		expect(collate('tone', [a, b])).toEqual([
			{ value: 'Neutral', sigla: ['a'] },
			{ value: 'Negative', sigla: ['b'] }
		]);
		expect(collate('tone', [{ siglum: 'c', answers: [{ key: 'tone', value: ' ' }] }])).toEqual([]);
	});

	it('marks every occurrence of a term in the source text', () => {
		expect(findTerm('À Bamako, la mosquée de bamako.', 'Bamako')).toEqual([
			{ text: 'À ', hit: false },
			{ text: 'Bamako', hit: true },
			{ text: ', la mosquée de ', hit: false },
			{ text: 'bamako', hit: true },
			{ text: '.', hit: false }
		]);
		expect(findTerm('a (b) c', '(b)')).toEqual([
			{ text: 'a ', hit: false },
			{ text: '(b)', hit: true },
			{ text: ' c', hit: false }
		]);
	});
});
