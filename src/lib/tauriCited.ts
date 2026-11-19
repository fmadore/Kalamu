// The bridge's commands for an analysis that quotes its text (an interview's themes and
// quotations), kept beside `tauri.ts` so the one bridge does not keep growing.

import type { CitedEdit } from './bindings/CitedEdit.js';
import type { CitedSource } from './bindings/CitedSource.js';
import { invoke } from './tauri.js';

export const citedApi = {
	/** Makes one change to the entries of a field that quotes the text (an entry's field typed,
	 * an entry or one of its quotations removed), in the piece's corrected answers. */
	editCited: (
		jobId: string,
		setupId: string,
		sourceId: string,
		index: number,
		key: string,
		change: CitedEdit
	) => invoke<void>('edit_cited', { jobId, setupId, sourceId, index, key, change }),
	/** The text one source of such an analysis read, with its turns, to show each quotation in
	 * place. */
	citedSource: (jobId: string, sourceId: string) =>
		invoke<CitedSource>('cited_source', { jobId, sourceId })
};
