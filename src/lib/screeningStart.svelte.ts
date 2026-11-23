// "Which of these speak of my question?": the chosen documents of a project, each in the text it
// was read in (frozen as *Use this text* freezes it, or a text file's own words), read by the
// ready-made screening prompt with the researcher's question, in one analysis job tied to the
// project. The model is the one New job would choose for an analysis; another one or the
// language of the answers only under "More choices". Start asks what New job's Start asks
// (consent to send to the provider, folders on the Mac App Store), then creates and starts the
// job as New job does (`launch`), and its page opens. Asked again of the documents added since,
// it starts from the question, the prompt, the model and the languages it was asked with.

import type { LanguageChoice } from './bindings/LanguageChoice.js';
import type { NewJob } from './bindings/NewJob.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { SourceFile } from './bindings/SourceFile.js';
import { providerNameOf } from './catalogText.js';
import { mayStart } from './dialogs.js';
import { draftOf, dropDraft, keepDraft } from './drafts.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { takesLanguage } from './languages.js';
import { jobSettings, offeredFormats, pagesPerCallOf, rulesFor } from './newJob.js';
import { modelInfo, newDraft, toRequest, type Draft } from './newJobModels.js';
import { launch } from './newJobStart.svelte.js';
import { suggestedName } from './research.js';
import { SCREENING_PRESET, screeningRequest } from './screening.js';
import { app, go, hasKey, type FromProject } from './state.svelte.js';
import { api } from './tauri.js';

/** A question kept with the project, to be asked again as it was asked. */
export interface Again {
	question: string;
	presetId: string | null;
	provider: string;
	model: string;
	languages: LanguageChoice;
	/** The fingerprint of the prompt it was asked with, for the dialog to say when the prompt
	 * has changed since. */
	prompt: string;
}

export class Screening {
	question = $state('');
	draft = $state<Draft>(
		newDraft(app.catalog!, rulesFor(app.rules, 'analysis'), SCREENING_PRESET, hasKey)
	);
	languageMode = $state<'auto' | 'explicit'>('auto');
	languages = $state<string[]>([]);
	/** Where the question typed waits when the dialog closes without starting (`drafts.ts`):
	 * one for each project, apart from a question asked again. */
	readonly #draftKey: string;
	/** Set once the job is made: the question typed is then spent, and kept no longer. */
	#started = false;
	constructor(again?: Again, projectId = '') {
		this.#draftKey = `screening:${projectId}:${again ? 'again' : 'new'}`;
		const typed = draftOf<string>(this.#draftKey);
		if (typed !== null) this.question = typed;
		if (!again) return;
		this.question = typed ?? again.question;
		this.draft = {
			...newDraft(
				app.catalog!,
				rulesFor(app.rules, 'analysis'),
				again.presetId ?? SCREENING_PRESET,
				hasKey
			),
			provider: again.provider,
			model: again.model
		};
		if (again.languages.mode === 'explicit') {
			this.languageMode = 'explicit';
			this.languages = [...again.languages.languages];
		}
	}

	/** The documents' texts, frozen when the dialog opens: what the estimate prices and the job
	 * reads. */
	sources = $state<SourceFile[] | null>(null);
	/** Documents left out, by title: nothing read into text yet, or several readings and none
	 * preferred. */
	noText = $state<string[]>([]);
	undecided = $state<string[]>([]);
	/** What the job would cost, in dollars, when the engine can tell. */
	cost = $state<number | null>(null);
	/** The documents the model chosen cannot read even in parts, left out of the job, and the
	 * sentence that names them; none when all can be read. */
	tooLong = $state<{ sources: string[]; said: string } | null>(null);
	error = $state<string | null>(null);
	busy = $state(false);
	/** Asked again, the prompt would not be the one the question was asked with: its preset
	 * was edited since, or the language of the answers changed, so the answers will stand apart. */
	promptChanged = $state(false);

	readonly catalog = $derived(app.catalog!);
	readonly rule = $derived(rulesFor(app.rules, 'analysis'));
	readonly model = $derived(modelInfo(this.catalog, this.draft));
	readonly providerName = $derived(providerNameOf(this.catalog, this.draft.provider));
	/** The provider can be sent the text: its key is saved. */
	readonly keyed = $derived(hasKey(this.draft.provider));
	/** The texts the job will read: those the model chosen can read. */
	readonly readable: SourceFile[] = $derived(
		(this.sources ?? []).filter((s) => !this.tooLong?.sources.includes(s.id))
	);
	readonly request: SetupRequest = $derived(
		screeningRequest(toRequest(this.catalog, 'analysis', this.draft), this.question)
	);
	/** Start can be pressed: a question, texts to read, a model the provider can be sent to. */
	readonly ready = $derived(
		!!this.question.trim() &&
			this.readable.length > 0 &&
			this.keyed &&
			!!this.draft.model &&
			!this.busy
	);

	/** Keeps the question typed for the dialog's next opening, as it closes without starting;
	 * a question left as it opened keeps nothing. */
	saveDraft(again?: Again): void {
		if (this.#started) return;
		const typed = this.question.trim() !== '' && this.question !== (again?.question ?? '');
		keepDraft(this.#draftKey, typed ? this.question : null);
	}

	/** Freezes the chosen documents' texts. */
	async prepare(projectId: string, documents: string[]): Promise<void> {
		try {
			const found = await api.documentSources(projectId, documents);
			this.noText = found.no_text;
			this.undecided = found.undecided;
			this.sources = found.sources;
		} catch (e) {
			this.error = describeError(e, messages());
		}
	}

	/** Asked again, whether the prompt the job would freeze differs from the one the question
	 * was first asked with (`promptChanged`). Only for the same question: another one is another
	 * group anyway. Read synchronously up to the call, so an effect follows the choices. */
	async checkPrompt(again?: Again): Promise<void> {
		if (!again || this.question.trim() !== again.question.trim()) {
			this.promptChanged = false;
			return;
		}
		const job = this.job([]);
		try {
			const [frozen] = await api.promptFingerprints(job);
			this.promptChanged = frozen !== undefined && frozen !== again.prompt;
		} catch {
			// A model or prompt the engine refuses: Start says why; nothing to compare.
			this.promptChanged = false;
		}
	}

	/** The job as New job would make it from these choices. */
	job(sources: SourceFile[]): NewJob {
		const m = messages();
		return {
			name: m.screening.jobName(suggestedName(this.question)),
			task: 'analysis',
			sources,
			setups: [this.request],
			settings: jobSettings({
				languageMode: this.languageMode,
				languages: this.languages,
				languagesTaken: takesLanguage(this.model),
				sample: false,
				samplePages: 3,
				paged: false,
				lineBreaks: 'running_text',
				linesTaken: false,
				pace: 'fast',
				pagesPerCall: pagesPerCallOf(this.rule, 'one', 1)
			}),
			output: {
				formats: app.settings!.text_formats.filter((f) =>
					offeredFormats(this.catalog, this.rule, [this.draft], sources).includes(f)
				),
				destination: { kind: 'beside_sources' },
				options: $state.snapshot(app.settings!.export_options)
			}
		};
	}

	/** Creates and starts the job, tied to the project, then opens its page; false when the
	 * researcher declined to send the texts, or the engine refused (the reason in `error`). */
	async start(project: FromProject): Promise<boolean> {
		const sources = $state.snapshot(this.readable);
		if (sources.length === 0 || !this.ready) return false;
		this.busy = true;
		this.error = null;
		try {
			const request = $state.snapshot(this.request) as SetupRequest;
			if (!(await mayStart([request], this.catalog, sources, true))) return false;
			const job = await api.createJob(this.job(sources));
			this.#started = true;
			dropDraft(this.#draftKey);
			await launch(job.id, project);
			go({ name: 'jobs', jobId: job.id });
			return true;
		} catch (e) {
			this.error = describeError(e, messages());
			return false;
		} finally {
			this.busy = false;
		}
	}
}
