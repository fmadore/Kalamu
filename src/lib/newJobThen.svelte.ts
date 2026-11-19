// The analysis New job can run on the text once it is read (and checked), when the task's
// results are a text: the prompt chosen, whether the text is checked first, the model that
// analyses it, and the two-step workflow that reads the text, then analyses it.

import type { OutputDestination } from './bindings/OutputDestination.js';
import type { Pace } from './bindings/Pace.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Preset } from './bindings/Preset.js';
import type { Workflow } from './bindings/Workflow.js';
import { messages } from './i18n/index.js';
import { followingAnalysis, type NewJobForm, rulesFor } from './newJob.js';
import type { JobChoices } from './newJobChoices.svelte.js';
import { newDraft, providerDefault, toRequest } from './newJobModels.js';
import { app, hasKey } from './state.svelte.js';
import { api } from './tauri.js';

export class FollowingAnalysis {
	readonly #choices: JobChoices;
	readonly #m = messages();
	/** An analysis to run on the text once it is read (and checked), or none. */
	presetId = $state<string | null>(null);
	check = $state(true);
	presets = $state<Preset[]>([]);
	readonly rule = $derived(rulesFor(app.rules, 'analysis'));
	/** Whether an analysis can follow: the task's results are a text. */
	readonly offered = $derived.by(() => this.#choices.rule.yields_text);
	/** The model that analyses the text: the reading provider's, when it has one and a key. */
	readonly draft = $derived.by(() => {
		const catalog = this.#choices.catalog;
		const draft = newDraft(catalog, this.rule, this.presetId, hasKey);
		const provider = this.#choices.drafts[0]?.provider;
		const same = provider && hasKey(provider) ? providerDefault(catalog, this.rule, provider) : '';
		return same ? { ...draft, provider: provider!, model: same } : draft;
	});

	constructor(choices: JobChoices, kept: NewJobForm | null) {
		this.#choices = choices;
		this.presetId = kept?.thenPreset ?? null;
		this.check = kept?.thenCheck ?? true;
	}

	/** Reads the analyses that can follow. */
	load() {
		api
			.listPresets('analysis')
			.then((p) => (this.presets = p))
			.catch(() => undefined);
	}

	/** The analysis chosen to follow, when one is offered. */
	chosen(): Preset | undefined {
		return this.offered ? this.presets.find((p) => p.id === this.presetId) : undefined;
	}

	/** The analysis that follows, as a pipeline: the researcher's usual formats for it. */
	pipeline(preset: Preset, pace: Pace, destination: OutputDestination): Pipeline {
		return followingAnalysis(
			preset,
			toRequest(this.#choices.catalog, 'analysis', this.draft),
			this.rule,
			app.settings!.text_formats,
			pace,
			destination
		);
	}

	/** Reads the text, then analyses it: a two-step workflow, unsaved, the reading run as `read`
	 * gives it. */
	workflow(read: Pipeline, preset: Preset, pace: Pace, destination: OutputDestination): Workflow {
		const task = this.#choices.task;
		return {
			id: '',
			name: this.#m.newJob.then.workflowName(this.#m.task[task], preset.name),
			version: 0,
			steps: [
				{ pipeline: read, input: { kind: 'sources' }, review: this.check },
				{
					pipeline: this.pipeline(preset, pace, destination),
					input: { kind: 'step', step: 0 },
					review: false
				}
			],
			updated_at: 0
		};
	}
}
