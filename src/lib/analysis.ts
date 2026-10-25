// The analysis review's collation: where set-ups answer a field differently, each value with the
// sigla of the set-ups that gave it, as an edition's index notes which witnesses carry a name.

import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { FieldValue } from './bindings/FieldValue.js';

/** One set-up's answers, named by its siglum. */
export interface Witness {
	siglum: string;
	answers: FieldAnswer[];
}

/** A value and the set-ups that gave it. */
export interface Reading {
	value: string;
	sigla: string[];
}

const same = (a: string, b: string) => a.trim().toLowerCase() === b.trim().toLowerCase();

function valuesOf(value: FieldValue | undefined): string[] {
	if (value === undefined) return [];
	return Array.isArray(value) ? value : value.trim() === '' ? [] : [value];
}

/**
 * Every value given for one field, in the order the witnesses give them (the lead's first), each
 * with the sigla of the witnesses that gave it, matched ignoring case. For a list, each item is
 * a value; for text or a choice, each whole answer is.
 */
export function collate(key: string, witnesses: Witness[]): Reading[] {
	const out: Reading[] = [];
	for (const w of witnesses) {
		const answer = w.answers.find((a) => a.key === key);
		for (const value of valuesOf(answer?.value)) {
			const known = out.find((r) => same(r.value, value));
			if (known) {
				if (!known.sigla.includes(w.siglum)) known.sigla.push(w.siglum);
			} else {
				out.push({ value, sigla: [w.siglum] });
			}
		}
	}
	return out;
}

/** The parts of a text, with each occurrence of `term` (ignoring case) marked. */
export function findTerm(text: string, term: string): { text: string; hit: boolean }[] {
	const needle = term.trim();
	if (needle === '') return [{ text, hit: false }];
	const pattern = new RegExp(needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
	const out: { text: string; hit: boolean }[] = [];
	let at = 0;
	for (const match of text.matchAll(pattern)) {
		const start = match.index ?? 0;
		if (start > at) out.push({ text: text.slice(at, start), hit: false });
		out.push({ text: match[0], hit: true });
		at = start + match[0].length;
	}
	if (at < text.length) out.push({ text: text.slice(at), hit: false });
	return out;
}
