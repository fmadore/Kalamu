// New job's suggestion for a long document (#215): try the model on a few pages spread through
// it, alone or beside a second model, then run the rest from the sample's page. Which pages are
// read is the engine's (`PageSample` in the core); this says only when the suggestion is shown.

import type { SourceFile } from './bindings/SourceFile.js';
import type { Task } from './bindings/Task.js';

/** Pages a document must have beyond this for the suggestion to show. */
export const SAMPLE_OVER = 15;
/** The pages the suggestion tries. */
export const SAMPLE_PAGES = 3;

/** The pages of the longest document, as New job counts them. */
export function longest(sources: SourceFile[]): number {
	return Math.max(0, ...sources.map((s) => s.page_count ?? 0));
}

/** Whether New job suggests trying a few pages first: a text extraction of a document of more
 * than `SAMPLE_OVER` pages (a PDF, or images read as one document), for a task that reads page
 * by page. Nothing changes for a short document. */
export function offersSample(task: Task, paged: boolean, sources: SourceFile[]): boolean {
	return task === 'text_extraction' && paged && longest(sources) > SAMPLE_OVER;
}
