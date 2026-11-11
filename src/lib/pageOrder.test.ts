import { describe, expect, it } from 'vitest';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { TaskRules } from './bindings/TaskRules.js';
import rulesJson from './dev/taskRules.json';
import { mockImageDocument } from './dev/mockPages.js';
import { en, fr } from './i18n/index.js';
import {
	imageSources,
	jobSources,
	movePage,
	noteTexts,
	offersOneDocument,
	sameImages
} from './pageOrder.js';

const rules = rulesJson as TaskRules[];
const rule = (task: string) => rules.find((r) => r.task === task)!;

function file(name: string, kind: SourceFile['kind'] = 'image'): InspectedFile {
	const path = `C:\\Archives\\Box 3\\${name}`;
	return {
		path,
		source: {
			id: `id-${name}`,
			path,
			file_name: name,
			sha256: name.padStart(64, '0'),
			size_bytes: 10,
			kind,
			page_count: 1,
			duration_ms: null,
			split_spreads: false,
			even_out_volume: false
		},
		warnings: [],
		error: null,
		pieces: null
	};
}

const photos = [file('p10.jpg'), file('p2.jpg'), file('p1.jpg')];

describe('images read as one document', () => {
	it('is offered for an extraction or a table of contents of two images or more', () => {
		const images = imageSources(photos);
		expect(offersOneDocument(rule('text_extraction'), images)).toBe(true);
		expect(offersOneDocument(rule('contents'), images)).toBe(true);
		expect(offersOneDocument(rule('analysis'), images)).toBe(false);
		expect(offersOneDocument(rule('text_extraction'), images.slice(0, 1))).toBe(false);
		// A PDF is not an image to put among the pages.
		expect(imageSources([file('a.pdf', 'pdf'), ...photos])).toHaveLength(3);
	});

	it('moves a page and keeps every page', () => {
		const doc = mockImageDocument('', imageSources(photos));
		const names = (pages = doc.source.pages ?? []) => pages.map((p) => p.file_name);
		expect(names()).toEqual(['p1.jpg', 'p2.jpg', 'p10.jpg']);
		expect(names(movePage(doc.source.pages ?? [], 2, 0))).toEqual(['p10.jpg', 'p1.jpg', 'p2.jpg']);
		expect(names(movePage(doc.source.pages ?? [], 0, 1))).toEqual(['p2.jpg', 'p1.jpg', 'p10.jpg']);
		// Out of range: nothing moves.
		expect(names(movePage(doc.source.pages ?? [], 0, 3))).toEqual(names());
		expect(names(movePage(doc.source.pages ?? [], -1, 0))).toEqual(names());
	});

	it('reads the document in place of its images, named as the researcher typed', () => {
		const files = [file('cover.pdf', 'pdf'), ...photos];
		const doc = mockImageDocument('', imageSources(files));
		expect(doc.source.file_name).toBe('Box 3');
		const one = jobSources(files, 'one', doc, '  Lettres 1923 ');
		expect(one.map((s) => s.file_name)).toEqual(['cover.pdf', 'Lettres 1923']);
		expect(one[1].pages).toHaveLength(3);
		// Each its own, as before.
		expect(jobSources(files, 'separate', doc, 'x')).toHaveLength(4);
		// Images added since the document was made: each its own until it is made again.
		const more = [...files, file('p11.jpg')];
		expect(sameImages(doc, imageSources(more))).toBe(false);
		expect(jobSources(more, 'one', doc, 'x')).toHaveLength(5);
		// A blank name keeps the engine's.
		expect(jobSources(files, 'one', doc, ' ')[1].file_name).toBe('Box 3');
	});

	it('says which numbers are missing or repeated, in both languages', () => {
		const doc = mockImageDocument('', imageSources([...photos, file('p02 copie.jpg')]));
		const at = (name: string) => {
			const page = doc.source.pages?.find((p) => p.file_name === name);
			return doc.notes.find((n) => n.page_id === page?.id);
		};
		expect(noteTexts(at('p10.jpg'), en)).toEqual(['No files numbered 3 to 9 before this one']);
		expect(noteTexts(at('p2.jpg'), en)).toEqual(['Another file is also numbered 2']);
		expect(noteTexts(at('p10.jpg'), fr)).toEqual([
			'Aucun fichier numéroté de 3 à 9 avant celui-ci'
		]);
		expect(noteTexts(at('p1.jpg'), en)).toEqual([]);
		expect(noteTexts(undefined, en)).toEqual([]);
	});
});
