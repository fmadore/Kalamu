import { describe, expect, it } from 'vitest';
import { menuTarget } from './menu.js';

describe('the keys of a menu', () => {
	it('go round with the arrows', () => {
		expect(menuTarget(3, 0, 'ArrowDown', true)).toBe(1);
		expect(menuTarget(3, 2, 'ArrowDown', true)).toBe(0);
		expect(menuTarget(3, 0, 'ArrowUp', true)).toBe(2);
		expect(menuTarget(3, 1, 'ArrowUp', false)).toBe(0);
	});

	it('start from the first or the last when nothing has the focus', () => {
		expect(menuTarget(3, -1, 'ArrowDown', true)).toBe(0);
		expect(menuTarget(3, -1, 'ArrowUp', true)).toBe(1);
	});

	it('go to either end with Home and End, only where the menu takes them', () => {
		expect(menuTarget(4, 2, 'Home', true)).toBe(0);
		expect(menuTarget(4, 1, 'End', true)).toBe(3);
		expect(menuTarget(4, 2, 'Home', false)).toBe(-1);
		expect(menuTarget(4, 1, 'End', false)).toBe(-1);
	});

	it('leave every other key alone', () => {
		expect(menuTarget(3, 0, 'Enter', true)).toBe(-1);
		expect(menuTarget(3, 0, 'ArrowRight', true)).toBe(-1);
		expect(menuTarget(3, 0, 'a', true)).toBe(-1);
	});
});
