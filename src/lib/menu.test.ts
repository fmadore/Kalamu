import { describe, expect, it } from 'vitest';
import { menuTarget } from './menu.js';

describe('the keys of a menu', () => {
	it('go round with the arrows', () => {
		expect(menuTarget(3, 0, 'ArrowDown')).toBe(1);
		expect(menuTarget(3, 2, 'ArrowDown')).toBe(0);
		expect(menuTarget(3, 0, 'ArrowUp')).toBe(2);
		expect(menuTarget(3, 1, 'ArrowUp')).toBe(0);
	});

	it('start from the first or the last when nothing has the focus', () => {
		expect(menuTarget(3, -1, 'ArrowDown')).toBe(0);
		expect(menuTarget(3, -1, 'ArrowUp')).toBe(1);
	});

	it('go to either end with Home and End, in every menu', () => {
		expect(menuTarget(4, 2, 'Home')).toBe(0);
		expect(menuTarget(4, 1, 'End')).toBe(3);
		// A name's menu has two items.
		expect(menuTarget(2, 0, 'End')).toBe(1);
		expect(menuTarget(2, 1, 'Home')).toBe(0);
	});

	it('leave every other key alone', () => {
		expect(menuTarget(3, 0, 'Enter')).toBe(-1);
		expect(menuTarget(3, 0, 'ArrowRight')).toBe(-1);
		expect(menuTarget(3, 0, 'a')).toBe(-1);
	});
});
