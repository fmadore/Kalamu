import { describe, expect, it } from 'vitest';
import type { DictionaryRow } from './bindings/DictionaryRow.js';
import { itemOf, labelChoices, proposedLabel, shownRows } from './dictionary.js';

function row(label: string, options: Partial<DictionaryRow> = {}): DictionaryRow {
	return {
		key: options.entry ? options.entry : `form:${label}`,
		entry: null,
		namespace: 'people',
		label,
		forms: [{ form: label, documents: options.documents ?? 1 }],
		documents: 1,
		listed: 1,
		read: 1,
		reviewed: false,
		unseen: false,
		apart: [],
		similar: [],
		...options
	};
}

const rows = [
	row('Moussa Traoré', { similar: ['form:Traore Moussa'], documents: 3 }),
	row('Traore Moussa', { similar: ['form:Moussa Traoré'] }),
	row('Père Éric', {
		entry: 'e1',
		reviewed: true,
		forms: [
			{ form: 'Père Éric', documents: 1 },
			{ form: 'Pere Eric', documents: 1 }
		]
	}),
	row('Old name', { entry: 'e2', unseen: true, documents: 0 }),
	row('Kankan', { namespace: 'places' })
];
const labels = (list: DictionaryRow[]) => list.map((r) => r.label);

describe('shownRows', () => {
	it('shows one field, filtered by what is left to do', () => {
		expect(labels(shownRows(rows, 'places', '', 'all'))).toEqual(['Kankan']);
		expect(labels(shownRows(rows, 'people', '', 'to_review'))).toEqual([
			'Moussa Traoré',
			'Traore Moussa',
			'Old name'
		]);
		expect(labels(shownRows(rows, 'people', '', 'similar'))).toEqual([
			'Moussa Traoré',
			'Traore Moussa'
		]);
		expect(labels(shownRows(rows, 'people', '', 'reviewed'))).toEqual(['Père Éric']);
		expect(labels(shownRows(rows, 'people', '', 'unseen'))).toEqual(['Old name']);
	});

	it('finds words in labels and forms, accents and capitals aside', () => {
		expect(labels(shownRows(rows, 'people', 'traore', 'all'))).toEqual([
			'Moussa Traoré',
			'Traore Moussa'
		]);
		expect(labels(shownRows(rows, 'people', 'pere eric', 'all'))).toEqual(['Père Éric']);
	});
});

describe('a selection', () => {
	it('stands for entries and loose forms', () => {
		expect(itemOf(rows[0])).toEqual({ kind: 'form', form: 'Moussa Traoré' });
		expect(itemOf(rows[2])).toEqual({ kind: 'entry', id: 'e1' });
	});

	it('proposes a reviewed label first, then the most named', () => {
		expect(proposedLabel([rows[1], rows[0]])).toBe('Moussa Traoré');
		expect(proposedLabel([rows[0], rows[2]])).toBe('Père Éric');
		expect(proposedLabel([])).toBe('');
		expect(labelChoices([rows[0], rows[2]])).toEqual(['Moussa Traoré', 'Père Éric', 'Pere Eric']);
	});
});
