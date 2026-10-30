// The analysis review's collation: where set-ups answer a field differently, each value with the
// sigla of the set-ups that gave it, as an edition's index notes which witnesses carry a name.

import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { FieldValue } from './bindings/FieldValue.js';
import { find } from './search.js';

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
	if (Array.isArray(value)) return value.filter((v): v is string => typeof v === 'string');
	return value.trim() === '' ? [] : [value];
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

/** Whether any witness's answer to a field, or its reason, holds the words searched for. */
export function holdsWords(key: string, witnesses: Witness[], query: string): boolean {
	return witnesses.some((w) => {
		const answer = w.answers.find((a) => a.key === key);
		const texts = [...valuesOf(answer?.value), answer?.reason ?? ''];
		return texts.some((t) => find(t, query).length > 0);
	});
}

/** The parts of a text, with each occurrence of `term` marked, found as the review's search
 * finds words (ignoring capitals, accents and runs of spaces). */
export function findTerm(text: string, term: string): { text: string; hit: boolean }[] {
	const out: { text: string; hit: boolean }[] = [];
	let at = 0;
	for (const match of find(text, term)) {
		if (match.start > at) out.push({ text: text.slice(at, match.start), hit: false });
		out.push({ text: text.slice(match.start, match.end), hit: true });
		at = match.end;
	}
	if (at < text.length || out.length === 0) out.push({ text: text.slice(at), hit: false });
	return out;
}
