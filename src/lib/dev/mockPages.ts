// The sample data's document of images (#52): a folder of photographed pages, and the engine's
// document made of them, roughly as the engine makes it (crates/core/src/ordered.rs):
// natural order of the names, the number each name carries, gaps and repeats.

import type { ImageDocument } from '../bindings/ImageDocument.js';
import type { PageFile } from '../bindings/PageFile.js';
import type { PageNote } from '../bindings/PageNote.js';
import type { SourceFile } from '../bindings/SourceFile.js';

/** The folder Add folder picks in the sample data, and the photographs it holds, in the order
 * the system lists them. */
export const PHOTO_FOLDER = 'C:\\Archives\\Box 3';
export const PHOTOS = ['IMG_10.jpg', 'IMG_2.jpg', 'IMG_1.jpg', 'IMG_3.jpg'].map(
	(name) => `${PHOTO_FOLDER}\\${name}`
);

function numberIn(name: string): number | null {
	const stem = name.replace(/\.[^.]*$/, '');
	const runs = stem.match(/[0-9]+/g);
	return runs ? Number(runs[runs.length - 1]) : null;
}

/** The images made one document, as `image_document` answers. */
export function mockImageDocument(name: string, sources: SourceFile[]): ImageDocument {
	const pages: PageFile[] = sources
		.map((s) => ({
			id: `page-${s.sha256.slice(-12)}`,
			path: s.path,
			file_name: s.file_name,
			sha256: s.sha256,
			size_bytes: s.size_bytes,
			frames: 1
		}))
		.sort((a, b) =>
			a.file_name.localeCompare(b.file_name, 'en', { numeric: true, sensitivity: 'base' })
		);
	const numbers = pages.map((p) => numberIn(p.file_name));
	const distinct = [...new Set(numbers.filter((n): n is number => n !== null))].sort(
		(a, b) => a - b
	);
	const notes: PageNote[] = pages.map((p, i) => {
		const n = numbers[i];
		const at = n === null ? -1 : distinct.indexOf(n);
		const before = at > 0 ? distinct[at - 1] : null;
		const gap = n !== null && before !== null && n > before + 1;
		return {
			page_id: p.id,
			number: n,
			duplicate: n !== null && numbers.filter((x) => x === n).length > 1,
			missing_from: gap && before !== null ? before + 1 : null,
			missing_to: gap && n !== null ? n - 1 : null
		};
	});
	const folder = sources[0]?.path.split('\\').slice(0, -1).join('\\') ?? '';
	const docName = name.trim() || folder.split('\\').pop() || 'document';
	return {
		source: {
			...sources[0],
			id: '00000000-0000-0000-0000-0000000005d0',
			path: `${folder}\\${docName}`,
			file_name: docName,
			sha256: pages.map((p) => p.sha256.slice(0, 4)).join(''),
			size_bytes: pages.reduce((sum, p) => sum + p.size_bytes, 0),
			kind: 'image',
			page_count: pages.length,
			pages
		},
		notes,
		warnings: []
	};
}
