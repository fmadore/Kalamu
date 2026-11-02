// Editing a workflow in the form: which inputs a step may read, and steps added, moved and
// removed with what they read kept pointing at the same steps. Whether a workflow can run is
// the engine's to say (`check_steps`, again when a run starts); this only offers what fits.

import type { Catalog } from './bindings/Catalog.js';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { InputKind } from './bindings/InputKind.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { RunStatus } from './bindings/RunStatus.js';
import type { StepInput } from './bindings/StepInput.js';
import type { StepState } from './bindings/StepState.js';
import type { Task } from './bindings/Task.js';
import type { Workflow } from './bindings/Workflow.js';
import type { WorkflowStep } from './bindings/WorkflowStep.js';
import { newDraft, type Rules, rulesFor, toRequest } from './newJob.js';

export const SOURCES: StepInput = { kind: 'sources' };

export function blankWorkflow(): Workflow {
	return { id: '', name: '', version: 0, steps: [], updated_at: 0 };
}

/**
 * A saved pipeline as a step. One that can read text, added after a step that yields it (an
 * analysis after an extraction, a correction after a transcription), reads the latest such
 * step; any other reads the files.
 */
export function stepFrom(pipeline: Pipeline, steps: WorkflowStep[], rules: Rules): WorkflowStep {
	const step: WorkflowStep = { pipeline: { ...pipeline }, input: SOURCES, review: false };
	const readable = textInputs([...steps, step], steps.length, rules);
	return { ...step, input: readable[readable.length - 1] ?? SOURCES };
}

/** The earlier steps whose text step `index` can read. */
export function textInputs(steps: WorkflowStep[], index: number, rules: Rules): StepInput[] {
	const step = steps[index];
	if (!step || !rulesFor(rules, step.pipeline.task).sources.includes('text')) return [];
	return steps
		.slice(0, index)
		.flatMap((s, i) =>
			rulesFor(rules, s.pipeline.task).yields_text ? [{ kind: 'step', step: i } as StepInput] : []
		);
}

/** Every input step `index` can read: the files, then the earlier steps' text. */
export function inputChoices(steps: WorkflowStep[], index: number, rules: Rules): StepInput[] {
	return [SOURCES, ...textInputs(steps, index, rules)];
}

export function sameInput(a: StepInput, b: StepInput): boolean {
	return a.kind === b.kind && (a.kind === 'sources' || (b.kind === 'step' && a.step === b.step));
}

/**
 * Whether a workflow can start on sources of these kinds: every step that works on the files
 * reads them. Text frozen from an earlier job suits a workflow that starts by correcting or
 * analysing, not one that starts by reading scans. The engine checks it again at the start.
 */
export function startsOn(workflow: Workflow, kinds: InputKind[], rules: Rules): boolean {
	return workflow.steps.every(
		(s) =>
			s.input.kind !== 'sources' ||
			kinds.every((k) => rulesFor(rules, s.pipeline.task).sources.includes(k))
	);
}

/** Whether a later step reads step `index`, so a review after it holds something. */
export function isRead(steps: WorkflowStep[], index: number): boolean {
	return steps.some((s) => s.input.kind === 'step' && s.input.step === index);
}

/**
 * The steps with one moved from `from` to `to`. What each step reads follows the step it read;
 * a step that would now read one after it reads the files instead.
 */
export function moveStep(steps: WorkflowStep[], from: number, to: number): WorkflowStep[] {
	if (from === to || to < 0 || to >= steps.length) return steps;
	const order = steps.map((_, i) => i);
	const [moved] = order.splice(from, 1);
	order.splice(to, 0, moved);
	const at = new Map(order.map((old, i) => [old, i]));
	return order.map((old, i) => remap(steps[old], (read) => at.get(read), i));
}

/** The steps without step `index`; a step that read it reads the files instead. */
export function removeStep(steps: WorkflowStep[], index: number): WorkflowStep[] {
	return steps
		.filter((_, i) => i !== index)
		.map((s, i) =>
			remap(s, (read) => (read === index ? undefined : read - Number(read > index)), i)
		);
}

function remap(
	step: WorkflowStep,
	to: (read: number) => number | undefined,
	position: number
): WorkflowStep {
	if (step.input.kind === 'sources') return step;
	const read = to(step.input.step);
	const input: StepInput =
		read === undefined || read >= position ? SOURCES : { kind: 'step', step: read };
	return { ...step, input };
}

/** The chip class for a run's or a step's status. */
export function tone(status: RunStatus | StepState): string {
	switch (status) {
		case 'needs_you':
		case 'review':
		case 'attention':
		case 'paused':
			return 'warn';
		case 'stopped':
		case 'cancelled':
		case 'blocked':
		case 'failed':
			return 'danger';
		case 'finished':
		case 'done':
			return 'ok';
		case 'running':
		case 'waiting':
		case 'done_with_errors':
			return '';
	}
}

/** Ready-made workflows for colleagues new to them: read, check, then analyse. */
export type Example = 'letters' | 'interviews' | 'library';

export const EXAMPLES: Record<Example, { task: Task; preset: string }[]> = {
	letters: [
		{ task: 'text_extraction', preset: 'text_extraction.printed' },
		{ task: 'analysis', preset: 'analysis.entities' }
	],
	interviews: [
		{ task: 'audio_transcription', preset: 'audio_transcription.interview' },
		{ task: 'analysis', preset: 'analysis.summary_research' }
	],
	library: [
		{ task: 'text_correction', preset: 'text_correction.general' },
		{ task: 'analysis', preset: 'analysis.summary_research' }
	]
};

/**
 * An example as a workflow to edit and save: each step on the model New job would offer (the
 * recommended one the researcher has a key for), the first checked before the second reads it.
 */
export function exampleWorkflow(
	kind: Example,
	words: { name: string; steps: string[] },
	catalog: Catalog,
	rules: Rules,
	hasKey: (provider: string) => boolean,
	formatsFor: (task: Task) => ExportFormat[]
): Workflow {
	const steps = EXAMPLES[kind].map(({ task, preset }, i): WorkflowStep => {
		const r = toRequest(catalog, task, newDraft(catalog, rulesFor(rules, task), preset, hasKey));
		const reads = rulesFor(rules, task).line_breaks;
		return {
			pipeline: {
				id: '',
				name: words.steps[i] ?? preset,
				task,
				setups: [
					{
						provider: r.provider,
						model: r.model,
						preset_id: r.preset_id,
						label: null,
						options: r.options
					}
				],
				settings: {
					languages: { mode: 'auto' },
					sample_pages: null,
					line_breaks: reads ? 'running_text' : null,
					pace: 'fast'
				},
				formats: formatsFor(task),
				destination: { kind: 'beside_sources' },
				updated_at: 0
			},
			input: i === 0 ? SOURCES : { kind: 'step', step: i - 1 },
			review: i === 0
		};
	});
	return { ...blankWorkflow(), name: words.name, steps };
}
