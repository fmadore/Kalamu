// Search in the review: words found in a document's text as the researcher reads it, ignoring
// capitals, accents and Arabic vowel marks, any run of spaces or line breaks matching one space.
// Matches are given in the text's own offsets, so they can be marked where they stand.

/** Where words were found: `[start, end)` in the text searched. */
export interface Match {
	start: number;
	end: number;
}

/** A text folded for matching, each folded character tied back to the stretch of the text it
 * came from. */
interface Folded {
	text: string;
	from: number[];
	to: number[];
}

const MARKS = /\p{M}/gu;
const SPACE = /\s/u;

function fold(text: string): Folded {
	const out: Folded = { text: '', from: [], to: [] };
	let i = 0;
	for (const ch of text) {
		const next = i + ch.length;
		if (SPACE.test(ch)) {
			// A run of white space is one space.
			if (out.text.endsWith(' ')) out.to[out.to.length - 1] = next;
			else {
				out.text += ' ';
				out.from.push(i);
				out.to.push(next);
			}
		} else {
			const letters = ch.normalize('NFD').replace(MARKS, '').toLowerCase();
			// A mark on its own (a vowel mark, an accent typed apart) belongs to the letter before.
			if (letters === '' && out.to.at(-1) === i) out.to[out.to.length - 1] = next;
			for (const c of letters) {
				out.text += c;
				out.from.push(i);
				out.to.push(next);
			}
		}
		i = next;
	}
	return out;
}

/** The query as it is matched: folded, without space at either end. */
export function folded(query: string): string {
	return fold(query).text.trim();
}

/** Whether a query is long enough to search for: two letters at least. */
export function searchable(query: string): boolean {
	return [...folded(query)].length >= 2;
}

/** Every place `query` is found in `text`, in order, none overlapping. */
export function find(text: string, query: string): Match[] {
	const needle = folded(query);
	if (!searchable(query)) return [];
	const hay = fold(text);
	const out: Match[] = [];
	let at = hay.text.indexOf(needle);
	while (at >= 0) {
		const last = at + needle.length - 1;
		out.push({ start: hay.from[at], end: hay.to[last] });
		at = hay.text.indexOf(needle, at + needle.length);
	}
	return out;
}

/** A match with some of the words around it, on one line, cut at word boundaries. */
export function context(
	text: string,
	m: Match,
	around = 48
): { before: string; words: string; after: string } {
	const line = (s: string) => s.replace(/\s+/g, ' ');
	let from = Math.max(0, m.start - around);
	let to = Math.min(text.length, m.end + around);
	if (from > 0) from = text.indexOf(' ', from) + 1 || from;
	if (to < text.length) to = text.lastIndexOf(' ', to) > m.end ? text.lastIndexOf(' ', to) : to;
	return {
		before: (from > 0 ? '…' : '') + line(text.slice(from, m.start)).trimStart(),
		words: line(text.slice(m.start, m.end)),
		after: line(text.slice(m.end, to)).trimEnd() + (to < text.length ? '…' : '')
	};
}

/** One piece of a source as it is searched: its text as the review shows it and, for a
 * recording, its timed lines. */
export interface Searched {
	index: number;
	text: string;
	/** Where the piece starts in the recording. */
	startMs: number;
	segments: { start_ms: number; text: string }[];
}

/** A match in one piece of a source. */
export interface Hit extends Match {
	/** The piece it is in. */
	index: number;
	/** Which match it is in that piece, from 0. */
	nth: number;
	/** In a recording, when the line it is in begins, from the piece's start; null when the
	 * line cannot be told (the text was corrected into other words). */
	lineMs: number | null;
}

/** Every match in a source's pieces, piece after piece. In a recording, each match is tied to
 * the timed line it is in when the lines hold as many matches as the text. */
export function search(pieces: Searched[], query: string): Hit[] {
	if (!searchable(query)) return [];
	const out: Hit[] = [];
	for (const p of pieces) {
		const found = find(p.text, query);
		const lines = p.segments.flatMap((s) => find(s.text, query).map(() => Number(s.start_ms)));
		const timed = lines.length === found.length;
		found.forEach((m, nth) =>
			out.push({ ...m, index: p.index, nth, lineMs: timed ? (lines[nth] ?? null) : null })
		);
	}
	return out;
}
