// An analysis that quotes its text, read for the review: each entry of such a field as a card
// (a theme's name over its description, its quotations with who speaks and when), what an
// entry's fields read as while they are typed anew, and where a quotation stands in the text
// shown. Locating and checking quotations is the engine's: this only lays out what it kept.

import type { Anchor } from './bindings/Anchor.js';
import type { CitedEntry } from './bindings/CitedEntry.js';
import type { CitedEvidence } from './bindings/CitedEvidence.js';
import type { CitedValue } from './bindings/CitedValue.js';
import type { Field } from './bindings/Field.js';
import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { FieldValue } from './bindings/FieldValue.js';
import { clock } from './pieces.js';

/** A field's value when it is entries with their quotations. */
export function citedValue(value: FieldValue | undefined): CitedValue | null {
	return typeof value === 'object' && value !== null && !Array.isArray(value) ? value : null;
}

/** The cited value of field `key` among the answers; none when it holds none. */
export function citedOf(answers: FieldAnswer[], key: string): CitedValue | null {
	return citedValue(answers.find((a) => a.key === key)?.value);
}

/** An entry's own field as text: a list's items between semicolons, as the engine reads them
 * back when the field is typed anew. */
export function asText(value: FieldValue | undefined): string {
	if (typeof value === 'string') return value;
	if (Array.isArray(value))
		return value.filter((v): v is string => typeof v === 'string').join('; ');
	return '';
}

/** One entry's own field as a card shows it. */
export interface CardNote {
	key: string;
	label: string;
	value: string;
}

/** One entry as a card: its title (the first of two fields or more, a theme's name), its other
 * fields, and its quotations in the order the engine kept them. An entry of one field (why a
 * passage matters) has no title: its quotations lead, and the field follows them. */
export interface Card {
	title: string | null;
	notes: CardNote[];
	quotes: CitedEvidence[];
	/** Whether the notes come before the quotations (a theme's description) or after them. */
	notesFirst: boolean;
}

export function cardOf(entry: CitedEntry, fields: Field[]): Card {
	const notes = fields.map((f) => ({
		key: f.key,
		label: f.label,
		value: asText(entry.answers.find((a) => a.key === f.key)?.value).trim()
	}));
	const titled = fields.length >= 2;
	return {
		title: titled ? notes[0].value || null : null,
		notes: (titled ? notes.slice(1) : notes).filter((n) => n.value !== ''),
		quotes: entry.evidence,
		notesFirst: titled
	};
}

export function cardsOf(value: CitedValue | null, fields: Field[]): Card[] {
	return (value?.entries ?? []).map((e) => cardOf(e, fields));
}

/** Who said a quotation and when, as the card writes it: `Mariam · 00:04:12`; empty when
 * neither is known. */
export function saidBy(e: CitedEvidence): string {
	return [e.speaker?.trim() || null, e.start_ms === null ? null : clock(e.start_ms)]
		.filter((p): p is string => p !== null)
		.join(' · ');
}

/** The first words of a quotation, to name it on a button. */
export function quoteStart(quote: string, words = 6): string {
	const all = quote.trim().split(/\s+/);
	return all.length > words ? `${all.slice(0, words).join(' ')}…` : all.join(' ');
}

/** An entry's own fields as they are typed anew, by key. */
export function entryDraft(entry: CitedEntry, fields: Field[]): Record<string, string> {
	return Object.fromEntries(
		fields.map((f) => [f.key, asText(entry.answers.find((a) => a.key === f.key)?.value)])
	);
}

/** The fields typed differently from how they stood, in the fields' order. */
export function draftChanges(
	draft: Record<string, string>,
	before: Record<string, string>
): { key: string; value: string }[] {
	return Object.keys(before)
		.filter((key) => (draft[key] ?? '').trim() !== (before[key] ?? '').trim())
		.map((key) => ({ key, value: (draft[key] ?? '').trim() }));
}

/**
 * Where `chars` Unicode characters into `text` fall in JavaScript's UTF-16 units: the engine
 * counts a quotation's place in characters (an emoji or a letter beyond the first plane is one),
 * the browser in units (two for those).
 */
export function utf16At(text: string, chars: number): number {
	let at = 0;
	for (let n = 0; n < chars && at < text.length; n++) {
		const code = text.codePointAt(at) ?? 0;
		at += code > 0xffff ? 2 : 1;
	}
	return at;
}

/** A quotation's place in `text`, in UTF-16 units, from its anchor in characters. */
export function utf16Span(text: string, anchor: Anchor): { start: number; end: number } {
	const start = utf16At(text, anchor.start);
	return { start, end: start + utf16At(text.slice(start), anchor.end - anchor.start) };
}
