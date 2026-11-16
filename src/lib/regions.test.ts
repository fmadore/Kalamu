import { describe, expect, it } from 'vitest';

import type { PageRegions } from './bindings/PageRegions.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import { levelFor, regionAt, turnBox } from './regions.js';
import { mockRegions } from './dev/mockRegions.js';
import { job, setupA, setupB, texts } from './dev/mockJobs.js';

const close = (a: object, b: object) =>
	Object.entries(b).forEach(([k, v]) => expect((a as Record<string, number>)[k]).toBeCloseTo(v));

describe('turnBox', () => {
	const box = { x: 0.1, y: 0.2, width: 0.3, height: 0.1 };

	it('leaves a box as read when the page is not turned', () => {
		expect(turnBox(box, 0)).toEqual(box);
	});

	it('turns a box with the page a quarter clockwise: its top-left goes top-right', () => {
		// The page's top edge becomes its right edge: y measured from the top is now measured
		// from the right.
		close(turnBox(box, 90), { x: 0.7, y: 0.1, width: 0.1, height: 0.3 });
	});

	it('turns it half round and a quarter anticlockwise', () => {
		close(turnBox(box, 180), { x: 0.6, y: 0.7, width: 0.3, height: 0.1 });
		close(turnBox(box, 270), { x: 0.2, y: 0.6, width: 0.1, height: 0.3 });
	});

	it('comes back to where it was after four quarter turns', () => {
		let b = box;
		for (let i = 0; i < 4; i++) b = turnBox(b, 90);
		close(b, box);
	});
});

const page = (blocks: number, lines: number, words: number): PageRegions => {
	const r: ShownRegion = {
		x: 0,
		y: 0,
		width: 1,
		height: 1,
		text: 'a',
		estimated: false,
		span: null
	};
	return {
		blocks: Array(blocks).fill(r),
		lines: Array(lines).fill(r),
		words: Array(words).fill(r),
		mapping: 'exact',
		corrected: false
	};
};

describe('levelFor', () => {
	it('draws blocks at the pane’s width, then lines and words as the reader zooms in', () => {
		const full = page(3, 30, 3000);
		expect(levelFor(1, full)).toBe('blocks');
		expect(levelFor(2, full)).toBe('lines');
		expect(levelFor(4, full)).toBe('words');
	});

	it('draws what the page has', () => {
		expect(levelFor(6, page(3, 0, 0))).toBe('blocks');
	});
});

describe('regionAt', () => {
	const at = (start: number, end: number): ShownRegion => ({
		x: 0,
		y: 0,
		width: 0,
		height: 0,
		text: '',
		estimated: false,
		span: { start, end }
	});

	it('finds the region whose place holds the offset, and none where nothing is placed', () => {
		const list = [at(0, 5), { ...at(0, 0), span: null }, at(7, 12)];
		expect(regionAt(list, 8)).toBe(2);
		// A caret just after a region's last letter, as a click past the end of a line leaves it.
		expect(regionAt(list, 5)).toBe(0);
		expect(regionAt(list, 6)).toBe(-1);
	});
});

describe('the sample regions', () => {
	it('place page 1 word by word, in two columns, each word where it stands in the text', () => {
		const regions = mockRegions(job.id, setupA.id, 0);
		expect(regions.mapping).toBe('exact');
		expect(regions.blocks).toHaveLength(8);
		expect(regions.words.every((w) => !w.estimated)).toBe(true);
		for (const w of regions.words) {
			expect(w.span).not.toBeNull();
			if (w.span) expect(texts[0].slice(w.span.start, w.span.end)).toBe(w.text);
		}
		// The paragraph running on into the second column.
		expect(regions.blocks[6].x).toBeGreaterThan(0.5);
	});

	it('estimate page 2’s line and words within its block, and place nothing on page 3', () => {
		const two = mockRegions(job.id, setupA.id, 1);
		expect(two.blocks).toHaveLength(1);
		expect(two.words.every((w) => w.estimated)).toBe(true);
		expect(mockRegions(job.id, setupA.id, 2).blocks).toHaveLength(0);
		expect(mockRegions(job.id, setupB.id, 0).blocks).toHaveLength(0);
	});
});
