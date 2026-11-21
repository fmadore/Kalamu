import { describe, expect, it } from 'vitest';
import { GAP, anchorLine, readingsTop, tallest, visibleBand } from './readingsPlace.js';

// A text pane at 1536 × 864: from 120 to 800 in the window.
const pane = { top: 120, bottom: 800 };

describe('readings placement', () => {
	it('opens under the words when they fit there, over them when they fit there', () => {
		expect(readingsTop({ top: 200, bottom: 220 }, pane, 300)).toBe(220 + GAP);
		expect(readingsTop({ top: 700, bottom: 720 }, pane, 300)).toBe(700 - GAP - 300);
	});

	it('keeps tall readings whole inside the pane when neither side has room', () => {
		// A long paragraph's last line in the middle: 380 px is too tall for either side.
		const line = { top: 450, bottom: 470 };
		const top = readingsTop(line, pane, 380);
		expect(top).toBeGreaterThanOrEqual(pane.top + GAP);
		expect(top + 380).toBeLessThanOrEqual(pane.bottom - GAP);
		// Near the bottom, the readings rise to fit rather than run under the pane's edge.
		const low = readingsTop({ top: 760, bottom: 780 }, pane, 640);
		expect(low).toBeGreaterThanOrEqual(pane.top + GAP);
		expect(low + 640).toBeLessThanOrEqual(pane.bottom - GAP);
		// Near the top, they come down rather than lose their first reading.
		const high = readingsTop({ top: 130, bottom: 150 }, pane, 600);
		expect(high).toBeGreaterThanOrEqual(pane.top + GAP);
		expect(high + 600).toBeLessThanOrEqual(pane.bottom - GAP);
	});

	it('caps the readings at the part of the pane that shows, cut by the window', () => {
		const visible = visibleBand({ top: 120, bottom: 1000 }, 864);
		expect(visible).toEqual({ top: 120, bottom: 864 });
		expect(tallest(visible)).toBe(864 - 120 - 2 * GAP);
		// Readings taller than the pane start at its top and scroll inside.
		expect(readingsTop({ top: 400, bottom: 420 }, visible, tallest(visible))).toBe(120 + GAP);
	});

	it('anchors on the last line of the words that shows', () => {
		const lines = [
			{ top: 700, bottom: 720 },
			{ top: 780, bottom: 800 },
			{ top: 900, bottom: 920 }
		];
		expect(anchorLine(lines, { top: 120, bottom: 864 })).toEqual(lines[1]);
		expect(anchorLine(lines, { top: 0, bottom: 100 })).toEqual(lines[2]);
		expect(anchorLine([], pane)).toBeNull();
	});
});
