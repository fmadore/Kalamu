// New job's model choice, kept out of the components so it can be tested on its own: which
// models a set-up is offered for a task and its files, which one it starts with, and what a
// set-up sends. Which models suit a task is the engine's to say (`TaskRules`).

import type { Catalog } from './bindings/Catalog.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { Task } from './bindings/Task.js';
import type { TaskRules } from './bindings/TaskRules.js';

/** Any image format, as `inputOf` asks it: the engine converts an image into one the model reads. */
export const IMAGES = 'image/*';

/** What every chosen model must read, each of the inputs listed; anything the task takes when
 * absent. */
export type Needs = string[] | undefined;

/** Whether a model reads every input in `needs`: a format it lists, or any image for `IMAGES`. */
export function readsInput(model: ModelInfo | undefined, needs: Needs): boolean {
	return (needs ?? []).every((input) =>
		input === IMAGES
			? (model?.inputs.some((i) => i.startsWith('image/')) ?? false)
			: (model?.inputs.includes(input) ?? false)
	);
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

/** Models that can do a task and, when `input` is given, read it. */
export function modelsFor(
	catalog: Catalog,
	rule: TaskRules,
	provider: string,
	input?: Needs
): ModelInfo[] {
	return forTask(catalog, rule, input).filter((m) => m.provider === provider);
}

/** Models offered for new work that can do a task and read `input`: a model replaced by a newer
 * one or retired is not offered, though a set-up that names it keeps it. */
function forTask(catalog: Catalog, rule: TaskRules, input?: Needs): ModelInfo[] {
	return catalog.models.filter((m) => offered(rule.models, m) && readsInput(m, input));
}

/** The models the picker lists for a set-up: those offered, and the one it names when that one
 * was replaced or retired, so a pipeline or comparison that names it shows it as it is. */
export function pickerModels(
	catalog: Catalog,
	rule: TaskRules,
	d: Draft,
	input?: Needs
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
	input?: Needs
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
	input?: Needs
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

/** The reasoning level the engine suggests for a draft's model and the task; none for a model
 * that does not reason. */
export function suggestedReasoning(rule: TaskRules, d: Draft): string | undefined {
	return rule.models.find((r) => r.provider === d.provider && r.model === d.model)
		?.suggested_reasoning;
}
