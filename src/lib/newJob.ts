// The New job form's logic, kept out of the components so it can be tested on its own. Which
// sources, models and formats suit a task is the engine's to say (`TaskRules`, from the
// `task_rules` command); the engine checks them again when the job is created. Model choice is
// in newJobModels.ts.

import type { Catalog } from './bindings/Catalog.js';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { ImageDocument } from './bindings/ImageDocument.js';
import type { InputKind } from './bindings/InputKind.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { JobSettings } from './bindings/JobSettings.js';
import type { LineBreaks } from './bindings/LineBreaks.js';
import type { OutputDestination } from './bindings/OutputDestination.js';
import type { Pace } from './bindings/Pace.js';
import type { PagesPerCall } from './bindings/PagesPerCall.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Preset } from './bindings/Preset.js';
import type { Refused } from './bindings/Refused.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { Task } from './bindings/Task.js';
import type { TaskRules } from './bindings/TaskRules.js';
import lists from './coreLists.json';
import {
	canonicalId,
	type Draft,
	IMAGES,
	modelInfo,
	type Needs,
	toRequest
} from './newJobModels.js';
import type { FromProject } from './state.svelte.js';
import type { Grouping } from './pageOrder.js';
import { baseName, pathParts } from './paths.js';
import { YOUTUBE_LINK } from './youtube.js';

/**
 * What New job offers: the tasks, and YouTube videos, which are transcribed like a recording
 * but added by their link and read only by models that take YouTube links.
 */
export type Mode = Task | 'youtube';

/**
 * In the order New job shows them: reading pages, then working on texts that exist, then
 * spoken sources.
 */
export const MODES: Mode[] = [
	'text_extraction',
	'text_correction',
	'translation',
	'analysis',
	'contents',
	'audio_transcription',
	'youtube'
];

export function taskOf(mode: Mode): Task {
	return mode === 'youtube' ? 'audio_transcription' : mode;
}

/**
 * The inputs every chosen model must read, as the engine refuses a job otherwise (media's
 * `readable`): YouTube links; images when an image is among the files, since a model that reads
 * text only fails on each of them; for an analysis, which reads a document whole, PDFs when a
 * PDF is among them. A correction or a translation reads a PDF's text layer, and other tasks
 * read a PDF's pages as text for a model that reads text only, so neither needs more.
 */
export function inputOf(mode: Mode, files: InspectedFile[] = []): Needs {
	if (mode === 'youtube') return [YOUTUBE_LINK];
	const has = (kind: string) => files.some((f) => f.source?.kind === kind);
	const needs = [
		...(has('image') ? [IMAGES] : []),
		...(mode === 'analysis' && has('pdf') ? ['application/pdf'] : [])
	];
	return needs.length > 0 ? needs : undefined;
}

/**
 * The prompt New job chooses for a task's files: a ready-made prompt made for their kind when
 * every file is of it (the image description for images, as the engine marks it), else the
 * task's first. The caller applies it only where the researcher has not chosen a prompt.
 */
export function suggestedPreset(presets: Preset[], files: InspectedFile[]): string | null {
	const kinds = files.map((f) => f.source?.kind);
	const made =
		kinds.length > 0
			? presets.find((p) => p.made_for !== undefined && kinds.every((k) => k === p.made_for))
			: undefined;
	return (made ?? presets[0])?.id ?? null;
}

/** The engine's rules for every task. */
export type Rules = TaskRules[];

/** The rules for one task; none before the engine has given them. */
export function rulesFor(rules: Rules, task: Task): TaskRules {
	return (
		rules.find((r) => r.task === task) ?? {
			task,
			sources: [],
			formats: [],
			answers_in_language: false,
			needs_language: false,
			models: [],
			merge_models: [],
			yields_text: false,
			line_breaks: false,
			samples: false,
			pages_per_call: false,
			splits_spreads: false,
			reads_text_layer: false,
			page_documents: false
		}
	);
}

/** Formats written only from a model that reports where text sits on the page, as
 * `ExportFormat::needs_layout` says (written by `cargo test`). */
export const LAYOUT_FORMATS = lists.layout_formats as ExportFormat[];

/** The extensions of each kind of file, from `InputKind::EXTENSIONS`: the table Rust reads a
 * file's kind from. */
const EXTENSIONS = lists.extensions as Partial<Record<InputKind, string[]>>;

/** The files a task reads, by extension: those of each kind of source it reads
 * (`Task::source_kinds`), so the picker offers what the engine reads and nothing else. */
export function taskExtensions(task: Task): string[] {
	const kinds = (lists.task_sources as Record<Task, InputKind[]>)[task] ?? [];
	return kinds.flatMap((kind) => EXTENSIONS[kind] ?? []);
}

/** Files of any kind some task reads, for a project's documents. */
export function documentExtensions(): string[] {
	return [...new Set((lists.tasks as Task[]).flatMap(taskExtensions))];
}

/**
 * The formats to offer: a searchable PDF and ALTO only when a chosen model reports where text
 * sits on the page (Tesseract, Mistral OCR).
 */
export function offeredFormats(catalog: Catalog, rule: TaskRules, drafts: Draft[]): ExportFormat[] {
	const layout = drafts.some((d) => modelInfo(catalog, d)?.layout === true);
	return rule.formats.filter((f) => !LAYOUT_FORMATS.includes(f) || layout);
}

/** Whether a mode reads a kind of source: YouTube links in the YouTube mode, files only in
 * the others. */
export function readsKind(rules: Rules, mode: Mode, kind: string | undefined): boolean {
	if (mode === 'youtube') return kind === 'youtube';
	return kind !== 'youtube' && rulesFor(rules, mode).sources.some((k) => k === kind);
}

/**
 * The other tasks that read the files a task set aside (an interview dropped on Extract text),
 * to offer one instead. Folders and files Kalamu reads in no task are left out; none is offered
 * when nothing set aside is a file of a kind some task reads.
 */
export function readersOf(rules: Rules, refused: Refused[], current: Mode): Mode[] {
	const kinds = [...new Set(refused.flatMap((r) => (r.kind ? [r.kind] : [])))];
	if (kinds.length === 0) return [];
	return MODES.filter(
		(m) =>
			m !== current && m !== 'youtube' && kinds.every((k) => rulesFor(rules, m).sources.includes(k))
	);
}

/** A job name from what was added: the folder's name, or the first file's. */
export function suggestName(added: string[], expanded: string[]): string {
	if (added.length === 1 && expanded.length > 1) return baseName(added[0]);
	return expanded[0] ? baseName(expanded[0]) : '';
}

/** What a file is called in a job name: a video's title, or the file's name, without the
 * `.txt` or `.pdf` of a text frozen from an earlier job. */
export function nameOfFile(file: InspectedFile, frozen: boolean): string {
	if (file.source?.title) return file.source.title;
	const name = file.source?.file_name ?? baseName(file.path);
	return frozen ? name.replace(/\.(txt|pdf)$/, '') : name;
}

/** Whether a name Kalamu suggested still fits the files: it names one of them, or a folder one
 * of them is in. Once the file it came from is removed, the name is suggested again. */
export function suggestionFits(
	suggested: string,
	files: InspectedFile[],
	frozen: boolean
): boolean {
	return files.some(
		(f) => nameOfFile(f, frozen) === suggested || pathParts(f.path).slice(0, -1).includes(suggested)
	);
}

/** How many pages of a PDF go in one call, as the form offers it. */
export type Together = 'one' | 'several' | 'whole';

/** What the job sends for how pages go together: nothing for one page at a time, as before,
 * nor for a task that sends pages one by one. */
export function pagesPerCallOf(
	rule: TaskRules,
	together: Together,
	pages: number
): PagesPerCall | undefined {
	if (!rule.pages_per_call || together === 'one') return undefined;
	return together === 'whole'
		? { kind: 'whole' }
		: { kind: 'several', pages: Math.max(2, Math.round(pages)) };
}

/** The form's choice back from what a pipeline sends; `pages` only when several. */
export function togetherOf(sent: PagesPerCall | undefined): { together: Together; pages?: number } {
	if (!sent || sent.kind === 'one') return { together: 'one' };
	return sent.kind === 'several'
		? { together: 'several', pages: sent.pages }
		: { together: 'whole' };
}

/** The choices of the form that every set-up shares. */
export interface SettingsForm {
	languageMode: 'auto' | 'explicit';
	languages: string[];
	/** Whether some chosen model takes the languages: if none does, they are not sent. */
	languagesTaken: boolean;
	sample: boolean;
	samplePages: number;
	/** Whether the task reads page by page, so a sample of pages applies. */
	paged: boolean;
	lineBreaks: LineBreaks;
	/** Whether line breaks apply to the task and some chosen model takes them. */
	linesTaken: boolean;
	pace: Pace;
	pagesPerCall: PagesPerCall | undefined;
}

/** The settings shared by every set-up, as a job and a pipeline store them. */
export function jobSettings(f: SettingsForm): JobSettings {
	return {
		languages:
			f.languageMode === 'explicit' && f.languages.length > 0 && f.languagesTaken
				? { mode: 'explicit', languages: [...f.languages] }
				: { mode: 'auto' },
		sample_pages: f.sample && f.paged ? f.samplePages : null,
		line_breaks: f.linesTaken ? f.lineBreaks : null,
		pace: f.pace,
		pages_per_call: f.pagesPerCall
	};
}

/** The form as a pipeline: everything but the files. */
export interface PipelineForm {
	name: string;
	task: Task;
	catalog: Catalog;
	rule: TaskRules;
	drafts: Draft[];
	settings: JobSettings;
	formats: ExportFormat[];
	destination: OutputDestination;
}

/** The form as a pipeline. Run now (`toRun`), it keeps only the formats the chosen models can
 * write, as a job does. */
export function formPipeline(f: PipelineForm, toRun = false): Pipeline {
	return {
		id: '',
		name: f.name,
		task: f.task,
		setups: f.drafts.map((d) => {
			const r = toRequest(f.catalog, f.task, d);
			return {
				provider: r.provider,
				model: r.model,
				preset_id: r.preset_id,
				label: r.label,
				options: r.options,
				merge: r.merge
			};
		}),
		settings: f.settings,
		formats: toRun
			? f.formats.filter((x) => offeredFormats(f.catalog, f.rule, f.drafts).includes(x))
			: [...f.formats],
		destination: f.destination,
		updated_at: 0
	};
}

/** The analysis that follows a reading, as a pipeline: the prompt chosen, the model `setup`
 * gives, and the researcher's usual formats for text that analysis can write. */
export function followingAnalysis(
	preset: Preset,
	setup: SetupRequest,
	rule: TaskRules,
	textFormats: ExportFormat[],
	pace: Pace,
	destination: OutputDestination
): Pipeline {
	const usual = rule.formats.filter((f) => textFormats.includes(f));
	return {
		id: '',
		name: preset.name,
		task: 'analysis',
		setups: [
			{
				provider: setup.provider,
				model: setup.model,
				preset_id: preset.id,
				label: null,
				options: setup.options
			}
		],
		settings: {
			languages: { mode: 'auto' },
			sample_pages: null,
			line_breaks: null,
			pace
		},
		formats: usual.length > 0 ? usual : rule.formats.slice(0, 1),
		destination,
		updated_at: 0
	};
}

/** A saved pipeline's set-ups as the form edits them, each model under the id the catalogue
 * writes now (a model replaced by a newer one is kept: New job offers its replacement). */
export function draftsOf(p: Pipeline, catalog?: Catalog): Draft[] {
	const id = (provider: string, model: string) =>
		catalog ? canonicalId(catalog, provider, model) : model;
	return p.setups.map((s) => ({
		provider: s.provider,
		model: id(s.provider, s.model),
		presetId: s.preset_id,
		label: s.label ?? '',
		diarize: s.options.diarize,
		reasoning: s.options.reasoning ?? null,
		timestamps: s.options.timestamps,
		vocabulary: s.options.vocabulary ?? '',
		translate: s.options.translate ?? false,
		mergeProvider: s.merge?.provider ?? s.provider,
		mergeModel: id(s.merge?.provider ?? s.provider, s.merge?.model ?? s.model)
	}));
}

/**
 * The New job form as the researcher left it: kept while they step away (to Settings for a
 * key, say) and given back when they come back to New job with nothing new to start from.
 */
export interface NewJobForm {
	mode: Mode;
	files: InspectedFile[];
	/** The files were text frozen from earlier results, not files to read. */
	frozen: boolean;
	/** The project the form was opened from, which the job joins. */
	project: FromProject | undefined;
	languageMode: 'auto' | 'explicit';
	languages: string[];
	lineBreaks: LineBreaks;
	pace: Pace;
	drafts: Draft[];
	sample: boolean;
	samplePages: number;
	together: Together;
	togetherPages: number;
	formats: ExportFormat[];
	destination: OutputDestination;
	name: string;
	suggested: string;
	/** The name the files suggest, while the document's name stands in for it. */
	fromFiles?: string;
	thenPreset: string | null;
	thenCheck: boolean;
	/** Several images: each its own document, or the pages of one (#52). */
	grouping?: Grouping;
	pageDoc?: ImageDocument | null;
	docName?: string;
	/** A page was moved by hand since the engine last put the pages in order. */
	byHand?: boolean;
}

let keptForm: NewJobForm | null = null;

/** Keeps the form as it stands, for the next time New job opens. */
export function keepForm(form: NewJobForm): void {
	keptForm = form;
}

/** Forgets the form kept: the job was started, or the researcher asked for a blank one. */
export function forgetForm(): void {
	keptForm = null;
}

/**
 * The form to open with: the one kept when New job opens plainly, none when it opens on
 * something to start from (a task, files, results or a project), which also forgets the one
 * kept so it cannot come back over the new start.
 */
export function formToOpen(opening: {
	task?: Task;
	paths?: string[];
	sources?: unknown[];
	project?: FromProject;
}): NewJobForm | null {
	const fresh =
		opening.task !== undefined ||
		opening.paths !== undefined ||
		opening.sources !== undefined ||
		opening.project !== undefined;
	if (fresh) keptForm = null;
	return keptForm;
}
