import { describe, expect, it } from 'vitest';
import type { DocumentRow } from './bindings/DocumentRow.js';
import type { InputKind } from './bindings/InputKind.js';
import { selectionPaths, shownDocuments, sortDocuments, tagsOf, taskFor } from './projects.js';

function row(
	title: string,
	options: {
		path?: string;
		kind?: InputKind;
		collections?: string[];
		added?: number;
		run?: number | null;
		missing?: boolean;
		tags?: string[];
	} = {}
): DocumentRow {
	const path = options.path ?? `C:\\Archives\\${title}.pdf`;
	return {
		document: {
			id: title,
			project_id: 'p',
			title,
			source: {
				id: `s-${title}`,
				path,
				file_name: path.split('\\').pop() ?? path,
				sha256: title,
				size_bytes: 1,
				kind: options.kind ?? 'pdf',
				page_count: 1,
				duration_ms: null,
				split_spreads: false,
				even_out_volume: false
			},
			created_at: options.added ?? 0,
			tags: options.tags ?? []
		},
		collections: options.collections ?? [],
		tasks: [],
		missing: options.missing ?? false,
		same_content: 0,
		last_run: options.run ?? null
	};
}

describe('the project document table', () => {
	const rows = [
		row('Lettre du Père Éric', { collections: ['letters'], added: 3, run: 10 }),
		row('page 10', { added: 1 }),
		row('page 2', { collections: ['letters'], added: 2, run: 20 }),
		row('Interview', { path: 'C:\\Fieldwork\\Ouaga\\int.mp3', kind: 'audio' })
	];

	it('shows a collection, and finds words in titles and folders without accents or capitals', () => {
		expect(shownDocuments(rows, 'letters', '').map((r) => r.document.title)).toEqual([
			'Lettre du Père Éric',
			'page 2'
		]);
		expect(shownDocuments(rows, undefined, 'pere eric').map((r) => r.document.id)).toEqual([
			'Lettre du Père Éric'
		]);
		expect(shownDocuments(rows, undefined, 'ouaga').map((r) => r.document.id)).toEqual([
			'Interview'
		]);
		expect(shownDocuments(rows, 'letters', 'interview')).toEqual([]);
	});

	it('sorts titles as a reader does, and puts documents never read last', () => {
		expect(sortDocuments(rows, 'title').map((r) => r.document.title)).toEqual([
			'Interview',
			'Lettre du Père Éric',
			'page 2',
			'page 10'
		]);
		expect(sortDocuments(rows, 'added').map((r) => r.document.created_at)).toEqual([3, 2, 1, 0]);
		expect(sortDocuments(rows, 'run').map((r) => r.document.title)).toEqual([
			'page 2',
			'Lettre du Père Éric',
			'Interview',
			'page 10'
		]);
	});

	it('narrows by tag, finds words in tags, and lists the tags of a selection', () => {
		const tagged = [
			row('Lettre', { collections: ['letters'], tags: ['Dakar', 'Pèlerinage'] }),
			row('Bulletin', { tags: ['Dakar'] }),
			row('Interview')
		];
		const titles = (list: DocumentRow[]) => list.map((r) => r.document.title);
		expect(titles(shownDocuments(tagged, undefined, '', 'Dakar'))).toEqual(['Lettre', 'Bulletin']);
		expect(titles(shownDocuments(tagged, 'letters', '', 'Dakar'))).toEqual(['Lettre']);
		expect(titles(shownDocuments(tagged, undefined, 'pelerinage'))).toEqual(['Lettre']);
		expect(tagsOf(tagged)).toEqual(['Dakar', 'Pèlerinage']);
		expect(tagsOf([tagged[2]])).toEqual([]);
	});

	it('opens New job on the right task with the files still on disk', () => {
		expect(taskFor([rows[3]])).toBe('audio_transcription');
		expect(taskFor(rows)).toBe('text_extraction');
		expect(taskFor([])).toBe('text_extraction');
		const gone = row('gone', { missing: true });
		const video = row('video', { kind: 'youtube', path: 'https://youtu.be/x' });
		expect(selectionPaths([rows[1], gone, video])).toEqual(['C:\\Archives\\page 10.pdf']);
	});
});
