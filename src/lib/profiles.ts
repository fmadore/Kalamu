// Export profiles as the dialog edits them: each column's value as one key a `<select>` can
// hold, and columns moved, added and removed. What a profile may be is checked in Rust
// (`check_export_profile`); here a draft is only edited. Pure, so it is tested without the app.

import type { ExportProfile } from './bindings/ExportProfile.js';
import type { ProfileColumn } from './bindings/ProfileColumn.js';
import type { ProfileValue } from './bindings/ProfileValue.js';
import { FIELDS } from './described.js';

/** The values taken from the document itself, in the order the dialog offers them. */
export const DOCUMENT_VALUES = [
	'document_id',
	'title',
	'file_name',
	'path',
	'project',
	'collections',
	'tags'
] as const;

export type DocumentValue = (typeof DOCUMENT_VALUES)[number];

/** `title`, `part:shelfmark`, `from:shelfmark` or `fixed`. */
export function valueKey(value: ProfileValue): string {
	return value.kind === 'part' || value.kind === 'from'
		? `${value.kind}:${value.field}`
		: value.kind;
}

/** The value a key stands for; a fixed text keeps the text it had. */
export function keyValue(key: string, previous?: ProfileValue): ProfileValue {
	const [kind, field] = key.split(':');
	if ((kind === 'part' || kind === 'from') && FIELDS.includes(field as never)) {
		return { kind, field: field as (typeof FIELDS)[number] };
	}
	if (kind === 'fixed') return { kind, text: previous?.kind === 'fixed' ? previous.text : '' };
	if ((DOCUMENT_VALUES as readonly string[]).includes(kind)) return { kind } as ProfileValue;
	return { kind: 'title' };
}

/** A new profile, its columns from a starter. */
export function draftProfile(name: string, columns: ProfileColumn[]): ExportProfile {
	return {
		id: '',
		name,
		columns,
		list_separator: '; ',
		missing: '',
		join_lines: false,
		updated_at: 0
	};
}

/** The columns with the one at `from` moved by `by` places, kept within the list. */
export function moved(columns: ProfileColumn[], from: number, by: number): ProfileColumn[] {
	const to = Math.max(0, Math.min(columns.length - 1, from + by));
	if (to === from) return columns;
	const out = [...columns];
	const [column] = out.splice(from, 1);
	out.splice(to, 0, column);
	return out;
}

/** Whether a profile has a fixed text that would leave with it when shared. */
export function hasFixedTexts(profile: ExportProfile): boolean {
	return profile.columns.some((c) => c.value.kind === 'fixed' && c.value.text.trim() !== '');
}
