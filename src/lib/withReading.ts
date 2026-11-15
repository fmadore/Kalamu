// A reading chosen in the review, taken into the text shown as a correction: the words of the
// place it reads differently are replaced by it, the rest of the text left as it was.

import type { Entry } from './apparatus.js';

/**
 * Text `base` with `text` in place of entry `e`'s words, `[e.start, e.end)` in it. The entry's
 * range may end in punctuation shared by every reading and left out of them: that stays after
 * the reading taken.
 */
export function withReading(
	base: string,
	e: Pick<Entry, 'start' | 'end' | 'lemma'>,
	text: string
): string {
	const range = base.slice(e.start, e.end);
	const tail = range.startsWith(e.lemma) ? range.slice(e.lemma.length) : '';
	return base.slice(0, e.start) + text + tail + base.slice(e.end);
}
