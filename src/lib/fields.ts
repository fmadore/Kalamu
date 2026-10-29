// Answers as the review's editors hold them, for an analysis and for the entries of a table of
// contents alike: one text per field, a list one item per line, the choices ticked.

import type { Field } from './bindings/Field.js';
import type { FieldAnswer } from './bindings/FieldAnswer.js';

/** The answers being corrected, by field key. */
export type Draft = Record<string, string | string[]>;

export function toDraft(fields: Field[], answers: FieldAnswer[]): Draft {
	const draft: Draft = {};
	for (const f of fields) {
		const value = answers.find((a) => a.key === f.key)?.value ?? '';
		const items = Array.isArray(value)
			? (value as unknown[]).filter((v): v is string => typeof v === 'string')
			: value
				? [value]
				: [];
		draft[f.key] =
			f.kind.type === 'list'
				? items.join('\n')
				: f.kind.type === 'choices'
					? items
					: (items[0] ?? '');
	}
	return draft;
}

/** The draft as answers again, each keeping the reason the model gave in `before`. */
export function fromDraft(
	fields: Field[],
	draft: Draft,
	before: FieldAnswer[] = []
): FieldAnswer[] {
	return fields.map((f) => {
		const raw = draft[f.key] ?? '';
		const value =
			f.kind.type === 'list'
				? String(raw)
						.split('\n')
						.map((l) => l.trim())
						.filter(Boolean)
				: f.kind.type === 'choices'
					? Array.isArray(raw)
						? raw
						: [raw].filter(Boolean)
					: String(raw).trim();
		const reason = before.find((a) => a.key === f.key)?.reason;
		return reason ? { key: f.key, value, reason } : { key: f.key, value };
	});
}
