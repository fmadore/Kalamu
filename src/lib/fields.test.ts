import { describe, expect, it } from 'vitest';
import type { Field } from './bindings/Field.js';
import { fromDraft, toDraft } from './fields.js';

const field = (key: string, kind: Field['kind'], reason = false): Field => ({
	key,
	label: key,
	instructions: '',
	kind,
	reason
});

const fields: Field[] = [
	field('summary', { type: 'text' }, true),
	field('people', { type: 'list', max: null }),
	field('genre', { type: 'choice', values: ['news', 'opinion'] }),
	field('topics', { type: 'choices', values: ['hajj', 'schools', 'press'] })
];

describe('answers in an editor', () => {
	it('holds each kind as its control does', () => {
		const draft = toDraft(fields, [
			{ key: 'summary', value: 'A pilgrimage.', reason: 'The headline says so.' },
			{ key: 'people', value: ['A. Diallo', 'B. Traoré'] },
			{ key: 'genre', value: 'news' },
			{ key: 'topics', value: ['hajj'] }
		]);
		expect(draft).toEqual({
			summary: 'A pilgrimage.',
			people: 'A. Diallo\nB. Traoré',
			genre: 'news',
			topics: ['hajj']
		});
	});

	it('gives empty controls for fields not answered', () => {
		expect(toDraft(fields, [])).toEqual({ summary: '', people: '', genre: '', topics: [] });
	});

	it('reads the draft back, one item per line, keeping the model’s reasons', () => {
		const before = [{ key: 'summary', value: 'A pilgrimage.', reason: 'The headline says so.' }];
		const answers = fromDraft(
			fields,
			{
				summary: ' The hajj of 1931. ',
				people: 'A. Diallo\n\n B. Traoré ',
				genre: 'opinion',
				topics: []
			},
			before
		);
		expect(answers).toEqual([
			{ key: 'summary', value: 'The hajj of 1931.', reason: 'The headline says so.' },
			{ key: 'people', value: ['A. Diallo', 'B. Traoré'] },
			{ key: 'genre', value: 'opinion' },
			{ key: 'topics', value: [] }
		]);
	});
});
