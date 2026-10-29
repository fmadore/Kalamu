// A table of contents in the review: its entries as the combining step wrote them, the pages
// each names, and the drafts the researcher edits. Kept out of the component to be tested.

import type { Field } from './bindings/Field.js';
import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { Setup } from './bindings/Setup.js';

export const TITLE = 'title';
export const PAGES = 'pages';
export const ENTRIES = 'entries';

/** The fields of each entry (title, pages, then the researcher's), from the set-up's merge step. */
export function entryFields(setup: Setup | undefined): Field[] {
	const kind = setup?.merge?.prompt.schema?.fields[0]?.kind;
	return kind?.type === 'entries' ? kind.fields : [];
}

/** The entries in a set of answers; an empty list reads back as a list of strings. */
export function entriesOf(answers: FieldAnswer[]): FieldAnswer[][] {
	const value = answers.find((a) => a.key === ENTRIES)?.value;
	return Array.isArray(value) && value.length > 0 && Array.isArray(value[0])
		? (value as FieldAnswer[][])
		: [];
}

/** An answer as one line of text: list items joined, as in the exports. */
export function valueText(entry: FieldAnswer[], key: string): string {
	const value = entry.find((a) => a.key === key)?.value;
	if (value === undefined) return '';
	if (typeof value === 'string') return value;
	return (value as unknown[]).filter((v) => typeof v === 'string').join('; ');
}

/** The pages an entry names (`"1, 3-5"` → 1, 3, 4, 5), at most a few hundred. */
export function pagesOf(pages: string): number[] {
	const out: number[] = [];
	for (const part of pages.split(/[,;]/)) {
		const range = part.match(/(\d+)\s*[-–]\s*(\d+)/);
		if (range) {
			const [from, to] = [Number(range[1]), Number(range[2])];
			for (let n = from; n <= Math.min(to, from + 500); n++) out.push(n);
		} else {
			const one = part.match(/\d+/);
			if (one) out.push(Number(one[0]));
		}
	}
	return out;
}

/** The answers that hold a corrected table of contents. */
export function asAnswers(entries: FieldAnswer[][]): FieldAnswer[] {
	return [{ key: ENTRIES, value: entries }];
}
