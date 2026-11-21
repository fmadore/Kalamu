// After a sample (#215), on its job's page: every page with one of its set-ups, priced with the
// pages it read taken from the cache, and another model on the same pages. The engine makes
// both jobs from the sample, copying its set-ups as they ran; here the researcher agrees to what
// is sent, as New job asks, and the job made starts and opens.

import type { Job } from './bindings/Job.js';
import type { RestEstimate } from './bindings/RestEstimate.js';
import type { Setup } from './bindings/Setup.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import { mayStart } from './dialogs.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { inputOf, type Mode, rulesFor } from './newJob.js';
import { type Draft, newDraft, toRequest } from './newJobModels.js';
import { launch } from './newJobStart.svelte.js';
import { app, go, hasKey } from './state.svelte.js';
import { api } from './tauri.js';

/** A set-up as it was asked for: what the questions before sending read. */
export function requestOf(setup: Setup): SetupRequest {
	return {
		provider: setup.provider,
		model: setup.model,
		preset_id: setup.prompt.preset_id,
		label: setup.label,
		options: setup.options,
		merge: setup.merge ? { provider: setup.merge.provider, model: setup.merge.model } : undefined
	};
}

/** Whether a job is a sample: so many pages of each document. */
export function isSample(job: Job): boolean {
	return job.settings.sample_pages !== null;
}

/** Asks what sending to `setups` needs, then makes the job with `make` and starts it, its page
 * opened; false when the researcher declines. */
async function makeAndStart(
	job: Job,
	setups: SetupRequest[],
	make: () => Promise<Job>
): Promise<boolean> {
	const beside = job.output.destination.kind === 'beside_sources';
	if (!(await mayStart(setups, app.catalog, job.sources, beside))) return false;
	const made = await make();
	await launch(made.id, undefined);
	go({ name: 'jobs', jobId: made.id });
	return true;
}

/** The sample's next steps on its page: the rest with each set-up, and what it costs. */
export class SampleNext {
	readonly #job: Job;
	/** What every page with each set-up costs, by set-up; null when it cannot be said. */
	rests = $state<Record<string, RestEstimate | null>>({});
	busy = $state(false);
	error = $state<string | null>(null);

	constructor(job: Job) {
		this.#job = job;
	}

	/** Asks the engine what the rest costs with each set-up. */
	load(): void {
		for (const setup of this.#job.setups) {
			api
				.restEstimate(this.#job.id, setup.id)
				.then((rest) => (this.rests[setup.id] = rest))
				.catch(() => (this.rests[setup.id] = null));
		}
	}

	/** Every page of the sample's documents with `setup`, started. */
	async runRest(setup: Setup): Promise<void> {
		const job = this.#job;
		this.busy = true;
		this.error = null;
		try {
			await makeAndStart(job, [requestOf(setup)], () =>
				api.restOfSample(job.id, setup.id, messages().jobs.sample.restName(job.name))
			);
		} catch (e) {
			this.error = describeError(e, messages());
		} finally {
			this.busy = false;
		}
	}
}

/** What every model compared with the sample must read: images when it read images. */
export function sampleInput(job: Job) {
	return inputOf(
		job.task as Mode,
		job.sources.map((source) => ({
			path: source.path,
			source,
			warnings: [],
			error: null,
			pieces: null
		}))
	);
}

/** The model first offered to compare with the sample: the first with a key (or on this
 * computer) that the sample does not use, asked what the sample asked. */
export function compareDraft(job: Job): Draft {
	const used = job.setups.map((s) => `${s.provider}/${s.model}`);
	const preset = job.setups.find((s) => s.prompt.preset_id)?.prompt.preset_id ?? null;
	return newDraft(
		app.catalog!,
		rulesFor(app.rules, job.task),
		preset,
		hasKey,
		used,
		sampleInput(job)
	);
}

/** Another model, `draft`'s, beside the sample's on its pages, started; false when the
 * researcher declines what would be sent. Throws what the engine refuses. */
export async function compareSample(job: Job, draft: Draft): Promise<boolean> {
	const request = toRequest(app.catalog!, job.task, draft);
	return makeAndStart(job, [request], () =>
		api.compareSample(
			job.id,
			draft.provider,
			draft.model,
			messages().jobs.sample.compareName(job.name)
		)
	);
}
