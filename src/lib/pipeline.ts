// What New job's diagram shows: the steps of a task's work, with the researcher's own files once
// they are added, and the measures its drawings are laid out by. Pure, so it is tested without
// the app; PipelineDiagram.svelte draws it.

import type { InspectedFile } from './bindings/InspectedFile.js';
import type { SourceFile } from './bindings/SourceFile.js';
import { listNames, type Messages } from './i18n/index.js';
import { duration, type Mode } from './newJob.js';

/** The drawing a step is shown by. */
export type Art =
	| 'stack'
	| 'pages'
	| 'read'
	| 'place'
	| 'joined'
	| 'searchable'
	| 'whole'
	| 'fields'
	| 'table'
	| 'wave'
	| 'cut'
	| 'transcribe'
	| 'transcript'
	| 'link'
	| 'send'
	| 'watch'
	| 'entries'
	| 'combine'
	| 'toc'
	| 'ocr'
	| 'correct'
	| 'compare';

/** `detail` names the user's file, in mono; `note` adds what the step does with it; `also`
 * marks a second result beside the one before it rather than a step after it. */
export type Stage = { art: Art; caption: string; detail?: string[]; note?: string; also?: boolean };

/** A file added to the job, with how many pieces a recording is sent in. */
export type DiagramSource = SourceFile & { pieces: number | null };

/** What the diagram is drawn for: the task, the files and the choices made. */
export interface Diagram {
	mode: Mode;
	files: InspectedFile[];
	/** The chosen prompt's fields, for an analysis or a table of contents' entries. */
	fields: string[];
	/** Names of the chosen models. */
	models: string[];
	/** Names of the models that combine a table of contents' pages. */
	mergeModels: string[];
	/** Names of the chosen models that report where text sits, so a searchable PDF can be
	 * written: none when they return text only. */
	searchable: string[];
}

/** The files read, each with its pieces. */
export function diagramSources(files: InspectedFile[]): DiagramSource[] {
	return files.flatMap((f) => (f.source ? [{ ...f.source, pieces: f.pieces }] : []));
}

/** Pages of the first file as pieces: a spread split in two counts twice; an image is one.
 * Three before any file is added. */
export function pageCount(first: DiagramSource | undefined): number {
	return first ? (first.page_count ?? 1) * (first.split_spreads ? 2 : 1) : 3;
}

/** The pieces a recording is sent in; three before any is added. */
export function pieceCount(first: DiagramSource | undefined): number {
	return first?.pieces ?? 3;
}

/** The fields shown: the prompt's, or examples until one with fields is chosen. */
export function fieldNamesOf(fields: string[], m: Messages): string[] {
	return fields.length > 0 ? fields : m.pipeline.exampleFields;
}

/** The steps of the task's work, in order, with the researcher's own file named. */
export function stagesFor(d: Diagram, m: Messages): Stage[] {
	const sources = diagramSources(d.files);
	const first = sources[0];
	const more = sources.length > 1 ? m.pipeline.moreFiles(sources.length - 1) : undefined;
	const model = d.models.length === 1 ? d.models[0] : m.pipeline.theModel;
	const merger = d.mergeModels.length === 1 ? d.mergeModels[0] : m.pipeline.theModel;
	/** Whether a chosen model reports where text sits; the others give text only. */
	const placed = d.searchable.length > 0;
	/** Under the reading: that it notes where text sits (and which model, when not all do), or
	 * that it gives text only. */
	const placeNote =
		placed && d.searchable.length === d.models.length
			? m.pipeline.extractPlace
			: placed
				? m.pipeline.extractPlaces(listNames(d.searchable))
				: d.models.length > 0
					? m.pipeline.extractTextOnly
					: undefined;
	/** A file's name, then its size as the file list shows it. */
	const fileDetail = first
		? [
				first.file_name,
				first.page_count ? m.newJob.pages(first.page_count) : '',
				first.duration_ms ? duration(Number(first.duration_ms), m) : ''
			].filter(Boolean)
		: undefined;
	const pieces = pieceCount(first);

	switch (d.mode) {
		case 'text_extraction':
			return [
				{ art: 'stack', caption: m.pipeline.extractSource, detail: fileDetail },
				{
					art: 'pages',
					caption: m.pipeline.extractSplit,
					note: first?.split_spreads ? m.pipeline.extractSpreads : undefined
				},
				{ art: placed ? 'place' : 'read', caption: m.pipeline.extractRead(model), note: placeNote },
				{ art: 'joined', caption: m.pipeline.extractJoin, note: more },
				...(placed
					? [
							{
								art: 'searchable' as const,
								caption: m.pipeline.extractSearchable,
								note: m.pipeline.extractPdfOnly,
								also: true
							}
						]
					: [])
			];
		case 'analysis':
			return [
				{ art: 'whole', caption: m.pipeline.analysisWhole, detail: fileDetail },
				{ art: 'fields', caption: m.pipeline.analysisFill(model) },
				{ art: 'table', caption: m.pipeline.analysisTable }
			];
		case 'audio_transcription':
			return [
				{ art: 'wave', caption: m.pipeline.audioSource, detail: fileDetail },
				{
					art: 'cut',
					caption: first?.pieces ? m.pipeline.audioCut(pieces) : m.pipeline.audioCutExample
				},
				{ art: 'transcribe', caption: m.pipeline.audioRead(model) },
				{
					art: 'transcript',
					caption: m.pipeline.audioJoin,
					note: pieces > 1 ? m.pipeline.audioSpeakers : more
				}
			];
		case 'contents':
			return [
				{ art: 'stack', caption: m.pipeline.contentsSource, detail: fileDetail },
				{
					art: 'entries',
					caption: m.pipeline.contentsRead(model),
					note: m.pipeline.contentsContinues
				},
				{ art: 'combine', caption: m.pipeline.contentsMerge(merger) },
				{ art: 'toc', caption: m.pipeline.contentsTable, note: more }
			];
		case 'text_correction':
			return [
				{ art: 'ocr', caption: m.pipeline.correctionSource, detail: fileDetail },
				{ art: 'pages', caption: m.pipeline.correctionSplit },
				{ art: 'correct', caption: m.pipeline.correctionRead(model) },
				{ art: 'compare', caption: m.pipeline.correctionCompare, note: more }
			];
		case 'youtube':
			return [
				{
					art: 'link',
					caption: m.pipeline.youtubeSource,
					detail: first?.title ? [first.title] : undefined
				},
				{ art: 'send', caption: m.pipeline.youtubeSend },
				{ art: 'watch', caption: m.pipeline.youtubeRead(model) },
				{ art: 'transcript', caption: m.pipeline.youtubeJoin, note: more }
			];
	}
}

// The drawings' measures, on a 140 × 72 grid.
export const W = 140;
/** A waveform's bars, how far apart, and the gap where it is cut. */
export const BARS = 32;
export const BAR_STEP = 3.5;
export const CUT_GAP = 6;
/** Where a page's text was misread, as [line, x, width] on a sheet's lines. */
export const FLAWS = [
	[1, 14, 8],
	[3, 8, 10],
	[5, 18, 7]
];

/** Left edges of `n` boxes `w` wide, `gap` apart, centred. */
export function row(n: number, w: number, gap: number): number[] {
	return Array.from({ length: n }, (_, i) => (W - (n * w + (n - 1) * gap)) / 2 + i * (w + gap));
}

/** The bars where a recording shown in `pieces` pieces is cut. */
export function cutsOf(pieces: number): number[] {
	return Array.from({ length: pieces - 1 }, (_, i) => Math.round(((i + 1) * BARS) / pieces));
}

/** The waveform's bars' heights, quiet where the recording is cut. */
export function barsOf(cuts: number[]): number[] {
	return Array.from({ length: BARS }, (_, k) =>
		cuts.some((c) => k === c || k === c - 1)
			? 2
			: 4 + Math.round(24 * Math.abs(Math.sin(k * 1.7) * Math.cos(k * 0.45)))
	);
}

/** Which piece bar `k` falls in. */
export function pieceOf(cuts: number[], k: number): number {
	return cuts.filter((c) => k >= c).length;
}

/** Where the waveform starts, centred with its cuts. */
export function waveLeft(pieces: number): number {
	return (W - (BARS * BAR_STEP + (pieces - 1) * CUT_GAP)) / 2;
}

/** The video's link without its scheme, or a stand-in until one is added. */
export function videoOf(first: DiagramSource | undefined): string {
	return first?.path.replace(/^https?:\/\/(www\.)?/, '') ?? 'youtube.com/watch?v=…';
}

/** A table of contents' rows: stand-in entries, with pages that run over the first pages. */
export function tocRows(pages: number, m: Messages): { title: string; pages: string }[] {
	return m.pipeline.exampleEntries.map((title, r) => ({
		title,
		pages: [pages > 2 ? '1, 3' : '1', '2', String(Math.max(pages, 3))][r] ?? ''
	}));
}

/** The table's rows: the user's documents, or stand-ins until there are some. */
export function tableRows(sources: DiagramSource[], m: Messages): string[] {
	return sources.length > 0
		? sources.slice(0, sources.length > 3 ? 2 : 3).map((s) => s.file_name)
		: [1, 2, 3].map((n) => m.pipeline.document(n));
}
