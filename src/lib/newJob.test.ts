import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import { en } from './i18n/index.js';
import {
	acceptsPrompt,
	answersInLanguage,
	inputOf,
	mergeDefault,
	mergeModelsFor,
	modelInfo,
	modelsFor,
	newDraft,
	providerDefault,
	readsKind,
	suggestName,
	taskOf,
	toRequest,
	warningText
} from './newJob.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;

describe('new job form', () => {
	it('starts with the recommended model the user has a key for', () => {
		const d = newDraft(catalog, 'text_extraction', 'p', () => true);
		expect(d.model).toBe('gemini-3.8-flash');
		const claudeOnly = newDraft(catalog, 'text_extraction', 'p', (p) => p === 'anthropic');
		expect(claudeOnly.provider).toBe('anthropic');
		// Each provider's everyday model, not its first or costliest.
		expect(claudeOnly.model).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, 'anthropic', 'analysis')).toBe('claude-sonnet-5-5');
		expect(providerDefault(catalog, 'mistral', 'audio_transcription')).toBe(
			'voxtral-small-transcribe-3-0'
		);
	});

	it('suggests a different model for a comparison', () => {
		const first = newDraft(catalog, 'text_extraction', 'p', () => true);
		const second = newDraft(catalog, 'text_extraction', 'p', () => true, [
			`${first.provider}/${first.model}`
		]);
		expect(`${second.provider}/${second.model}`).not.toBe(`${first.provider}/${first.model}`);
	});

	it('sends no prompt to models that take none', () => {
		const ocr = {
			...newDraft(catalog, 'text_extraction', 'preset', () => true),
			provider: 'mistral',
			model: 'mistral-ocr-4-1'
		};
		expect(acceptsPrompt(modelInfo(catalog, ocr))).toBe(false);
		expect(toRequest(catalog, 'text_extraction', ocr).preset_id).toBeNull();
		const flash = newDraft(catalog, 'text_extraction', 'preset', () => true);
		expect(toRequest(catalog, 'text_extraction', flash).preset_id).toBe('preset');
		expect(toRequest(catalog, 'text_extraction', flash).options.diarize).toBe(false);
	});

	it('summarises texts and PDFs with the recommended model', () => {
		const d = newDraft(catalog, 'analysis', 'analysis.summary_research', () => true);
		expect(d.model).toBe('gemini-3.8-flash');
		expect(readsKind('analysis', 'text')).toBe(true);
		expect(readsKind('analysis', 'pdf')).toBe(true);
		expect(readsKind('analysis', 'audio')).toBe(false);
		// Whisper never offers speaker labels, whatever the box says.
		const whisper = {
			...newDraft(catalog, 'audio_transcription', null, () => true),
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
		expect(readsKind('youtube', 'youtube')).toBe(true);
		expect(readsKind('youtube', 'audio')).toBe(false);
		expect(readsKind('audio_transcription', 'youtube')).toBe(false);
		const input = inputOf('youtube');
		const d = newDraft(catalog, 'audio_transcription', null, () => true, [], input);
		expect(d.provider).toBe('gemini');
		// No key for Gemini still proposes a Gemini model, never one that cannot read the link.
		const noGemini = newDraft(
			catalog,
			'audio_transcription',
			null,
			(p) => p !== 'gemini',
			[],
			input
		);
		expect(noGemini.provider).toBe('gemini');
		expect(modelsFor(catalog, 'mistral', 'audio_transcription', input)).toEqual([]);
		expect(modelsFor(catalog, 'gemini', 'audio_transcription', input).length).toBeGreaterThan(0);
		expect(providerDefault(catalog, 'gemini', 'audio_transcription', input)).toBe(
			'gemini-3.8-flash'
		);
	});
});

describe('tables of contents and corrections', () => {
	it('asks for the language of the answers where the model writes them', () => {
		expect(answersInLanguage('analysis')).toBe(true);
		expect(answersInLanguage('contents')).toBe(true);
		expect(answersInLanguage('text_correction')).toBe(false);
		expect(answersInLanguage('text_extraction')).toBe(false);
	});

	it('reads pages with a vision model and combines them with any text model', () => {
		const pages = modelsFor(catalog, 'mistral', 'contents').map((m) => m.id);
		expect(pages).not.toContain('deepseek-v4-1-flash-1017');
		expect(pages).not.toContain('mistral-ocr-4-1');
		expect(modelsFor(catalog, 'mistral', 'text_correction').map((m) => m.id)).toContain(
			'deepseek-v4-1-flash-1017'
		);
		expect(mergeModelsFor(catalog, 'mistral').map((m) => m.id)).toContain(
			'deepseek-v4-1-flash-1017'
		);
		const d = newDraft(catalog, 'contents', 'contents.periodical', () => true);
		expect(d.model).toBe('gemini-3.8-flash');
		expect(d.mergeProvider).toBe('gemini');
		expect(mergeModelsFor(catalog, 'gemini').map((m) => m.id)).toContain(d.mergeModel);
		// Gemini Pro reads the pages, Mistral combines them.
		const mixed = { ...d, model: 'gemini-3.1-pro-preview', mergeProvider: 'mistral' };
		mixed.mergeModel = mergeDefault(catalog, 'mistral');
		const request = toRequest(catalog, 'contents', mixed);
		expect(request.merge).toEqual({ provider: 'mistral', model: mixed.mergeModel });
		expect(toRequest(catalog, 'analysis', mixed).merge).toBeUndefined();
	});

	it('takes the files each task reads', () => {
		expect(readsKind('contents', 'image')).toBe(true);
		expect(readsKind('contents', 'text')).toBe(false);
		expect(readsKind('text_correction', 'text')).toBe(true);
		expect(readsKind('text_correction', 'image')).toBe(false);
		expect(taskOf('text_correction')).toBe('text_correction');
	});
});
