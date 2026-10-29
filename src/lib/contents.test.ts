import { describe, expect, it } from 'vitest';
import type { Field } from './bindings/Field.js';
import { asAnswers, entriesOf, pagesOf, valueText } from './contents.js';
import { fromDraft, toDraft } from './fields.js';

const fields: Field[] = [
	{ key: 'title', label: 'Title', instructions: '', kind: { type: 'text' }, reason: false },
	{ key: 'pages', label: 'Pages', instructions: '', kind: { type: 'text' }, reason: false },
	{
		key: 'authors',
		label: 'Authors',
		instructions: '',
		kind: { type: 'list', max: null },
		reason: false
	}
];

describe('a table of contents in the review', () => {
	it('reads the pages an entry names', () => {
		expect(pagesOf('1, 3-5')).toEqual([1, 3, 4, 5]);
		expect(pagesOf('p. 12–13; 20')).toEqual([12, 13, 20]);
		expect(pagesOf('')).toEqual([]);
	});

	it('edits an entry and stores it back as answers', () => {
		const entry = [
			{ key: 'title', value: '[Éditorial] Le Hajj' },
			{ key: 'pages', value: '1-2' },
			{ key: 'authors', value: ['La Rédaction', 'A. Diallo'] }
		];
		const draft = toDraft(fields, entry);
		expect(draft.authors).toBe('La Rédaction\nA. Diallo');
		draft.pages = ' 1-3 ';
		const back = fromDraft(fields, draft);
		expect(valueText(back, 'pages')).toBe('1-3');
		expect(valueText(back, 'authors')).toBe('La Rédaction; A. Diallo');
		expect(entriesOf(asAnswers([back]))).toEqual([back]);
		// No entries read back as an empty list.
		expect(entriesOf([{ key: 'entries', value: [] }])).toEqual([]);
	});
});
