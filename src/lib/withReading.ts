// A reading chosen in the review, taken into the text shown as a correction: the words of the
// place it reads differently are replaced by it, the rest of the text left as it was.

import type { Entry } from './apparatus.js';

/** A space or tab: what stands between two words on a line. */
const SPACE = /[ \t  ]/;
/** Punctuation that follows a word without a space in English (French spaces are kept). */
const CLOSING = /^[.,;:!?)\]}»”’…]/;

/**
 * Text `base` with `text` in place of entry `e`'s words, `[e.start, e.end)` in it. The entry's
 * range may end in punctuation shared by every reading and left out of them: that stays after
 * the reading taken. An empty reading leaves the words out with one of the spaces around them,
 * so no double space is left, nor a space at the start or end of a line or before punctuation.
 */
export function withReading(
	base: string,
	e: Pick<Entry, 'start' | 'end' | 'lemma'>,
	text: string
): string {
	const range = base.slice(e.start, e.end);
	const tail = range.startsWith(e.lemma) ? range.slice(e.lemma.length) : '';
	let head = base.slice(0, e.start);
	let rest = tail + base.slice(e.end);
	const removed = range.length > tail.length;
	if (text === '' && removed) {
		const spaceBefore = head.length > 0 && SPACE.test(head.slice(-1));
		const lineStart = head === '' || head.endsWith('\n');
		if (
			spaceBefore &&
			(rest === '' || rest.startsWith('\n') || SPACE.test(rest[0]) || CLOSING.test(rest))
		) {
			head = head.slice(0, -1);
		} else if (lineStart && rest.length > 0 && SPACE.test(rest[0])) {
			rest = rest.slice(1);
		}
	}
	return head + text + rest;
}
