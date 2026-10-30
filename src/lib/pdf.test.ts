import { describe, expect, it } from 'vitest';
import { turn } from './pdf.js';

describe('turning a page', () => {
	it('goes round by quarter turns either way', () => {
		expect(turn(0, 1)).toBe(90);
		expect(turn(270, 1)).toBe(0);
		expect(turn(0, -1)).toBe(270);
		expect(turn(90, -1)).toBe(0);
	});
});
