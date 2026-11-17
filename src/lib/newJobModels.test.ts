import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { Task } from './bindings/Task.js';
import rulesJson from './dev/taskRules.json';
import { draftsOf, inputOf, readsKind, type Rules, rulesFor, taskOf } from './newJob.js';
import {
	IMAGES,
	acceptsPrompt,
	dictionaryNames,
	mergeDefault,
	mergeModelsFor,
	mergePickerModels,
	modelInfo,
	modelsFor,
	newDraft,
	pickerModels,
	providerDefault,
	readsInput,
	replacedModel,
	toRequest
} from './newJobModels.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
/** The engine's rules for the bundled catalogue, written by `cargo test`. */
const rules = rulesJson as Rules;
const rule = (task: Task) => rulesFor(rules, task);

describe('new job form', () => {
	it('starts with the recommended model the user has a key for', () => {
		const d = newDraft(catalog, rule('text_extraction'), 'p', () => true);
		expect(d.model).toBe('gemini-4-hydrogen');
		const claudeOnly = newDraft(catalog, rule('text_extraction'), 'p', (p) => p === 'anthropic');
		expect(claudeOnly.provider).toBe('anthropic');
		// Each provider's everyday model, not its first or costliest.
		expect(claudeOnly.model).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, rule('analysis'), 'anthropic')).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, rule('audio_transcription'), 'mistral')).toBe(
			'voxtral-small-transcribe-3-0'
		);
		// Google's speech-to-text model for recordings; it takes no prompt.
		const transcribe = providerDefault(catalog, rule('audio_transcription'), 'gemini', [
			'audio/mpeg'
		]);
		expect(transcribe).toBe('gemini-3.5-transcribe');
		expect(acceptsPrompt(catalog.models.find((m) => m.id === transcribe))).toBe(false);
	});

	it('suggests a different model for a comparison', () => {
		const first = newDraft(catalog, rule('text_extraction'), 'p', () => true);
		const second = newDraft(catalog, rule('text_extraction'), 'p', () => true, [
			`${first.provider}/${first.model}`
		]);
		expect(`${second.provider}/${second.model}`).not.toBe(`${first.provider}/${first.model}`);
	});

	it('offers only the newest model of each line, and keeps an older one a set-up names', () => {
		const analysis = rule('analysis');
		const claude = modelsFor(catalog, analysis, 'anthropic').map((m) => m.id);
		expect(claude).toContain('claude-fable-5-2');
		expect(claude).not.toContain('claude-fable-5-1');
		// Argon, Carbon and Hydrogen are three lines of Gemini 4, all offered; Hydrogen replaces
		// 3.8 Flash and Argon 3.1 Pro.
		const gemini = modelsFor(catalog, analysis, 'gemini').map((m) => m.id);
		expect(gemini).toEqual(
			expect.arrayContaining(['gemini-4-carbon', 'gemini-4-argon', 'gemini-4-hydrogen'])
		);
		expect(gemini).not.toContain('gemini-3.8-flash');
		expect(gemini).not.toContain('gemini-3.1-pro-preview');
		// GPT-6.2 Sol replaces 6.1 Sol, and 6.2 Astra the first Astra.
		const openai = modelsFor(catalog, analysis, 'openai').map((m) => m.id);
		expect(openai).toEqual(expect.arrayContaining(['gpt-6.2-sol', 'gpt-6.2-astra']));
		expect(openai).not.toContain('gpt-6.1-sol');
		expect(openai).not.toContain('gpt-6-astra');
		expect(providerDefault(catalog, analysis, 'openai')).toBe('gpt-6.2-sol');
		expect(providerDefault(catalog, rule('text_extraction'), 'openai')).toBe('gpt-6.2-sol');
		expect(providerDefault(catalog, analysis, 'mistral')).toBe('mistral-large-4-0');
		// No new set-up is given a replaced model, even when nothing else is left.
		const used = catalog.models.map((m) => `${m.provider}/${m.id}`);
		const d = newDraft(catalog, analysis, null, (p) => p === 'anthropic', used.slice(0, -1));
		expect(d.model).not.toBe('claude-fable-5-1');

		// A pipeline saved with Fable 5.1 shows it, marked, with Fable 5.2 to switch to.
		const old = {
			...newDraft(catalog, analysis, null, () => true),
			provider: 'anthropic',
			model: 'claude-fable-5-1'
		};
		const listed = pickerModels(catalog, analysis, old).map((m) => m.id);
		expect(listed).toContain('claude-fable-5-1');
		const replaced = replacedModel(catalog, analysis.models, modelInfo(catalog, old));
		expect(replaced?.model.name).toBe('Claude Fable 5.1');
		expect(replaced?.retired).toBe(false);
		expect(replaced?.replacement?.id).toBe('claude-fable-5-2');
		expect(
			replacedModel(
				catalog,
				analysis.models,
				modelInfo(catalog, { ...old, model: 'claude-fable-5-2' })
			)
		).toBeUndefined();
		// The model that combines a table of contents' pages is kept the same way.
		const contents = rule('contents');
		const merge = { ...old, mergeProvider: 'mistral', mergeModel: 'mistral-medium-3-5' };
		expect(mergeModelsFor(catalog, contents, 'mistral').map((m) => m.id)).not.toContain(
			'mistral-medium-3-5'
		);
		expect(mergePickerModels(catalog, contents, merge).map((m) => m.id)).toContain(
			'mistral-medium-3-5'
		);
	});

	it('reads a model under an id the catalogue once gave it', () => {
		const saved = {
			setups: [
				{ provider: 'mistral', model: 'deepseek-v4p1-flash-1017', preset_id: null, options: {} }
			]
		} as unknown as Parameters<typeof draftsOf>[0];
		const [d] = draftsOf(saved, catalog);
		expect(d.model).toBe('deepseek-v4-1-flash-1017');
		expect(d.mergeModel).toBe('deepseek-v4-1-flash-1017');
		expect(modelInfo(catalog, { ...d, model: 'deepseek-v4p1-flash-1017' })?.name).toBe(
			'DeepSeek V4.1 Flash'
		);
	});

	it('sends no prompt to models that take none', () => {
		const ocr = {
			...newDraft(catalog, rule('text_extraction'), 'preset', () => true),
			provider: 'mistral',
			model: 'mistral-ocr-4-1'
		};
		expect(acceptsPrompt(modelInfo(catalog, ocr))).toBe(false);
		expect(toRequest(catalog, 'text_extraction', ocr).preset_id).toBeNull();
		const flash = newDraft(catalog, rule('text_extraction'), 'preset', () => true);
		expect(toRequest(catalog, 'text_extraction', flash).preset_id).toBe('preset');
		expect(toRequest(catalog, 'text_extraction', flash).options.diarize).toBe(false);
	});

	it('summarises texts and PDFs with the recommended model', () => {
		const d = newDraft(catalog, rule('analysis'), 'analysis.summary_research', () => true);
		expect(d.model).toBe('gemini-4-hydrogen');
		expect(readsKind(rules, 'analysis', 'text')).toBe(true);
		expect(readsKind(rules, 'analysis', 'pdf')).toBe(true);
		// A photographed page is read as one document, as the engine allows.
		expect(readsKind(rules, 'analysis', 'image')).toBe(true);
		expect(readsKind(rules, 'analysis', 'audio')).toBe(false);
		// Whisper never offers speaker labels, whatever the box says.
		const whisper = {
			...newDraft(catalog, rule('audio_transcription'), null, () => true),
			provider: 'whisper',
			model: 'whisper-large-v3-turbo',
			diarize: true
		};
		expect(toRequest(catalog, 'audio_transcription', whisper).options.diarize).toBe(false);
	});

	it('sends names to spell and translation only to a model that uses them', () => {
		const base = newDraft(catalog, rule('audio_transcription'), null, () => true);
		const whisper = (model: string) => ({
			...base,
			provider: 'whisper',
			model,
			vocabulary: 'Samory Touré, Ouagadougou\n\nsamory touré',
			translate: true
		});
		const large = toRequest(catalog, 'audio_transcription', whisper('whisper-large-v3')).options;
		expect(large.vocabulary).toBe('Samory Touré\nOuagadougou');
		expect(large.translate).toBe(true);
		// Turbo was trained without translation.
		const turbo = toRequest(catalog, 'audio_transcription', whisper('whisper-large-v3-turbo'));
		expect(turbo.options.translate).toBeUndefined();
		expect(turbo.options.vocabulary).toBe('Samory Touré\nOuagadougou');
		// A cloud model is sent neither, so its answer and its key stay as they were.
		const cloud = toRequest(catalog, 'audio_transcription', {
			...whisper('x'),
			...base,
			vocabulary: 'Samory',
			translate: true
		});
		expect(cloud.options).not.toHaveProperty('vocabulary');
		expect(cloud.options).not.toHaveProperty('translate');
	});

	it('offers a project dictionary’s most found names to spell', () => {
		expect(
			dictionaryNames([
				{ label: 'Bamako', documents: 2 },
				{ label: 'Samory Touré', documents: 9 },
				{ label: ' ', documents: 4 },
				{ label: 'Bamako', documents: 1 }
			])
		).toEqual(['Samory Touré', 'Bamako']);
	});

	it('offers YouTube videos to the models that read their links only', () => {
		expect(taskOf('youtube')).toBe('audio_transcription');
		expect(readsKind(rules, 'youtube', 'youtube')).toBe(true);
		expect(readsKind(rules, 'youtube', 'audio')).toBe(false);
		expect(readsKind(rules, 'audio_transcription', 'youtube')).toBe(false);
		const input = inputOf('youtube');
		const d = newDraft(catalog, rule('audio_transcription'), null, () => true, [], input);
		expect(d.provider).toBe('gemini');
		// No key for Gemini still proposes a Gemini model, never one that cannot read the link.
		const noGemini = newDraft(
			catalog,
			rule('audio_transcription'),
			null,
			(p) => p !== 'gemini',
			[],
			input
		);
		expect(noGemini.provider).toBe('gemini');
		expect(modelsFor(catalog, rule('audio_transcription'), 'mistral', input)).toEqual([]);
		expect(modelsFor(catalog, rule('audio_transcription'), 'gemini', input).length).toBeGreaterThan(
			0
		);
		expect(providerDefault(catalog, rule('audio_transcription'), 'gemini', input)).toBe(
			'gemini-4-hydrogen'
		);
	});

	it('offers images to an analysis only on models that read them', () => {
		const inspected = (kind: SourceFile['kind']) =>
			({ path: `a.${kind}`, source: { kind } as SourceFile }) as InspectedFile;
		const analysis = rule('analysis');
		const texts = inputOf('analysis', [inspected('text')]);
		expect(texts).toBeUndefined();
		expect(modelsFor(catalog, analysis, 'mistral', texts).map((m) => m.id)).toContain(
			'zai-glm-5-3'
		);
		const images = inputOf('analysis', [inspected('text'), inspected('image')]);
		const mistral = modelsFor(catalog, analysis, 'mistral', images).map((m) => m.id);
		expect(mistral).toContain('mistral-large-4-0');
		expect(mistral).not.toContain('zai-glm-5-3');
		expect(mistral).not.toContain('deepseek-v4-1-flash-1017');
		// Any image format will do: the engine converts a TIFF into one the model reads.
		const glm = catalog.models.find((m) => m.id === 'zai-glm-5-3');
		expect(readsInput(glm, images)).toBe(false);
		expect(
			readsInput(
				modelInfo(
					catalog,
					newDraft(catalog, analysis, null, () => true)
				),
				images
			)
		).toBe(true);
		expect(inputOf('youtube', [inspected('image')])).toEqual(inputOf('youtube'));
	});

	it('offers PDFs to an analysis only on models that read them whole', () => {
		const inspected = (kind: SourceFile['kind']) =>
			({ path: `a.${kind}`, source: { kind } as SourceFile }) as InspectedFile;
		const analysis = rule('analysis');
		const pdfs = inputOf('analysis', [inspected('text'), inspected('pdf')]);
		const mistral = modelsFor(catalog, analysis, 'mistral', pdfs).map((m) => m.id);
		expect(mistral).toContain('mistral-large-4-0');
		expect(mistral).not.toContain('zai-glm-5-3');
		// Gemma reads page images, not whole PDFs.
		expect(modelsFor(catalog, analysis, 'local-llm', pdfs)).toEqual([]);
		// Images and PDFs together need both.
		const both = inputOf('analysis', [inspected('image'), inspected('pdf')]);
		expect(both).toEqual([IMAGES, 'application/pdf']);
		// A translation reads a PDF's text layer: a model that reads text only is offered.
		expect(inputOf('translation', [inspected('pdf')])).toBeUndefined();
	});
});

describe('tables of contents and corrections', () => {
	it('reads pages with a vision model and combines them with any text model', () => {
		const pages = modelsFor(catalog, rule('contents'), 'mistral').map((m) => m.id);
		expect(pages).not.toContain('deepseek-v4-1-flash-1017');
		expect(pages).not.toContain('mistral-ocr-4-1');
		expect(modelsFor(catalog, rule('text_correction'), 'mistral').map((m) => m.id)).toContain(
			'deepseek-v4-1-flash-1017'
		);
		expect(mergeModelsFor(catalog, rule('contents'), 'mistral').map((m) => m.id)).toContain(
			'deepseek-v4-1-flash-1017'
		);
		const d = newDraft(catalog, rule('contents'), 'contents.periodical', () => true);
		expect(d.model).toBe('gemini-4-hydrogen');
		expect(d.mergeProvider).toBe('gemini');
		expect(mergeModelsFor(catalog, rule('contents'), 'gemini').map((m) => m.id)).toContain(
			d.mergeModel
		);
		// Gemini 4 Argon reads the pages, Mistral combines them.
		const mixed = { ...d, model: 'gemini-4-argon', mergeProvider: 'mistral' };
		mixed.mergeModel = mergeDefault(catalog, rule('contents'), 'mistral');
		const request = toRequest(catalog, 'contents', mixed);
		expect(request.merge).toEqual({ provider: 'mistral', model: mixed.mergeModel });
		expect(toRequest(catalog, 'analysis', mixed).merge).toBeUndefined();
	});
});
