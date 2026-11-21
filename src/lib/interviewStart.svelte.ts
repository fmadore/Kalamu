// "Analyse this interview": a transcript's text, frozen as the researcher corrected it, read by
// the ready-made interview prompt with their question, in an analysis job of its own. The model
// is the one New job would choose for an analysis (`newDraft`), another one or the language of
// the answers only under "More choices". Start asks what New job's Start asks (consent to send
// to the provider, folders on the Mac App Store), then creates and starts the job as New job
// does (`launch`), tied to the project it was opened from. A description of the research can be
// kept as a prompt of the researcher's own and chosen again (`research.ts`). A transcript too long
// for the model chosen is said where the cost would be, from the same estimate, and Start waits
// for another model (`tooLongOf`).

import type { Catalog } from './bindings/Catalog.js';
import type { Estimate } from './bindings/Estimate.js';
import type { NewJob } from './bindings/NewJob.js';
import type { Preset } from './bindings/Preset.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { SourceFile } from './bindings/SourceFile.js';
import { modelNameOf, providerNameOf } from './catalogText.js';
import { mayStart } from './dialogs.js';
import { describeError } from './errors.js';
import { messages, type Messages } from './i18n/index.js';
import { takesLanguage } from './languages.js';
import { jobSettings, offeredFormats, pagesPerCallOf, rulesFor } from './newJob.js';
import { modelInfo, newDraft, toRequest, type Draft } from './newJobModels.js';
import { launch } from './newJobStart.svelte.js';
import {
	firstChoice,
	keepChoice,
	keptPrompt,
	lastChoice,
	NEW_RESEARCH,
	researchPrompts,
	suggestedName
} from './research.js';
import { app, go, hasKey, type FromProject } from './state.svelte.js';
import { api } from './tauri.js';

/** The ready-made prompt that reads an interview for its themes and quotations. */
export const INTERVIEW_PRESET = 'analysis.interview';

/** The setup an interview is analysed with: the draft's, the researcher's question and their
 * own themes added when they typed them (each left out when empty, as the engine's cache keys
 * expect). */
export function interviewRequest(
	request: SetupRequest,
	question: string,
	themes = ''
): SetupRequest {
	const asked = question.trim();
	const listed = themes.trim();
	if (!asked && !listed) return request;
	return {
		...request,
		options: {
			...request.options,
			...(asked ? { question: asked } : {}),
			...(listed ? { themes: listed } : {})
		}
	};
}

/** The researcher's themes typed for this interview, added to a kept description's setup (they
 * replace the prompt's own); the setup as it is when none were typed. */
export function withThemes(request: SetupRequest, themes: string): SetupRequest {
	const listed = themes.trim();
	return listed ? { ...request, options: { ...request.options, themes: listed } } : request;
}

/** The setup with a kept description's prompt in place of the ready-made one, whose question
 * the engine sends as if it had been typed. */
export function withPrompt(request: SetupRequest, presetId: string): SetupRequest {
	return request.preset_id === null ? request : { ...request, preset_id: presetId };
}

/** The sentence saying the transcript is too long for a model, when the estimate says so: the
 * engine measured the text it would send, and would refuse the job for it. */
export function tooLongOf(
	estimate: Estimate | null,
	catalog: Catalog,
	m: Messages = messages()
): string | null {
	const row = estimate?.rows.find((r) => r.too_long?.length);
	const long = row?.too_long?.[0];
	if (!row || !long) return null;
	const count = new Intl.NumberFormat(m.locale.tag);
	return m.interview.tooLong(
		modelNameOf(catalog, row.provider, row.model),
		count.format(long.words),
		count.format(long.most_words)
	);
}

export class InterviewAnalysis {
	question = $state('');
	/** The researcher's own themes or interview guide, one per line, under More choices. */
	themes = $state('');
	/** The analysis prompts, read as the dialog opens: the ready-made one, and the researcher's
	 * kept descriptions among them. */
	presets = $state<Preset[]>([]);
	/** A kept description's prompt, or `NEW_RESEARCH` for the box. */
	choice = $state(NEW_RESEARCH);
	/** Keep what is typed as a prompt of the researcher's own, under `keptName`. */
	keep = $state(false);
	keptName = $state('');
	draft = $state<Draft>(
		newDraft(app.catalog!, rulesFor(app.rules, 'analysis'), INTERVIEW_PRESET, hasKey)
	);
	languageMode = $state<'auto' | 'explicit'>('auto');
	languages = $state<string[]>([]);
	/** The transcript's text, frozen when the dialog opens: what the estimate prices and the job
	 * reads. */
	sources = $state<SourceFile[] | null>(null);
	/** What the job would cost, in dollars, when the engine can tell. */
	cost = $state<number | null>(null);
	/** The transcript is too long for the model chosen, in a sentence; none when it fits. */
	tooLong = $state<string | null>(null);
	/** How many parts a transcript too long to be read at once will be read in; none when it
	 * is read whole. */
	parts = $state<number | null>(null);
	error = $state<string | null>(null);
	busy = $state(false);

	readonly catalog = $derived(app.catalog!);
	readonly rule = $derived(rulesFor(app.rules, 'analysis'));
	readonly model = $derived(modelInfo(this.catalog, this.draft));
	readonly providerName = $derived(providerNameOf(this.catalog, this.draft.provider));
	/** The provider can be sent the text: its key is saved. */
	readonly keyed = $derived(hasKey(this.draft.provider));
	/** The kept descriptions the dialog offers; none on first use. */
	readonly kept = $derived(researchPrompts(this.presets));
	/** The kept description chosen, if one is. */
	readonly chosen = $derived(this.kept.find((p) => p.id === this.choice) ?? null);
	readonly request = $derived.by(() => {
		const request = toRequest(this.catalog, 'analysis', this.draft);
		return this.chosen
			? withThemes(withPrompt(request, this.chosen.id), this.themes)
			: interviewRequest(request, this.question, this.themes);
	});

	/** Freezes the transcript's text, every source's or one source's, as the other choices of
	 * "Use this text" do. */
	async prepare(jobId: string, setupId: string, sourceId: string | null): Promise<void> {
		void this.readPrompts();
		try {
			this.sources = await api.snapshotJob(jobId, setupId, sourceId);
		} catch (e) {
			this.error = describeError(e, messages());
		}
	}

	/** Reads the analysis prompts and opens on the description used last. Without them the
	 * dialog is as on first use: one box. */
	async readPrompts(): Promise<void> {
		try {
			this.presets = await api.listPresets('analysis');
		} catch {
			this.presets = [];
		}
		this.choice = firstChoice(this.kept, lastChoice());
		this.themes = this.chosen?.themes ?? '';
	}

	/** Chooses a kept description, or a new one: the box of themes shows the kept themes, to
	 * change for this interview. */
	choose(choice: string): void {
		this.choice = choice;
		this.themes = this.chosen?.themes ?? '';
	}

	/** Ticks "Keep this…", with a name to start from when none is written yet. */
	setKeep(on: boolean, project: string | undefined): void {
		this.keep = on;
		if (on && !this.keptName.trim()) this.keptName = suggestedName(this.question, project);
	}

	/** Saves what was typed as a prompt of the researcher's own, when they asked to keep it;
	 * the setup to start then names it. */
	async keepTyped(request: SetupRequest, project: string | undefined): Promise<SetupRequest> {
		const base = this.presets.find((p) => p.id === INTERVIEW_PRESET);
		if (this.chosen || !this.keep || !this.question.trim() || !base) return request;
		const name = this.keptName.trim() || suggestedName(this.question, project);
		const saved = await api.savePreset(
			keptPrompt($state.snapshot(base), name, this.question, this.themes)
		);
		this.presets = [...this.presets, saved];
		this.choice = saved.id;
		return withThemes(withPrompt(request, saved.id), this.themes);
	}

	/** The job as New job would make it from these choices. */
	job(sources: SourceFile[], request: SetupRequest = this.request): NewJob {
		const m = messages();
		const draft = this.draft;
		return {
			name: m.interview.jobName(sources[0]?.file_name ?? ''),
			task: 'analysis',
			sources,
			setups: [request],
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
					offeredFormats(this.catalog, this.rule, [draft], sources).includes(f)
				),
				destination: { kind: 'beside_sources' },
				options: $state.snapshot(app.settings!.export_options)
			}
		};
	}

	/** Creates and starts the analysis, then opens its page; false when the researcher declined
	 * to send the text, or the engine refused (the reason in `error`). */
	async start(project: FromProject | undefined): Promise<boolean> {
		const sources = $state.snapshot(this.sources);
		if (!sources || this.busy || this.tooLong) return false;
		this.busy = true;
		this.error = null;
		try {
			if (!(await mayStart([this.request], this.catalog, sources, true))) return false;
			const request = await this.keepTyped($state.snapshot(this.request), project?.name);
			keepChoice(this.choice);
			const job = await api.createJob(this.job(sources, request));
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
