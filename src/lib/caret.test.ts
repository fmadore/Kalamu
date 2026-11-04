import { describe, expect, it } from 'vitest';
import { lineStarts } from './caret.js';

describe('the caret in the text as read', () => {
	it('knows where each line starts', () => {
		expect(lineStarts('ab\n\ncde\n')).toEqual([0, 3, 4, 8]);
		expect(lineStarts('')).toEqual([0]);
	});
});
