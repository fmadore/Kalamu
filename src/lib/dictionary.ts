// A project's dictionary as the review shows it: which rows a field, a search and a filter
// show, what a selection stands for, and the label a merge proposes. Pure, so it is tested
// without the app.

import type { DictionaryRow } from './bindings/DictionaryRow.js';
import type { Item } from './bindings/Item.js';
import { folded } from './search.js';

export type RowFilter = 'all' | 'to_review' | 'similar' | 'reviewed' | 'unseen';

/** The rows of a field that the search and filter let through; the search reads labels and
 *  every form, capitals and accents ignored. */
export function shownRows(
	rows: DictionaryRow[],
	namespace: string,
	query: string,
	filter: RowFilter
): DictionaryRow[] {
	const words = folded(query).split(/\s+/).filter(Boolean);
	return rows.filter((row) => {
		if (row.namespace !== namespace) return false;
		const passes =
			filter === 'all' ||
			(filter === 'to_review' && !row.reviewed) ||
			(filter === 'reviewed' && row.reviewed) ||
			(filter === 'similar' && row.similar.length > 0) ||
			(filter === 'unseen' && row.unseen);
		if (!passes) return false;
		if (words.length === 0) return true;
		const haystack = folded(`${row.label} ${row.forms.map((f) => f.form).join(' ')}`);
		return words.every((w) => haystack.includes(w));
	});
}

/** What a row stands for in a decision: its entry, or its one form. */
export function itemOf(row: DictionaryRow): Item {
	return row.entry !== null
		? { kind: 'entry', id: row.entry }
		: { kind: 'form', form: row.forms[0]?.form ?? row.label };
}

/** The label a merge proposes: a reviewed entry's own, otherwise the label named by the most
 *  documents, the first in reading order on a tie. */
export function proposedLabel(rows: DictionaryRow[]): string {
	const reviewed = rows.find((r) => r.entry !== null && r.reviewed);
	if (reviewed) return reviewed.label;
	const best = [...rows].sort(
		(a, b) => b.documents - a.documents || a.label.localeCompare(b.label)
	)[0];
	return best?.label ?? '';
}

/** Every label and form among the rows, each once: the choices offered for a merged label. */
export function labelChoices(rows: DictionaryRow[]): string[] {
	return [...new Set(rows.flatMap((r) => [r.label, ...r.forms.map((f) => f.form)]))];
}
