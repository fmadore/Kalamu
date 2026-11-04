import { describe, expect, it } from 'vitest';
import { between, clampZoom, keepUnder, MAX_ZOOM, MIN_ZOOM, wheelFactor } from './zoom.js';

describe('zoom', () => {
	it('zooms in proportion to the wheel, the same in and out', () => {
		const notch = wheelFactor(-100, 0);
		expect(notch).toBeGreaterThan(1.2);
		expect(notch).toBeLessThan(1.35);
		expect(wheelFactor(100, 0) * notch).toBeCloseTo(1);
		// Ten small pinch steps zoom as far as one notch of the same length.
		expect(wheelFactor(-10, 0) ** 10).toBeCloseTo(notch);
		// A wheel counting in lines: three lines are about a notch.
		expect(wheelFactor(-3, 1)).toBeCloseTo(Math.exp(0.15));
	});

	it('keeps a fling of the wheel from leaping', () => {
		expect(wheelFactor(-5000, 0)).toBe(wheelFactor(-150, 0));
	});

	it('stays within its limits', () => {
		expect(clampZoom(100)).toBe(MAX_ZOOM);
		expect(clampZoom(0)).toBe(MIN_ZOOM);
		expect(clampZoom(1.5)).toBe(1.5);
	});

	it('keeps the point under the pointer in place', () => {
		const before = { left: 100, top: 50, width: 400, height: 600 };
		// Doubled from its top left corner: the pointer at the page's middle would now see
		// a point 200 and 300 pixels further on, unless scrolled by as much.
		const after = { left: 100, top: 50, width: 800, height: 1200 };
		expect(keepUnder(before, after, { x: 300, y: 350 })).toEqual({ x: 200, y: 300 });
		// The pointer at the page's corner needs no scroll.
		expect(keepUnder(before, after, { x: 100, y: 50 })).toEqual({ x: 0, y: 0 });
	});

	it('glides in equal ratios from one zoom to another', () => {
		expect(between(1, 4, 0)).toBe(1);
		expect(between(1, 4, 1)).toBe(4);
		const half = between(1, 4, 0.5);
		expect(half).toBeGreaterThan(2);
		expect(half).toBeLessThan(4);
	});
});
