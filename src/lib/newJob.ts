// The New job form's logic, kept out of the components so it can be tested on its own. Which
// sources, models and formats suit a task is the engine's to say (`TaskRules`, from the
// `task_rules` command); the engine checks them again when the job is created.

import type { Catalog } from './bindings/Catalog.js';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { ImageDocument } from './bindings/ImageDocument.js';
import type { InputWarning } from './bindings/InputWarning.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { JobSettings } from './bindings/JobSettings.js';
import type { LineBreaks } from './bindings/LineBreaks.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { OutputDestination } from './bindings/OutputDestination.js';
import type { Pace } from './bindings/Pace.js';
import type { PagesPerCall } from './bindings/PagesPerCall.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Preset } from './bindings/Preset.js';
import type { Refused } from './bindings/Refused.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { Task } from './bindings/Task.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { Messages } from './i18n/index.js';
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

/** Any image format, as `inputOf` asks it: the engine converts an image into one the model reads. */
export const IMAGES = 'image/*';

/**
 * The input every chosen model must read: YouTube links; images when an image is among the
 * files, since a model that reads text only fails on each of them; else anything the task takes.
 */
export function inputOf(mode: Mode, files: InspectedFile[] = []): string | undefined {
	if (mode === 'youtube') return YOUTUBE_LINK;
	return files.some((f) => f.source?.kind === 'image') ? IMAGES : undefined;
}

/** Whether a model reads `input`: a format it lists, or any image for `IMAGES`. */
export function readsInput(model: ModelInfo | undefined, input: string | undefined): boolean {
	if (input === undefined) return true;
	if (input === IMAGES) return model?.inputs.some((i) => i.startsWith('image/')) ?? false;
	return model?.inputs.includes(input) ?? false;
}

/** One model set-up as edited in the form. */
export interface Draft {
	provider: string;
	model: string;
	presetId: string | null;
	label: string;
	diarize: boolean;
	timestamps: boolean;
	/** Names and words to spell as written, one per line, for a model that takes them. */
	vocabulary: string;
	/** Write the speech in English, for a model that can. */
	translate: boolean;
	/** How much the model thinks; `null` for the level suggested for the task. */
	reasoning: string | null;
	/** For a table of contents, the model that combines the pages: any provider's. */
	mergeProvider: string;
	mergeModel: string;
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

/** The engine's entry for a model in a task's rules. */
const refOf = (refs: TaskRules['models'], m: ModelInfo) =>
	refs.find((r) => r.provider === m.provider && r.model === m.id);
const listed = (refs: TaskRules['models'], m: ModelInfo) => refOf(refs, m) !== undefined;
/** Listed and offered for new work: neither replaced by a newer model nor retired. */
const offered = (refs: TaskRules['models'], m: ModelInfo) => {
	const ref = refOf(refs, m);
	return ref !== undefined && ref.replaced_by === undefined && ref.retired !== true;
};
/** The app's choice for the task, and each provider's, as the engine reads the catalogue. */
const recommended = (rule: TaskRules, m: ModelInfo) => refOf(rule.models, m)?.recommended === true;
const providerChoice = (rule: TaskRules, m: ModelInfo) =>
	refOf(rule.models, m)?.provider_default === true;

/** Whether a model can do the task. */
export function doesTask(rule: TaskRules, model: ModelInfo): boolean {
	return listed(rule.models, model);
}

/** Models that can combine a table of contents' pages, for one provider. */
export function mergeModelsFor(catalog: Catalog, rule: TaskRules, provider: string): ModelInfo[] {
	return catalog.models.filter((m) => m.provider === provider && offered(rule.merge_models, m));
}

/** The models the merge picker lists: those offered, and the one the set-up names when that one
 * was replaced, so a pipeline that names it shows it as it is. */
export function mergePickerModels(catalog: Catalog, rule: TaskRules, d: Draft): ModelInfo[] {
	const models = mergeModelsFor(catalog, rule, d.mergeProvider);
	const chosen = mergeInfo(catalog, d);
	return chosen && !models.includes(chosen) && listed(rule.merge_models, chosen)
		? [...models, chosen]
		: models;
}

/** Providers with a model that can combine the pages. */
export function mergeProviders(catalog: Catalog, rule: TaskRules): string[] {
	return catalog.providers
		.map((p) => p.id)
		.filter((id) => mergeModelsFor(catalog, rule, id).length > 0);
}

/** The model offered to combine the pages when the user switches provider: its smallest
 * listed, the cheapest per output token. */
export function mergeDefault(catalog: Catalog, rule: TaskRules, provider: string): string {
	const models = mergeModelsFor(catalog, rule, provider);
	const price = (m: ModelInfo) => m.pricing.output_per_mtok ?? Number.POSITIVE_INFINITY;
	return [...models].sort((a, b) => price(a) - price(b))[0]?.id ?? '';
}

/** Formats written only from a model that reports where text sits on the page, as the
 * engine's `ExportFormat::needs_layout` says. */
const LAYOUT_FORMATS: ExportFormat[] = ['searchable_pdf', 'alto'];

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

/** Models that can do a task and, when `input` is given, read it. */
export function modelsFor(
	catalog: Catalog,
	rule: TaskRules,
	provider: string,
	input?: string
): ModelInfo[] {
	return forTask(catalog, rule, input).filter((m) => m.provider === provider);
}

/** Models offered for new work that can do a task and read `input`: a model replaced by a newer
 * one or retired is not offered, though a set-up that names it keeps it. */
function forTask(catalog: Catalog, rule: TaskRules, input?: string): ModelInfo[] {
	return catalog.models.filter((m) => offered(rule.models, m) && readsInput(m, input));
}

/** The models the picker lists for a set-up: those offered, and the one it names when that one
 * was replaced or retired, so a pipeline or comparison that names it shows it as it is. */
export function pickerModels(
	catalog: Catalog,
	rule: TaskRules,
	d: Draft,
	input?: string
): ModelInfo[] {
	const models = modelsFor(catalog, rule, d.provider, input);
	const chosen = modelInfo(catalog, d);
	return chosen && !models.includes(chosen) && doesTask(rule, chosen)
		? [...models, chosen]
		: models;
}

/** A model replaced by a newer one or retired, as New job shows it, with the model offered in
 * its place. */
export interface Replaced {
	model: ModelInfo;
	retired: boolean;
	replacement: ModelInfo | undefined;
}

/** How a model was replaced; nothing for a model still offered or one the rules do not list. */
export function replacedModel(
	catalog: Catalog,
	refs: TaskRules['models'],
	model: ModelInfo | undefined
): Replaced | undefined {
	const ref = model && refOf(refs, model);
	if (!model || !ref || (ref.replaced_by === undefined && ref.retired !== true)) return undefined;
	const replacement = catalog.models.find(
		(m) => m.provider === model.provider && m.id === ref.replaced_by
	);
	return { model, retired: ref.retired === true, replacement };
}

/** The catalogue's entry for a model, by its id or by an id the catalogue once gave it. */
export function findModel(catalog: Catalog, provider: string, id: string): ModelInfo | undefined {
	const of = (id: string) => catalog.models.find((m) => m.provider === provider && m.id === id);
	const alias = catalog.aliases?.find((a) => a.provider === provider && a.id === id);
	return of(id) ?? (alias ? of(alias.model) : undefined);
}

/** A model's id as the catalogue writes it now; an id it does not know, as it is. */
export function canonicalId(catalog: Catalog, provider: string, id: string): string {
	return findModel(catalog, provider, id)?.id ?? id;
}

export function modelInfo(catalog: Catalog, d: Draft): ModelInfo | undefined {
	return findModel(catalog, d.provider, d.model);
}

/** Whether a transcription model takes a list of names and words to spell (Whisper). */
export function takesVocabulary(model: ModelInfo | undefined, task: Task): boolean {
	return task === 'audio_transcription' && model?.vocabulary === true;
}

/** Whether a transcription model can write the speech in English instead (Whisper but Turbo). */
export function translates(model: ModelInfo | undefined, task: Task): boolean {
	return task === 'audio_transcription' && model?.translates === true;
}

/**
 * The names and words typed, one a line, without blank lines or repeats; the engine reads
 * commas and semicolons as separators too.
 */
export function vocabularyOf(text: string): string {
	const seen = new Set<string>();
	return text
		.split(/[\n,;]/)
		.map((t) => t.trim())
		.filter((t) => t && !seen.has(t.toLowerCase()) && seen.add(t.toLowerCase()))
		.join('\n');
}

/**
 * The names a project's dictionary holds, most found first, to spell in a transcription: as
 * many as Whisper keeps in its prompt.
 */
export function dictionaryNames(rows: { label: string; documents: number }[], max = 60): string[] {
	return [...rows]
		.sort((a, b) => b.documents - a.documents)
		.map((r) => r.label.trim())
		.filter((label, i, all) => label && all.indexOf(label) === i)
		.slice(0, max);
}

/** Whisper cannot tell speakers apart; a missing flag means the model can. */
export function labelsSpeakers(model: ModelInfo | undefined): boolean {
	return model?.speakers !== false;
}

/** Dedicated OCR and speech models take no prompt; a missing flag means they do. */
export function acceptsPrompt(model: ModelInfo | undefined): boolean {
	return model?.accepts_prompt !== false;
}

/**
 * The model chosen when the user switches to a provider: its default for the task in the
 * catalogue (Claude Sonnet, not the costlier Fable listed first), else its first model.
 */
export function providerDefault(
	catalog: Catalog,
	rule: TaskRules,
	provider: string,
	input?: string
): string {
	const models = modelsFor(catalog, rule, provider, input);
	return (models.find((m) => providerChoice(rule, m)) ?? models[0])?.id ?? '';
}

/**
 * A set-up for a usable model: the recommended one first, then each provider's default, then
 * any model with a key, avoiding
 * models already in `used` so a second set-up compares against something different. With
 * `input`, only models that read it are considered.
 */
export function newDraft(
	catalog: Catalog,
	rule: TaskRules,
	presetId: string | null,
	hasKey: (provider: string) => boolean,
	used: string[] = [],
	input?: string
): Draft {
	const offered = forTask(catalog, rule, input);
	const usable = offered.filter((m) => hasKey(m.provider));
	const fresh = usable.filter((m) => !used.includes(`${m.provider}/${m.id}`));
	const pick =
		fresh.find((m) => recommended(rule, m)) ??
		fresh.find((m) => providerChoice(rule, m)) ??
		fresh[0] ??
		usable.find((m) => recommended(rule, m)) ??
		usable[0] ??
		offered[0];
	const provider = pick?.provider ?? '';
	return {
		provider,
		model: pick?.id ?? '',
		presetId,
		label: '',
		diarize: true,
		timestamps: true,
		vocabulary: '',
		translate: false,
		reasoning: null,
		// The pages' provider combines them, with its cheapest model: one key is enough.
		mergeProvider: provider,
		mergeModel: mergeDefault(catalog, rule, provider)
	};
}

export function toRequest(catalog: Catalog, task: Task, d: Draft): SetupRequest {
	const model = modelInfo(catalog, d);
	const vocabulary = takesVocabulary(model, task) ? vocabularyOf(d.vocabulary) : '';
	return {
		provider: d.provider,
		model: d.model,
		preset_id: acceptsPrompt(modelInfo(catalog, d)) ? d.presetId : null,
		label: d.label.trim() || null,
		options: {
			temperature: null,
			diarize: task === 'audio_transcription' && d.diarize && labelsSpeakers(modelInfo(catalog, d)),
			timestamps: task === 'audio_transcription' && d.timestamps,
			speakers: null,
			reasoning: d.reasoning ?? undefined,
			...(vocabulary ? { vocabulary } : {}),
			...(translates(model, task) && d.translate ? { translate: true } : {})
		},
		merge:
			task === 'contents' && d.mergeModel
				? { provider: d.mergeProvider, model: d.mergeModel }
				: undefined
	};
}

/** The draft's merge model, for the catalogue's name. */
export function mergeInfo(catalog: Catalog, d: Draft): ModelInfo | undefined {
	return findModel(catalog, d.mergeProvider, d.mergeModel);
}

export function warningText(w: InputWarning, m: Messages): string {
	const pages = (list: number[]) =>
		list.length > 6 ? `${list.slice(0, 6).join(', ')}…` : list.join(', ');
	switch (w.kind) {
		case 'double_page_spread':
			return m.warning.double_page_spread(pages(w.pages));
		case 'low_resolution':
			return m.warning.low_resolution(w.dpi, pages(w.pages));
		case 'has_text_layer':
			return m.warning.has_text_layer(w.pages_with_text);
		case 'unsupported_image':
			return m.warning.unsupported_image(w.detail);
		case 'large_file':
			return m.warning.large_file;
		case 'tool_missing':
			return m.warning.tool_missing;
		case 'no_sound_track':
			return m.warning.no_sound_track;
		case 'long_recording':
			return m.warning.long_recording(duration(w.minutes * 60_000, m), w.pieces);
		case 'quiet_recording':
			return m.warning.quiet_recording(w.mean_db);
	}
}

/** "1 h 05 min", "12 min", "45 s". */
export function duration(ms: number, m: Messages): string {
	const total = Math.round(ms / 1000);
	const h = Math.floor(total / 3600);
	const min = Math.floor((total % 3600) / 60);
	if (h > 0) return m.newJob.hoursMinutes(h, String(min).padStart(2, '0'));
	if (min > 0) return m.newJob.minutes(min);
	return m.newJob.seconds(total);
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

/** The reasoning level the engine suggests for a draft's model and the task; none for a model
 * that does not reason. */
export function suggestedReasoning(rule: TaskRules, d: Draft): string | undefined {
	return rule.models.find((r) => r.provider === d.provider && r.model === d.model)
		?.suggested_reasoning;
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
	thenPreset: string | null;
	thenCheck: boolean;
	/** Several images: each its own document, or the pages of one (#52). */
	grouping?: Grouping;
	pageDoc?: ImageDocument | null;
	docName?: string;
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
