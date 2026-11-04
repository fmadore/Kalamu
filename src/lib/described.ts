// A source's description as the researcher gives it: its parts in the order the app shows them,
// and a draft compared with what is kept. Inheritance from the project is decided in Rust
// (`kalamu-projects`); here a draft is only edited. Pure, so it is tested without the app.

import type { DescriptionField } from './bindings/DescriptionField.js';
import lists from './coreLists.json';

/** Every part of a description, as `DescriptionField::ALL` orders them (written by `cargo
 * test`). */
export const FIELDS = lists.description_fields as DescriptionField[];

/** Parts written over several lines. */
export const LONG: DescriptionField[] = ['citation', 'rights'];

export type Draft = Partial<Record<DescriptionField, string>>;

/** A draft as it would be kept: its values trimmed, the empty ones left out. */
export function kept(draft: Draft): Draft {
	const out: Draft = {};
	for (const field of FIELDS) {
		const value = draft[field]?.trim();
		if (value) out[field] = value;
	}
	return out;
}

/** Whether a draft would change what is kept. */
export function changed(draft: Draft, stored: Draft): boolean {
	const a = kept(draft);
	const b = kept(stored);
	return FIELDS.some((f) => (a[f] ?? '') !== (b[f] ?? ''));
}
