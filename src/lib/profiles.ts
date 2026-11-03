// Export profiles as the dialog edits them: each column's value as one key a `<select>` can
// hold, and columns moved, added and removed. What a profile may be is checked in Rust
// (`check_export_profile`); here a draft is only edited. Pure, so it is tested without the app.

import type { ExportProfile } from './bindings/ExportProfile.js';
import type { ProfileColumn } from './bindings/ProfileColumn.js';
import type { ProfileValue } from './bindings/ProfileValue.js';
import type { Task } from './bindings/Task.js';
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

/** The tasks whose result a column can read, in the order the dialog offers them. */
export const RESULT_TASKS: Task[] = [
	'text_extraction',
	'text_correction',
	'audio_transcription',
	'analysis',
	'contents'
];

/** The tasks that yield a text, which a column can take. */
export const TEXT_TASKS: Task[] = ['text_extraction', 'text_correction', 'audio_transcription'];

/** What a column can read of a task's result. */
export const RESULT_VALUES = ['status', 'model', 'text'] as const;

/**
 * `title`, `part:shelfmark`, `from:shelfmark`, `fixed`, `status:analysis`, `text:text_extraction`,
 * `answer` or `labelled`.
 */
export function valueKey(value: ProfileValue): string {
	if (value.kind === 'part' || value.kind === 'from') return `${value.kind}:${value.field}`;
	if (value.kind === 'status' || value.kind === 'model' || value.kind === 'text') {
		return `${value.kind}:${value.task}`;
	}
	return value.kind;
}

/** The field's key of an answer's column, through the dictionary or not. */
function answerKey(value?: ProfileValue): string {
	return value?.kind === 'answer' || value?.kind === 'labelled' ? value.key : '';
}

/** The value a key stands for; a fixed text or an answer keeps the text or key it had. */
export function keyValue(key: string, previous?: ProfileValue): ProfileValue {
	const [kind, field] = key.split(':');
	if ((kind === 'part' || kind === 'from') && FIELDS.includes(field as never)) {
		return { kind, field: field as (typeof FIELDS)[number] };
	}
	if ((kind === 'status' || kind === 'model') && RESULT_TASKS.includes(field as Task)) {
		return { kind, task: field as Task };
	}
	if (kind === 'text' && TEXT_TASKS.includes(field as Task)) {
		return { kind, task: field as Task };
	}
	if (kind === 'fixed') return { kind, text: previous?.kind === 'fixed' ? previous.text : '' };
	if (kind === 'answer' || kind === 'labelled') return { kind, key: answerKey(previous) };
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
