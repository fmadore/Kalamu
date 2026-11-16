// Records (#171): an analysis whose only field is a repeating group, one row per entry, read
// page by page. The engine decides that from the fields (`kalamu_core::records`) and does the
// reading and the corrections; the interface finds the group to show its rows, and offers what
// the engine says a task read page by page offers.

import type { AnalysisSchema } from './bindings/AnalysisSchema.js';
import type { Field } from './bindings/Field.js';
import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { TaskRules } from './bindings/TaskRules.js';

/** The repeating group of a records analysis: its only field, when that is a group. */
export function groupOf(schema: AnalysisSchema | null | undefined): Field | null {
	const fields = schema?.fields ?? [];
	return fields.length === 1 && fields[0].kind.type === 'entries' ? fields[0] : null;
}

/** The group's columns, in order; none for an analysis that is not records. */
export function columnsOf(schema: AnalysisSchema | null | undefined): Field[] {
	const kind = groupOf(schema)?.kind;
	return kind?.type === 'entries' ? kind.fields : [];
}

/** A page's rows in its answers, in reading order. An empty list of rows reads back as a list
 * of strings, which holds none. */
export function rowsOf(
	schema: AnalysisSchema | null | undefined,
	answers: FieldAnswer[]
): FieldAnswer[][] {
	const group = groupOf(schema);
	const value = answers.find((a) => a.key === group?.key)?.value;
	return Array.isArray(value) && value.length > 0 && Array.isArray(value[0])
		? (value as FieldAnswer[][])
		: [];
}

/** A cell as one line of text: a list's items joined by semicolons, as typed back. */
export function cellText(row: FieldAnswer[], key: string): string {
	const value = row.find((a) => a.key === key)?.value;
	if (value === undefined) return '';
	if (typeof value === 'string') return value;
	return (value as unknown[]).filter((v) => typeof v === 'string').join('; ');
}

/** The task's rules for these set-ups' fields: when every one is records, what the engine says
 * the task offers read page by page (samples, double pages split, images as one document). */
export function readingRule(
	rule: TaskRules,
	schemas: (AnalysisSchema | null | undefined)[]
): TaskRules {
	if (!rule.by_page || schemas.length === 0 || !schemas.every((s) => groupOf(s))) return rule;
	return { ...rule, ...rule.by_page };
}
