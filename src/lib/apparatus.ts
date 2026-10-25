// The review view's apparatus criticus: where other witnesses (another set-up's text, or the
// model's own text under a correction) disagree with the reading text, entry by entry and line
// by line, as a critical edition sets its variants below the text.

import type { DiffSpan } from './bindings/DiffSpan.js';

/** a, b, c…: the letter that names a job's set-up as a witness, wherever set-ups are listed. */
export function siglum(index: number): string {
	return String.fromCharCode(97 + index);
}

/** A reading of one witness at one place. `text` is empty when the witness omits the lemma. */
export interface Reading {
	siglum: string;
	text: string;
}

/** One place of variation in the reading text: `[start, end)` in it, and what others read. */
export interface Entry {
	id: number;
	line: number;
	start: number;
	end: number;
	/** The reading text's words at this place; empty when a witness adds words here. */
	lemma: string;
	readings: Reading[];
}

/** One line of the reading text, cut where entries begin and end. Blank lines, which separate
 * paragraphs, are not counted: their number is 0. */
export interface Line {
	number: number;
	parts: { text: string; entry: number | null }[];
}

interface Variant {
	start: number;
	end: number;
	text: string;
}

/** The places where a witness differs from the base, from a word diff of the two. */
export function variants(spans: DiffSpan[]): Variant[] {
	const out: Variant[] = [];
	let pos = 0;
	let open: Variant | null = null;
	const close = () => {
		if (open) out.push(open);
		open = null;
	};
	for (const span of spans) {
		const length = span.text.length;
		if (span.kind === 'same') {
			// Whitespace between two changed words keeps them one variant.
			if (open && span.text.trim() === '' && length < 3) {
				open.end = pos + length;
				open.text += span.text;
			} else {
				close();
			}
			pos += length;
		} else if (span.kind === 'removed') {
			open ??= { start: pos, end: pos, text: '' };
			open.end = pos + length;
			pos += length;
		} else {
			open ??= { start: pos, end: pos, text: '' };
			open.text += span.text;
		}
	}
	close();
	return out.map((v) => trimVariant(v));
}

/** Space at either end of a lemma belongs to the text around it. */
function trimVariant(v: Variant): Variant {
	return { ...v, text: v.text.trim() };
}

/** Each raw line's number as an edition counts it: blank lines are skipped (0). */
function lineNumbers(base: string): number[] {
	let n = 0;
	return base.split('\n').map((line) => (line.trim() === '' ? 0 : ++n));
}

function lineAt(starts: number[], numbers: number[], offset: number): number {
	let line = 0;
	while (line + 1 < starts.length && starts[line + 1] <= offset) line++;
	return numbers[line] || 1;
}

/** A place in the base without the space at either end, as entries are keyed. */
function withoutSpace(base: string, start: number, end: number): [number, number] {
	while (start < end && /\s/.test(base[start])) start++;
	while (end > start && /\s/.test(base[end - 1])) end--;
	return [start, end];
}

/** Entries for the base text against each witness, merged where they vary at the same place. */
export function apparatus(
	base: string,
	witnesses: { siglum: string; spans: DiffSpan[] }[]
): Entry[] {
	const starts = [0];
	for (let i = 0; i < base.length; i++) if (base[i] === '\n') starts.push(i + 1);
	const numbers = lineNumbers(base);
	const byPlace = new Map<string, Entry>();
	for (const witness of witnesses) {
		for (const v of variants(witness.spans)) {
			const [start, end] = withoutSpace(base, v.start, v.end);
			const key = `${start}:${end}`;
			const entry = byPlace.get(key) ?? {
				id: 0,
				line: lineAt(starts, numbers, start),
				start,
				end,
				lemma: base.slice(start, end),
				readings: []
			};
			entry.readings.push({ siglum: witness.siglum, text: v.text });
			byPlace.set(key, entry);
		}
	}
	return [...byPlace.values()]
		.sort((a, b) => a.start - b.start || a.end - b.end)
		.map((e, i) => ({ ...withoutSharedPunctuation(e), id: i + 1 }));
}

/** Punctuation the lemma and every reading end with is not part of the variant, as an edition
 * writes "minaret ] marché", not "minaret. ] marché.". The highlighted range is unchanged. */
function withoutSharedPunctuation(entry: Entry): Entry {
	const tail = /[.,;:!?»”)]+$/.exec(entry.lemma)?.[0];
	if (!tail || !entry.readings.every((r) => r.text.endsWith(tail))) return entry;
	return {
		...entry,
		lemma: entry.lemma.slice(0, -tail.length),
		readings: entry.readings.map((r) => ({ ...r, text: r.text.slice(0, -tail.length) }))
	};
}

/** The reading text as numbered lines, cut at entries so their lemmata can be highlighted. */
export function lines(base: string, entries: Entry[]): Line[] {
	const out: Line[] = [];
	const numbers = lineNumbers(base);
	let offset = 0;
	base.split('\n').forEach((text, i) => {
		const lineStart = offset;
		const lineEnd = offset + text.length;
		const cuts = new Set([lineStart, lineEnd]);
		for (const e of entries) {
			if (e.end > lineStart && e.start < lineEnd) {
				cuts.add(Math.max(e.start, lineStart));
				cuts.add(Math.min(e.end, lineEnd));
			}
		}
		const points = [...cuts].sort((a, b) => a - b);
		const parts: Line['parts'] = [];
		// Words another witness adds have no lemma: they show as an empty part where they go.
		const additions = (at: number) =>
			entries.filter((e) => e.start === at && e.end === at).map((e) => ({ text: '', entry: e.id }));
		for (let k = 0; k + 1 < points.length; k++) {
			const [a, b] = [points[k], points[k + 1]];
			parts.push(...additions(a));
			const entry = entries.find((e) => e.start <= a && e.end >= b && e.end > e.start);
			parts.push({ text: base.slice(a, b), entry: entry?.id ?? null });
		}
		parts.push(...additions(lineEnd));
		if (parts.length === 0) parts.push({ text: '', entry: null });
		out.push({ number: numbers[i], parts });
		offset = lineEnd + 1;
	});
	return out;
}

/** Whether a line number is printed in the margin: every fifth line and every line with a variant. */
export function numbered(line: number, entries: Entry[]): boolean {
	return line > 0 && (line % 5 === 0 || entries.some((e) => e.line === line));
}

/** A witness's own text, cut where it differs from the base, each difference tied to the entry
 * it belongs to, so the two texts can be read side by side. Words the witness leaves out are an
 * empty part. */
export function witnessParts(base: string, spans: DiffSpan[], entries: Entry[]): Line['parts'] {
	const out: Line['parts'] = [];
	let pos = 0;
	let open: Variant | null = null;
	const close = () => {
		if (!open) return;
		const [start, end] = withoutSpace(base, open.start, open.end);
		const entry = entries.find((e) => e.start === start && e.end === end)?.id ?? null;
		const before = /^\s*/.exec(open.text)?.[0] ?? '';
		const words = open.text.slice(before.length).trimEnd();
		const after = open.text.slice(before.length + words.length);
		if (before) out.push({ text: before, entry: null });
		out.push({ text: words, entry });
		if (after) out.push({ text: after, entry: null });
		open = null;
	};
	// As in `variants`, a short space between two changed words keeps them one difference.
	for (const span of spans) {
		const length = span.text.length;
		if (span.kind === 'same') {
			if (open && span.text.trim() === '' && length < 3) {
				open.end = pos + length;
				open.text += span.text;
			} else {
				close();
				out.push({ text: span.text, entry: null });
			}
			pos += length;
		} else if (span.kind === 'removed') {
			open ??= { start: pos, end: pos, text: '' };
			open.end = pos + length;
			pos += length;
		} else {
			open ??= { start: pos, end: pos, text: '' };
			open.text += span.text;
		}
	}
	close();
	return out;
}
