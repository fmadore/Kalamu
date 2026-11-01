import { describe, expect, it } from 'vitest';
import type { Pipeline } from './bindings/Pipeline.js';
import type { Task } from './bindings/Task.js';
import type { WorkflowStep } from './bindings/WorkflowStep.js';
import rulesJson from './dev/taskRules.json';
import type { Rules } from './newJob.js';
import {
	inputChoices,
	isRead,
	moveStep,
	removeStep,
	SOURCES,
	stepFrom,
	textInputs
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
