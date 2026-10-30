// The New job form's logic, kept out of the components so it can be tested on its own. Which
// sources, models and formats suit a task is the engine's to say (`TaskRules`, from the
// `task_rules` command); the engine checks them again when the job is created.

import type { Catalog } from './bindings/Catalog.js';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { InputWarning } from './bindings/InputWarning.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { Task } from './bindings/Task.js';
import type { TaskRules } from './bindings/TaskRules.js';
import type { Messages } from './i18n/index.js';
import { YOUTUBE_LINK } from './youtube.js';

/**
 * What New job offers: the tasks, and YouTube videos, which are transcribed like a recording
 * but added by their link and read only by models that take YouTube links.
 */
export type Mode = Task | 'youtube';

/**
 * In the order New job shows them, two by two: reading pages, then working on texts, then
 * spoken sources.
 */
export const MODES: Mode[] = [
	'text_extraction',
	'text_correction',
	'analysis',
	'contents',
	'audio_transcription',
	'youtube'
];

export function taskOf(mode: Mode): Task {
	return mode === 'youtube' ? 'audio_transcription' : mode;
}

/** The input every chosen model must read: YouTube links, or anything the task takes. */
export function inputOf(mode: Mode): string | undefined {
	return mode === 'youtube' ? YOUTUBE_LINK : undefined;
}

/** One model set-up as edited in the form. */
export interface Draft {
	provider: string;
	model: string;
	presetId: string | null;
	label: string;
	diarize: boolean;
	timestamps: boolean;
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
			models: [],
			merge_models: []
		}
	);
}

/**
 * The task the catalogue lists a model's defaults under: a table of contents and a correction
 * are made of analyses (`Task::request_task` in Rust), so they take the analysis defaults.
 */
export function requestTask(task: Task): Task {
	return task === 'contents' || task === 'text_correction' ? 'analysis' : task;
}

const listed = (refs: TaskRules['models'], m: ModelInfo) =>
	refs.some((r) => r.provider === m.provider && r.model === m.id);

/** Whether a model can do the task. */
export function doesTask(rule: TaskRules, model: ModelInfo): boolean {
	return listed(rule.models, model);
}

/** Models that can combine a table of contents' pages, for one provider. */
export function mergeModelsFor(catalog: Catalog, rule: TaskRules, provider: string): ModelInfo[] {
	return catalog.models.filter((m) => m.provider === provider && listed(rule.merge_models, m));
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

/**
 * The formats to offer: a searchable PDF only when a chosen model reports where text sits on
 * the page (Mistral OCR).
 */
export function offeredFormats(catalog: Catalog, rule: TaskRules, drafts: Draft[]): ExportFormat[] {
	const layout = drafts.some((d) => modelInfo(catalog, d)?.layout === true);
	return rule.formats.filter((f) => f !== 'searchable_pdf' || layout);
}

/** Whether a mode reads a kind of source: YouTube links in the YouTube mode, files only in
 * the others. */
export function readsKind(rules: Rules, mode: Mode, kind: string | undefined): boolean {
	if (mode === 'youtube') return kind === 'youtube';
	return kind !== 'youtube' && rulesFor(rules, mode).sources.some((k) => k === kind);
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

function forTask(catalog: Catalog, rule: TaskRules, input?: string): ModelInfo[] {
	return catalog.models.filter(
		(m) => doesTask(rule, m) && (input === undefined || m.inputs.includes(input))
	);
}

export function modelInfo(catalog: Catalog, d: Draft): ModelInfo | undefined {
	return catalog.models.find((m) => m.provider === d.provider && m.id === d.model);
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
	const listedAs = requestTask(rule.task);
	return (models.find((m) => m.provider_default_for.includes(listedAs)) ?? models[0])?.id ?? '';
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
	const listedAs = requestTask(rule.task);
	const pick =
		fresh.find((m) => m.recommended_for.includes(listedAs)) ??
		fresh.find((m) => m.provider_default_for.includes(listedAs)) ??
		fresh[0] ??
		usable.find((m) => m.recommended_for.includes(listedAs)) ??
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
		// The pages' provider combines them, with its cheapest model: one key is enough.
		mergeProvider: provider,
		mergeModel: mergeDefault(catalog, rule, provider)
	};
}

export function toRequest(catalog: Catalog, task: Task, d: Draft): SetupRequest {
	return {
		provider: d.provider,
		model: d.model,
		preset_id: acceptsPrompt(modelInfo(catalog, d)) ? d.presetId : null,
		label: d.label.trim() || null,
		options: {
			temperature: null,
			diarize: task === 'audio_transcription' && d.diarize && labelsSpeakers(modelInfo(catalog, d)),
			timestamps: task === 'audio_transcription' && d.timestamps,
			speakers: null
		},
		merge:
			task === 'contents' && d.mergeModel
				? { provider: d.mergeProvider, model: d.mergeModel }
				: undefined
	};
}

/** The draft's merge model, for the catalogue's name. */
export function mergeInfo(catalog: Catalog, d: Draft): ModelInfo | undefined {
	return catalog.models.find((m) => m.provider === d.mergeProvider && m.id === d.mergeModel);
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
	const last = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? '';
	if (added.length === 1 && expanded.length > 1) return last(added[0]);
	return expanded[0] ? last(expanded[0]) : '';
}
