// The arithmetic of a box the researcher draws or adjusts over a scan (#84): a pointer's place
// on the page, a box between two points, kept on the page and never smaller than a stroke, moved
// or resized by its grip or by the keys, in steps that suit the zoom. All of it on the page as
// drawn, after the reader's quarter turns, which is what the researcher sees and points at; the
// box goes back on the page as read (`unturnBox`) only when it is kept.

import type { Box } from './regions.js';

/** A point in fractions of the page as drawn. */
export interface Point {
	x: number;
	y: number;
}

/** What a box is held by: a corner, a side, or the whole of it. */
export type Grip = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw' | 'move';

/** The smallest side a box is given while it is adjusted, in fractions of the page: a little
 * over the smallest the app keeps (`core::regions::MIN_SIZE`, 0.004), so a box made small by the
 * keys is still kept. */
export const MIN_SIDE = 0.005;

/** On screen, the least a drawn box must measure either way, in CSS pixels, to be kept: less is
 * a slip of the hand, not a region. */
export const SLIP_PX = 6;

const clamp01 = (v: number) => Math.min(Math.max(v, 0), 1);

/** Where the pointer is on the page drawn in `rect`, in fractions of it, kept on the page. */
export function pointIn(
	rect: { left: number; top: number; width: number; height: number },
	clientX: number,
	clientY: number
): Point {
	return {
		x: rect.width > 0 ? clamp01((clientX - rect.left) / rect.width) : 0,
		y: rect.height > 0 ? clamp01((clientY - rect.top) / rect.height) : 0
	};
}

/** The box between two points, whichever way it was drawn, on the page. */
export function boxBetween(a: Point, b: Point): Box {
	const [x0, x1] = [clamp01(Math.min(a.x, b.x)), clamp01(Math.max(a.x, b.x))];
	const [y0, y1] = [clamp01(Math.min(a.y, b.y)), clamp01(Math.max(a.y, b.y))];
	return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
}

/** A box kept on the page: given from its top-left corner, cut at the page's edges. */
export function clampBox(b: Box): Box {
	return boxBetween({ x: b.x, y: b.y }, { x: b.x + b.width, y: b.y + b.height });
}

/** Whether a box drawn on a page shown `page` CSS pixels large is a slip: too small on screen
 * either way, or too small to keep. */
export function isSlip(b: Box, page: { width: number; height: number }): boolean {
	return (
		b.width * page.width < SLIP_PX ||
		b.height * page.height < SLIP_PX ||
		b.width < MIN_SIDE ||
		b.height < MIN_SIDE
	);
}

/** `b` held by `grip` and moved `dx`, `dy` (fractions of the page): moved whole within the
 * page, or its corner or side moved, never past the side across from it nor off the page. */
export function dragBox(b: Box, grip: Grip, dx: number, dy: number): Box {
	if (grip === 'move') {
		return {
			x: Math.min(Math.max(b.x + dx, 0), 1 - b.width),
			y: Math.min(Math.max(b.y + dy, 0), 1 - b.height),
			width: b.width,
			height: b.height
		};
	}
	let [left, top, right, bottom] = [b.x, b.y, b.x + b.width, b.y + b.height];
	if (grip.includes('w')) left = Math.min(Math.max(left + dx, 0), right - MIN_SIDE);
	if (grip.includes('e')) right = Math.max(Math.min(right + dx, 1), left + MIN_SIDE);
	if (grip.includes('n')) top = Math.min(Math.max(top + dy, 0), bottom - MIN_SIDE);
	if (grip.includes('s')) bottom = Math.max(Math.min(bottom + dy, 1), top + MIN_SIDE);
	return { x: left, y: top, width: right - left, height: bottom - top };
}

/** How far one press of an arrow moves or resizes a box at `zoom`: a hundredth of the page at
 * the pane's width, finer as the page is zoomed, so a step is about as long on screen. */
export function stepFor(zoom: number): number {
	return Math.max(0.01 / Math.max(zoom, 1), 0.001);
}

/** What a step did to a box, as a screen reader is told it. */
export type Said = 'right' | 'left' | 'up' | 'down' | 'wider' | 'narrower' | 'taller' | 'shorter';

/** What one press of a key does to a box: the arrows move it, Shift and the arrows resize it
 * from its right and bottom sides. Null for another key. */
export function keyStep(
	key: string,
	shift: boolean,
	step: number
): { grip: Grip; dx: number; dy: number; said: Said } | null {
	const along: Record<string, [number, number, Said, Said]> = {
		ArrowRight: [1, 0, 'right', 'wider'],
		ArrowLeft: [-1, 0, 'left', 'narrower'],
		ArrowDown: [0, 1, 'down', 'taller'],
		ArrowUp: [0, -1, 'up', 'shorter']
	};
	const at = along[key];
	if (!at) return null;
	const [sx, sy, moved, sized] = at;
	if (!shift) return { grip: 'move', dx: sx * step, dy: sy * step, said: moved };
	return { grip: sx !== 0 ? 'e' : 's', dx: sx * step, dy: sy * step, said: sized };
}

/** A new box in the middle of `view`, the part of the page in view (fractions of the page as
 * drawn): a third of its width and a tenth of its height, on the page. */
export function middleOf(view: Box | null): Box {
	const v = view ? clampBox(view) : { x: 0, y: 0, width: 1, height: 1 };
	const seen = v.width > 0 && v.height > 0 ? v : { x: 0, y: 0, width: 1, height: 1 };
	const width = Math.max(seen.width / 3, MIN_SIDE * 4);
	const height = Math.max(seen.height / 10, MIN_SIDE * 4);
	return clampBox({
		x: seen.x + (seen.width - width) / 2,
		y: seen.y + (seen.height - height) / 2,
		width,
		height
	});
}

/** Whether two boxes are the same, to a hair. */
export function sameBox(a: Box, b: Box): boolean {
	const near = (p: number, q: number) => Math.abs(p - q) < 1e-6;
	return near(a.x, b.x) && near(a.y, b.y) && near(a.width, b.width) && near(a.height, b.height);
}

/** A stretch of the window from top to bottom, in pixels. */
export interface Span {
	top: number;
	bottom: number;
}

/**
 * Whether the card that describes a region, at its place under the pane's bar, would cover the
 * region on the screen (with a little room around it): it then goes to the foot of the scan
 * instead, so the box it speaks of stays in sight.
 */
export function noteCovers(region: Span, note: Span, room = 8): boolean {
	return region.top < note.bottom + room && region.bottom > note.top - room;
}
