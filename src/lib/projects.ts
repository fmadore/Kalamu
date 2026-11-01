// The project's document table: which documents a collection and a search show, in which
// order, and the paths a selection hands to New job. Pure, so it is tested without the app.

import type { DocumentRow } from './bindings/DocumentRow.js';
import type { Task } from './bindings/Task.js';
import { folded } from './search.js';

export type DocumentSort = 'title' | 'added' | 'run';

/** The documents of a collection (all with `undefined`) whose title, file name or folder
 *  holds the words, capitals and accents ignored. */
export function shownDocuments(
	rows: DocumentRow[],
	collectionId: string | undefined,
	query: string
): DocumentRow[] {
	const words = folded(query).split(/\s+/).filter(Boolean);
	return rows.filter((row) => {
		if (collectionId && !row.collections.includes(collectionId)) return false;
		if (words.length === 0) return true;
		const haystack = folded(
			`${row.document.title} ${row.document.source.file_name} ${row.document.source.path}`
		);
		return words.every((w) => haystack.includes(w));
	});
}

/** Sorted by title (as a reader sorts: `page 2` before `page 10`), newest added first, or
 *  most recently read first, with documents never read last. */
export function sortDocuments(rows: DocumentRow[], by: DocumentSort): DocumentRow[] {
	const title = (a: DocumentRow, b: DocumentRow) =>
		a.document.title.localeCompare(b.document.title, undefined, {
			numeric: true,
			sensitivity: 'base'
		});
	const sorted = [...rows];
	switch (by) {
		case 'title':
			return sorted.sort(title);
		case 'added':
			return sorted.sort((a, b) => b.document.created_at - a.document.created_at || title(a, b));
		case 'run':
			return sorted.sort((a, b) => (b.last_run ?? -1) - (a.last_run ?? -1) || title(a, b));
	}
}

/** The task New job opens on for a selection: transcription when every document is a
 *  recording, otherwise text extraction (the researcher can change it there). */
export function taskFor(rows: DocumentRow[]): Task {
	const recordings = rows.every((r) => ['audio', 'video'].includes(r.document.source.kind));
	return rows.length > 0 && recordings ? 'audio_transcription' : 'text_extraction';
}

/** The files a selection hands to New job: those still on disk, YouTube links aside. */
export function selectionPaths(rows: DocumentRow[]): string[] {
	return rows
		.filter((r) => !r.missing && r.document.source.kind !== 'youtube')
		.map((r) => r.document.source.path);
}
