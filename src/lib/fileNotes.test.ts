import { describe, expect, it } from 'vitest';
import { warningText } from './fileNotes.js';
import { en } from './i18n/index.js';

describe('new job form', () => {
	it('words input warnings', () => {
		const text = warningText({ source_id: 'x', kind: 'double_page_spread', pages: [2, 3] }, en);
		expect(text).toContain('page 2, 3');
	});
});
