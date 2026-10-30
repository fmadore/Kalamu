import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { Task } from './bindings/Task.js';
import rulesJson from './dev/taskRules.json';
import { en } from './i18n/index.js';
import {
	acceptsPrompt,
	inputOf,
	mergeDefault,
	mergeModelsFor,
	modelInfo,
	modelsFor,
	newDraft,
	providerDefault,
	readsKind,
	type Rules,
	rulesFor,
	suggestName,
	taskOf,
	toRequest,
	warningText
} from './newJob.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
/** The engine's rules for the bundled catalogue, written by `cargo test`. */
const rules = rulesJson as Rules;
const rule = (task: Task) => rulesFor(rules, task);

describe('new job form', () => {
	it('starts with the recommended model the user has a key for', () => {
		const d = newDraft(catalog, rule('text_extraction'), 'p', () => true);
		expect(d.model).toBe('gemini-3.8-flash');
		const claudeOnly = newDraft(catalog, rule('text_extraction'), 'p', (p) => p === 'anthropic');
		expect(claudeOnly.provider).toBe('anthropic');
		// Each provider's everyday model, not its first or costliest.
		expect(claudeOnly.model).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, rule('analysis'), 'anthropic')).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, rule('audio_transcription'), 'mistral')).toBe(
			'voxtral-small-transcribe-3-0'
		);
	});

	it('suggests a different model for a comparison', () => {
		const first = newDraft(catalog, rule('text_extraction'), 'p', () => true);
		const second = newDraft(catalog, rule('text_extraction'), 'p', () => true, [
			`${first.provider}/${first.model}`
		]);
		expect(`${second.provider}/${second.model}`).not.toBe(`${first.provider}/${first.model}`);
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
		expect(d.model).toBe('gemini-3.8-flash');
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

	it('words input warnings and names jobs after what was added', () => {
		const text = warningText({ source_id: 'x', kind: 'double_page_spread', pages: [2, 3] }, en);
		expect(text).toContain('page 2, 3');
		expect(
			suggestName(
				['C:\\Archives\\Bulletin'],
				['C:\\Archives\\Bulletin\\a.pdf', 'C:\\Archives\\Bulletin\\b.pdf']
			)
		).toBe('Bulletin');
		expect(suggestName(['C:\\x\\lettre.jpg'], ['C:\\x\\lettre.jpg'])).toBe('lettre.jpg');
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
			'gemini-3.8-flash'
		);
	});
});

describe('tables of contents and corrections', () => {
	it('asks for the language of the answers where the model writes them', () => {
		expect(rule('analysis').answers_in_language).toBe(true);
		expect(rule('contents').answers_in_language).toBe(true);
		expect(rule('text_correction').answers_in_language).toBe(false);
		expect(rule('text_extraction').answers_in_language).toBe(false);
	});

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
		expect(d.model).toBe('gemini-3.8-flash');
		expect(d.mergeProvider).toBe('gemini');
		expect(mergeModelsFor(catalog, rule('contents'), 'gemini').map((m) => m.id)).toContain(
			d.mergeModel
		);
		// Gemini Pro reads the pages, Mistral combines them.
		const mixed = { ...d, model: 'gemini-3.1-pro-preview', mergeProvider: 'mistral' };
		mixed.mergeModel = mergeDefault(catalog, rule('contents'), 'mistral');
		const request = toRequest(catalog, 'contents', mixed);
		expect(request.merge).toEqual({ provider: 'mistral', model: mixed.mergeModel });
		expect(toRequest(catalog, 'analysis', mixed).merge).toBeUndefined();
	});

	it('takes the files each task reads', () => {
		expect(readsKind(rules, 'contents', 'image')).toBe(true);
		expect(readsKind(rules, 'contents', 'text')).toBe(false);
		expect(readsKind(rules, 'text_correction', 'text')).toBe(true);
		expect(readsKind(rules, 'text_correction', 'image')).toBe(false);
		expect(taskOf('text_correction')).toBe('text_correction');
	});
});
