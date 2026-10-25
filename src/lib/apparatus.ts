// The review view's apparatus criticus: where other witnesses (another set-up's text, or the
// model's own text under a correction) disagree with the reading text, entry by entry and line
// by line, as a critical edition sets its variants below the text.

import type { DiffSpan } from './bindings/DiffSpan.js';

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
			let { start, end } = v;
			while (start < end && /\s/.test(base[start])) start++;
			while (end > start && /\s/.test(base[end - 1])) end--;
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
		.map((e, i) => ({ ...e, id: i + 1 }));
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
		for (let k = 0; k + 1 < points.length; k++) {
			const [a, b] = [points[k], points[k + 1]];
			if (a === b) continue;
			const entry = entries.find((e) => e.start <= a && e.end >= b && e.end > e.start);
			parts.push({ text: base.slice(a, b), entry: entry?.id ?? null });
		}
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
