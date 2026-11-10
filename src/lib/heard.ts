// A recording's transcript is shown once, as the text read and corrected; the passage the player
// is at is found in that text and marked where it stands through the browser's own highlights
// (`::highlight(heard)` in app.css), as a search's words are, so the text's elements stay as
// they are.

import { located, supported } from './found.js';

/** A stretch of a text, from `start` to `end` (UTF-16 offsets). */
export interface Span {
	start: number;
	end: number;
}

const WORD = /[\p{L}\p{N}]+/gu;

/** The words of a text, lower-cased, with where each stands. */
function words(text: string): { word: string; start: number; end: number }[] {
	return [...text.matchAll(WORD)].map((m) => ({
		word: m[0].toLowerCase(),
		start: m.index,
		end: m.index + m[0].length
	}));
}

/** Where a run of words begins among `tokens`, looked for from `from` over `within` words; -1
 * when it is not there. */
function findRun(tokens: { word: string }[], run: string[], from: number, within: number): number {
	const last = Math.min(tokens.length - run.length, from + within);
	for (let i = from; i <= last; i++) {
		if (run.every((w, k) => tokens[i + k].word === w)) return i;
	}
	return -1;
}

/** How far ahead of the line before a line's words are looked for: past a few lines the text
 * no longer holds, never across the whole text. */
const AHEAD = 400;

/**
 * Where each timed line stands in the text shown, in order: found by its first words (else by
 * its last, when a correction changed how it begins) after where the line before ended,
 * ignoring capitals and punctuation, so a speaker's label or a time written before the words
 * is passed over. Null for a line the text no longer holds, corrected away.
 */
export function placeLines(text: string, lines: string[]): (Span | null)[] {
	const tokens = words(text);
	let from = 0;
	return lines.map((line) => {
		const said = words(line).map((t) => t.word);
		if (said.length === 0 || from >= tokens.length) return null;
		const k = Math.min(3, said.length);
		const at = findRun(tokens, said.slice(0, k), from, AHEAD);
		if (at >= 0) {
			const last = Math.min(at + said.length - 1, tokens.length - 1);
			from = last + 1;
			return { start: tokens[at].start, end: tokens[last].end };
		}
		const tail = findRun(tokens, said.slice(-k), from, AHEAD + said.length);
		if (tail < 0) return null;
		const last = tail + k - 1;
		const first = Math.max(from, last - said.length + 1);
		from = last + 1;
		return { start: tokens[first].start, end: tokens[last].end };
	});
}

/** Takes the mark off. */
export function clearHeard(): void {
	if (supported()) CSS.highlights.delete('heard');
}

/** Marks `span` of the text of the `blocks` under `root`, each block read as a line of its own,
 * as the passage being heard, and returns where it stands; none marks nothing. */
export function markHeard(root: ParentNode, blocks: string, span: Span | null): Range | null {
	clearHeard();
	if (!span || !supported()) return null;
	const range = located(root, blocks).range(span.start, span.end);
	if (range) CSS.highlights.set('heard', new Highlight(range));
	return range;
}
