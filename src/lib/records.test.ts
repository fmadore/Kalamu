import { describe, expect, it } from 'vitest';
import type { AnalysisSchema } from './bindings/AnalysisSchema.js';
import type { Field } from './bindings/Field.js';
import type { TaskRules } from './bindings/TaskRules.js';
import { cellText, columnsOf, groupOf, readingRule, rowsOf } from './records.js';

const text = (key: string): Field => ({
	key,
	label: key,
	instructions: '',
	kind: { type: 'text' },
	reason: false
});

const register: AnalysisSchema = {
	fields: [
		{
			key: 'entries',
			label: 'Entries',
			instructions: '',
			kind: { type: 'entries', fields: [text('name'), text('date')] },
			reason: false
		}
	]
};

const summary: AnalysisSchema = { fields: [text('summary')] };

describe('records', () => {
	it('are an analysis whose only field is a repeating group', () => {
		expect(groupOf(register)?.key).toBe('entries');
		expect(columnsOf(register).map((c) => c.key)).toEqual(['name', 'date']);
		expect(groupOf(summary)).toBeNull();
		expect(groupOf({ fields: [...register.fields, text('street')] })).toBeNull();
		expect(columnsOf(undefined)).toEqual([]);
	});

	it('read a page’s rows and their cells, an empty page holding none', () => {
		const rows = rowsOf(register, [
			{
				key: 'entries',
				value: [
					[
						{ key: 'name', value: 'Diallo' },
						{ key: 'date', value: '〃' }
					]
				]
			}
		]);
		expect(rows).toHaveLength(1);
		expect(cellText(rows[0], 'date')).toBe('〃');
		expect(cellText([{ key: 'people', value: ['Awa', 'Moussa'] }], 'people')).toBe('Awa; Moussa');
		expect(cellText(rows[0], 'missing')).toBe('');
		expect(rowsOf(register, [{ key: 'entries', value: [] }])).toEqual([]);
		expect(rowsOf(summary, [{ key: 'summary', value: 'x' }])).toEqual([]);
	});

	it('take what a task offers page by page only when every set-up reads rows', () => {
		const rule = {
			task: 'analysis',
			samples: false,
			splits_spreads: false,
			page_documents: false,
			by_page: { samples: true, splits_spreads: true, page_documents: true }
		} as TaskRules;
		expect(readingRule(rule, [register]).page_documents).toBe(true);
		expect(readingRule(rule, [register]).samples).toBe(true);
		expect(readingRule(rule, [register, summary]).page_documents).toBe(false);
		expect(readingRule(rule, []).page_documents).toBe(false);
		expect(readingRule({ ...rule, by_page: undefined }, [register]).page_documents).toBe(false);
	});
});
