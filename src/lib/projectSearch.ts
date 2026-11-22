// A project's search, as the interface shows it: where each match stands and what it was found
// in, and what the review is asked to open. The search itself runs in Rust (`kalamu-search`).
// Pure, so it is tested without the app.

import type { Hit } from './bindings/Hit.js';
import type { MatchMode } from './bindings/MatchMode.js';
import type { ResultHits } from './bindings/ResultHits.js';
import type { SearchRequest } from './bindings/SearchRequest.js';
import type { SearchView } from './bindings/SearchView.js';
import { fieldLabel, type Messages } from './i18n/index.js';
import { clock, pieceLabel } from './pieces.js';
import { folded } from './search.js';
import rules from './searchRules.json';
import type { ReviewFind } from './state.svelte.js';

/** The fewest letters a word searched for has, from Rust (`kalamu_search::MIN_QUERY`): two,
 * found as a word on its own; three or more are found anywhere, inside words too. */
export const MIN_QUERY: number = rules.min_query;

/** Quotes that make several words one term, as Rust reads them. */
const QUOTES = /["“”«»]/;
/** Punctuation set aside at either end of a word. */
const PUNCTUATION = /^[!-/:-@[-`{-~]+|[!-/:-@[-`{-~]+$/g;
/** Written in capitals between two words or phrases: either will do. Both languages' words are
 * read whatever the interface's language, so a query means the same in each. */
const OPERATORS: string[] = rules.operators;
/** Scripts written without spaces between words, as ranges of code points: two of their letters
 * are not a word on its own. */
const UNSPACED: number[][] = rules.unspaced;
const LETTER = /^[\p{Alphabetic}\p{N}]$/u;

/** A letter or digit of a script written with spaces between words. */
function spacedLetter(c: string): boolean {
	const code = c.codePointAt(0) ?? 0;
	return LETTER.test(c) && !UNSPACED.some(([a, b]) => code >= a && code <= b);
}

/** A word or words looked for, as `terms::Term`. */
export interface Term {
	/** As typed, which the review searches for when a match opens. */
	typed: string;
	needle: string;
	/** Two letters, found only as a word on its own. */
	word: boolean;
}

/** What a query looks for, as `terms::parse` reads it. */
export interface ParsedQuery {
	/** Every group is looked for; each holds one or more terms, any of which will do. */
	groups: Term[][];
	/** Words too short to look for, left out of the query. */
	ignored: string[];
}

/**
 * The terms of a query as `mode` reads it, as `kalamu_search` does: for a phrase the words
 * between operators, otherwise each word, words between quotes kept together and punctuation
 * around a word set aside, OR or OU between two of them making them alternatives. A term that
 * folds to two letters is a word on its own; one shorter, or of two characters that are not
 * letters of a script written with spaces, is left out and named.
 */
export function parseQuery(query: string, mode: MatchMode): ParsedQuery {
	type Token = { text: string; quoted: boolean };
	const tokens = query.split(QUOTES).flatMap((chunk, i): Token[] =>
		i % 2 === 1
			? [{ text: chunk.split(/\s+/).filter(Boolean).join(' '), quoted: true }]
			: chunk
					.split(/\s+/)
					.filter(Boolean)
					.map((w) => ({ text: mode === 'phrase' ? w : w.replace(PUNCTUATION, ''), quoted: false }))
	);
	const operator = (t: Token) => !t.quoted && OPERATORS.includes(t.text);
	const written: string[][] = [];
	if (mode === 'phrase') {
		const alternatives: string[] = [];
		let run: string[] = [];
		for (const t of tokens) {
			if (operator(t)) {
				alternatives.push(run.join(' '));
				run = [];
			} else if (t.text) run.push(t.text);
		}
		alternatives.push(run.join(' '));
		written.push(alternatives);
	} else {
		let or = false;
		for (const t of tokens.filter((t) => t.text)) {
			if (operator(t)) or = written.length > 0;
			else {
				if (or) written[written.length - 1].push(t.text);
				else written.push([t.text]);
				or = false;
			}
		}
	}
	const out: ParsedQuery = { groups: [], ignored: [] };
	const seen = new Set<string>();
	for (const alternatives of written) {
		const group: Term[] = [];
		for (const typed of alternatives.filter(Boolean)) {
			const needle = folded(typed);
			const chars = [...needle];
			const word = chars.length === 2 && chars.every(spacedLetter);
			if (chars.length < 3 && !word) {
				if (!out.ignored.includes(typed)) out.ignored.push(typed);
			} else if (!seen.has(needle)) {
				seen.add(needle);
				group.push({ typed, needle, word });
			}
		}
		if (group.length > 0) out.groups.push(group);
	}
	return out;
}

/**
 * Whether a query has a word long enough to search for, as `kalamu_search` parses it. Until it
 * has, the search waits rather than answering that the words are too short.
 */
export function searchableQuery(query: string, mode: MatchMode): boolean {
	return parseQuery(query, mode).groups.length > 0;
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
	const parts = [hit.entry, fieldLabel(hit.field.label, hit.field.key)].filter(
		(p): p is string => !!p
	);
	return parts.join(' › ') + (hit.reason ? ` (${m.search.reason})` : '');
}

/** Whether a result is said to be written by a model: an analysis or a table of contents,
 * unless every match shown in it is in a name the researcher listed by hand. */
export function writtenByModel(result: ResultHits): boolean {
	return result.generated && result.hits.some((h) => !h.by_hand);
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

/** The last search of each project in this session, with what it found, so that coming back to
 * the search (Back from a match's review, say) shows it again without searching anew. Kept in
 * memory only. */
const last = new Map<string, { request: SearchRequest; view: SearchView | null }>();

export function lastSearch(
	projectId: string
): { request: SearchRequest; view: SearchView | null } | undefined {
	return last.get(projectId);
}

export function keepSearch(projectId: string, request: SearchRequest, view: SearchView | null) {
	last.set(projectId, { request, view });
}
