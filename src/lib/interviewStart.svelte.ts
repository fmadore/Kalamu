// "Analyse this interview": a transcript's text, frozen as the researcher corrected it, read by
// the ready-made interview prompt with their question, in an analysis job of its own. The model
// is the one New job would choose for an analysis (`newDraft`), another one or the language of
// the answers only under "More choices". Start asks what New job's Start asks (consent to send
// to the provider, folders on the Mac App Store), then creates and starts the job as New job
// does (`launch`), tied to the project it was opened from.

import type { NewJob } from './bindings/NewJob.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { SourceFile } from './bindings/SourceFile.js';
import { providerNameOf } from './catalogText.js';
import { mayStart } from './dialogs.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { takesLanguage } from './languages.js';
import { jobSettings, offeredFormats, pagesPerCallOf, rulesFor } from './newJob.js';
import { modelInfo, newDraft, toRequest, type Draft } from './newJobModels.js';
import { launch } from './newJobStart.svelte.js';
import { app, go, hasKey, type FromProject } from './state.svelte.js';
import { api } from './tauri.js';

/** The ready-made prompt that reads an interview for its themes and quotations. */
export const INTERVIEW_PRESET = 'analysis.interview';

/** The setup an interview is analysed with: the draft's, the researcher's question added when
 * they typed one (left out when empty, as the engine's cache keys expect). */
export function interviewRequest(request: SetupRequest, question: string): SetupRequest {
	const asked = question.trim();
	return asked ? { ...request, options: { ...request.options, question: asked } } : request;
}

export class InterviewAnalysis {
	question = $state('');
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
	error = $state<string | null>(null);
	busy = $state(false);

	readonly catalog = $derived(app.catalog!);
	readonly rule = $derived(rulesFor(app.rules, 'analysis'));
	readonly model = $derived(modelInfo(this.catalog, this.draft));
	readonly providerName = $derived(providerNameOf(this.catalog, this.draft.provider));
	/** The provider can be sent the text: its key is saved. */
	readonly keyed = $derived(hasKey(this.draft.provider));
	readonly request = $derived(
		interviewRequest(toRequest(this.catalog, 'analysis', this.draft), this.question)
	);

	/** Freezes the transcript's text, every source's or one source's, as the other choices of
	 * "Use this text" do. */
	async prepare(jobId: string, setupId: string, sourceId: string | null): Promise<void> {
		try {
			this.sources = await api.snapshotJob(jobId, setupId, sourceId);
		} catch (e) {
			this.error = describeError(e, messages());
		}
	}

	/** The job as New job would make it from these choices. */
	job(sources: SourceFile[]): NewJob {
		const m = messages();
		const draft = this.draft;
		return {
			name: m.interview.jobName(sources[0]?.file_name ?? ''),
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
		if (!sources || this.busy) return false;
		this.busy = true;
		this.error = null;
		try {
			if (!(await mayStart([this.request], this.catalog, sources, true))) return false;
			const job = await api.createJob(this.job(sources));
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
