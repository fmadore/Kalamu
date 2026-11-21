import { describe, expect, it } from 'vitest';

import {
	boxBetween,
	clampBox,
	dragBox,
	isSlip,
	keyStep,
	middleOf,
	MIN_SIDE,
	noteCovers,
	pointIn,
	sameBox,
	stepFor
} from './regionBox.js';
import { turnBox, unturnBox } from './regions.js';
import type { Rotation } from './pdf.js';

const close = (a: object, b: object) =>
	Object.entries(b).forEach(([k, v]) => expect((a as Record<string, number>)[k]).toBeCloseTo(v));

describe('unturnBox', () => {
	const box = { x: 0.1, y: 0.2, width: 0.3, height: 0.1 };

	it('undoes the reader’s quarter turns at 0, 90, 180 and 270', () => {
		for (const r of [0, 90, 180, 270] as Rotation[]) close(unturnBox(turnBox(box, r), r), box);
	});

	it('takes a box drawn on a page turned a quarter back where it is on the page as read', () => {
		// Drawn at the top right of a page turned clockwise: the top left of the page as read.
		close(unturnBox({ x: 0.7, y: 0.1, width: 0.1, height: 0.3 }, 90), box);
		close(unturnBox({ x: 0.6, y: 0.7, width: 0.3, height: 0.1 }, 180), box);
		close(unturnBox({ x: 0.2, y: 0.6, width: 0.1, height: 0.3 }, 270), box);
	});
});

describe('a box drawn with the pointer', () => {
	const rect = { left: 100, top: 50, width: 400, height: 800 };

	it('is placed in fractions of the page, the pointer off the page kept at its edge', () => {
		expect(pointIn(rect, 300, 450)).toEqual({ x: 0.5, y: 0.5 });
		expect(pointIn(rect, 20, 2000)).toEqual({ x: 0, y: 1 });
	});

	it('is the same whichever way it was drawn, and cut at the page’s edges', () => {
		const b = boxBetween({ x: 0.6, y: 0.4 }, { x: 0.2, y: 0.1 });
		close(b, { x: 0.2, y: 0.1, width: 0.4, height: 0.3 });
		close(clampBox({ x: -0.1, y: 0.9, width: 0.3, height: 0.3 }), {
			x: 0,
			y: 0.9,
			width: 0.2,
			height: 0.1
		});
	});

	it('is a slip when it is too small on screen or to keep', () => {
		const page = { width: 400, height: 800 };
		expect(isSlip({ x: 0, y: 0, width: 0.01, height: 0.1 }, page)).toBe(true); // 4 px wide
		expect(isSlip({ x: 0, y: 0, width: 0.1, height: 0.1 }, page)).toBe(false);
		expect(isSlip({ x: 0, y: 0, width: 0.004, height: 0.1 }, { width: 4000, height: 800 })).toBe(
			true
		);
	});
});

describe('a box adjusted', () => {
	const box = { x: 0.2, y: 0.2, width: 0.2, height: 0.1 };

	it('moves whole within the page', () => {
		close(dragBox(box, 'move', 0.1, 0.05), { x: 0.3, y: 0.25, width: 0.2, height: 0.1 });
		close(dragBox(box, 'move', 2, -2), { x: 0.8, y: 0, width: 0.2, height: 0.1 });
	});

	it('is resized by a corner or a side, never past the side across nor off the page', () => {
		close(dragBox(box, 'se', 0.1, 0.1), { x: 0.2, y: 0.2, width: 0.3, height: 0.2 });
		close(dragBox(box, 'nw', -0.5, -0.5), { x: 0, y: 0, width: 0.4, height: 0.3 });
		const crossed = dragBox(box, 'e', -1, 0);
		expect(crossed.x).toBeCloseTo(0.2);
		expect(crossed.width).toBeCloseTo(MIN_SIDE);
		close(dragBox(box, 'n', 0, 0.05), { y: 0.25, height: 0.05 });
	});

	it('steps by a hundredth of the page at the pane’s width, finer zoomed in', () => {
		expect(stepFor(1)).toBeCloseTo(0.01);
		expect(stepFor(4)).toBeCloseTo(0.0025);
		expect(stepFor(0.5)).toBeCloseTo(0.01);
	});

	it('moves with the arrows and is resized with Shift and the arrows, each said in a word', () => {
		expect(keyStep('ArrowRight', false, 0.01)).toEqual({
			grip: 'move',
			dx: 0.01,
			dy: 0,
			said: 'right'
		});
		expect(keyStep('ArrowUp', true, 0.01)).toEqual({
			grip: 's',
			dx: 0,
			dy: -0.01,
			said: 'shorter'
		});
		expect(keyStep('ArrowLeft', true, 0.01)?.said).toBe('narrower');
		expect(keyStep('Enter', false, 0.01)).toBeNull();
	});

	it('is placed in the middle of what is in view, on the page', () => {
		close(middleOf({ x: 0.5, y: 0.5, width: 0.3, height: 0.5 }), {
			x: 0.6,
			y: 0.725,
			width: 0.1,
			height: 0.05
		});
		const anywhere = middleOf(null);
		expect(anywhere.x + anywhere.width).toBeLessThanOrEqual(1);
		expect(sameBox(middleOf({ x: -1, y: -1, width: 3, height: 3 }), middleOf(null))).toBe(true);
	});
});

describe('the card of a region chosen', () => {
	it('moves out of the way of a region it would cover, and stays where it is otherwise', () => {
		const note = { top: 100, bottom: 260 };
		// Drawn at the top of the page, under the card.
		expect(noteCovers({ top: 110, bottom: 180 }, note)).toBe(true);
		// Just below it, within the room kept around it.
		expect(noteCovers({ top: 265, bottom: 300 }, note)).toBe(true);
		// Further down the page, or above it.
		expect(noteCovers({ top: 400, bottom: 480 }, note)).toBe(false);
		expect(noteCovers({ top: 20, bottom: 80 }, note)).toBe(false);
	});
});
