// A region of a page read again by a model, as the dialog needs it: where an accepted reading
// goes (the region's own text, the page's text in place of its words, or nowhere), the text as
// the engine writes it (`core::region_reading`), and the size to draw the region's picture at.

import type { PageRegions } from './bindings/PageRegions.js';
import type { ReadingTarget } from './bindings/ReadingTarget.js';
import type { ShownRegion } from './bindings/ShownRegion.js';

/** The piece whose text the review shows beside the page: its set-up, index and text. */
export interface ShownPiece {
	setupId: string;
	index: number;
	text: string;
}

/**
 * Where a reading of `region` would go: a region the researcher drew takes it as its text; an
 * engine's region whose words have a certain place in `piece`'s text (`span`) has them
 * replaced; any other can only be read and copied. `page` is the page's regions as shown, whose
 * `drawn` are the researcher's own; `piece` is null on a page with no text of its own (an
 * analysis's or a table of contents' review).
 */
export function targetFor(
	region: ShownRegion,
	page: PageRegions,
	piece: ShownPiece | null
): ReadingTarget {
	if (page.drawn.some((d) => d.id === region.id)) return { kind: 'region' };
	const span = region.span;
	if (span && piece && span.end > span.start && span.end <= piece.text.length) {
		return {
			kind: 'text',
			setup_id: piece.setupId,
			index: piece.index,
			start: span.start,
			end: span.end,
			was: piece.text.slice(span.start, span.end)
		};
	}
	return { kind: 'nowhere' };
}

/** Where the target's words stand in `text`: at their place, or else their only place; null
 * when the text changed there, as `core::region_reading::locate` decides. */
export function locateWords(text: string, target: ReadingTarget): [number, number] | null {
	if (target.kind !== 'text' || !target.was || target.end < target.start) return null;
	if (text.slice(target.start, target.end) === target.was) return [target.start, target.end];
	const first = text.indexOf(target.was);
	if (first < 0 || text.indexOf(target.was, first + 1) >= 0) return null;
	return [first, first + target.was.length];
}

/** `text` with the target's words replaced by `replacement`; null where they changed. */
export function withReading(
	text: string,
	target: ReadingTarget,
	replacement: string
): string | null {
	const at = locateWords(text, target);
	return at ? text.slice(0, at[0]) + replacement + text.slice(at[1]) : null;
}

/** The width to draw the whole page at so that the region comes out about `wanted` pixels wide,
 * never past `most`, so that a narrow note in a margin is still drawn large enough to read. */
export function pageWidthFor(region: { width: number }, wanted = 480, most = 4000): number {
	const share = Math.max(region.width, 0.01);
	return Math.round(Math.min(most, Math.max(wanted, wanted / share)));
}

/** The pixels a region covers on a drawing `width` × `height`: x, y, width, height, at least
 * one each way and within the drawing. */
export function pixelsOf(
	region: { x: number; y: number; width: number; height: number },
	width: number,
	height: number
): [number, number, number, number] {
	const clamp = (v: number) => Math.min(1, Math.max(0, v));
	const x = Math.round(clamp(region.x) * width);
	const y = Math.round(clamp(region.y) * height);
	const right = Math.round(clamp(region.x + region.width) * width);
	const bottom = Math.round(clamp(region.y + region.height) * height);
	return [
		Math.min(x, width - 1),
		Math.min(y, height - 1),
		Math.max(1, right - x),
		Math.max(1, bottom - y)
	];
}
