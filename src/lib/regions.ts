// Where a page's text sits, drawn over the scan in the text review (#84). The boxes come from
// the app (`page_regions`) in fractions of the page as read, worked out in Rust
// (crates/core/src/regions.rs); here they are only turned with the reader's quarter turns, drawn
// at the granularity that suits the zoom, and their places in the text marked through the
// browser's own highlights (`::highlight(region)` in app.css), as a search's words are.

import type { PageRegions } from './bindings/PageRegions.js';
import type { ShownRegion } from './bindings/ShownRegion.js';
import type { TextSpan } from './bindings/TextSpan.js';
import { located, supported } from './found.js';
import type { Rotation } from './pdf.js';

/** A granularity the app gives regions at. */
export type Level = keyof Pick<PageRegions, 'blocks' | 'lines' | 'words'>;

/** A box in fractions of a page's width and height, from its top-left corner. */
export interface Box {
	x: number;
	y: number;
	width: number;
	height: number;
}

/** A box on the page as read, on the page as drawn after the reader's quarter turns clockwise. */
export function turnBox(b: Box, rotation: Rotation): Box {
	switch (rotation) {
		case 90:
			return { x: 1 - b.y - b.height, y: b.x, width: b.height, height: b.width };
		case 180:
			return { x: 1 - b.x - b.width, y: 1 - b.y - b.height, width: b.width, height: b.height };
		case 270:
			return { x: b.y, y: 1 - b.x - b.width, width: b.height, height: b.width };
		case 0:
			return { x: b.x, y: b.y, width: b.width, height: b.height };
	}
}

/** Whether two boxes share any part. */
export function overlaps(a: Box, b: Box): boolean {
	return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

/** Zooms from which lines, then words, are drawn rather than blocks: at the pane's width a
 * newspaper's thousands of words would be slivers too small to point at, and too many to draw
 * smoothly. */
const LINES_FROM = 1.75;
const WORDS_FROM = 3.5;

/** The finest granularity worth drawing at `zoom` that the page has. */
export function levelFor(zoom: number, page: PageRegions): Level {
	if (zoom >= WORDS_FROM && page.words.length > 0) return 'words';
	if (zoom >= LINES_FROM && page.lines.length > 0) return 'lines';
	return page.blocks.length > 0 ? 'blocks' : page.lines.length > 0 ? 'lines' : 'words';
}

/** The region of `list` whose place in the text holds `offset` (a caret just after its last
 * letter included), or -1: only a region the app placed, so nothing is guessed from the text
 * back to the page. */
export function regionAt(list: ShownRegion[], offset: number): number {
	return list.findIndex((r) => r.span !== null && r.span.start <= offset && offset <= r.span.end);
}

/** Takes the mark off. */
export function clearPlace(): void {
	if (supported()) CSS.highlights.delete('region');
}

/** Marks `span` of the text of the `blocks` under `root`, each block read as a line of its own,
 * as the place of the region chosen, and returns where it stands; none marks nothing. */
export function markPlace(root: ParentNode, blocks: string, span: TextSpan | null): Range | null {
	clearPlace();
	if (!span || !supported()) return null;
	const range = located(root, blocks).range(span.start, span.end);
	if (range) CSS.highlights.set('region', new Highlight(range));
	return range;
}
