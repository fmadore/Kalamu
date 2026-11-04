// The arithmetic of zooming a page in the review's source pane, apart from the page, so it can
// be tested: how far a turn of the wheel or a pinch zooms, the limits, and how far to scroll so
// the point under the pointer stays under it.

export const MIN_ZOOM = 0.25;
export const MAX_ZOOM = 6;

export function clampZoom(zoom: number): number {
	return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
}

/**
 * How much one wheel event zooms, as a factor: in proportion to how far the wheel turned, so a
 * trackpad's many small steps and a mouse's notches zoom alike (a notch, about 100 pixels, by
 * about a quarter). `deltaMode` is the event's: pixels, lines or pages.
 */
export function wheelFactor(deltaY: number, deltaMode: number): number {
	const unit = deltaMode === 1 ? 0.05 : deltaMode === 2 ? 1 : 0.0025;
	return Math.exp(-Math.max(-150, Math.min(150, deltaY)) * unit);
}

type Box = { left: number; top: number; width: number; height: number };
type Point = { x: number; y: number };
/** A place on the page, as fractions of its width and height from its top left corner. */
export type Place = { u: number; v: number };

/** The place on the page, its box on screen, under a point of the window. */
export function placeUnder(page: Box, at: Point): Place {
	return {
		u: (at.x - page.left) / Math.max(page.width, 1),
		v: (at.y - page.top) / Math.max(page.height, 1)
	};
}

/** How far to scroll, across and down, to bring a place on the page under a point. */
export function scrollToPlace(page: Box, place: Place, at: Point): Point {
	return { x: page.left + place.u * page.width - at.x, y: page.top + place.v * page.height - at.y };
}

/**
 * How far to scroll, across and down, so that the point of the page under `at` before a zoom is
 * under it again after: the page's box on screen before and after.
 */
export function keepUnder(before: Box, after: Box, at: Point): Point {
	return scrollToPlace(after, placeUnder(before, at), at);
}

/** The zoom `t` of the way (0 to 1, eased) from one zoom to another, in equal ratios so zooming
 * in and out feel alike. */
export function between(from: number, to: number, t: number): number {
	const eased = 1 - (1 - t) ** 3;
	return from * (to / from) ** eased;
}
