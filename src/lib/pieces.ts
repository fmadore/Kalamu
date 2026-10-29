// The names of a source's pieces in the review: in the menu of pages, over the source, under
// each page in the strip.

import type { PieceRef } from './bindings/PieceRef.js';
import type { Messages } from './i18n/index.js';

/** A time in a recording, as hours, minutes and seconds: "01:02:03". */
export function clock(ms: number): string {
	const s = Math.floor(ms / 1000);
	return [s / 3600, (s / 60) % 60, s % 60]
		.map((n) => String(Math.floor(n)).padStart(2, '0'))
		.join(':');
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
