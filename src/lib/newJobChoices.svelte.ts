// New job's choices: the task and the engine's rules for it, the set-ups (model and prompt),
// the languages, how the text is read, the pace, the sample and the formats, as the researcher
// leaves them; filled from a task picked or a saved pipeline, and given as the settings a job
// and a pipeline store. A set-up follows the files while the researcher has not chosen for it.

import { untrack } from 'svelte';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { LineBreaks } from './bindings/LineBreaks.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { OutputDestination } from './bindings/OutputDestination.js';
import type { Pace } from './bindings/Pace.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Preset } from './bindings/Preset.js';
import { messages } from './i18n/index.js';
import { modelsNotDetecting, takesLanguage } from './languages.js';
import {
	draftsOf,
	formPipeline,
	inputOf,
	jobSettings,
	type Mode,
	type NewJobForm,
	pagesPerCallOf,
	readsKind,
	rulesFor,
	suggestedPreset,
	taskOf,
	togetherOf,
	type Together
} from './newJob.js';
import type { JobFiles } from './newJobFiles.svelte.js';
import {
	acceptsPrompt,
	type Draft,
	modelInfo,
	newDraft,
	providerDefault,
	readsInput
} from './newJobModels.js';
import { allRecords, readingRule } from './records.js';
import { app, hasKey, missingLanguageData, toast } from './state.svelte.js';
import { api } from './tauri.js';

export class JobChoices {
	readonly #files: JobFiles;
	readonly #m = messages();
	mode = $state<Mode>('text_extraction');
	readonly task = $derived(taskOf(this.mode));
	/** The engine's rules for the task: which files, models and formats suit it. */
	readonly taskRule = $derived(rulesFor(app.rules, this.task));
	/** What every model must read: YouTube links for videos, images when there are images,
	 * else anything the task takes. */
	readonly input = $derived.by(() => inputOf(this.mode, this.#files.list));
	languageMode = $state<'auto' | 'explicit'>('auto');
	languages = $state<string[]>([]);
	lineBreaks = $state<LineBreaks>('running_text');
	pace = $state<Pace>('fast');
	drafts = $state<Draft[]>([]);
	presets = $state<Preset[]>([]);
	/** The chosen analyses' fields, one for each set-up. */
	readonly schemas = $derived(
		this.drafts.map((d) => this.presets.find((p) => p.id === d.presetId)?.schema)
	);
	/** The task's rules, and for records chosen in every set-up what the task offers read page
	 * by page. */
	readonly rule = $derived(readingRule(this.taskRule, this.schemas));
	/** Records chosen in every set-up: one row per entry, read page by page. */
	readonly records = $derived(this.task === 'analysis' && allRecords(this.schemas));
	sample = $state(false);
	samplePages = $state(3);
	/** How many pages of a PDF go in one call: one, several, or as many as fit. */
	together = $state<Together>('one');
	togetherPages = $state(4);
	formats = $state<ExportFormat[]>([]);
	destination = $state<OutputDestination>({ kind: 'beside_sources' });
	readonly catalog = $derived(app.catalog!);
	readonly models = $derived(this.drafts.map((d) => modelInfo(this.catalog, d)));
	/** Distinct names of the chosen models matching a test, for the notes under a choice. */
	readonly namesOf = (test: (model: ModelInfo | undefined) => boolean) => [
		...new Set(this.models.filter(test).map((model) => model?.name ?? ''))
	];
	readonly languageIgnored = $derived(this.namesOf((model) => !takesLanguage(model)));
	readonly noPrompt = $derived(this.namesOf((model) => !acceptsPrompt(model)));
	/** Where samples and line breaks apply, as the engine says. */
	readonly paged = $derived(this.rule.samples);
	readonly linesApply = $derived(this.rule.line_breaks);
	/** What the job sends for it; nothing for one page at a time, as before. */
	readonly pagesPerCall = $derived(pagesPerCallOf(this.rule, this.together, this.togetherPages));
	/** A model that cannot detect the language needs the languages chosen and downloaded; a
	 * translation, the language it translates into. */
	readonly languagesReady = $derived(
		(modelsNotDetecting(this.models).length === 0 ||
			(this.languageMode === 'explicit' &&
				this.languages.length > 0 &&
				missingLanguageData(this.languages).length === 0)) &&
			(!this.rule.needs_language || (this.languageMode === 'explicit' && this.languages.length > 0))
	);

	/** The prompt Kalamu last chose for the files; a set-up still holding it was not chosen by the
	 * researcher, and follows the files. Not state: it only remembers what was chosen. */
	#autoPreset: string | null = null;
	/** Whether records were chosen when last looked at; null until the task's prompts are read,
	 * or once a kept form or a pipeline has filled the form, which keeps the formats it has. Not
	 * state: it only remembers. */
	#recordsBefore: boolean | null = null;
	/** The spreadsheet was ticked for the records, not by the researcher. */
	#sheetForRecords = false;
	/** Raised by each task picked, so the prompts of a task picked earlier and read later are
	 * not put under the one picked since. */
	#taskTicket = 0;

	/** Called while New job is set up, so its effects end with it. */
	constructor(files: JobFiles, kept: NewJobForm | null) {
		this.#files = files;
		this.mode = kept?.mode ?? 'text_extraction';
		this.languageMode = kept?.languageMode ?? 'auto';
		this.languages = kept?.languages ?? [];
		this.lineBreaks = kept?.lineBreaks ?? 'running_text';
		this.pace = kept?.pace ?? 'fast';
		this.drafts = kept?.drafts ?? [];
		this.sample = kept?.sample ?? false;
		this.samplePages = kept?.samplePages ?? 3;
		this.together = kept?.together ?? 'one';
		this.togetherPages = kept?.togetherPages ?? 4;
		this.formats = kept?.formats ?? [];
		this.destination = kept?.destination ?? { kind: 'beside_sources' };

		// A set-up chosen before images were added leaves a model that cannot read them for its
		// provider's default among those that can, rather than leaving Start refused unsaid.
		$effect(() => {
			for (const d of this.drafts) {
				if (!d.model || readsInput(modelInfo(this.catalog, d), this.input)) continue;
				const next = providerDefault(this.catalog, this.rule, d.provider, this.input);
				if (next) d.model = next;
			}
		});
		// Files that are all images suggest the prompt made for images (and others the task's
		// first), in every set-up whose prompt is still Kalamu's choice; one the researcher chose
		// stays.
		$effect(() => {
			const next = suggestedPreset(this.presets, files.list);
			if (next === this.#autoPreset) return;
			for (const d of this.drafts) if (d.presetId === this.#autoPreset) d.presetId = next;
			this.#autoPreset = next;
		});
		// Records are read for their table: choosing them ticks the spreadsheet, and leaving them
		// takes back that tick. A spreadsheet ticked or unticked by hand stays as it is.
		$effect(() => {
			if (this.presets.length === 0) return;
			const now = this.records;
			const before = this.#recordsBefore;
			this.#recordsBefore = now;
			if (before === null || now === before) return;
			untrack(() => {
				if (now && !this.formats.includes('xlsx') && this.rule.formats.includes('xlsx')) {
					this.formats = [...this.formats, 'xlsx'];
					this.#sheetForRecords = true;
				} else if (!now && this.#sheetForRecords) {
					this.formats = this.formats.filter((f) => f !== 'xlsx');
					this.#sheetForRecords = false;
				} else {
					this.#sheetForRecords = false;
				}
			});
		});
	}

	async loadTask(next: Mode) {
		const files = this.#files;
		if (next !== this.mode) files.setAside = null;
		this.mode = next;
		const ticket = ++this.#taskTicket;
		const listed = await api.listPresets(this.task);
		if (ticket !== this.#taskTicket) return;
		this.presets = listed;
		this.#autoPreset = suggestedPreset(this.presets, files.list);
		this.drafts = [newDraft(this.catalog, this.rule, this.#autoPreset, hasKey, [], this.input)];
		// A table of contents reads best as a document and a spreadsheet for Excel.
		this.formats =
			this.task === 'contents'
				? ['markdown', 'xlsx']
				: [
						...(this.task === 'audio_transcription'
							? app.settings!.audio_formats
							: app.settings!.text_formats)
					].filter((f) => this.rule.formats.includes(f));
		// The usual formats, before any records: the prompt chosen with them may tick the
		// spreadsheet.
		this.#recordsBefore = false;
		this.#sheetForRecords = false;
		// An analysis or a table of contents is written in one language.
		if (this.rule.answers_in_language) this.languages = this.languages.slice(0, 1);
		// A video has no folder of its own: offer the folder last chosen.
		const last = app.settings!.last_folder;
		if (this.mode === 'youtube' && this.destination.kind !== 'folder' && last) {
			this.destination = { kind: 'folder', path: last };
		}
		files.list = files.list.filter((f) => readsKind(app.rules, this.mode, f.source?.kind));
	}

	/** Fills the form from a saved pipeline; the files stay as they are. */
	async applyPipeline(p: Pipeline) {
		// A transcription pipeline applied to videos keeps the videos.
		await this.loadTask(
			this.mode === 'youtube' && p.task === 'audio_transcription' ? 'youtube' : p.task
		);
		this.drafts = draftsOf(p, this.catalog);
		this.languageMode = p.settings.languages.mode;
		this.languages =
			p.settings.languages.mode === 'explicit' ? [...p.settings.languages.languages] : [];
		this.lineBreaks = p.settings.line_breaks ?? 'running_text';
		this.pace = p.settings.pace ?? 'fast';
		const sent = togetherOf(p.settings.pages_per_call);
		this.together = sent.together;
		if (sent.pages !== undefined) this.togetherPages = sent.pages;
		this.sample = p.settings.sample_pages !== null;
		this.samplePages = p.settings.sample_pages ?? this.samplePages;
		this.formats = p.formats.filter((f) => rulesFor(app.rules, p.task).formats.includes(f));
		// The pipeline's formats stand, records or not.
		this.#recordsBefore = null;
		this.#sheetForRecords = false;
		this.destination = p.destination;
		toast(this.#m.newJob.pipelineApplied(p.name));
	}

	/** The settings shared by every set-up, as a job and a pipeline store them, at `pace`. */
	settings(pace: Pace) {
		return jobSettings({
			languageMode: this.languageMode,
			languages: this.languages,
			languagesTaken: this.languageIgnored.length < this.models.length,
			sample: this.sample,
			samplePages: this.samplePages,
			paged: this.paged,
			lineBreaks: this.lineBreaks,
			linesTaken: this.linesApply && this.noPrompt.length < this.models.length,
			pace,
			pagesPerCall: this.pagesPerCall
		});
	}

	/** The form as a pipeline: everything but the files. Run now, it keeps only the formats the
	 * chosen models can write, as a job does. */
	pipeline(name: string, pace: Pace, toRun = false): Pipeline {
		return formPipeline(
			{
				name,
				task: this.task,
				catalog: this.catalog,
				rule: this.rule,
				drafts: this.drafts,
				settings: this.settings(pace),
				formats: this.formats,
				destination: $state.snapshot(this.destination)
			},
			toRun
		);
	}
}
