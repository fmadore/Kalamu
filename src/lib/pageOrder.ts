// Images read as the pages of one document (#52): which files can be, the document's sources
// for the job, and a page moved. The order itself (natural, by the numbers in the names), the
// numbers' gaps and repeats, the document's hash and its place are the engine's: the interface
// only moves pages and shows what the engine says.

import type { ImageDocument } from './bindings/ImageDocument.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { PageFile } from './bindings/PageFile.js';
import type { PageNote } from './bindings/PageNote.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { Messages } from './i18n/index.js';

/** Each image a document of its own, or the pages of one document. */
export type Grouping = 'separate' | 'one';

/** The images among the files: inspected, read from disk, not already a document. */
export function imageSources(files: InspectedFile[]): SourceFile[] {
	return files.flatMap((f) =>
		f.source && f.source.kind === 'image' && !f.source.derived_from && !f.source.pages?.length
			? [f.source]
			: []
	);
}

/** Whether New job offers to read the images as one document: the task reads documents of
 * pages, and there are at least two images. */
export function offersOneDocument(rule: TaskRules, images: SourceFile[]): boolean {
	return rule.page_documents && images.length >= 2;
}

/** Whether the document holds exactly these images, whatever their order. */
export function sameImages(doc: ImageDocument | null, images: SourceFile[]): boolean {
	if (!doc) return false;
	const pages = doc.source.pages ?? [];
	if (pages.length !== images.length) return false;
	const paths = new Set(pages.map((p) => p.path));
	return images.every((s) => paths.has(s.path));
}

/** The pages with the one at `from` moved to `to`; the same pages when either is out of range. */
export function movePage(pages: PageFile[], from: number, to: number): PageFile[] {
	if (from === to || from < 0 || to < 0 || from >= pages.length || to >= pages.length) {
		return pages;
	}
	const next = [...pages];
	const [moved] = next.splice(from, 1);
	next.splice(to, 0, moved);
	return next;
}

/** What the job reads: the document named `name` in place of its images when they are read as
 * one, every other file as it is. Before the engine has made the document of these images, each
 * is still its own. */
export function jobSources(
	files: InspectedFile[],
	grouping: Grouping,
	doc: ImageDocument | null,
	name: string
): SourceFile[] {
	const all = files.flatMap((f) => (f.source ? [f.source] : []));
	const images = imageSources(files);
	if (grouping !== 'one' || !doc || !sameImages(doc, images)) return all;
	const inDoc = new Set(images.map((s) => s.id));
	const document: SourceFile = {
		...doc.source,
		file_name: name.trim() || doc.source.file_name
	};
	// The document stands where its first image was among the files.
	const at = all.findIndex((s) => inDoc.has(s.id));
	const others = all.filter((s) => !inDoc.has(s.id));
	others.splice(Math.max(0, at), 0, document);
	return others;
}

/** What the engine noticed in a page's number, in plain words: numbers missing just before it,
 * and another file carrying the same number. */
export function noteTexts(note: PageNote | undefined, m: Messages): string[] {
	if (!note) return [];
	const out: string[] = [];
	if (note.missing_from !== null && note.missing_to !== null) {
		out.push(m.newJob.grouping.missing(note.missing_from, note.missing_to));
	}
	if (note.duplicate && note.number !== null) {
		out.push(m.newJob.grouping.duplicate(note.number));
	}
	return out;
}
