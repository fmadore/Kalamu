import { describe, expect, it } from 'vitest';
import { warningText } from './fileNotes.js';
import { en } from './i18n/index.js';

describe('new job form', () => {
	it('words input warnings', () => {
		const text = warningText({ source_id: 'x', kind: 'double_page_spread', pages: [2, 3] }, en);
		expect(text).toContain('page 2, 3');
	});

	it('names both photographs of one page, the earlier first', () => {
		const names: Record<string, string> = { a: 'IMG_0142.jpg', b: 'IMG_0143.jpg' };
		const text = warningText(
			{ source_id: 'b', kind: 'same_page', other: 'a', keep: 'b', clearly_sharper: true },
			en,
			(id) => names[id]
		);
		expect(text).toBe('IMG_0142.jpg and IMG_0143.jpg look like the same page');
		expect(warningText({ source_id: 'a', kind: 'blurred' }, en, (id) => names[id])).toBe(
			'IMG_0142.jpg looks blurred'
		);
	});
});
