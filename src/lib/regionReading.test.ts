import { describe, expect, it } from 'vitest';

import type { PageRegions } from './bindings/PageRegions.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import { locateWords, pageWidthFor, pixelsOf, targetFor, withReading } from './regionReading.js';

const TEXT = 'Le Gouverneur a reçu deux cents fideles.';

function region(id: string, span: { start: number; end: number } | null): ShownRegion {
	return {
		id,
		x: 0.1,
		y: 0.2,
		width: 0.3,
		height: 0.05,
		text: '',
		granularity: 'line',
		origin: 'provider',
		span
	};
}

const page = (drawn: ShownRegion[] = []): PageRegions => ({
	blocks: [],
	lines: [],
	words: [],
	mapping: 'unique',
	corrected: false,
	result: 'key',
	drawn
});

describe('targetFor', () => {
	const piece = { setupId: 's1', index: 0, text: TEXT };

	it('gives a region the researcher drew its reading as its text', () => {
		const own = { ...region('6f1c', null), origin: 'manual' as const };
		expect(targetFor(own, page([own]), piece)).toEqual({ kind: 'region' });
	});

	it('replaces the words of an engine’s region placed in the text', () => {
		const start = TEXT.indexOf('deux');
		const line = region('r1_l2', { start, end: start + 18 });
		expect(targetFor(line, page(), piece)).toEqual({
			kind: 'text',
			setup_id: 's1',
			index: 0,
			start,
			end: start + 18,
			was: 'deux cents fideles'
		});
	});

	it('writes nowhere a region with no certain place, or a page with no text', () => {
		expect(targetFor(region('r2', null), page(), piece)).toEqual({ kind: 'nowhere' });
		expect(targetFor(region('r2', { start: 0, end: 2 }), page(), null)).toEqual({
			kind: 'nowhere'
		});
		expect(targetFor(region('r2', { start: 0, end: 99 }), page(), piece)).toEqual({
			kind: 'nowhere'
		});
	});
});

describe('withReading', () => {
	const target = (start: number, end: number, was: string) => ({
		kind: 'text' as const,
		setup_id: 's1',
		index: 0,
		start,
		end,
		was
	});

	it('replaces the words where they stand, or where they alone moved to', () => {
		const at = TEXT.indexOf('deux');
		expect(withReading(TEXT, target(at, at + 18, 'deux cents fideles'), '200 fidèles')).toBe(
			'Le Gouverneur a reçu 200 fidèles.'
		);
		expect(withReading(TEXT, target(0, 4, 'Gouverneur'), 'Gouverneur général')).toBe(
			'Le Gouverneur général a reçu deux cents fideles.'
		);
	});

	it('refuses words that changed, or that stand twice away from their place', () => {
		expect(withReading(TEXT, target(3, 9, 'Gouvemeur'), 'x')).toBeNull();
		expect(locateWords('la fete, la fete', target(1, 5, 'fete'))).toBeNull();
		expect(locateWords(TEXT, { kind: 'nowhere' })).toBeNull();
	});
});

describe('the region’s picture', () => {
	it('draws a narrow region large enough to read, within reason', () => {
		expect(pageWidthFor({ width: 0.5 })).toBe(960);
		expect(pageWidthFor({ width: 1 })).toBe(480);
		expect(pageWidthFor({ width: 0.001 })).toBe(4000);
	});

	it('cuts the region’s pixels, kept within the drawing', () => {
		expect(pixelsOf({ x: 0.1, y: 0.2, width: 0.3, height: 0.1 }, 1000, 500)).toEqual([
			100, 100, 300, 50
		]);
		expect(pixelsOf({ x: 0.9, y: 0.9, width: 0.5, height: 0.5 }, 100, 100)).toEqual([
			90, 90, 10, 10
		]);
	});
});
