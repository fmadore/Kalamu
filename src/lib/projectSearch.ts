// A project's search, as the interface shows it: where each match stands and what it was found
// in, and what the review is asked to open. The search itself runs in Rust (`kalamu-search`).
// Pure, so it is tested without the app.

import type { Hit } from './bindings/Hit.js';
import type { MatchMode } from './bindings/MatchMode.js';
import type { ResultHits } from './bindings/ResultHits.js';
import type { SearchRequest } from './bindings/SearchRequest.js';
import type { SearchView } from './bindings/SearchView.js';
import type { Messages } from './i18n/index.js';
import { clock, pieceLabel } from './pieces.js';
import { folded } from './search.js';
import rules from './searchRules.json';
import type { ReviewFind } from './state.svelte.js';

/** The fewest letters a word searched for has, from Rust (`kalamu_search::MIN_QUERY`). */
export const MIN_QUERY: number = rules.min_query;

/** Quotes that make several words one term, as Rust reads them. */
const QUOTES = /["“”«»]/;

/**
 * Whether a query has a word long enough to search for, as `kalamu_search` parses it: the whole
 * phrase, or else each word or quoted run, punctuation at its ends set aside. Until it has, the
 * search waits rather than answering that the words are too short.
 */
export function searchableQuery(query: string, mode: MatchMode): boolean {
	const long = (typed: string) => [...folded(typed)].length >= MIN_QUERY;
	if (mode === 'phrase') return long(query);
	return query.split(QUOTES).some((chunk, i) =>
		i % 2 === 1
			? long(chunk)
			: chunk
					.split(/\s+/)
					.map((w) => w.replace(/^[!-/:-@[-`{-~]+|[!-/:-@[-`{-~]+$/g, ''))
					.some(long)
	);
}

/** How many words may stand between words looked for near each other, unless changed. */
export const NEAR = 10;

/** The search a project opens with: a phrase, in every document, the preferred readings, the
 * text only. */
export function blankRequest(): SearchRequest {
	return {
		query: '',
		mode: 'phrase',
		near: NEAR,
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
 * finds there for the words of the match; for an answer, its field, and its entry in a table of
 * contents. */
export function reviewFind(result: ResultHits, hit: Hit): ReviewFind {
	return {
		query: hit.term,
		setupId: result.setup_id,
		index: hit.index,
		nth: hit.nth,
		field: hit.field?.key ?? null,
		entry: hit.entry_number
	};
}

/** A match's words cut where the words looked for stand, so that only those are marked: for
 * words near each other, the words between them are not. */
export function hitWords(hit: Hit): { text: string; found: boolean }[] {
	const chars = [...hit.words];
	const out: { text: string; found: boolean }[] = [];
	let at = 0;
	const marks = [...hit.marks].sort((a, b) => a.start - b.start || b.end - a.end);
	for (const { start, end } of marks) {
		if (end <= at) continue;
		const from = Math.max(start, at);
		if (from > at) out.push({ text: chars.slice(at, from).join(''), found: false });
		out.push({ text: chars.slice(from, end).join(''), found: true });
		at = end;
	}
	if (at < chars.length) out.push({ text: chars.slice(at).join(''), found: false });
	return out;
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
