import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { Task } from './bindings/Task.js';
import rulesJson from './dev/taskRules.json';
import { en } from './i18n/index.js';
import type { Preset } from './bindings/Preset.js';
import type { Refused } from './bindings/Refused.js';
import {
	acceptsPrompt,
	dictionaryNames,
	draftsOf,
	followingAnalysis,
	formPipeline,
	inputOf,
	jobSettings,
	mergeDefault,
	mergeModelsFor,
	modelInfo,
	modelsFor,
	nameOfFile,
	newDraft,
	pagesPerCallOf,
	providerDefault,
	readersOf,
	readsKind,
	type Rules,
	rulesFor,
	suggestName,
	suggestionFits,
	taskOf,
	togetherOf,
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
		// Google's speech-to-text model for recordings; it takes no prompt.
		const transcribe = providerDefault(
			catalog,
			rule('audio_transcription'),
			'gemini',
			'audio/mpeg'
		);
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

	it('suggests the name again once the file it came from is removed', () => {
		const entry = (path: string): InspectedFile => ({
			path,
			source: { file_name: path.split('\\').pop()! } as SourceFile,
			warnings: [],
			error: null,
			pieces: null
		});
		const issue = entry('C:\\Newspapers\\1911-04-05.pdf');
		expect(suggestionFits('bulletin-page.jpg', [issue], false)).toBe(false);
		expect(suggestionFits('1911-04-05.pdf', [issue], false)).toBe(true);
		expect(suggestionFits('Newspapers', [issue], false)).toBe(true);
		expect(suggestionFits('anything', [], false)).toBe(false);
		const frozen = entry('C:\\data\\1911-04-05.txt');
		expect(nameOfFile(frozen, true)).toBe('1911-04-05');
		expect(suggestionFits('1911-02-15', [frozen], true)).toBe(false);
	});

	it('offers the tasks that read files dropped on the wrong one', () => {
		const file = (file_name: string, kind: Refused['kind'], folder = false): Refused => ({
			path: `C:/data/${file_name}`,
			file_name,
			kind,
			folder
		});
		const interview = file('interview.mp3', 'audio');
		expect(readersOf(rules, [interview], 'text_extraction')).toEqual(['audio_transcription']);
		// A scan dropped on a transcription can be read, corrected… whatever reads images.
		const offered = readersOf(rules, [file('page.jpg', 'image')], 'audio_transcription');
		expect(offered).toContain('text_extraction');
		expect(offered).not.toContain('audio_transcription');
		// A recording and a scan together: no one task reads both.
		expect(readersOf(rules, [interview, file('page.jpg', 'image')], 'analysis')).toEqual([]);
		// A folder or an archive: nothing to offer.
		expect(readersOf(rules, [file('letters.zip', null)], 'text_extraction')).toEqual([]);
		expect(readersOf(rules, [file('empty', null, true)], 'text_extraction')).toEqual([]);
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

describe('the New job form as a pipeline', () => {
	const settings = {
		languageMode: 'explicit' as const,
		languages: ['fr', 'mos'],
		languagesTaken: true,
		sample: true,
		samplePages: 3,
		paged: true,
		lineBreaks: 'as_on_page' as const,
		linesTaken: true,
		pace: 'economy' as const,
		pagesPerCall: undefined
	};

	it('sends pages together only where the task can and when asked', () => {
		const extraction = rule('text_extraction');
		expect(pagesPerCallOf(extraction, 'one', 4)).toBeUndefined();
		expect(pagesPerCallOf(extraction, 'several', 1.4)).toEqual({ kind: 'several', pages: 2 });
		expect(pagesPerCallOf(extraction, 'whole', 4)).toEqual({ kind: 'whole' });
		expect(pagesPerCallOf(rule('analysis'), 'whole', 4)).toBeUndefined();
		expect(togetherOf(undefined)).toEqual({ together: 'one' });
		expect(togetherOf({ kind: 'several', pages: 6 })).toEqual({ together: 'several', pages: 6 });
		expect(togetherOf({ kind: 'whole' })).toEqual({ together: 'whole' });
	});

	it('keeps the settings that apply, and leaves out those no model takes', () => {
		expect(jobSettings(settings)).toEqual({
			languages: { mode: 'explicit', languages: ['fr', 'mos'] },
			sample_pages: 3,
			line_breaks: 'as_on_page',
			pace: 'economy',
			pages_per_call: undefined
		});
		expect(
			jobSettings({ ...settings, languagesTaken: false, paged: false, linesTaken: false })
		).toMatchObject({ languages: { mode: 'auto' }, sample_pages: null, line_breaks: null });
	});

	it('saves the form as a pipeline, and runs only the formats its models can write', () => {
		const d = newDraft(catalog, rule('text_extraction'), 'p', () => true);
		const form = {
			name: 'Bulletins',
			task: 'text_extraction' as const,
			catalog,
			rule: rule('text_extraction'),
			drafts: [d],
			settings: jobSettings(settings),
			formats: ['txt', 'alto'] as const satisfies string[],
			destination: { kind: 'beside_sources' as const }
		};
		const saved = formPipeline({ ...form, formats: [...form.formats] });
		expect(saved).toMatchObject({ id: '', name: 'Bulletins', formats: ['txt', 'alto'] });
		expect(saved.setups).toEqual([toRequest(catalog, 'text_extraction', d)]);
		// Applied again, the pipeline gives the same models and prompt.
		expect(draftsOf(saved)).toMatchObject([
			{
				provider: d.provider,
				model: d.model,
				presetId: 'p',
				label: '',
				mergeProvider: d.provider,
				mergeModel: d.model
			}
		]);
		const run = formPipeline({ ...form, formats: [...form.formats] }, true);
		const offered = run.formats;
		expect(offered).toContain('txt');
		expect(offered.every((f) => ['txt', 'alto'].includes(f))).toBe(true);
	});

	it('follows a reading with an analysis in the researcher’s usual formats', () => {
		const d = newDraft(catalog, rule('analysis'), null, () => true);
		const preset = { id: 'analysis.people', name: 'People' } as Preset;
		const then = followingAnalysis(
			preset,
			toRequest(catalog, 'analysis', d),
			rule('analysis'),
			['markdown', 'txt'],
			'fast',
			{ kind: 'beside_sources' }
		);
		expect(then).toMatchObject({ name: 'People', task: 'analysis' });
		expect(then.setups[0]).toMatchObject({ preset_id: 'analysis.people', label: null });
		expect(then.formats.every((f) => ['markdown', 'txt'].includes(f))).toBe(true);
		const none = followingAnalysis(preset, then.setups[0], rule('analysis'), [], 'fast', {
			kind: 'beside_sources'
		});
		expect(none.formats).toEqual(rule('analysis').formats.slice(0, 1));
	});
});
