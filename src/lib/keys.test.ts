import { describe, expect, it } from 'vitest';
import { panStep, rovingStop, rovingTarget, scrollsItself } from './keys.js';

describe('marks sharing one tab stop', () => {
	const order = [3, 5, 8, 13];

	it('moves to the next and previous mark, and to either end', () => {
		expect(rovingTarget(order, 5, 'ArrowRight')).toBe(8);
		expect(rovingTarget(order, 5, 'ArrowLeft')).toBe(3);
		expect(rovingTarget(order, 8, 'Home')).toBe(3);
		expect(rovingTarget(order, 5, 'End')).toBe(13);
	});

	it('stays at either end, and leaves other keys alone', () => {
		expect(rovingTarget(order, 13, 'ArrowRight')).toBeNull();
		expect(rovingTarget(order, 3, 'ArrowLeft')).toBeNull();
		expect(rovingTarget(order, 3, 'Home')).toBeNull();
		expect(rovingTarget(order, 5, 'ArrowDown')).toBeNull();
		expect(rovingTarget(order, 5, 'Enter')).toBeNull();
		expect(rovingTarget(order, 4, 'ArrowRight')).toBeNull();
	});

	it('gives the stop to the mark wanted, or to the first once it is gone', () => {
		expect(rovingStop(order, 8)).toBe(8);
		expect(rovingStop(order, 4)).toBe(3);
		expect(rovingStop(order, null)).toBe(3);
		expect(rovingStop([], 4)).toBeNull();
	});
});

describe('panning a zoomed page', () => {
	const pane = { width: 500, height: 800 };

	it('moves a short step with the arrows, most of the pane with Shift', () => {
		expect(panStep('ArrowDown', false, pane)).toEqual({ x: 0, y: 48 });
		expect(panStep('ArrowLeft', false, pane)).toEqual({ x: -48, y: 0 });
		expect(panStep('ArrowUp', true, pane)).toEqual({ x: 0, y: -640 });
		expect(panStep('ArrowRight', true, pane)).toEqual({ x: 400, y: 0 });
	});

	it('leaves other keys alone', () => {
		expect(panStep('PageDown', false, pane)).toBeNull();
		expect(panStep('a', true, pane)).toBeNull();
	});
});

describe('a pane that scrolls', () => {
	it('scrolls when what it holds is taller and its overflow allows it', () => {
		expect(scrollsItself({ scrollHeight: 900, clientHeight: 400 }, 'auto')).toBe(true);
		expect(scrollsItself({ scrollHeight: 900, clientHeight: 400 }, 'scroll')).toBe(true);
		expect(scrollsItself({ scrollHeight: 900, clientHeight: 400 }, 'visible')).toBe(false);
		expect(scrollsItself({ scrollHeight: 400, clientHeight: 400 }, 'auto')).toBe(false);
		// A pixel of rounding is not something to scroll.
		expect(scrollsItself({ scrollHeight: 401, clientHeight: 400 }, 'auto')).toBe(false);
	});
});
