import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Preset } from './bindings/Preset.js';
import type { Task } from './bindings/Task.js';
import type { WorkflowStep } from './bindings/WorkflowStep.js';
import rulesJson from './dev/taskRules.json';
import type { Rules } from './newJob.js';
import { en } from './i18n/index.js';
import {
	EXAMPLES,
	type Example,
	exampleWorkflow,
	inputChoices,
	isRead,
	moveStep,
	presetPipeline,
	removeStep,
	SOURCES,
	startsOn,
	stepFrom,
	textInputs,
	withPreset
} from './workflows.js';

const rules = rulesJson as Rules;

function pipeline(name: string, task: Task): Pipeline {
	return {
		id: `pipeline-${name}`,
		name,
		task,
		setups: [],
		settings: { languages: { mode: 'auto' }, sample_pages: null, line_breaks: null, pace: 'fast' },
		formats: [],
		destination: { kind: 'beside_sources' },
		updated_at: 0
	};
}

function build(...tasks: Task[]): WorkflowStep[] {
	return tasks.reduce<WorkflowStep[]>(
		(steps, task, i) => [...steps, stepFrom(pipeline(`s${i}`, task), steps, rules)],
		[]
	);
}

const inputs = (steps: WorkflowStep[]) =>
	steps.map((s) => (s.input.kind === 'sources' ? 'files' : s.input.step));

describe('workflow steps', () => {
	it('read the latest text before them when they can read text', () => {
		const steps = build('text_extraction', 'analysis', 'analysis');
		expect(inputs(steps)).toEqual(['files', 0, 0]);
		// A transcript is corrected, and the correction analysed.
		expect(inputs(build('audio_transcription', 'text_correction', 'analysis'))).toEqual([
			'files',
			0,
			1
		]);
		// A text is translated, and the translation analysed.
		expect(inputs(build('text_extraction', 'translation', 'analysis'))).toEqual(['files', 0, 1]);
		// A table of contents reads scans, never text.
		expect(inputs(build('text_extraction', 'contents'))).toEqual(['files', 'files']);
		expect(textInputs(steps, 0, rules)).toEqual([]);
		// An analysis's fields are not a text to read.
		expect(inputChoices(steps, 2, rules)).toEqual([SOURCES, { kind: 'step', step: 0 }]);
		expect(isRead(steps, 0)).toBe(true);
		expect(isRead(steps, 1)).toBe(false);
	});

	it('keep reading the same step when moved, or the files when it would come after', () => {
		const steps = build('text_extraction', 'analysis', 'text_extraction');
		// The second extraction moved first: the analysis still reads the first extraction.
		const moved = moveStep(steps, 2, 0);
		expect(moved.map((s) => s.pipeline.name)).toEqual(['s2', 's0', 's1']);
		expect(inputs(moved)).toEqual(['files', 'files', 1]);
		// The analysis moved before what it read reads the files.
		expect(inputs(moveStep(steps, 1, 0))).toEqual(['files', 'files', 'files']);
		expect(moveStep(steps, 0, 5)).toBe(steps);
	});

	it('read the files when the step they read is removed', () => {
		const steps = build('text_extraction', 'text_extraction', 'analysis');
		expect(inputs(steps)).toEqual(['files', 'files', 1]);
		expect(inputs(removeStep(steps, 1))).toEqual(['files', 'files']);
		expect(inputs(removeStep(steps, 0))).toEqual(['files', 0]);
	});
});

const catalog = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;
const presets = readFileSync('crates/prompts/src/defaults.rs', 'utf8');

describe('example workflows', () => {
	it('check the text read before the analysis reads it, with presets that exist', () => {
		for (const kind of Object.keys(EXAMPLES) as Example[]) {
			const w = exampleWorkflow(
				kind,
				en.workflows.examples[kind],
				catalog,
				rules,
				() => true,
				() => ['txt']
			);
			expect(w.name).toBe(en.workflows.examples[kind].name);
			expect(en.workflows.examples[kind].steps).toHaveLength(w.steps.length);
			// Each later step reads the one before it, and only the first is checked.
			expect(inputs(w.steps)).toEqual(['files', ...w.steps.slice(1).map((_, i) => i)]);
			expect(w.steps.map((s) => s.review)).toEqual(w.steps.map((_, i) => i === 0));
			// The second step reads only what the first can give it.
			expect(textInputs(w.steps, 1, rules)).toEqual([{ kind: 'step', step: 0 }]);
			for (const s of w.steps) {
				expect(s.pipeline.setups[0].model).not.toBe('');
				// A model that takes no prompt (Voxtral's transcription) has none, as in New job.
				const preset = s.pipeline.setups[0].preset_id;
				if (preset !== null) expect(presets).toContain(`"${preset}"`);
			}
		}
	});

	it('translates into a language left for the researcher to choose', () => {
		const w = exampleWorkflow(
			'translated',
			en.workflows.examples.translated,
			catalog,
			rules,
			() => true,
			() => ['txt']
		);
		expect(w.steps.map((s) => s.pipeline.task)).toEqual([
			'text_extraction',
			'translation',
			'analysis'
		]);
		// No language chosen: the engine refuses to save it until one is.
		expect(w.steps[1].pipeline.settings.languages).toEqual({ mode: 'auto' });
		// The analysis reads the translation.
		expect(w.steps[2].input).toEqual({ kind: 'step', step: 1 });
	});
});

describe('steps made from prompts', () => {
	const preset = (id: string, name: string): Preset => ({
		id,
		task: 'analysis',
		name,
		description: '',
		instructions: '',
		user_prompt: '',
		builtin: true,
		modified: false
	});
	const entities = preset('analysis.entities', 'People and places');
	const summary = preset('analysis.summary_research', 'Research summary');

	it('runs a prompt on the model New job would offer, as a step to add', () => {
		const p = presetPipeline(
			'analysis',
			entities.id,
			entities.name,
			catalog,
			rules,
			() => true,
			() => ['txt']
		);
		expect(p.name).toBe('People and places');
		expect(p.setups).toHaveLength(1);
		expect(p.setups[0].model).not.toBe('');
		expect(p.setups[0].preset_id).toBe('analysis.entities');
		expect(p.formats).toEqual(['txt']);
		const [first] = build('text_extraction');
		const step = stepFrom(p, [first], rules);
		// After a step that reads the text, it reads that text.
		expect(step.input).toEqual({ kind: 'step', step: 0 });
	});

	it('changes a step’s prompt, and its name while it still names the prompt', () => {
		const p = presetPipeline(
			'analysis',
			entities.id,
			entities.name,
			catalog,
			rules,
			() => true,
			() => []
		);
		const step: WorkflowStep = { pipeline: p, input: SOURCES, review: false };
		const changed = withPreset(step, summary, entities);
		expect(changed.pipeline.setups[0].preset_id).toBe('analysis.summary_research');
		expect(changed.pipeline.name).toBe('Research summary');
		const renamed = { ...step, pipeline: { ...p, name: 'Who is named' } };
		expect(withPreset(renamed, summary, entities).pipeline.name).toBe('Who is named');
		// A comparison keeps its own prompts.
		const compared = { ...step, pipeline: { ...p, setups: [p.setups[0], p.setups[0]] } };
		expect(withPreset(compared, summary, entities)).toBe(compared);
	});
});

describe('starting a workflow on text already read', () => {
	it('suits workflows that start by correcting or analysing, not by reading scans', () => {
		const words = en.workflows.examples;
		const make = (kind: Example) =>
			exampleWorkflow(
				kind,
				words[kind],
				catalog,
				rules,
				() => true,
				() => []
			);
		expect(startsOn(make('library'), ['text'], rules)).toBe(true);
		expect(startsOn(make('letters'), ['text'], rules)).toBe(false);
		expect(startsOn(make('letters'), ['pdf', 'image'], rules)).toBe(true);
		expect(startsOn(make('interviews'), ['text'], rules)).toBe(false);
	});
});
