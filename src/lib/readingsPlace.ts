// Where the readings of one difference open in the text pane: under the words, over them when
// there is no room below, and otherwise wherever they fit whole in the part of the pane that
// shows, so a long paragraph near either edge never cuts off a reading.

/** A stretch of the window from top to bottom, in the window's pixels. */
export interface Band {
	top: number;
	bottom: number;
}

/** The space kept between the readings, the words and the pane's edges. */
export const GAP = 8;

/** The part of the pane the researcher sees: the pane, cut by the window's own edges when the
 * pane runs under them. */
export function visibleBand(pane: Band, windowHeight: number): Band {
	const top = Math.max(pane.top, 0);
	return { top, bottom: Math.max(top, Math.min(pane.bottom, windowHeight)) };
}

/** The tallest the readings may be to show whole in that part: what is taller scrolls inside. */
export function tallest(visible: Band): number {
	return Math.max(0, visible.bottom - visible.top - 2 * GAP);
}

/** The line of the words the readings belong to: the last one that shows, as a long paragraph
 * may run on past the pane's edge; the last line when none shows. */
export function anchorLine<T extends Band>(lines: T[], visible: Band): T | null {
	const shown = lines.filter((l) => l.bottom > visible.top && l.top < visible.bottom);
	return shown.at(-1) ?? lines.at(-1) ?? null;
}

/** The top of readings `height` tall, in the window's pixels: under the line when they fit
 * there, over it when they fit there, else kept whole inside the visible part, on the side with
 * more room. */
export function readingsTop(line: Band, visible: Band, height: number): number {
	const below = line.bottom + GAP;
	const above = line.top - GAP - height;
	if (below + height <= visible.bottom - GAP) return below;
	if (above >= visible.top + GAP) return above;
	const first = visible.top + GAP;
	const last = Math.max(first, visible.bottom - GAP - height);
	const roomBelow = visible.bottom - line.bottom;
	const roomAbove = line.top - visible.top;
	const wanted = roomBelow >= roomAbove ? below : above;
	return Math.min(Math.max(wanted, first), last);
}
