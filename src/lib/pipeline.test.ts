import { describe, expect, it } from 'vitest';
import type { InspectedFile } from './bindings/InspectedFile.js';
import { source } from './dev/mockJobs.js';
import { en } from './i18n/en.js';
import {
	barsOf,
	BARS,
	cutsOf,
	diagramSources,
	pageCount,
	pieceOf,
	row,
	stagesFor,
	tableRows,
	tocRows,
	videoOf,
	W,
	type Diagram
} from './pipeline.js';

const file = (over: Partial<typeof source> = {}, pieces: number | null = null): InspectedFile => ({
	path: source.path,
	source: { ...source, ...over },
	warnings: [],
	error: null,
	pieces
});
const diagram = (over: Partial<Diagram>): Diagram => ({
	mode: 'text_extraction',
	files: [],
	fields: [],
	models: [],
	mergeModels: [],
	searchable: [],
	...over
});
const arts = (d: Diagram) => stagesFor(d, en).map((s) => s.art);

describe('New job’s diagram', () => {
	it('joins the pages of a document of images into one text for the document', () => {
		const joined = (files: InspectedFile[]) =>
			stagesFor(diagram({ files }), en).find((s) => s.art === 'joined')?.caption;
		expect(joined([file()])).toBe(en.pipeline.extractJoin);
		const page = { id: 'p', path: 'a.jpg', file_name: 'a.jpg', sha256: 'x', size_bytes: 1 };
		const pages = [{ ...page, frames: 1 }];
		const doc: InspectedFile = { ...file(), source: { ...source, kind: 'image', pages } };
		expect(joined([doc])).toBe(en.pipeline.extractJoinDocument);
	});

	it('reads a scan page by page, and adds a searchable PDF when a model places the text', () => {
		expect(arts(diagram({ models: ['Flash'] }))).toEqual(['stack', 'pages', 'read', 'joined']);
		const placed = stagesFor(diagram({ models: ['Flash'], searchable: ['Flash'] }), en);
		expect(placed.map((s) => s.art)).toEqual(['stack', 'pages', 'place', 'joined', 'searchable']);
		expect(placed[2].note).toBe(en.pipeline.extractPlace);
		expect(placed[4].also).toBe(true);
	});

	it('names the models that place the text when not all of them do', () => {
		const some = stagesFor(diagram({ models: ['Flash', 'OCR'], searchable: ['OCR'] }), en);
		expect(some[2].note).toBe(en.pipeline.extractPlaces('OCR'));
		expect(some[2].caption).toBe(en.pipeline.extractRead(en.pipeline.theModel));
	});

	it('names the researcher’s file and how many more there are', () => {
		const stages = stagesFor(
			diagram({ files: [file({ page_count: 12 }), file({ file_name: 'b.pdf' })] }),
			en
		);
		expect(stages[0].detail).toEqual([source.file_name, en.newJob.pages(12)]);
		expect(stages[3].note).toBe(en.pipeline.moreFiles(1));
	});

	it('draws every task', () => {
		expect(arts(diagram({ mode: 'analysis' }))).toEqual(['whole', 'fields', 'table']);
		expect(arts(diagram({ mode: 'audio_transcription' }))).toEqual([
			'wave',
			'cut',
			'transcribe',
			'transcript'
		]);
		expect(arts(diagram({ mode: 'contents' }))).toEqual(['stack', 'entries', 'combine', 'toc']);
		expect(arts(diagram({ mode: 'text_correction' }))).toEqual([
			'ocr',
			'pages',
			'correct',
			'compare'
		]);
		expect(arts(diagram({ mode: 'translation' }))).toEqual([
			'text',
			'pages',
			'translate',
			'beside'
		]);
		expect(arts(diagram({ mode: 'youtube' }))).toEqual(['link', 'send', 'watch', 'transcript']);
	});

	it('counts a recording’s pieces and a scan’s pages, spreads twice', () => {
		const heard = stagesFor(diagram({ mode: 'audio_transcription', files: [file({}, 4)] }), en);
		expect(heard[1].caption).toBe(en.pipeline.audioCut(4));
		expect(heard[3].note).toBe(en.pipeline.audioSpeakers);
		const [spread] = diagramSources([file({ page_count: 5, split_spreads: true })]);
		expect(pageCount(spread)).toBe(10);
		expect(pageCount(undefined)).toBe(3);
	});

	it('lays its drawings out on the grid', () => {
		const xs = row(3, 22, 8);
		expect(xs).toEqual([29, 59, 89]);
		expect(xs[2] + 22 + xs[0]).toBe(W);
		const cuts = cutsOf(3);
		expect(cuts).toEqual([11, 21]);
		const bars = barsOf(cuts);
		expect(bars).toHaveLength(BARS);
		expect([bars[10], bars[11], bars[20], bars[21]]).toEqual([2, 2, 2, 2]);
		expect([pieceOf(cuts, 0), pieceOf(cuts, 11), pieceOf(cuts, 31)]).toEqual([0, 1, 2]);
	});

	it('stands in for what the researcher has not added yet', () => {
		expect(tableRows([], en)).toEqual(['Document 1', 'Document 2', 'Document 3']);
		expect(tocRows(2, en).map((r) => r.pages)).toEqual(
			['1', '2', '3'].slice(0, en.pipeline.exampleEntries.length)
		);
		expect(videoOf(undefined)).toBe('youtube.com/watch?v=…');
		const [video] = diagramSources([file({ path: 'https://www.youtube.com/watch?v=x' })]);
		expect(videoOf(video)).toBe('youtube.com/watch?v=x');
	});
});
