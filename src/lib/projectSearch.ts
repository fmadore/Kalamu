// A project's search, as the interface shows it: where each match stands and what it was found
// in, and what the review is asked to open. The search itself runs in Rust (`kalamu-search`).
// Pure, so it is tested without the app.

import type { Hit } from './bindings/Hit.js';
import type { ResultHits } from './bindings/ResultHits.js';
import type { SearchRequest } from './bindings/SearchRequest.js';
import type { SearchView } from './bindings/SearchView.js';
import type { Messages } from './i18n/index.js';
import { clock, pieceLabel } from './pieces.js';
import type { ReviewFind } from './state.svelte.js';

/** The search a project opens with: every document, the preferred readings, the text only. */
export function blankRequest(): SearchRequest {
	return {
		query: '',
		collection: null,
		tag: null,
		task: null,
		review: 'any',
		results: 'preferred',
		generated: false
	};
}

/** Where a match stands: the time of its line in a recording, else its page or part. */
export function hitPlace(hit: Hit, m: Messages): string {
	if (hit.at_ms !== null) return clock(Number(hit.at_ms));
	return pieceLabel(hit.piece, m);
}

/** What a match was found in, for an answer: the entry and the field, and whether it was the
 * model's reason; empty for text read from the document. */
export function hitOrigin(hit: Hit, m: Messages): string {
	if (!hit.field) return '';
	const parts = [hit.entry, hit.field.label].filter((p): p is string => !!p);
	return parts.join(' › ') + (hit.reason ? ` (${m.search.reason})` : '');
}

/** What the review opens at for a match: its piece, and the match among those its search
 * finds there with the same words. */
export function reviewFind(result: ResultHits, hit: Hit, query: string): ReviewFind {
	return { query, setupId: result.setup_id, index: hit.index, nth: hit.nth };
}

/** The last search of each project in this session, so that coming back from a review shows
 * it again. Kept in memory only. */
const last = new Map<string, { request: SearchRequest; view: SearchView | null }>();

export function lastSearch(
	projectId: string
): { request: SearchRequest; view: SearchView | null } | undefined {
	return last.get(projectId);
}

export function keepSearch(projectId: string, request: SearchRequest, view: SearchView | null) {
	last.set(projectId, { request, view });
}
