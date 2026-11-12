// The names of a source's pieces in the review: in the menu of pages, over the source, under
// each page in the strip.

import type { PieceRef } from './bindings/PieceRef.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Segment } from './bindings/Segment.js';
import { formatUsd, type Messages } from './i18n/index.js';

/** A time in a recording, as hours, minutes and seconds: "01:02:03". */
export function clock(ms: number): string {
	const s = Math.floor(ms / 1000);
	return [s / 3600, (s / 60) % 60, s % 60]
		.map((n) => String(Math.floor(n)).padStart(2, '0'))
		.join(':');
}

/** Whether a line is being heard `seconds` into its recording. */
export function sounding(segment: Segment, seconds: number): boolean {
	return seconds * 1000 >= Number(segment.start_ms) && seconds * 1000 < Number(segment.end_ms);
}

/** A piece's full name: "Page 3", "Page 3, left half", "Passage 2", "From 00:10:00". A whole
 * YouTube video is a video, any other whole source a document. */
export function pieceLabel(piece: PieceRef, m: Messages, online = false): string {
	switch (piece.kind) {
		case 'page':
			return piece.half
				? m.review.pageHalf(piece.number, piece.half)
				: m.review.page(String(piece.number));
		case 'whole':
			return online ? m.review.video : m.review.document;
		case 'passage':
			return m.review.passage(piece.number);
		case 'segment':
			return m.review.segment(clock(piece.start_ms));
	}
}

/** A piece's short name in the strip: a page's number ("3", "3l"), any other piece its place
 * among the source's pieces, counted from 0 and named from 1. */
export function pieceNumber(piece: PieceRef, place: number, m: Messages): string {
	if (piece.kind !== 'page') return String(place + 1);
	return piece.half ? m.review.pageHalfShort(piece.number, piece.half) : String(piece.number);
}

/**
 * A list of pages as the engine writes it ("1, 3-5, 8 (left)"), with the halves named in the
 * interface language ("8 (gauche)"), for reading and correcting.
 */
export function typedPages(pages: string, m: Messages): string {
	return pages.replace(
		/\((left|right)\)/g,
		(_, half: 'left' | 'right') => `(${m.units.halves[half]})`
	);
}

/** A list of pages as typed in the interface language, with the halves as the engine reads them. */
export function enginePages(pages: string, m: Messages): string {
	// The words are letters only, so they need no escaping in a pattern.
	return (['left', 'right'] as const).reduce(
		(text, half) => text.replace(new RegExp(m.units.halves[half], 'gi'), half),
		pages
	);
}

/** What a piece cost, after its model's name: " · $0.003", or " · From cache" for a piece
 * answered from an earlier reading, which was not paid for again (the result keeps the cost
 * of the call that made it); nothing when the cost is not known. */
export function pieceCost(view: PieceView | undefined, m: Messages): string {
	if (view?.status === 'cached') return ` · ${m.status.cached}`;
	return view?.cost_usd ? ` · ${formatUsd(view.cost_usd)}` : '';
}
