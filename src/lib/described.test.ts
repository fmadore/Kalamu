import { describe, expect, it } from 'vitest';
import { FIELDS, changed, kept } from './described.js';
import { en, fr } from './i18n/index.js';

describe('source descriptions', () => {
	it('keeps trimmed values and drops empty ones', () => {
		expect(kept({ repository: '  ANS ', fonds: '   ', citation: 'ANS,\n19G 1' })).toEqual({
			repository: 'ANS',
			citation: 'ANS,\n19G 1'
		});
	});

	it('tells a real change from spaces typed', () => {
		expect(changed({ repository: 'ANS ' }, { repository: 'ANS' })).toBe(false);
		expect(changed({ repository: '' }, {})).toBe(false);
		expect(changed({ shelfmark: '19G 1' }, {})).toBe(true);
		expect(changed({}, { shelfmark: '19G 1' })).toBe(true);
	});

	it('names every part in each language', () => {
		for (const m of [en, fr]) {
			for (const f of FIELDS) expect(m.projects.describe.fields[f]).toBeTruthy();
		}
	});
});
