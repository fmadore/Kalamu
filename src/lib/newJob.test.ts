import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { SourceFile } from './bindings/SourceFile.js';
import type { Task } from './bindings/Task.js';
import rulesJson from './dev/taskRules.json';
import { en } from './i18n/en.js';
import { fr } from './i18n/fr.js';
import type { Preset } from './bindings/Preset.js';
import type { Refused } from './bindings/Refused.js';
import {
	suggestedPreset,
	documentExtensions,
	draftsOf,
	followingAnalysis,
	forgetForm,
	formPipeline,
	formToOpen,
	jobSettings,
	keepForm,
	LAYOUT_FORMATS,
	nameOfFile,
	offeredFormats,
	pagesPerCallOf,
	readersOf,
	readsKind,
	type Rules,
	rulesFor,
	suggestName,
	suggestionFits,
	taskExtensions,
	taskOf,
	TIMED_FORMATS,
	togetherOf,
	type NewJobForm
} from './newJob.js';
import { modelsFor, newDraft, toRequest } from './newJobModels.js';

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
/** The engine's rules for the bundled catalogue, written by `cargo test`. */
const rules = rulesJson as Rules;
const rule = (task: Task) => rulesFor(rules, task);

describe('new job form', () => {
	it('names jobs after what was added', () => {
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

	it('suggests the prompt made for images when every file is one', () => {
		const inspected = (kind: SourceFile['kind']) =>
			({ path: `a.${kind}`, source: { kind } as SourceFile }) as InspectedFile;
		const preset = (id: string, made_for?: SourceFile['kind']): Preset => ({
			id,
			task: 'analysis',
			name: id,
			description: '',
			instructions: '',
			user_prompt: '',
			builtin: true,
			modified: false,
			...(made_for ? { made_for } : {})
		});
		const presets = [
			preset('analysis.summary_research'),
			preset('analysis.image_description', 'image')
		];
		expect(suggestedPreset(presets, [])).toBe('analysis.summary_research');
		expect(suggestedPreset(presets, [inspected('image'), inspected('image')])).toBe(
			'analysis.image_description'
		);
		expect(suggestedPreset(presets, [inspected('image'), inspected('pdf')])).toBe(
			'analysis.summary_research'
		);
		expect(suggestedPreset([], [inspected('image')])).toBeNull();
	});
});

describe('tables of contents and corrections', () => {
	it('asks for the language of the answers where the model writes them', () => {
		expect(rule('analysis').answers_in_language).toBe(true);
		expect(rule('contents').answers_in_language).toBe(true);
		expect(rule('text_correction').answers_in_language).toBe(false);
		expect(rule('text_extraction').answers_in_language).toBe(false);
		// A translation answers in the language chosen, which must be chosen.
		expect(rule('translation').answers_in_language).toBe(true);
		expect(rule('translation').needs_language).toBe(true);
		expect(rule('analysis').needs_language).toBe(false);
		expect(rule('translation').line_breaks).toBe(false);
	});

	it('takes the files each task reads', () => {
		expect(readsKind(rules, 'contents', 'image')).toBe(true);
		expect(readsKind(rules, 'contents', 'text')).toBe(false);
		expect(readsKind(rules, 'text_correction', 'text')).toBe(true);
		expect(readsKind(rules, 'text_correction', 'image')).toBe(false);
		expect(taskOf('text_correction')).toBe('text_correction');
		expect(readsKind(rules, 'translation', 'text')).toBe(true);
		expect(readsKind(rules, 'translation', 'pdf')).toBe(true);
		expect(readsKind(rules, 'translation', 'image')).toBe(false);
		expect(modelsFor(catalog, rule('translation'), 'mistral').map((m) => m.id)).toEqual(
			modelsFor(catalog, rule('text_correction'), 'mistral').map((m) => m.id)
		);
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

describe('the New job form kept between visits', () => {
	const form: NewJobForm = {
		mode: 'analysis',
		files: [],
		frozen: false,
		project: undefined,
		languageMode: 'auto',
		languages: [],
		lineBreaks: 'running_text',
		pace: 'fast',
		drafts: [],
		sample: false,
		samplePages: 3,
		together: 'one',
		togetherPages: 4,
		formats: ['markdown'],
		destination: { kind: 'beside_sources' },
		name: 'Letters',
		suggested: '',
		thenPreset: null,
		thenCheck: true
	};

	it('comes back when New job opens plainly, until forgotten', () => {
		keepForm(form);
		expect(formToOpen({})).toBe(form);
		expect(formToOpen({})).toBe(form);
		forgetForm();
		expect(formToOpen({})).toBeNull();
	});

	it('gives way to anything new to start from, and does not come back after it', () => {
		for (const opening of [
			{ task: 'analysis' as const },
			{ paths: ['a.pdf'] },
			{ sources: [] },
			{ project: { id: 'p', name: 'Project' } }
		]) {
			keepForm(form);
			expect(formToOpen(opening)).toBeNull();
			expect(formToOpen({})).toBeNull();
		}
	});
});

describe('the files each task’s picker offers', () => {
	const PAGES = ['pdf', 'png', 'jpg', 'jpeg', 'tif', 'tiff', 'webp', 'bmp', 'heic', 'heif'];
	const TEXT = ['txt', 'md', 'docx', 'csv'];

	it('offers the extensions of each kind of source the task reads, as Rust lists them', () => {
		expect(taskExtensions('text_extraction')).toEqual(PAGES);
		expect(taskExtensions('contents')).toEqual(PAGES);
		expect(taskExtensions('text_correction')).toEqual([...TEXT, 'pdf']);
		expect(taskExtensions('translation')).toEqual([...TEXT, 'pdf']);
		expect(taskExtensions('analysis')).toEqual([...TEXT, ...PAGES]);
		const recordings = taskExtensions('audio_transcription');
		for (const ext of ['mp3', 'aiff', 'aif', 'wma', 'mp4', 'webm', 'wmv', 'm4v']) {
			expect(recordings).toContain(ext);
		}
		expect(recordings).not.toContain('pdf');
	});

	it('agrees with the kinds the engine’s rules say each task reads', () => {
		for (const r of rules) {
			const offered = taskExtensions(r.task);
			expect(offered.length > 0).toBe(true);
			if (!r.sources.includes('text')) expect(offered).not.toContain('txt');
			if (!r.sources.includes('pdf')) expect(offered).not.toContain('pdf');
			if (!r.sources.includes('audio')) expect(offered).not.toContain('mp3');
		}
	});

	it('never offers a spreadsheet, which no task reads', () => {
		expect(documentExtensions()).not.toContain('xlsx');
	});

	it('names no spreadsheet among the files Kalamu reads, in every language', () => {
		for (const m of [en, fr]) expect(m.newJob.setAside.kinds).not.toMatch(/Excel|xlsx/i);
	});

	it('offers a project every file some task reads, once each', () => {
		const all = documentExtensions();
		expect(new Set(all).size).toBe(all.length);
		for (const ext of [...PAGES, ...TEXT, 'mp3', 'mov']) expect(all).toContain(ext);
	});

	it('names the formats that need a model reporting layout, from Rust', () => {
		expect(LAYOUT_FORMATS).toEqual(['searchable_pdf', 'alto']);
		expect(TIMED_FORMATS).toEqual(['srt', 'vtt']);
	});

	it('offers a translation subtitles only of a transcript sent on with its times', () => {
		const text = { file_name: 'lettre.txt', kind: 'text' } as SourceFile;
		const timed = {
			file_name: 'entretien.txt',
			kind: 'text',
			derived_from: { timed: true }
		} as SourceFile;
		const offered = (sources?: SourceFile[]) =>
			offeredFormats(catalog, rule('translation'), [], sources);
		expect(offered([text])).not.toContain('srt');
		expect(offered([text])).toContain('docx');
		expect(offered([text, timed])).toEqual(expect.arrayContaining(['srt', 'vtt']));
		// A pipeline's texts come later: the engine leaves out what has no times.
		expect(offered()).toContain('vtt');
		// A transcription always has its own.
		expect(offeredFormats(catalog, rule('audio_transcription'), [], [])).toContain('srt');
	});
});
